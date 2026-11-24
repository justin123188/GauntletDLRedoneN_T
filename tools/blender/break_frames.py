"""Make the breaking frames of a replaced object for the model pack
(decisions/0023, models-plan M5): the new model is cut into the pieces of
the original's break, and every piece moves as the original's piece moves,
frame by frame.

    blender --background --factory-startup --python tools/blender/break_frames.py -- \
        ORIGINAL_DIR NEW.glb OUT_DIR FRAME... [--thickness T] [--keep-base 0|1] \
        [--inside-from WOOD.glb] [--report FILE.json] [--preview DIR]

ORIGINAL_DIR is the package's folder of the model export
(objects/<package>); NEW.glb is the fitted whole model (the standing
object's model pack file, from fit_model.py); FRAME... are the labels of
the original's breaking flip-book in order (G1's barrel: BAROBJACTI16F01
... BAROBJACTI16F24 BAROBJDONE17F24). OUT_DIR gets FRAME.glb for each,
for model_pack/<package>/.

How the original is read (its frames, not guesses):
- frames with the first frame's shape before the first broken one are the
  whole object: NEW.glb itself is written for them;
- the first frame whose triangles differ is the rest frame: the pieces in
  their places;
- the later frames must share one vertex layout; their pieces are the
  connected triangles (copies at one place, the original's two-sided
  faces, are one piece), and each moves rigidly (checked);
- each piece's place in the rest frame is found by stepping its motion
  back once and matching it to the rest frame's corners (checked; when the
  rest frame shares the layout, by index).
The new model, in these steps:
- welded at its seams, normals from its shape (40 degrees smooth);
- cut where the pieces meet at rest: a plane through each shared edge and
  the object's centre, and halfway between two pieces of one material that
  share a long border on the new model but no edge at rest (G1's barrel:
  the six corners, the middle, and between the lid halves);
- each face sorted into the part of the object it is (the material of the
  nearest surface of the original's first frame, the normals counting: for
  G1's barrel its side or its top), then into the nearest piece of that
  material at rest; every part between the cuts goes whole to the piece
  most of its area is nearest to, so the pieces end at the cuts;
- the base (faces looking down at the bottom) and what only it held (loose
  parts that touched it, each under 15 % of the area) removed, unless
  --keep-base 1: the original's pieces have no base;
- each piece keeps its largest connected part; smaller parts go to the
  piece they share the longest border with;
- each piece thickened inwards by T (default 3 % of the height), with
  plain wood inside and on its broken edges (the middle tones of the new
  texture, or of WOOD.glb's for a painted model: G1's red and poison
  barrels take the brown barrel's; darker, in streaks with stave seams; a
  second material);
- placed in each frame as its piece is (one glTF node per piece), never
  lower than the original's piece or the ground.
With --preview, pictures of the original's and the new frames from one
camera, and a sheet of both. Every run with the same files writes the same
frames.
"""
import json
import math
import os
import shutil
import struct
import sys
import tempfile

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Vector

MAX_BYTES = 512 * 1024 * 1024
# glTF (y up) to Blender (z up), as the importer and exporter convert.
TO_BLENDER = np.array([[1, 0, 0, 0], [0, 0, -1, 0], [0, 1, 0, 0], [0, 0, 0, 1]], float)
FROM_BLENDER = TO_BLENDER.T


# --- the original's frames, read exactly (no importer) ---------------------

COMPONENT = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
WIDTH = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}


def glb(path):
    if os.path.getsize(path) > MAX_BYTES:
        raise SystemExit(f'{path} is larger than the size policy')
    with open(path, 'rb') as f:
        b = f.read()
    magic, _, length = struct.unpack_from('<III', b, 0)
    if magic != 0x46546C67:
        raise SystemExit(f'{path}: not a .glb')
    at, doc, data = 12, None, b''
    while at + 8 <= min(length, len(b)):
        size, kind = struct.unpack_from('<II', b, at)
        chunk = b[at + 8:at + 8 + size]
        if kind == 0x4E4F534A:
            doc = json.loads(chunk)
        elif kind == 0x004E4942:
            data = chunk
        at += 8 + size
    if doc is None:
        raise SystemExit(f'{path}: no JSON chunk')
    return doc, data


def accessor(doc, data, i):
    a = doc['accessors'][i]
    v = doc['bufferViews'][a['bufferView']]
    kind, n = COMPONENT[a['componentType']], WIDTH[a['type']]
    start = v.get('byteOffset', 0) + a.get('byteOffset', 0)
    item = np.dtype(kind).itemsize * n
    stride = v.get('byteStride', 0) or item
    raw = np.frombuffer(data, np.uint8, count=stride * (a['count'] - 1) + item, offset=start)
    rows = np.lib.stride_tricks.as_strided(raw, (a['count'], item), (stride, 1)).copy()
    out = rows.view(kind).reshape(a['count'], n)
    if a.get('normalized'):
        out = out.astype(np.float64) / np.iinfo(kind).max
    return out.astype(np.float64) if out.dtype.kind == 'f' else out.astype(np.int64)


