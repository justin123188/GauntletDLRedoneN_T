"""Fit a new model (from an image-to-3D tool, or made by hand) to an
original object of the model export, and write it as a model pack file
(decisions/0022, models-plan M5).

    blender --background --factory-startup --python tools/blender/fit_model.py -- \
        ORIGINAL.glb NEW.glb OUT.glb [--triangles N] [--texture PX] [--yaw DEG] \
        [--fit height|box] [--bake auto|on] [--report FILE.json] [--preview DIR]

ORIGINAL.glb is the object's file from `gdl_play --export-models`
(objects/<package>/<LABEL>.glb); OUT.glb goes to
model_pack/<package>/<LABEL>.glb beside the game. The steps:
- every mesh of NEW joined into one object, its transforms applied;
- turned by --yaw degrees about the up axis (default 0);
- scaled uniformly so its height matches the original's (--fit height,
  the default) or so it fits inside the original's box (--fit box);
- centred over the original's centre and stood on the original's base;
- with more than --triangles triangles (default 3000), or with --bake on:
  a copy welded at its seams, reduced to that many triangles, given new
  UVs, and the full model's base colour baked onto it (Cycles, the GPU
  when it can) in a square --texture image;
- every texture but the base colour removed, the base colour at most
  --texture pixels on its longer side (default 512) and written as PNG;
- exported as glTF binary with positions, normals, one UV set and the
  base colour texture (no Draco, no vertex colours).
With --preview, pictures of the original and the fitted model from the
same camera.
"""
import json
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

MAX_BYTES = 512 * 1024 * 1024


def parse_args():
    a = sys.argv[sys.argv.index('--') + 1:]
    if len(a) < 3:
        raise SystemExit(__doc__)
    opts = {'triangles': 3000, 'texture': 512, 'yaw': 0.0, 'fit': 'height', 'bake': 'auto', 'report': '', 'preview': ''}
    original, new, out, rest = a[0], a[1], a[2], a[3:]
    while rest:
        key = rest.pop(0)
        if not key.startswith('--') or key[2:] not in opts or not rest:
            raise SystemExit(f'unknown or incomplete option {key}')
        value = rest.pop(0)
        opts[key[2:]] = type(opts[key[2:]])(value)
    if opts['fit'] not in ('height', 'box'):
        raise SystemExit('--fit is height or box')
    if opts['bake'] not in ('auto', 'on'):
        raise SystemExit('--bake is auto or on')
    for f in (original, new):
        if os.path.getsize(f) > MAX_BYTES:
            raise SystemExit(f'{f} is larger than the size policy')
    return original, new, out, opts


def empty_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def meshes():
    return [o for o in bpy.context.scene.objects if o.type == 'MESH']


def bounds(objects):
    lo = Vector((math.inf,) * 3)
    hi = Vector((-math.inf,) * 3)
    for o in objects:
        for c in o.bound_box:
            w = o.matrix_world @ Vector(c)
            lo = Vector(map(min, lo, w))
            hi = Vector(map(max, hi, w))
    return lo, hi


def triangles(objects):
    n = 0
    for o in objects:
        o.data.calc_loop_triangles()
        n += len(o.data.loop_triangles)
    return n


def render(path, centre, radius):
    scene = bpy.context.scene
    cam_data = bpy.data.cameras.new('fit')
    cam_data.lens_unit = 'FOV'
    cam_data.angle = math.radians(40)
    cam_data.clip_start, cam_data.clip_end = radius / 1000, radius * 20
    cam = bpy.data.objects.new('fit', cam_data)
    scene.collection.objects.link(cam)
    direction = Vector((0.9, -1.2, 0.8)).normalized()
    cam.location = centre + direction * (radius / math.sin(cam_data.angle / 2) * 1.05)
    cam.rotation_euler = (centre - cam.location).to_track_quat('-Z', 'Y').to_euler()
    scene.camera = cam
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.display.shading.light = 'STUDIO'
    scene.display.shading.color_type = 'TEXTURE'
    scene.render.resolution_x, scene.render.resolution_y = 640, 640
    scene.render.image_settings.file_format = 'PNG'
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam)


