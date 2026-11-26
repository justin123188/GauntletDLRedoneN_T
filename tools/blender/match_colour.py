"""Give one colour group of a model pack file the colour of another file's
paint (models-plan M5): the owner's chest's wood in the owner's barrel's
wood colour, its brass kept.

    blender --background --factory-startup --python tools/blender/match_colour.py -- \
        NEW.glb REF.glb OUT.glb [--group darker|lighter|all] [--image NAME]...
        [--ref-group all|darker|lighter] [--match mean|spread] [--strength S]
        [--report FILE.json] [--preview DIR]

NEW.glb's base colour textures (those named by --image, else all) have
their coloured pixels (saturation and value high enough, as recolour.py
takes them: paint and wood, not iron, seams or the black between a baked
texture's islands) split into two colour groups (two means over value,
saturation and hue), and the chosen group (--group, default darker: a
chest's wood beside its brass) is moved in HSV so its mean hue, saturation
and value are REF.glb's coloured pixels' (for the chest: the barrel's wood;
its dark iron is not coloured), or those of one of REF's own two colour
groups (--ref-group lighter: the barrel's staves without the dark seams
between them). Each pixel keeps its own variation (grain,
dirt, painted light) and is moved by how coloured and how much in the group
it is, so the edges blend. With --match spread, the group's saturation and
value also take REF's spread (standard deviation) about the mean, so its
lights and darks are as far apart as REF's (the barrel's bright staves and
dark seams); --match mean (the default) scales them. --strength 0..1 goes
part of the way (the hue turn times S, the saturation and value moved S of
the way).

Every other byte of NEW.glb's data stays; textures beside REF.glb or
NEW.glb (share_data.py's textures/ folder) are read, OUT.glb embeds its
own. The same texture bytes give the same output bytes, so files sharing a
texture still share it after share_data.py.
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
    opts = {'group': 'darker', 'ref-group': 'all', 'match': 'mean', 'strength': '1', 'report': '', 'preview': ''}
    images = []
    pos = []
    while a:
        k = a.pop(0)
        if k == '--image' and a:
            images.append(a.pop(0))
        elif k.startswith('--'):
            if k[2:] not in opts or not a:
                raise SystemExit(f'unknown or incomplete option {k}')
            opts[k[2:]] = a.pop(0)
        else:
            pos.append(k)
    groups = ('darker', 'lighter', 'all')
    if len(pos) != 3 or opts['group'] not in groups or opts['ref-group'] not in groups or opts['match'] not in ('mean', 'spread'):
        raise SystemExit(__doc__)
    s = float(opts['strength'])
    if not 0.0 <= s <= 1.0:
        raise SystemExit('--strength is between 0 and 1')
    return pos, opts, images, s


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
    return doc, data


def beside(path, uri):
    """A file beside `path` named by a plain relative uri (share_data.py)."""
    parts = uri.split('/')
    if not parts or any(p in ('', '.', '..') or not all(c.isalnum() or c in '_.-' for c in p) for p in parts):
        raise SystemExit(f'{path}: only a plain relative path beside the file is read ({uri!r})')
    full = os.path.join(os.path.dirname(os.path.abspath(path)), *parts)
    if os.path.getsize(full) > MAX_BYTES:
        raise SystemExit(f'{full} is larger than the size policy')
    with open(full, 'rb') as f:
        return f.read()


def view_bytes(doc, data, i):
    v = doc['bufferViews'][i]
    start = v.get('byteOffset', 0)
    return data[start:start + v['byteLength']]


def png_of(path, doc, data, img):
    if img.get('mimeType', 'image/png') != 'image/png' and not img.get('uri', '').lower().endswith('.png'):
        raise SystemExit(f'{path}: an image that is not PNG')
    if 'bufferView' in img:
        return view_bytes(doc, data, img['bufferView'])
    if 'uri' in img:
        return beside(path, img['uri'])
    raise SystemExit(f'{path}: an image without data')


def write_glb(path, doc, data, replace):
    """NEW's data with the images in `replace` (image index -> PNG bytes)
    embedded, every view laid out again in order (4-byte aligned)."""
    if any('uri' in b for b in doc.get('buffers', [])):
        raise SystemExit('NEW.glb keeps its buffer beside it: run on the embedded file (before share_data.py)')
    views = doc['bufferViews']
    by_view = {doc['images'][i]['bufferView']: png for i, png in replace.items() if 'bufferView' in doc['images'][i]}
    out = bytearray()
    for i, v in enumerate(views):
        chunk = by_view.get(i, view_bytes(doc, data, i))
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


def pixels(png, tmp, name):
    """A PNG's pixels as an (h, w, 4) float array (Blender's order)."""
    path = os.path.join(tmp, name + '.png')
    with open(path, 'wb') as f:
        f.write(png)
    im = bpy.data.images.load(path)
    w, h = im.size
    px = np.empty(w * h * 4, np.float32)
    im.pixels.foreach_get(px)
    return px.reshape(h, w, 4)


def save_png(px, tmp, name):
    h, w = px.shape[:2]
    im = bpy.data.images.new(name, w, h, alpha=True)
    im.pixels.foreach_set(px.astype(np.float32).ravel())
    path = os.path.join(tmp, name + '.png')
    im.filepath_raw = path
    im.file_format = 'PNG'
    im.save()
    with open(path, 'rb') as f:
        return f.read()


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
    """How much a pixel is paint or wood (0..1), as recolour.py takes it."""
    s, v = hsv[..., 1], hsv[..., 2]
    return np.clip((s - 0.15) / 0.2, 0, 1) * np.clip((v - 0.06) / 0.1, 0, 1)


def features(hsv):
    """Value, saturation and the hue as a point on a circle (scaled by the
    saturation), for the two colour groups."""
    ang = hsv[..., 0] * 2 * math.pi
    return np.stack([hsv[..., 2], hsv[..., 1], 0.5 * hsv[..., 1] * np.cos(ang), 0.5 * hsv[..., 1] * np.sin(ang)], -1)


def two_groups(f, w):
    """Two weighted means over features f (n, 4), started from the darkest
    and the brightest tenth by value: (darker centre, lighter centre)."""
    keep = w > 0.05
    f, w = f[keep], w[keep]
    if len(f) < 16:
        raise SystemExit('too few coloured pixels to split')
    order = np.argsort(f[:, 0])
    k = max(1, len(f) // 10)
    c = np.stack([np.average(f[order[:k]], 0, w[order[:k]]), np.average(f[order[-k:]], 0, w[order[-k:]])])
    for _ in range(30):
        d = ((f[:, None, :] - c[None]) ** 2).sum(-1)
        near = d.argmin(1)
        new = np.stack([np.average(f[near == j], 0, w[near == j]) if (near == j).any() else c[j] for j in (0, 1)])
        if np.allclose(new, c, atol=1e-6):
            break
        c = new
    return c if c[0, 0] <= c[1, 0] else c[::-1]


def membership(f, centres, group):
    """How much each pixel is in the chosen group (0..1), soft at the
    border between the two groups."""
    if group == 'all':
        return np.ones(f.shape[:-1], np.float32)
    own, other = (centres[0], centres[1]) if group == 'darker' else (centres[1], centres[0])
    ds = ((f - own) ** 2).sum(-1)
    do = ((f - other) ** 2).sum(-1)
    m = do / np.maximum(ds + do, 1e-9)
    t = np.clip((m - 0.35) / 0.3, 0, 1)
    return t * t * (3 - 2 * t)


def stats(hsv, w):
    if w.sum() < 1:
        raise SystemExit('no coloured pixels to measure')
    ang = hsv[..., 0] * 2 * math.pi
    hue = math.atan2(float((np.sin(ang) * w * hsv[..., 1]).sum()), float((np.cos(ang) * w * hsv[..., 1]).sum())) / (2 * math.pi) % 1.0
    s = float((hsv[..., 1] * w).sum() / w.sum())
    v = float((hsv[..., 2] * w).sum() / w.sum())
    return {'hue': hue, 'saturation': s, 'value': v, 'share': float(w.mean()),
            'saturation_spread': float(np.sqrt(((hsv[..., 1] - s) ** 2 * w).sum() / w.sum())),
            'value_spread': float(np.sqrt(((hsv[..., 2] - v) ** 2 * w).sum() / w.sum()))}


def moved_channel(x, own_mean, own_spread, ref_mean, ref_spread, match, strength):
    """A saturation or value channel moved towards REF's: scaled by the
    means' ratio (mean), or by the spreads' ratio about the means (spread);
    `strength` of the way."""
    if match == 'mean':
        to = x * (ref_mean / max(own_mean, 1e-6))
    else:
        to = (x - own_mean) * (ref_spread / max(own_spread, 1e-6)) + ref_mean
    return np.clip(x + (to - x) * strength, 0, 1)


def main():
    (new, ref, out), opts, names, strength = parse_args()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    tmp = tempfile.mkdtemp(prefix='match-colour-')
    r_doc, r_data = read_glb(ref)
    r_hsv = [rgb_to_hsv(pixels(png_of(ref, r_doc, r_data, img), tmp, f'ref{i}')[..., :3]).reshape(-1, 3) for i, img in enumerate(r_doc.get('images', []))]
    if not r_hsv:
        raise SystemExit(f'{ref}: no image')
    rh = np.concatenate(r_hsv)
    rw = coloured(rh)
    rf = features(rh)
    target = stats(rh, rw * membership(rf, two_groups(rf, rw), opts['ref-group']))
    doc, data = read_glb(new)
    chosen = [i for i, img in enumerate(doc.get('images', [])) if not names or img.get('name') in names]
    if not chosen:
        raise SystemExit(f'{new}: no image named {names}')
    report = {'group': opts['group'], 'ref_group': opts['ref-group'], 'match': opts['match'], 'strength': strength, 'reference': target, 'images': []}
    replace = {}
    for i in chosen:
        px = pixels(png_of(new, doc, data, doc['images'][i]), tmp, f'new{i}')
        hsv = rgb_to_hsv(px[..., :3])
        w = coloured(hsv)
        f = features(hsv)
        centres = two_groups(f.reshape(-1, 4), w.ravel())
        m = membership(f, centres, opts['group'])
        own = stats(hsv, w * m)
        turn = ((target['hue'] - own['hue']) + 0.5) % 1.0 - 0.5
        sat = target['saturation'] / max(own['saturation'], 1e-6)
        val = target['value'] / max(own['value'], 1e-6)
        moved = hsv.copy()
        moved[..., 0] = (hsv[..., 0] + turn * strength) % 1.0
        m_ = opts['match']
        moved[..., 1] = moved_channel(hsv[..., 1], own['saturation'], own['saturation_spread'], target['saturation'], target['saturation_spread'], m_, strength)
        moved[..., 2] = moved_channel(hsv[..., 2], own['value'], own['value_spread'], target['value'], target['value_spread'], m_, strength)
        k = (w * m)[..., None]
        rgb = hsv_to_rgb(moved) * k + px[..., :3] * (1 - k)
        result = np.concatenate([rgb, px[..., 3:]], -1)
        replace[i] = save_png(result, tmp, f'out{i}')
        after = stats(rgb_to_hsv(rgb), w * m)
        report['images'].append({'image': i, 'name': doc['images'][i].get('name'), 'size': [px.shape[1], px.shape[0]],
                                 'groups_value': [round(float(centres[0, 0]), 4), round(float(centres[1, 0]), 4)],
                                 'group_share': round(float((w * m).mean()), 4), 'before': own, 'after': after,
                                 'hue_turn_degrees': round(turn * strength * 360, 2), 'saturation_scale': round(sat, 4),
                                 'value_scale': round(val, 4), 'png_bytes': len(replace[i])})
        if opts['preview']:
            os.makedirs(opts['preview'], exist_ok=True)
            side = np.concatenate([px, result], 1)
            mask = np.concatenate([np.repeat(m[..., None], 3, -1), np.ones_like(m)[..., None]], -1)
            for tag, im in (('texture', side), ('group', mask)):
                s = bpy.data.images.new(f'{tag}{i}', im.shape[1], im.shape[0], alpha=True)
                s.pixels.foreach_set(im.astype(np.float32).ravel())
                s.filepath_raw = os.path.join(opts['preview'], f'{os.path.splitext(os.path.basename(out))[0]}-{tag}{i}.png')
                s.file_format = 'PNG'
                s.save()
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    write_glb(out, doc, data, replace)
    report['out'] = os.path.basename(out)
    report['out_bytes'] = os.path.getsize(out)
    if opts['report']:
        with open(opts['report'], 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=1)
    print('match_colour:', json.dumps({'target': {k: round(v, 4) for k, v in target.items()},
                                       'images': [{k: x[k] for k in ('name', 'group_share', 'hue_turn_degrees', 'saturation_scale', 'value_scale')} for x in report['images']]}))


main()