def node_matrix(n):
    if 'matrix' in n:
        return np.array(n['matrix'], float).reshape(4, 4).T
    x, y, z, w = n.get('rotation', [0, 0, 0, 1])
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * np.array(n.get('scale', [1, 1, 1]), float)
    m[:3, 3] = n.get('translation', [0, 0, 0])
    return m


def read_frame(path):
    """The frame's primitives in its space: material name, positions (node
    transforms applied), UVs and triangle indices."""
    doc, data = glb(path)
    out = []

    def walk(i, parent):
        n = doc['nodes'][i]
        m = parent @ node_matrix(n)
        for p in doc['meshes'][n['mesh']]['primitives'] if 'mesh' in n else []:
            if p.get('mode', 4) != 4:
                raise SystemExit(f'{path}: only triangle lists')
            at = p['attributes']
            pos = accessor(doc, data, at['POSITION'])
            pos = (np.c_[pos, np.ones(len(pos))] @ m.T)[:, :3]
            idx = accessor(doc, data, p['indices']).ravel() if 'indices' in p else np.arange(len(pos))
            name = doc['materials'][p['material']].get('name', str(p['material'])) if 'material' in p else ''
            uv = accessor(doc, data, at['TEXCOORD_0']) if 'TEXCOORD_0' in at else np.zeros((len(pos), 2))
            out.append({'material': name, 'pos': pos, 'uv': uv, 'idx': idx})
        for c in n.get('children', []):
            walk(c, m)

    for r in doc['scenes'][doc.get('scene', 0)]['nodes']:
        walk(r, np.eye(4))
    return out


def same_layout(a, b):
    return len(a) == len(b) and all(
        x['material'] == y['material'] and len(x['pos']) == len(y['pos']) and np.array_equal(x['idx'], y['idx'])
        for x, y in zip(a, b))


def same_shape(a, b, tol):
    return same_layout(a, b) and all(np.abs(x['pos'] - y['pos']).max() <= tol for x, y in zip(a, b))


def components(idx, n):
    parent = list(range(n))

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    used = set()
    for t in idx.reshape(-1, 3):
        used.update(int(v) for v in t)
        for a, b in ((t[0], t[1]), (t[1], t[2])):
            ra, rb = find(a), find(b)
            if ra != rb:
                parent[ra] = rb
    groups = {}
    for v in sorted(used):
        groups.setdefault(find(v), []).append(v)
    return [np.array(g) for g in groups.values()]


def apply(m, p):
    return p @ m[:3, :3].T + m[:3, 3]


def kabsch(a, b):
    """A 4x4 rigid transform taking points a onto b, and the worst distance left."""
    ca, cb = a.mean(0), b.mean(0)
    u, _, vt = np.linalg.svd((a - ca).T @ (b - cb))
    d = np.sign(np.linalg.det(vt.T @ u.T)) or 1.0
    r = vt.T @ np.diag([1, 1, d]) @ u.T
    m = np.eye(4)
    m[:3, :3] = r
    m[:3, 3] = cb - r @ ca
    left = np.linalg.norm(apply(m, a) - b, axis=1).max()
    return m, float(left)


def pieces_of(frame):
    """Connected triangles per primitive; copies at one place are one piece."""
    out, seen = [], {}
    for pi, p in enumerate(frame):
        for c in components(p['idx'], len(p['pos'])):
            key = (p['material'], tuple(sorted(map(tuple, np.round(p['pos'][c], 3).tolist()))))
            if key in seen:
                seen[key]['copies'].append((pi, c))
                continue
            members = set(c.tolist())
            tris = [t for t in p['idx'].reshape(-1, 3) if int(t[0]) in members]
            seen[key] = {'material': p['material'], 'prim': pi, 'verts': c, 'tris': np.array(tris), 'copies': [(pi, c)]}
            out.append(seen[key])
    return out


def place_at_rest(a1, a2, cloud, rounds=100):
    """Rest positions of a piece seen at a1 and then a2: its step a1 -> a2
    undone once from a1, then matched to the rest frame's corners."""
    back, _ = kabsch(a2, a1)
    guess = apply(back, a1)
    cur, target = guess, guess
    for _ in range(rounds):
        target = cloud[((cur[:, None, :] - cloud[None, :, :]) ** 2).sum(-1).argmin(1)]
        m, _ = kabsch(a1, target)
        new = apply(m, a1)
        if np.abs(new - cur).max() < 1e-9:
            break
        cur = new
    m, left = kabsch(a1, target)
    return apply(m, a1), left, float(np.linalg.norm(guess.mean(0) - target.mean(0)))


