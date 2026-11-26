"""Check a replacement ground object against the level's collision (models-plan
M6, decisions/0035): where heroes walk, the new surface must stay within a
tolerance of the collision the game keeps using.

    blender --background --factory-startup --python tools/blender/check_ground.py -- \
        LEVEL.glb LABEL NEW.glb [--tolerance 0.10] [--report FILE.json] [--map FILE.png]

or with Blender's own Python (numpy): python check_ground.py LEVEL.glb LABEL NEW.glb ...

LEVEL.glb is the level's scene from `gdl_play --export-models`
(levels/<LEVEL>.glb: every placed object, the collision as COL_ nodes);
LABEL the object replaced (its node name there, as objects/<LEVEL>/<LABEL>.glb);
NEW.glb the replacement in that object's own frame (as the model pack takes it).

Where it looks: the walkable collision triangles (|normal.y| at least 0.35)
under the original object's drawn surface (within 1.5 units above or below
the collision), sampled every 0.25 units; the area this object draws. At each
sample, the new surface's height straight above or below, the one nearest
the collision: above it by more than the tolerance, a hero's feet sink into
the new ground there; below it, they float over it; no new surface within
1.5 units, a hole. The original's own surface is measured the same way, for
reference (the disc's drawn ground and collision differ a little too).
Beside it, over walkable collision the original did not draw (another
object's ground), the new surface must not stand up to HEADROOM above it: a
hero would walk through it there.

Prints a summary; --report writes it as JSON with the worst spots (game
coordinates: the export negates x); --map writes a top-down picture of the
samples (green within the tolerance, red the new ground above the collision,
blue below, black a hole; beside it dark green, orange where the new stands in the
way; grey not sampled; brighter the further off).
"""
import json
import math
import struct
import sys
import zlib

import numpy as np

MAX_BYTES = 512 * 1024 * 1024
WINDOW = 1.5
STEP = 0.25
MIN_UP = 0.35
# How far above walkable ground a new surface stands in a hero's way.
HEADROOM = 2.0
CT = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
NC = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}


def parse_args():
    a = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else sys.argv[1:]
    if len(a) < 3:
        raise SystemExit(__doc__)
    level, label, new, rest = a[0], a[1], a[2], a[3:]
    opts = {'tolerance': 0.10, 'report': '', 'map': ''}
    while rest:
        key = rest.pop(0)
        if not key.startswith('--') or key[2:] not in opts or not rest:
            raise SystemExit(f'unknown or incomplete option {key}')
        value = rest.pop(0)
        opts[key[2:]] = type(opts[key[2:]])(value)
    return level, label, new, opts


def load(path):
    with open(path, 'rb') as f:
        b = f.read(MAX_BYTES + 1)
    if len(b) > MAX_BYTES:
        raise SystemExit(f'{path} is larger than the size policy')
    magic, _, length = struct.unpack_from('<III', b, 0)
    if magic != 0x46546C67:
        raise SystemExit(f'{path} is not glTF binary')
    off, js, bin_ = 12, None, b''
    while off < min(length, len(b)):
        cl, ct = struct.unpack_from('<II', b, off)
        chunk = b[off + 8:off + 8 + cl]
        off += 8 + cl
        if ct == 0x4E4F534A:
            js = json.loads(chunk)
        elif ct == 0x004E4942:
            bin_ = chunk
    if js is None:
        raise SystemExit(f'{path} has no JSON chunk')
    return js, bin_


def accessor(js, bin_, i):
    a = js['accessors'][i]
    v = js['bufferViews'][a['bufferView']]
    if v.get('buffer', 0) != 0:
        raise SystemExit('only a buffer inside the .glb is read')
    dt, n = CT[a['componentType']], NC[a['type']]
    start = v.get('byteOffset', 0) + a.get('byteOffset', 0)
    stride, size = v.get('byteStride', 0), np.dtype(dt).itemsize * n
    if stride and stride != size:
        raw = np.frombuffer(bin_, np.uint8, count=stride * a['count'], offset=start).reshape(a['count'], stride)[:, :size]
        arr = raw.copy().view(dt).reshape(a['count'], n)
    else:
        arr = np.frombuffer(bin_, dt, count=a['count'] * n, offset=start).reshape(a['count'], n)
    return arr.astype(np.float64)