def base_colour_only(material):
    """Remove every image node that does not feed the base colour."""
    if not material or not material.use_nodes:
        return
    tree = material.node_tree
    bsdf = next((n for n in tree.nodes if n.type == 'BSDF_PRINCIPLED'), None)
    keep = set()
    if bsdf:
        stack = [link.from_node for link in bsdf.inputs['Base Color'].links]
        while stack:
            n = stack.pop()
            keep.add(n)
            stack.extend(link.from_node for i in n.inputs for link in i.links)
    for n in list(tree.nodes):
        if n.type == 'TEX_IMAGE' and n not in keep:
            tree.nodes.remove(n)


def gpu_cycles(scene):
    """Cycles on the GPU when one is usable (OptiX, then CUDA), else the CPU."""
    scene.render.engine = 'CYCLES'
    try:
        prefs = bpy.context.preferences.addons['cycles'].preferences
        for kind in ('OPTIX', 'CUDA'):
            prefs.compute_device_type = kind
            prefs.get_devices()
            devices = [d for d in prefs.devices if d.type == kind]
            if devices:
                for d in prefs.devices:
                    d.use = d.type == kind
                scene.cycles.device = 'GPU'
                return kind
    except Exception:
        pass
    scene.cycles.device = 'CPU'
    return 'CPU'


def select_only(objects, active):
    for o in bpy.context.scene.objects:
        o.select_set(o in objects)
    bpy.context.view_layer.objects.active = active