def shared_edge_planes(pieces, centre, tol):
    """Cutting planes where two pieces meet at rest: through each shared
    edge and the object's centre (for G1's barrel: the six corners and the
    middle)."""
    edges = []
    for k, p in enumerate(pieces):
        for t in p['tris']:
            for a, b in ((t[0], t[1]), (t[1], t[2]), (t[2], t[0])):
                edges.append((k, p['rest'][int(a)], p['rest'][int(b)]))
    planes, touching = [], set()
    for i, (ki, a, b) in enumerate(edges):
        for kj, c, d in edges[i + 1:]:
            if ki == kj:
                continue
            same = (np.linalg.norm(a - c) < tol and np.linalg.norm(b - d) < tol) or \
                   (np.linalg.norm(a - d) < tol and np.linalg.norm(b - c) < tol)
            if same:
                touching.add((min(ki, kj), max(ki, kj)))
                add_plane(planes, np.cross(a - centre, b - centre), centre, tol)
    return planes, touching


def add_plane(planes, normal, point, tol):
    length = np.linalg.norm(normal)
    if length < 1e-9:
        return
    normal = normal / length
    for n2, p2 in planes:
        if abs(abs(normal @ n2) - 1) < 1e-3 and abs((point - p2) @ n2) < tol:
            return
    planes.append((normal, point))


def base_faces(centres, normals, opts):
    """Faces looking down at the bottom (none with --keep-base 1)."""
    h = float(np.ptp(centres[:, 1])) or 1.0
    base = (normals[:, 1] < -0.5) & (centres[:, 1] < centres[:, 1].min() + 0.05 * h)
    if opts['keep-base']:
        base[:] = False
    return base


def surface_samples(frame, centre):
    """Points over an original frame's triangles, each with its material and
    its normal turned away from `centre` (the export keeps the original's
    triangle order, so the winding alone does not say which way is out)."""
    pts, mats, nrm = [], [], []
    for p in frame:
        for t in p['idx'].reshape(-1, 3):
            c = p['pos'][t]
            n = np.cross(c[1] - c[0], c[2] - c[0])
            length = np.linalg.norm(n)
            if length < 1e-12:
                continue
            n = n / length
            if n @ (c.mean(0) - centre) < 0:
                n = -n
            s = triangle_samples(c, 8)
            pts.append(s)
            mats += [p['material']] * len(s)
            nrm.append(np.repeat(n[None], len(s), 0))
    return np.vstack(pts), np.array(mats), np.vstack(nrm)


def classify(centres, normals, pts, mats, nrm, weight):
    """The material of the nearest original surface, a facing the other way
    counting as `weight` more distance per unit of the normals' difference."""
    out = np.empty(len(centres), dtype=object)
    for s in range(0, len(centres), 256):
        c, n = centres[s:s + 256], normals[s:s + 256]
        d = np.sqrt(((c[:, None, :] - pts[None, :, :]) ** 2).sum(-1)) + weight * (1 - n @ nrm.T)
        out[s:s + 256] = mats[d.argmin(1)]
    return out


def by_cells(centres, areas, kind, labels, planes):
    """Each part between the cutting planes (per material) given whole to the
    piece most of its area is nearest to."""
    sides = np.stack([(centres - p) @ nm >= 0 for nm, p in planes], 1) if planes else np.zeros((len(centres), 0), bool)
    out = labels.copy()
    groups = {}
    for i, (m, s) in enumerate(zip(kind.tolist(), map(bytes, np.packbits(sides, axis=1)))):
        groups.setdefault((m, s), []).append(i)
    for members in groups.values():
        members = np.array(members)
        votes = {}
        for i in members:
            votes[labels[i]] = votes.get(labels[i], 0.0) + areas[i]
        out[members] = max(votes, key=votes.get)
    return out


def loose_with_base(me, base, areas, share=0.15):
    """Parts of the model that the base held (they touched it and are loose
    without it), each under `share` of the area left: G1's new barrel has a
    bottom ring and two battens under its base."""
    if not base.any():
        return base
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.faces.ensure_lookup_table()
    seen = np.zeros(len(base), bool)
    out = base.copy()
    total = areas[~base].sum()
    for f in bm.faces:
        if seen[f.index] or base[f.index]:
            continue
        seen[f.index] = True
        stack, members, touches = [f], [], False
        while stack:
            g = stack.pop()
            members.append(g.index)
            for e in g.edges:
                for h in e.link_faces:
                    if base[h.index]:
                        touches = True
                    elif not seen[h.index]:
                        seen[h.index] = True
                        stack.append(h)
        members = np.array(members)
        if touches and areas[members].sum() < share * total:
            out[members] = True
    bm.free()
    return out