def node_matrix(n):
    if 'matrix' in n:
        return np.array(n['matrix'], float).reshape(4, 4).T
    t = np.array(n.get('translation', [0, 0, 0]), float)
    x, y, z, w = n.get('rotation', [0, 0, 0, 1])
    s = np.array(n.get('scale', [1, 1, 1]), float)
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * s
    m[:3, 3] = t
    return m


def mesh_triangles(js, bin_, mesh_index, matrix):
    """A mesh's triangles (N x 3 x 3) under `matrix`."""
    out = []
    for p in js['meshes'][mesh_index]['primitives']:
        if p.get('mode', 4) != 4:
            continue
        pos = accessor(js, bin_, p['attributes']['POSITION'])
        pos = (matrix @ np.c_[pos, np.ones(len(pos))].T).T[:, :3]
        idx = accessor(js, bin_, p['indices']).astype(np.int64).ravel() if 'indices' in p else np.arange(len(pos))
        out.append(pos[idx[:len(idx) // 3 * 3].reshape(-1, 3)])
    return np.concatenate(out) if out else np.zeros((0, 3, 3))


def scene_nodes(js):
    """Every node with its world matrix and name."""
    out = []

    def walk(i, parent):
        n = js['nodes'][i]
        m = parent @ node_matrix(n)
        out.append((i, n, m))
        for c in n.get('children', []):
            walk(c, m)
    scene = js['scenes'][js.get('scene', 0)]
    for r in scene['nodes']:
        walk(r, np.eye(4))
    return out


def file_triangles(path):
    js, bin_ = load(path)
    tris = [mesh_triangles(js, bin_, n['mesh'], m) for _, n, m in scene_nodes(js) if 'mesh' in n]
    return np.concatenate(tris) if tris else np.zeros((0, 3, 3))


class Heights:
    """Vertical rays against triangles: the surface heights at (x, z)."""

    def __init__(self, tris, cell=1.0):
        self.tris, self.cell, self.grid = tris, cell, {}
        lo, hi = tris.min(1), tris.max(1)
        for k in range(len(tris)):
            for i in range(int(math.floor(lo[k, 0] / cell)), int(math.floor(hi[k, 0] / cell)) + 1):
                for j in range(int(math.floor(lo[k, 2] / cell)), int(math.floor(hi[k, 2] / cell)) + 1):
                    self.grid.setdefault((i, j), []).append(k)

    def at(self, x, z):
        ys = []
        for k in self.grid.get((int(math.floor(x / self.cell)), int(math.floor(z / self.cell))), ()):
            a, b, c = self.tris[k]
            d = (b[0] - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (b[2] - a[2])
            if abs(d) < 1e-12:
                continue
            wb = ((x - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (z - a[2])) / d
            wc = ((b[0] - a[0]) * (z - a[2]) - (x - a[0]) * (b[2] - a[2])) / d
            wa = 1.0 - wb - wc
            if min(wa, wb, wc) >= -1e-6:
                ys.append(a[1] * wa + b[1] * wb + c[1] * wc)
        return ys


def nearest(ys, y):
    best = None
    for v in ys:
        if abs(v - y) <= WINDOW and (best is None or abs(v - y) < abs(best - y)):
            best = v
    return best


def samples(tri):
    """The points of the level-wide grid (every STEP in x and z) inside a
    triangle seen from above, at the triangle's height there."""
    a, b, c = tri
    d = (b[0] - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (b[2] - a[2])
    if abs(d) < 1e-12:
        return []
    lo, hi = tri.min(0), tri.max(0)
    out = []
    for i in range(int(math.ceil(lo[0] / STEP)), int(math.floor(hi[0] / STEP)) + 1):
        for j in range(int(math.ceil(lo[2] / STEP)), int(math.floor(hi[2] / STEP)) + 1):
            x, z = i * STEP, j * STEP
            wb = ((x - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (z - a[2])) / d
            wc = ((b[0] - a[0]) * (z - a[2]) - (x - a[0]) * (b[2] - a[2])) / d
            wa = 1.0 - wb - wc
            if min(wa, wb, wc) >= -1e-9:
                out.append(np.array([x, a[1] * wa + b[1] * wb + c[1] * wc, z]))
    return out


def write_png(path, w, h, rows):
    raw = b''.join(b'\x00' + bytes(r) for r in rows)
    def chunk(t, d):
        return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)
    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b''))


def main():
    level, label, new_path, opts = parse_args()
    tol = opts['tolerance']
    js, bin_ = load(level)
    nodes = scene_nodes(js)
    places = [(n, m) for _, n, m in nodes if n.get('name') == label and 'mesh' in n]
    if not places:
        raise SystemExit(f'{label}: no node of that name with a mesh in {level}')
    collision = [mesh_triangles(js, bin_, n['mesh'], m) for _, n, m in nodes if n.get('name', '').startswith('COL_') and 'mesh' in n]
    collision = np.concatenate(collision) if collision else np.zeros((0, 3, 3))
    nrm = np.cross(collision[:, 1] - collision[:, 0], collision[:, 2] - collision[:, 0])
    up = np.abs(nrm[:, 1]) / np.maximum(1e-12, np.linalg.norm(nrm, axis=1))
    walkable = collision[MIN_UP <= up]
    new_local = file_triangles(new_path)
    if not len(new_local):
        raise SystemExit(f'{new_path}: no triangles')
    original, new = [], []
    for n, m in places:
        original.append(mesh_triangles(js, bin_, n['mesh'], m))
        new.append((m @ np.c_[new_local.reshape(-1, 3), np.ones(new_local.size // 3)].T).T[:, :3].reshape(-1, 3, 3))
    original, new = np.concatenate(original), np.concatenate(new)
    h_orig, h_new = Heights(original), Heights(new)
    # Where to look: the walkable collision under either surface.
    lo = np.minimum(original.min((0, 1)), new.min((0, 1)))
    hi = np.maximum(original.max((0, 1)), new.max((0, 1)))
    near = walkable[(walkable.max(1)[:, 0] >= lo[0] - 1) & (walkable.min(1)[:, 0] <= hi[0] + 1) & (walkable.max(1)[:, 2] >= lo[2] - 1) & (walkable.min(1)[:, 2] <= hi[2] + 1)]
    rows, intrusions, seen, elsewhere, beside = [], [], set(), 0, []
    for tri in near:
        for p in samples(tri):
            # Collision at another height there (a bridge, a ledge) samples
            # apart.
            key = (round(p[0] / STEP), round(p[2] / STEP), round(p[1] / WINDOW))
            if key in seen:
                continue
            seen.add(key)
            ys_new = h_new.at(p[0], p[2])
            yo = nearest(h_orig.at(p[0], p[2]), p[1])
            if yo is not None:
                yn = nearest(ys_new, p[1])
                rows.append((p[0], p[1], p[2], yo - p[1], None if yn is None else yn - p[1]))
                continue
            # Walkable ground this object did not draw (another object's):
            # the new surface must not stand in it, where a hero walks
            # through it.
            elsewhere += 1
            beside.append((p[0], p[2]))
            up = [y - p[1] for y in ys_new if tol < y - p[1] <= HEADROOM]
            if up:
                intrusions.append((p[0], p[1], p[2], max(up)))
    if not rows and not elsewhere:
        raise SystemExit(f'{label}: no walkable collision under or beside it')
    d_orig = np.array([r[3] for r in rows]) if rows else np.zeros(0)
    holes = [r for r in rows if r[4] is None]
    d_new = np.array([r[4] for r in rows if r[4] is not None])
    within = float(np.mean(np.abs(d_new) <= tol)) if len(d_new) else None
    worst = sorted((r for r in rows if r[4] is not None), key=lambda r: -abs(r[4]))[:10]
    game = lambda r: [round(float(-r[0]), 3), round(float(r[1]), 3), round(float(r[2]), 3)]
    stats = lambda d: {'mean_abs': float(np.mean(np.abs(d))), 'max_above': float(max(0.0, d.max())), 'max_below': float(max(0.0, -d.min())), 'within': float(np.mean(np.abs(d) <= tol))} if len(d) else None
    report = {
        'level': level, 'label': label, 'new': new_path, 'tolerance': tol, 'places': len(places), 'step': STEP,
        'samples': len(rows), 'original': stats(d_orig), 'new': stats(d_new), 'holes': len(holes),
        'worst': [{'game': game(r), 'off': round(float(r[4]), 3)} for r in worst],
        'holes_at': [game(r) for r in holes[:200]],
        'beside': elsewhere, 'intrusions': len(intrusions),
        'intrusions_at': [{'game': game(r), 'up': round(float(r[3]), 3)} for r in sorted(intrusions, key=lambda r: -r[3])[:200]],
    }
    print(f"{label}: tolerance {tol}, {len(places)} placed")
    if rows:
        o = report['original']
        print(f"  under it: {len(rows)} samples of walkable collision; the original mean {o['mean_abs']:.3f}, above by up to {o['max_above']:.3f}, below by up to {o['max_below']:.3f}, {o['within'] * 100:.1f} % within")
        if len(d_new):
            nw = report['new']
            print(f"  the new: mean {nw['mean_abs']:.3f}, above by up to {nw['max_above']:.3f}, below by up to {nw['max_below']:.3f}, {nw['within'] * 100:.1f} % within, {len(holes)} holes")
        else:
            print(f"  the new: no surface over any of them ({len(holes)} holes)")
        for w in report['worst'][:5]:
            print(f"  worst at game {w['game']}: {w['off']:+.3f}")
    else:
        print("  under it: no walkable collision (not ground heroes walk on)")
    print(f"  beside it: {elsewhere} samples of walkable collision it did not draw; the new stands in {len(intrusions)} of them (up to {HEADROOM} above)")
    for w in report['intrusions_at'][:5]:
        print(f"  standing in the way at game {w['game']}: {w['up']:+.3f}")
    if opts['report']:
        with open(opts['report'], 'w') as f:
            json.dump(report, f, indent=1)
    if opts['map']:
        cells = [(x, z, "beside") for x, z in beside] + [(r[0], r[2], "hole" if r[4] is None else r[4]) for r in rows] + [(r[0], r[2], "in") for r in intrusions]
        if not cells:
            return
        xs, zs = np.array([c[0] for c in cells]), np.array([c[1] for c in cells])
        x0, z0 = xs.min(), zs.min()
        w, h = int(round((xs.max() - x0) / STEP)) + 1, int(round((zs.max() - z0) / STEP)) + 1
        # PX pixels a sample; game x to the right (the export's x mirrored),
        # z down. Grey: not sampled.
        PX = 3
        img = [[40, 40, 40] * (w * PX) for _ in range(h * PX)]
        for x, z, v in cells:
            i, j = w - 1 - int(round((x - x0) / STEP)), int(round((z - z0) / STEP))
            if v == "beside":
                c = (30, 90, 30)
            elif v == "hole":
                c = (0, 0, 0)
            elif v == "in":
                c = (240, 150, 30)
            else:
                k = min(1.0, abs(v) / max(tol * 5, 1e-6))
                c = (60, 200, 60) if abs(v) <= tol else ((int(120 + 135 * k), 40, 40) if v > 0 else (40, 40, int(120 + 135 * k)))
            for y in range(j * PX, j * PX + PX):
                img[y][3 * i * PX:3 * (i + 1) * PX] = c * PX
        write_png(opts['map'], w * PX, h * PX, img)


if __name__ == '__main__':
    main()
