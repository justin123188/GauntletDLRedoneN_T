"""Import exported .glb files into empty Blender scenes, one at a time, and
report what Blender made of them (decisions/0021, the model exporter).

Read only: nothing is saved but the JSON report and, with --render, one
preview picture per file (Workbench, textured); for a level file, one of
the scenery alone, one of the collision alone and one of both.

    blender --background --factory-startup --python tools/blender/check_glb.py -- \
        REPORT.json [--render DIR] FILE.glb ...
"""
import json
import math
import os
import sys

import bpy
from mathutils import Vector

MAX_FILES = 200
MAX_BYTES = 256 * 1024 * 1024


def parse_args():
    a = sys.argv[sys.argv.index('--') + 1:]
    if not a:
        raise SystemExit('usage: ... -- REPORT.json [--render DIR] FILE.glb ...')
    report, rest, render = a[0], a[1:], None
    if rest[:1] == ['--render']:
        render, rest = rest[1], rest[2:]
    if not rest or len(rest) > MAX_FILES:
        raise SystemExit(f'give 1..{MAX_FILES} files')
    return report, render, rest


def empty_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def bounds(objects):
    lo = Vector((math.inf,) * 3)
    hi = Vector((-math.inf,) * 3)
    for o in objects:
        for c in o.bound_box:
            w = o.matrix_world @ Vector(c)
            lo = Vector(map(min, lo, w))
            hi = Vector(map(max, hi, w))
    return lo, hi


def render(objects, path):
    """A textured Workbench picture of `objects`, seen from above at an angle."""
    scene = bpy.context.scene
    lo, hi = bounds(objects)
    centre = (lo + hi) / 2
    radius = max((hi - lo).length / 2, 0.01)
    cam_data = bpy.data.cameras.new('check')
    cam_data.lens_unit = 'FOV'
    cam_data.angle = math.radians(40)
    cam_data.clip_end = radius * 20
    cam_data.clip_start = radius / 1000
    cam = bpy.data.objects.new('check', cam_data)
    scene.collection.objects.link(cam)
    direction = Vector((0.9, -1.2, 1.1)).normalized()
    cam.location = centre + direction * (radius / math.sin(cam_data.angle / 2) * 1.05)
    cam.rotation_euler = (centre - cam.location).to_track_quat('-Z', 'Y').to_euler()
    scene.camera = cam
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.display.shading.light = 'STUDIO'
    scene.display.shading.color_type = 'TEXTURE'
    scene.render.resolution_x, scene.render.resolution_y = 960, 640
    scene.render.film_transparent = False
    scene.render.image_settings.file_format = 'PNG'
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(cam)


def check(path, render_dir):
    out = {'file': os.path.basename(path)}
    if os.path.getsize(path) > MAX_BYTES:
        out['error'] = 'larger than the check policy'
        return out
    empty_scene()
    try:
        out['import'] = sorted(bpy.ops.import_scene.gltf(filepath=path))
    except Exception as e:  # the importer's own message
        out['error'] = str(e)
        return out
    objects = list(bpy.context.scene.objects)
    meshes = [o for o in objects if o.type == 'MESH']
    triangles = 0
    for o in meshes:
        o.data.calc_loop_triangles()
        triangles += len(o.data.loop_triangles)
    lo, hi = bounds(meshes) if meshes else (Vector(), Vector())
    tagged = [o for o in objects if 'gdl' in o.keys()]
    out.update({
        'objects': len(objects),
        'mesh_objects': len(meshes),
        'triangles': triangles,
        'materials': len(bpy.data.materials),
        'images': len(bpy.data.images),
        'images_with_pixels': sum(1 for i in bpy.data.images if i.size[0] > 0 and i.size[1] > 0),
        'uv_layers': sorted({len(o.data.uv_layers) for o in meshes}),
        'colour_attributes': sorted({a.name for o in meshes for a in o.data.color_attributes}),
        'bounds_blender_zup': [list(map(lambda v: round(v, 4), lo)), list(map(lambda v: round(v, 4), hi))],
        'tagged_objects': len(tagged),
        'first_tag': tagged[0]['gdl'].to_dict() if tagged else None,
        'layers': sorted(o.name for o in objects if o.name in ('SCENERY', 'COLLISION', 'WATER')),
    })
    # A level: where a world node is both drawn and collides, the two
    # objects' bounding-box centres (the same node, so they should agree).
    named = {o.name: o for o in meshes}
    gaps = []
    for o in meshes:
        layer, _, label = o.name.partition('_')
        if layer in ('COL', 'WATER') and label in named:
            a, b = bounds([o]), bounds([named[label]])
            gaps.append(((a[0] + a[1]) / 2 - (b[0] + b[1]) / 2).length)
    if gaps:
        gaps.sort()
        out['drawn_and_colliding'] = len(gaps)
        out['centre_gap'] = {'median': round(gaps[len(gaps) // 2], 4), 'p90': round(gaps[len(gaps) * 9 // 10], 4), 'max': round(gaps[-1], 4)}
    if render_dir and meshes:
        stem = os.path.splitext(os.path.basename(path))[0]
        layer = {o.name: o for o in objects if o.parent is None}
        scenery = [o for o in meshes if not o.name.startswith(('COL_', 'WATER_'))]
        collision = [o for o in meshes if o.name.startswith(('COL_', 'WATER_'))]
        if collision:
            for o in collision:
                o.hide_render = True
            render(scenery, os.path.join(render_dir, f'{stem}-scenery.png'))
            for o in collision:
                o.hide_render = False
            for o in scenery:
                o.hide_render = True
            render(collision, os.path.join(render_dir, f'{stem}-collision.png'))
            for o in scenery:
                o.hide_render = False
            render(meshes, os.path.join(render_dir, f'{stem}-both.png'))
        else:
            render(meshes, os.path.join(render_dir, f'{stem}.png'))
        del layer
    return out


def main():
    report, render_dir, files = parse_args()
    if render_dir:
        os.makedirs(render_dir, exist_ok=True)
    results = [check(f, render_dir) for f in files]
    doc = {'blender': bpy.app.version_string, 'files': results}
    with open(report, 'w', encoding='utf-8') as f:
        json.dump(doc, f, indent=1)
    print(f'check_glb: {len(results)} files, {sum(1 for r in results if "error" in r)} failed: {report}')


main()