def islands_to_neighbours(me, labels, areas, rounds=4):
    """Each piece keeps its largest connected part; a smaller part goes to
    the piece it shares the longest border with (stray bits, such as rim
    pieces cut off from a lid)."""
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.faces.ensure_lookup_table()
    moved = 0
    for _ in range(rounds):
        part = np.full(len(labels), -1)
        parts = []
        for f in bm.faces:
            if part[f.index] >= 0 or labels[f.index] < 0:
                continue
            part[f.index] = len(parts)
            stack, members = [f], []
            while stack:
                g = stack.pop()
                members.append(g.index)
                for e in g.edges:
                    for h in e.link_faces:
                        if part[h.index] < 0 and labels[h.index] == labels[f.index]:
                            part[h.index] = len(parts)
                            stack.append(h)
            parts.append(np.array(members))
        biggest = {}
        for c, members in enumerate(parts):
            lab, area = labels[members[0]], areas[members].sum()
            if lab not in biggest or area > biggest[lab][1]:
                biggest[lab] = (c, area)
        changed = False
        for c, members in enumerate(parts):
            lab = labels[members[0]]
            if biggest[lab][0] == c:
                continue
            border = {}
            for i in members:
                for e in bm.faces[i].edges:
                    for h in e.link_faces:
                        other = labels[h.index]
                        if other != lab and other >= 0:
                            border[other] = border.get(other, 0.0) + e.calc_length()
            if border:
                labels[members] = max(border, key=border.get)
                moved += len(members)
                changed = True
        if not changed:
            break
    bm.free()
    return labels, moved


def nearest_piece(centres, samples, owner):
    out = np.empty(len(centres), int)
    for s in range(0, len(centres), 512):
        d = ((centres[s:s + 512, None, :] - samples[None, :, :]) ** 2).sum(-1)
        out[s:s + 512] = owner[d.argmin(1)]
    return out


def triangle_samples(corners, steps=12):
    """Points spread over a triangle (barycentric grid)."""
    pts = []
    for i in range(steps + 1):
        for j in range(steps + 1 - i):
            a, b = i / steps, j / steps
            pts.append(corners[0] * (1 - a - b) + corners[1] * a + corners[2] * b)
    return np.array(pts)


# --- Blender ----------------------------------------------------------------

def parse_args():
    a = sys.argv[sys.argv.index('--') + 1:]
    opts = {'thickness': -1.0, 'keep-base': 0, 'inside-from': '', 'report': '', 'preview': ''}
    pos = []
    while a:
        k = a.pop(0)
        if k.startswith('--'):
            if k[2:] not in opts or not a:
                raise SystemExit(f'unknown or incomplete option {k}')
            opts[k[2:]] = type(opts[k[2:]])(a.pop(0))
        else:
            pos.append(k)
    if len(pos) < 5:
        raise SystemExit(__doc__)
    return pos[0], pos[1], pos[2], pos[3:], opts


def select_only(objects, active=None):
    for o in bpy.context.scene.objects:
        o.select_set(o in objects)
    bpy.context.view_layer.objects.active = active or (objects[0] if objects else None)


def blender_points(me):
    pts = np.empty(len(me.vertices) * 3)
    me.vertices.foreach_get('co', pts)
    return pts.reshape(-1, 3)


def inside_image(src, size=256):
    """Plain wood for the pieces' inside and broken edges: the new model's
    wood colour (its texture's middle-bright pixels) in soft streaks."""
    px = np.empty(src.size[0] * src.size[1] * 4, np.float32)
    src.pixels.foreach_get(px)
    rgb = px.reshape(-1, 4)[:, :3]
    lum = rgb @ np.array([0.2126, 0.7152, 0.0722])
    used = lum > 0.03  # not the black between a baked texture's islands
    lo, hi = np.percentile(lum[used], [40, 80]) if used.any() else (0.0, 1.0)
    pick = used & (lum >= lo) & (lum <= hi)
    wood = np.median(rgb[pick], 0) if pick.any() else np.array([0.4, 0.25, 0.12])
    # The inside, out of the weather: darker and a little greyer than the
    # outside's middle tones.
    wood = (0.85 * wood + 0.15 * float(wood @ np.array([0.2126, 0.7152, 0.0722]))) * 0.72
    rng = np.random.default_rng(23)  # fixed, so every run writes the same frames

    def periodic(n, width):
        x = np.tile(rng.random(n), 3)
        return np.convolve(x, np.ones(width) / width, 'same')[n:2 * n]

    streak = periodic(size, 9)
    streak = (streak - streak.min()) / max(float(np.ptp(streak)), 1e-6)
    grain = rng.random((size, size)) * 0.08
    # Soft streaks of grain, and a dark seam between staves three times
    # across the texture.
    value = 0.85 + 0.25 * streak[None, :] + grain
    u = np.arange(size) / size * 3 % 1.0
    seam = np.where(np.minimum(u, 1 - u) < 0.03, 0.4, 1.0)
    img = np.clip(wood[None, None, :] * (value * seam[None, :])[:, :, None], 0, 1)
    out = bpy.data.images.new('inside', size, size, alpha=False)
    out.pixels.foreach_set(np.concatenate([img, np.ones((size, size, 1))], 2).astype(np.float32).ravel())
    out.file_format = 'PNG'
    out.pack()
    return out, wood


