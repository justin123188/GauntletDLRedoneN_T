"""Recolour a model pack file the way one original differs from another
(decisions/0024, models-plan M5): G1's red and poison barrels are the brown
barrel painted red and green, so the owner's barrel painted the same way
stands in for them.

    blender --background --factory-startup --python tools/blender/recolour.py -- \
        NEW.glb FROM.glb TO.glb OUT.glb [--report FILE.json] [--preview DIR]

NEW.glb is a model pack file (from fit_model.py); FROM.glb and TO.glb are
two originals of the model export with one shape and different paint (for
G1: objects/ITEMS_G/BAROBJIDLE15F01.glb and BAREXPIDLE18F01.glb). OUT.glb
is NEW.glb with only its textures changed: every other byte of the file's
data stays (the geometry is not re-exported).

The change, measured and applied in HSV on the textures' coloured pixels
(saturation and value high enough: paint and wood, not iron, seams or the
black between a baked texture's islands), each pixel weighted by how
coloured it is so the edges blend:
- the hue turned so the mean hue (circular, weighted by saturation) is
  TO's: the paint's own colour;
- the saturation and the value scaled by the ratio of FROM's and TO's
  means, so the new model keeps its own brightness against its original.
With --preview, the new texture beside the old one.
"""
import json
import math
import os
import struct
import sys
import tempfile

import bpy
import numpy as np

MAX_BYTES = 512 * 1024 * 1024


def parse_args():
    a = sys.argv[sys.argv.index('--') + 1:]
    opts = {'report': '', 'preview': ''}
    pos = []
    while a:
        k = a.pop(0)
        if k.startswith('--'):
            if k[2:] not in opts or not a:
                raise SystemExit(f'unknown or incomplete option {k}')
            opts[k[2:]] = a.pop(0)
        else:
            pos.append(k)
    if len(pos) != 4:
        raise SystemExit(__doc__)
    return pos, opts


def read_glb(path):
    if os.path.getsize(path) > MAX_BYTES:
        raise SystemExit(f'{path} is larger than the size policy')
    with open(path, 'rb') as f:
        b = f.read()
    magic, version, length = struct.unpack_from('<III', b, 0)
    if magic != 0x46546C67 or version != 2:
        raise SystemExit(f'{path}: not a glTF 2.0 binary')
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
    if any('uri' in buf for buf in doc.get('buffers', [])):
        raise SystemExit(f'{path}: a buffer outside the file')
    return doc, data


def view_bytes(doc, data, i):
    v = doc['bufferViews'][i]
    start = v.get('byteOffset', 0)
    return data[start:start + v['byteLength']]


def write_glb(path, doc, data, replace):
    """`data` with the views in `replace` (index -> bytes) swapped, every
    view laid out again in order (4-byte aligned)."""
    views = doc['bufferViews']
    out = bytearray()
    for i, v in enumerate(views):
        chunk = replace.get(i, view_bytes(doc, data, i))
        while len(out) % 4:
            out.append(0)
        v['byteOffset'] = len(out)
        v['byteLength'] = len(chunk)
        out.extend(chunk)
    while len(out) % 4:
        out.append(0)
    doc['buffers'][0]['byteLength'] = len(out)
    js = json.dumps(doc, separators=(',', ':')).encode()
    js += b' ' * (-len(js) % 4)
    total = 12 + 8 + len(js) + 8 + len(out)
    with open(path, 'wb') as f:
        f.write(struct.pack('<III', 0x46546C67, 2, total))
        f.write(struct.pack('<II', len(js), 0x4E4F534A))
        f.write(js)
        f.write(struct.pack('<II', len(out), 0x004E4942))
        f.write(bytes(out))


def png_pixels(png, tmp, name):
    """A PNG's pixels as an (h, w, 4) float array (Blender's order: bottom row first)."""
    path = os.path.join(tmp, name + '.png')
    with open(path, 'wb') as f:
        f.write(png)
    im = bpy.data.images.load(path)
    w, h = im.size
    px = np.empty(w * h * 4, np.float32)
    im.pixels.foreach_get(px)
    return px.reshape(h, w, 4), im


def images_of(doc, data, tmp, tag):
    out = []
    for i, img in enumerate(doc.get('images', [])):
        if img.get('mimeType') != 'image/png' or 'bufferView' not in img:
            raise SystemExit(f'{tag}: image {i} is not an embedded PNG')
        px, _ = png_pixels(view_bytes(doc, data, img['bufferView']), tmp, f'{tag}{i}')
        out.append((i, img['bufferView'], px))
    return out


def rgb_to_hsv(rgb):
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    mx, mn = rgb.max(-1), rgb.min(-1)
    d = mx - mn
    h = np.zeros_like(mx)
    nz = d > 1e-6
    rm = nz & (mx == r)
    gm = nz & (mx == g) & ~rm
    bm = nz & ~rm & ~gm
    h[rm] = ((g - b)[rm] / d[rm]) % 6
    h[gm] = (b - r)[gm] / d[gm] + 2
    h[bm] = (r - g)[bm] / d[bm] + 4
    h = h / 6.0
    s = np.where(mx > 1e-6, d / np.maximum(mx, 1e-6), 0)
    return np.stack([h, s, mx], -1)