def baked_copy(high, opts, report):
    """A reduced copy of `high` (welded at its seams, at most
    opts['triangles'] triangles, smart-projected UVs) with the base colour
    baked from `high` into a new square image; `high` is removed."""
    scene = bpy.context.scene
    lo, hi = bounds([high])
    extent = max((hi - lo).length, 1e-6)
    low = high.copy()
    low.data = high.data.copy()
    scene.collection.objects.link(low)
    select_only([low], low)
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.remove_doubles(threshold=extent * 1e-5)
    bpy.ops.object.mode_set(mode='OBJECT')
    have = triangles([low])
    report['welded_triangles'] = have
    if have > opts['triangles']:
        mod = low.modifiers.new('fit', 'DECIMATE')
        mod.ratio = opts['triangles'] / have
        bpy.ops.object.modifier_apply(modifier=mod.name)
    me = low.data
    while me.uv_layers:
        me.uv_layers.remove(me.uv_layers[0])
    me.uv_layers.new(name='UVMap')
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.uv.smart_project(angle_limit=math.radians(66), island_margin=0.004)
    bpy.ops.object.mode_set(mode='OBJECT')
    try:
        bpy.ops.object.shade_smooth_by_angle(angle=math.radians(40))
    except Exception:
        bpy.ops.object.shade_smooth()
    size = opts['texture']
    img = bpy.data.images.new('baked', size, size, alpha=False)
    mat = bpy.data.materials.new('baked')
    mat.use_nodes = True
    tex = mat.node_tree.nodes.new('ShaderNodeTexImage')
    tex.image = img
    mat.node_tree.nodes.active = tex
    me.materials.clear()
    me.materials.append(mat)
    report['bake_device'] = gpu_cycles(scene)
    scene.cycles.samples = 4
    select_only([high, low], low)
    bpy.ops.object.bake(
        type='DIFFUSE',
        pass_filter={'COLOR'},
        use_selected_to_active=True,
        cage_extrusion=extent * 0.01,
        max_ray_distance=extent * 0.05,
        margin=max(4, size // 128),
    )
    bsdf = mat.node_tree.nodes['Principled BSDF']
    mat.node_tree.links.new(tex.outputs['Color'], bsdf.inputs['Base Color'])
    bpy.data.objects.remove(high)
    report['baked'] = [size, size]
    return low


def main():
    original, new, out, opts = parse_args()
    report = {'original': os.path.basename(original), 'new': os.path.basename(new), 'options': opts}
    # The original's box (Blender space: z up).
    empty_scene()
    bpy.ops.import_scene.gltf(filepath=original)
    lo, hi = bounds(meshes())
    report['original_bounds'] = [list(lo), list(hi)]
    report['original_triangles'] = triangles(meshes())
    if opts['preview']:
        os.makedirs(opts['preview'], exist_ok=True)
        render(os.path.join(opts['preview'], 'original.png'), (lo + hi) / 2, max((hi - lo).length / 2, 0.01))
    # The new model as one object.
    empty_scene()
    bpy.ops.import_scene.gltf(filepath=new)
    parts = meshes()
    if not parts:
        raise SystemExit(f'{new} has no mesh')
    for o in bpy.context.scene.objects:
        o.select_set(False)
    for o in parts:
        world = o.matrix_world.copy()
        o.parent = None
        o.matrix_world = world
        o.select_set(True)
    for o in [o for o in bpy.context.scene.objects if o.type != 'MESH']:
        bpy.data.objects.remove(o)
    bpy.context.view_layer.objects.active = parts[0]
    if len(parts) > 1:
        bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    report['new_triangles'] = triangles([obj])
    nlo, nhi = bounds([obj])
    report['new_bounds'] = [list(nlo), list(nhi)]
    # Turn, scale, place.
    if opts['yaw']:
        obj.data.transform(Matrix.Rotation(math.radians(opts['yaw']), 4, 'Z'))
        obj.data.update()
        nlo, nhi = bounds([obj])
    size, nsize = hi - lo, nhi - nlo
    if opts['fit'] == 'height' and nsize.z > 0:
        k = size.z / nsize.z
    else:
        k = min(s / n for s, n in zip(size, nsize) if n > 0)
    centre = Vector(((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, lo.z))
    ncentre = Vector(((nlo.x + nhi.x) / 2, (nlo.y + nhi.y) / 2, nlo.z))
    obj.data.transform(Matrix.Translation(centre) @ Matrix.Scale(k, 4) @ Matrix.Translation(-ncentre))
    obj.data.update()
    report['scale'] = k
    # Fewer triangles: a reduced copy with new UVs, its base colour baked
    # from the full model (reducing an image-to-3D model's own UV atlas
    # tears it: thousands of islands, split at every seam).
    have = triangles([obj])
    if have > opts['triangles'] or opts['bake'] == 'on':
        obj = baked_copy(obj, opts, report)
    report['triangles'] = triangles([obj])
    flo, fhi = bounds([obj])
    report['bounds'] = [list(flo), list(fhi)]
    # The base colour texture only, small and PNG.
    for slot in obj.material_slots:
        base_colour_only(slot.material)
    # Each image is copied into a new generated one: the exporter writes an
    # imported JPEG as JPEG even after scaling, and a generated image as PNG.
    images = []
    for img in [i for i in bpy.data.images if i.users and i.size[0] > 0]:
        w, h = img.size
        longest = max(w, h)
        if longest > opts['texture']:
            img.scale(max(1, round(w * opts['texture'] / longest)), max(1, round(h * opts['texture'] / longest)))
        png = bpy.data.images.new(img.name + '_png', img.size[0], img.size[1], alpha=True)
        png.pixels = img.pixels[:]
        png.file_format = 'PNG'
        img.user_remap(png)
        images.append({'name': img.name, 'from': [w, h], 'to': list(png.size)})
        bpy.data.images.remove(img)
    report['textures'] = images
    # Export only what the loader reads (options this Blender has).
    props = bpy.ops.export_scene.gltf.get_rna_type().properties.keys()
    wanted = {
        'filepath': out,
        'export_format': 'GLB',
        'use_selection': True,
        'export_apply': True,
        'export_yup': True,
        'export_texcoords': True,
        'export_normals': True,
        'export_tangents': False,
        'export_materials': 'EXPORT',
        'export_image_format': 'AUTO',
        'export_draco_mesh_compression_enable': False,
        'export_vertex_color': 'NONE',
        'export_colors': False,
        'export_animations': False,
        'export_skins': False,
        'export_morph': False,
        'export_cameras': False,
        'export_lights': False,
        'export_extras': False,
    }
    kwargs = {k: v for k, v in wanted.items() if k in props}
    report['export_options'] = sorted(kwargs)
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    for o in bpy.context.scene.objects:
        o.select_set(o == obj)
    bpy.ops.export_scene.gltf(**kwargs)
    report['out'] = out
    report['out_bytes'] = os.path.getsize(out)
    if opts['preview']:
        render(os.path.join(opts['preview'], 'fitted.png'), (lo + hi) / 2, max((hi - lo).length / 2, 0.01))
    text = json.dumps(report, indent=1, default=list)
    if opts['report']:
        with open(opts['report'], 'w', encoding='utf-8') as f:
            f.write(text)
    print('fit_model:', json.dumps({k: report[k] for k in ('scale', 'new_triangles', 'triangles', 'out_bytes')}))


main()