def inside_material(image):
    mat = bpy.data.materials.new('inside')
    mat.use_nodes = True
    tex = mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex.image = image
    bsdf = mat.node_tree.nodes['Principled BSDF']
    mat.node_tree.links.new(tex.outputs['Color'], bsdf.inputs['Base Color'])
    bsdf.inputs['Roughness'].default_value = 1.0
    mat.use_backface_culling = True
    return mat


def inside_uvs(me, centre, radius, scale):
    """Cylindrical UVs about the vertical axis for the inside faces
    (planar for flat ones), `scale` model units per texture."""
    uv = me.uv_layers.active.data
    for poly in me.polygons:
        if poly.material_index != 1:
            continue
        loops = list(poly.loop_indices)
        cos = [me.vertices[me.loops[li].vertex_index].co for li in loops]
        if abs(poly.normal.z) > 0.7:
            for li, co in zip(loops, cos):
                uv[li].uv = ((co.x - centre[0]) / scale, (co.y - centre[1]) / scale)
            continue
        ang = [math.atan2(co.y - centre[1], co.x - centre[0]) for co in cos]
        if max(ang) - min(ang) > math.pi:
            ang = [a + 2 * math.pi if a < 0 else a for a in ang]
        for li, co, a in zip(loops, cos, ang):
            uv[li].uv = (a * radius / scale, co.z / scale)


def export_selected(path):
    props = bpy.ops.export_scene.gltf.get_rna_type().properties.keys()
    wanted = {
        'filepath': path, 'export_format': 'GLB', 'use_selection': True, 'export_apply': True,
        'export_yup': True, 'export_texcoords': True, 'export_normals': True, 'export_tangents': False,
        'export_materials': 'EXPORT', 'export_image_format': 'AUTO',
        'export_draco_mesh_compression_enable': False, 'export_vertex_color': 'NONE', 'export_colors': False,
        'export_animations': False, 'export_skins': False, 'export_morph': False, 'export_cameras': False,
        'export_lights': False, 'export_extras': False,
    }
    bpy.ops.export_scene.gltf(**{k: v for k, v in wanted.items() if k in props})


def render(path, centre, radius):
    scene = bpy.context.scene
    cam_data = bpy.data.cameras.new('preview')
    cam_data.lens_unit = 'FOV'
    cam_data.angle = math.radians(40)
    cam_data.clip_start, cam_data.clip_end = radius / 1000, radius * 20
    cam = bpy.data.objects.new('preview', cam_data)
    scene.collection.objects.link(cam)
    c = Vector(centre.tolist())
    direction = Vector((0.9, -1.2, 0.8)).normalized()
    cam.location = c + direction * (radius / math.sin(cam_data.angle / 2) * 1.05)
    cam.rotation_euler = (c - cam.location).to_track_quat('-Z', 'Y').to_euler()
    scene.camera = cam
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.display.shading.light = 'STUDIO'
    scene.display.shading.color_type = 'TEXTURE'
    scene.display.shading.show_backface_culling = True
    scene.render.resolution_x = scene.render.resolution_y = 360
    scene.render.image_settings.file_format = 'PNG'
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam)


def sheet(rows, path):
    """Rows of equal pictures side by side in one PNG (the first row on top)."""
    imgs = [[bpy.data.images.load(p) for p in row] for row in rows]
    w, h = imgs[0][0].size
    out = np.zeros((h * len(rows), w * len(rows[0]), 4), np.float32)
    for r, row in enumerate(imgs):
        for c, im in enumerate(row):
            px = np.empty(w * h * 4, np.float32)
            im.pixels.foreach_get(px)
            out[(len(rows) - 1 - r) * h:(len(rows) - r) * h, c * w:(c + 1) * w] = px.reshape(h, w, 4)
    s = bpy.data.images.new('sheet', out.shape[1], out.shape[0], alpha=True)
    s.pixels.foreach_set(out.ravel())
    s.filepath_raw = path
    s.file_format = 'PNG'
    s.save()