def hsv_to_rgb(hsv):
    h, s, v = hsv[..., 0] % 1.0, hsv[..., 1], hsv[..., 2]
    i = np.floor(h * 6).astype(int) % 6
    f = h * 6 - np.floor(h * 6)
    p, q, t = v * (1 - s), v * (1 - f * s), v * (1 - (1 - f) * s)
    choices = [(v, t, p), (q, v, p), (p, v, t), (p, q, v), (t, p, v), (v, p, q)]
    out = np.zeros(h.shape + (3,), np.float32)
    for k, (a, b, c) in enumerate(choices):
        m = i == k
        out[m] = np.stack([a[m], b[m], c[m]], -1)
    return out


def coloured(hsv):
    """How much a pixel is paint or wood (0..1): saturated and not dark."""
    s, v = hsv[..., 1], hsv[..., 2]
    return np.clip((s - 0.15) / 0.2, 0, 1) * np.clip((v - 0.06) / 0.1, 0, 1)


def stats(images):
    """Mean hue (circular), saturation and value of the coloured pixels."""
    hs = np.concatenate([rgb_to_hsv(px[..., :3]).reshape(-1, 3) for _, _, px in images])
    w = coloured(hs)
    if w.sum() < 1:
        raise SystemExit('no coloured pixels to measure')
    ang = hs[:, 0] * 2 * math.pi
    hue = math.atan2(float((np.sin(ang) * w * hs[:, 1]).sum()), float((np.cos(ang) * w * hs[:, 1]).sum())) / (2 * math.pi) % 1.0
    return {'hue': hue, 'saturation': float((hs[:, 1] * w).sum() / w.sum()), 'value': float((hs[:, 2] * w).sum() / w.sum()),
            'coloured_share': float(w.mean())}


def main():
    (new, src, dst, out), opts = parse_args()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    tmp = tempfile.mkdtemp(prefix='recolour-')
    n_doc, n_data = read_glb(new)
    a = stats(images_of(*read_glb(src), tmp, 'from'))
    b = stats(images_of(*read_glb(dst), tmp, 'to'))
    c = stats(images_of(n_doc, n_data, tmp, 'own'))
    # The hue to TO's own (the paint's colour), the saturation and value by
    # the originals' ratios (the new model keeps its own brightness).
    turn = (b['hue'] - c['hue'] + 0.5) % 1.0 - 0.5
    sat = b['saturation'] / max(a['saturation'], 1e-6)
    val = b['value'] / max(a['value'], 1e-6)
    report = {'from': a, 'to': b, 'new': c, 'hue_turn_degrees': round(turn * 360, 2), 'saturation_scale': round(sat, 4),
              'value_scale': round(val, 4), 'images': []}
    replace = {}
    for i, view, px in images_of(n_doc, n_data, tmp, 'new'):
        hsv = rgb_to_hsv(px[..., :3])
        w = coloured(hsv)
        moved = hsv.copy()
        moved[..., 0] = (hsv[..., 0] + turn) % 1.0
        moved[..., 1] = np.clip(hsv[..., 1] * sat, 0, 1)
        moved[..., 2] = np.clip(hsv[..., 2] * val, 0, 1)
        rgb = hsv_to_rgb(moved) * w[..., None] + px[..., :3] * (1 - w[..., None])
        h, wd = px.shape[:2]
        im = bpy.data.images.new(f'recoloured{i}', wd, h, alpha=True)
        im.pixels.foreach_set(np.concatenate([rgb, px[..., 3:]], -1).astype(np.float32).ravel())
        path = os.path.join(tmp, f'out{i}.png')
        im.filepath_raw = path
        im.file_format = 'PNG'
        im.save()
        with open(path, 'rb') as f:
            replace[view] = f.read()
        report['images'].append({'image': i, 'size': [wd, h], 'coloured_share': round(float(w.mean()), 4),
                                 'png_bytes': len(replace[view])})
        if opts['preview']:
            os.makedirs(opts['preview'], exist_ok=True)
            side = np.concatenate([px, np.concatenate([rgb, px[..., 3:]], -1)], 1)
            s = bpy.data.images.new(f'preview{i}', side.shape[1], side.shape[0], alpha=True)
            s.pixels.foreach_set(side.astype(np.float32).ravel())
            s.filepath_raw = os.path.join(opts['preview'], f'texture{i}.png')
            s.file_format = 'PNG'
            s.save()
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    write_glb(out, n_doc, n_data, replace)
    report['out'] = out
    report['out_bytes'] = os.path.getsize(out)
    text = json.dumps(report, indent=1)
    if opts['report']:
        with open(opts['report'], 'w', encoding='utf-8') as f:
            f.write(text)
    print('recolour:', json.dumps({k: report[k] for k in ('hue_turn_degrees', 'saturation_scale', 'value_scale', 'out_bytes')}))


main()