def read_original(orig_dir, labels):
    report = {'frames': {}, 'pieces': []}
    frames = [read_frame(os.path.join(orig_dir, f + '.glb')) for f in labels]
    first = frames[0]
    allp = np.vstack([p['pos'] for p in first])
    height = float(np.ptp(allp[:, 1]))
    tol = 1e-3 * max(height, 1e-6)
    rest_i = next((i for i, f in enumerate(frames) if not same_layout(f, first)), None)
    if rest_i is None:
        raise SystemExit("no frame breaks: every frame has the first frame's triangles")
    for i in range(rest_i):
        if not same_shape(frames[i], first, tol):
            raise SystemExit(f'{labels[i]} moves the whole object before it breaks (not handled)')
    tracked = list(range(rest_i + 1, len(frames)))
    if len(tracked) < 2:
        raise SystemExit('fewer than two frames after the rest frame')
    for i in tracked:
        if not same_layout(frames[i], frames[tracked[0]]):
            raise SystemExit(f'{labels[i]}: the pieces change after {labels[tracked[0]]} (not handled)')
    pieces = pieces_of(frames[tracked[0]])
    rest_frame = frames[rest_i]
    by_index = same_layout(rest_frame, frames[tracked[0]])
    for k, piece in enumerate(pieces):
        pi, v = piece['prim'], piece['verts']
        seen = {i: frames[i][pi]['pos'][v] for i in tracked}
        worst = 0.0
        for i in tracked:
            _, left = kabsch(seen[tracked[0]], seen[i])
            worst = max(worst, left)
            if left > 0.02 * height:
                raise SystemExit(f'piece {k} is not rigid in {labels[i]} ({left:.4f})')
        if by_index:
            rest, left, guess = rest_frame[pi]['pos'][v], 0.0, 0.0
        else:
            cloud = np.vstack([p['pos'] for p in rest_frame if p['material'] == piece['material']])
            rest, left, guess = place_at_rest(seen[tracked[0]], seen[tracked[1]], cloud)
        if left > 0.02 * height:
            raise SystemExit(f'piece {k}: no place in the rest frame {labels[rest_i]} ({left:.4f})')
        piece['rest'] = dict(zip(v.tolist(), rest))
        piece['moves'] = {rest_i: np.eye(4)}
        for i in tracked:
            piece['moves'][i], _ = kabsch(rest, seen[i])
        piece['low'] = {i: min(float(frames[i][c_pi]['pos'][c][:, 1].min()) for c_pi, c in piece['copies']) for i in tracked}
        piece['low'][rest_i] = float(rest[:, 1].min())
        report['pieces'].append({'material': piece['material'], 'vertices': len(v), 'triangles': len(piece['tris']),
                                 'copies': len(piece['copies']), 'rigid_left': round(worst, 5),
                                 'rest_left': round(left, 5), 'step_back_error': round(guess, 4),
                                 'centre_at_rest': np.round(rest.mean(0), 3).tolist()})
    report['rest_frame'] = labels[rest_i]
    report['tracked_frames'] = [labels[i] for i in tracked]
    return frames, pieces, rest_i, tracked, float(allp[:, 1].min()), report


def main():
    orig_dir, new_path, out_dir, labels, opts = parse_args()
    frames, pieces, rest_i, tracked, ground, report = read_original(orig_dir, labels)

    # The new model, welded, with normals from its shape (so moved copies
    # keep them right).
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=new_path)
    objs = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    select_only(objs)
    if len(objs) > 1:
        bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    pts = blender_points(obj.data)
    lo, hi = pts.min(0), pts.max(0)
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.remove_doubles(threshold=float(np.linalg.norm(hi - lo)) * 1e-5)
    bpy.ops.object.mode_set(mode='OBJECT')
    try:
        bpy.ops.mesh.customdata_custom_splitnormals_clear()
    except Exception:
        pass
    try:
        bpy.ops.object.shade_smooth_by_angle(angle=math.radians(40))
    except Exception:
        bpy.ops.object.shade_smooth()
    me = obj.data
    samples, owner = [], []
    for k, piece in enumerate(pieces):
        for t in piece['tris']:
            s = triangle_samples(np.array([piece['rest'][int(v)] for v in t]))
            samples.append(s)
            owner += [k] * len(s)
    samples, owner = np.vstack(samples), np.array(owner)
    owner_mat = np.array([pieces[k]['material'] for k in owner])
    # Which part of the object each new face is (G1's barrel: its top or its
    # side), from the original standing object's surfaces.
    standing = frames[0]
    every = np.vstack([p['pos'] for p in standing])
    s_pts, s_mat, s_nrm = surface_samples(standing, (every.min(0) + every.max(0)) / 2)
    weight = 0.05 * float(np.ptp(every[:, 1]))

    def faces():
        n = len(me.polygons)
        c = np.empty(n * 3)
        nn = np.empty(n * 3)
        area = np.empty(n)
        me.polygons.foreach_get('center', c)
        me.polygons.foreach_get('normal', nn)
        me.polygons.foreach_get('area', area)
        return apply(FROM_BLENDER, c.reshape(n, 3)), nn.reshape(n, 3) @ FROM_BLENDER[:3, :3].T, area

    def assign(centres, normals):
        kind = classify(centres, normals, s_pts, s_mat, s_nrm, weight)
        out = np.full(len(centres), -1)
        for m in sorted(set(kind.tolist())):
            sel = kind == m
            keep = owner_mat == m
            if not keep.any():
                keep[:] = True
            out[sel] = nearest_piece(centres[sel], samples[keep], owner[keep])
        return kind, out

    # Clean breaks: cut the new model where the pieces meet at rest (shared
    # edges), and halfway between pieces of one material that touch on the
    # new model without sharing an edge (G1's barrel: the two lid halves).
    rest_pts = np.vstack([np.array(list(p['rest'].values())) for p in pieces])
    middle = (rest_pts.min(0) + rest_pts.max(0)) / 2
    tol = 0.01 * float(np.ptp(rest_pts[:, 1]))
    planes, touching = shared_edge_planes(pieces, middle, tol)
    centres, normals, _ = faces()
    _, first = assign(centres, normals)
    first[base_faces(centres, normals, opts)] = -1
    border = {}
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.faces.ensure_lookup_table()
    for e in bm.edges:
        if len(e.link_faces) == 2:
            a, b = first[e.link_faces[0].index], first[e.link_faces[1].index]
            if a != b and min(a, b) >= 0:
                key = (min(a, b), max(a, b))
                border[key] = border.get(key, 0.0) + e.calc_length()
    # A long border only (at least a quarter of the height): faces meeting
    # by chance are no break line.
    report["borders"] = {f"{a}-{b}": round(length, 3) for (a, b), length in sorted(border.items())}
    for (a, b), length in sorted(border.items()):
        if (a, b) not in touching and length >= 0.25 * float(np.ptp(rest_pts[:, 1])) and pieces[a]['material'] == pieces[b]['material']:
            ca = np.array(list(pieces[a]['rest'].values())).mean(0)
            cb = np.array(list(pieces[b]['rest'].values())).mean(0)
            add_plane(planes, ca - cb, (ca + cb) / 2, tol)
    for normal, point in planes:
        geom = list(bm.verts) + list(bm.edges) + list(bm.faces)
        bmesh.ops.bisect_plane(bm, geom=geom, dist=tol * 1e-3,
                               plane_co=apply(TO_BLENDER, point[None])[0].tolist(),
                               plane_no=(TO_BLENDER[:3, :3] @ normal).tolist())
    bm.to_mesh(me)
    bm.free()
    report['cut_planes'] = [{'normal': np.round(nm, 3).tolist(), 'point': np.round(pt, 3).tolist()} for nm, pt in planes]
    centres, normals, areas = faces()
    n = len(centres)
    new_h = float(np.ptp(centres[:, 1])) or 1.0
    base = base_faces(centres, normals, opts)
    report['base_faces'] = int(base.sum())
    base = loose_with_base(me, base, areas)
    kind, nearest = assign(centres, normals)
    # Every part between the cuts goes whole to the piece most of it is
    # nearest to, so the pieces end exactly at the cuts.
    nearest = by_cells(centres, areas, kind, nearest, planes)
    nearest[base] = -1
    nearest, report['stray_faces_moved'] = islands_to_neighbours(me, nearest, areas)
    report['new_faces'] = n
    report['faces_by_part'] = {m: int((kind == m).sum()) for m in sorted(set(kind.tolist()))}
    report['base_faces_removed'] = int(base.sum())

    outer = me.materials[0] if me.materials else None
    if outer:
        outer.use_backface_culling = True
    src = next((im for im in bpy.data.images if im.size[0] > 0), None)
    if opts['inside-from']:
        # Bare wood inside a painted model: another model's first texture.
        doc, data = glb(opts['inside-from'])
        img = next((i for i in doc.get('images', []) if i.get('mimeType') == 'image/png' and 'bufferView' in i), None)
        if img is None:
            raise SystemExit(f"{opts['inside-from']}: no embedded PNG")
        v = doc['bufferViews'][img['bufferView']]
        path = os.path.join(bpy.app.tempdir or tempfile.gettempdir(), 'inside-from.png')
        with open(path, 'wb') as f:
            f.write(data[v.get('byteOffset', 0):v.get('byteOffset', 0) + v['byteLength']])
        src = bpy.data.images.load(path)  # Blender's temp folder, gone at exit
        report['inside_from'] = opts['inside-from']
    if src:
        inner_img, wood = inside_image(src)
        report['inside_colour'] = np.round(wood, 3).tolist()
    else:
        inner_img = bpy.data.images.new('inside', 4, 4)
    inner = inside_material(inner_img)
    thickness = opts['thickness'] if opts['thickness'] >= 0 else 0.03 * new_h
    report['thickness'] = round(thickness, 4)
    centre_b = (lo + hi) / 2
    radius_b = float(max(hi[0] - lo[0], hi[1] - lo[1]) / 2)
    parts = []
    for k, piece in enumerate(pieces):
        keep = nearest == k
        report['pieces'][k]['new_faces'] = int(keep.sum())
        if not keep.any():
            continue
        p = obj.copy()
        p.data = obj.data.copy()
        p.name = p.data.name = f'piece{k:02d}'
        bpy.context.scene.collection.objects.link(p)
        bm = bmesh.new()
        bm.from_mesh(p.data)
        bm.faces.ensure_lookup_table()
        bmesh.ops.delete(bm, geom=[f for f in bm.faces if not keep[f.index]], context='FACES')
        bm.to_mesh(p.data)
        bm.free()
        if len(p.data.materials) < 2:
            p.data.materials.append(inner)
        select_only([p], p)
        if thickness > 0:
            mod = p.modifiers.new('thick', 'SOLIDIFY')
            mod.thickness = thickness
            mod.offset = -1.0
            mod.use_rim = True
            mod.material_offset = 1
            mod.material_offset_rim = 1
            bpy.ops.object.modifier_apply(modifier=mod.name)
            inside_uvs(p.data, centre_b, radius_b, new_h / 2)
            try:
                bpy.ops.object.shade_smooth_by_angle(angle=math.radians(40))
            except Exception:
                pass
        p.data.calc_loop_triangles()
        report['pieces'][k]['new_triangles'] = len(p.data.loop_triangles)
        parts.append((k, p, apply(FROM_BLENDER, blender_points(p.data))))
    bpy.data.objects.remove(obj)

    os.makedirs(out_dir, exist_ok=True)
    preview = opts['preview']
    picks = []
    if preview:
        os.makedirs(preview, exist_ok=True)
        every_b = apply(TO_BLENDER, np.vstack([np.vstack([p['pos'] for p in f]) for f in frames]))
        view = ((every_b.min(0) + every_b.max(0)) / 2, float(np.linalg.norm(np.ptp(every_b, 0)) / 2))
        picks = sorted({rest_i, tracked[0], tracked[len(tracked) // 4], tracked[len(tracked) // 2], tracked[-1]})
    for i, label in enumerate(labels):
        out = os.path.join(out_dir, label + '.glb')
        if i < rest_i:
            shutil.copyfile(new_path, out)
            report['frames'][label] = 'whole (NEW.glb)'
            continue
        lifted = 0.0
        for k, p, local in parts:
            piece = pieces[k]
            m = piece['moves'][i].copy()
            floor = min(ground, piece['low'][i])
            lift = max(0.0, floor - float(apply(m, local)[:, 1].min()))
            lifted = max(lifted, lift)
            m[1, 3] += lift
            p.matrix_world = Matrix((TO_BLENDER @ m @ FROM_BLENDER).tolist())
        select_only([p for _, p, _ in parts])
        export_selected(out)
        report['frames'][label] = {'bytes': os.path.getsize(out), 'lifted': round(lifted, 4)}
        if i in picks:
            render(os.path.join(preview, f'new-{label}.png'), *view)
    if preview:
        for _, p, _ in parts:
            p.hide_render = True
        for i in picks:
            before = set(bpy.context.scene.objects)
            bpy.ops.import_scene.gltf(filepath=os.path.join(orig_dir, labels[i] + '.glb'))
            added = [o for o in bpy.context.scene.objects if o not in before]
            render(os.path.join(preview, f'original-{labels[i]}.png'), *view)
            for o in added:
                bpy.data.objects.remove(o)
        sheet([[os.path.join(preview, f'original-{labels[i]}.png') for i in picks],
               [os.path.join(preview, f'new-{labels[i]}.png') for i in picks]], os.path.join(preview, 'sheet.png'))
        report['preview'] = [labels[i] for i in picks]
    text = json.dumps(report, indent=1)
    if opts['report']:
        with open(opts['report'], 'w', encoding='utf-8') as f:
            f.write(text)
    print('break_frames:', json.dumps({'pieces': len(pieces), 'rest': labels[rest_i], 'frames': len(labels),
                                       'base_faces_removed': report['base_faces_removed'],
                                       'triangles': sum(x.get('new_triangles', 0) for x in report['pieces'])}))


main()
