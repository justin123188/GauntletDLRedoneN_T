"""Compare a replacement's colour and contrast with its original's, as the
textures lay them on the surface (models-plan M6, decisions/0037).

    python check_colour.py ORIG.glb NEW.glb [--tolerance 0.10] [--contrast 0.80]
    python check_colour.py ORIG_DIR NEW_DIR [...]   (every NEW_DIR/*.glb with
                                                     an ORIG_DIR file of its name)

Blender's own Python (numpy) runs it, or blender --background
--factory-startup --python check_colour.py -- ORIG NEW ...

ORIG is the object's model export (objects/<package>/<LABEL>.glb from
`gdl_play --export-models`, its textures embedded); NEW the replacement as
the model pack takes it (textures embedded or beside it). A replaced piece
of scenery keeps the original's light (decisions/0035, 0037), so the
colour its textures lay on the surface sets its colour in the game.

What it measures, per piece: each triangle's texture sampled at random
points, weighted by area (the base colour texture times its factor, the
UVs repeating), from box-filtered mips at fixed sizes in game units, so
that a 128 px original and a 2048 px replacement are seen alike (each
primitive's texel size from its UVs and its texture's width;
decisions/0048): 0.5 units (close up) for the mean colour per material and
in all, in texels 0..255; and the relative contrast (the luma's standard
deviation over its mean) at 0.5 units and at 2 units (the pattern that
survives at the game's camera distance). Marks a channel of the mean off
by more than the tolerance (new / original), and a contrast at the camera
below that share of the original's.
"""
import json
import os
import struct
import sys
import zlib

import numpy as np

CT = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
NC = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
SAMPLES = 6
# The contrast's sizes in game units: close up, and at the camera.
CLOSE, CAMERA = 0.5, 2.0


def load(path):
    b = open(path, "rb").read()
    magic, _, length = struct.unpack_from("<III", b, 0)
    if magic != 0x46546C67:
        raise ValueError("%s: not glTF binary" % path)
    off, js, bin_ = 12, None, None
    while off < length:
        cl, ct = struct.unpack_from("<II", b, off)
        off += 8
        chunk = b[off:off + cl]
        off += cl
        if ct == 0x4E4F534A:
            js = json.loads(chunk)
        elif ct == 0x004E4942:
            bin_ = chunk
    return js, bin_


def accessor(js, bin_, i):
    a = js["accessors"][i]
    v = js["bufferViews"][a["bufferView"]]
    dt, n = CT[a["componentType"]], NC[a["type"]]
    start = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    size = np.dtype(dt).itemsize * n
    stride = v.get("byteStride", 0)
    if stride and stride != size:
        raw = np.frombuffer(bin_, np.uint8, count=stride * a["count"], offset=start).reshape(a["count"], stride)[:, :size]
        arr = raw.copy().view(dt).reshape(a["count"], n)
    else:
        arr = np.frombuffer(bin_, dt, count=a["count"] * n, offset=start).reshape(a["count"], n)
    out = arr.astype(np.float64)
    if a.get("normalized") and dt != np.float32:
        out /= np.iinfo(dt).max
    return out


def node_matrix(n):
    if "matrix" in n:
        return np.array(n["matrix"], float).reshape(4, 4).T
    t = np.array(n.get("translation", [0, 0, 0]), float)
    x, y, z, w = n.get("rotation", [0, 0, 0, 1])
    s = np.array(n.get("scale", [1, 1, 1]), float)
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * s
    m[:3, 3] = t
    return m


def primitives(js, bin_):
    out = []

    def walk(ni, parent):
        n = js["nodes"][ni]
        m = parent @ node_matrix(n)
        if "mesh" in n:
            for p in js["meshes"][n["mesh"]]["primitives"]:
                at = p["attributes"]
                if p.get("mode", 4) != 4 or "TEXCOORD_0" not in at:
                    continue
                pos = accessor(js, bin_, at["POSITION"])
                pos = (m @ np.c_[pos, np.ones(len(pos))].T).T[:, :3]
                idx = accessor(js, bin_, p["indices"]).ravel().astype(np.int64) if "indices" in p else np.arange(len(pos))
                out.append((pos, accessor(js, bin_, at["TEXCOORD_0"]), idx, p.get("material")))
        for c in n.get("children", []):
            walk(c, m)

    for r in js["scenes"][js.get("scene", 0)]["nodes"]:
        walk(r, np.eye(4))
    return out


def png_decode(b):
    if b[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    off, idat, pal = 8, [], None
    while off < len(b):
        n, t = struct.unpack(">I4s", b[off:off + 8])
        d = b[off + 8:off + 8 + n]
        off += 12 + n
        if t == b"IHDR":
            w, h, depth, ct, _, _, il = struct.unpack(">IIBBBBB", d)
        elif t == b"PLTE":
            pal = np.frombuffer(d, np.uint8).reshape(-1, 3)
        elif t == b"IDAT":
            idat.append(d)
        elif t == b"IEND":
            break
    if depth != 8 or il:
        raise ValueError("only 8-bit, non-interlaced PNGs")
    ch = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[ct]
    raw = np.frombuffer(zlib.decompress(b"".join(idat)), np.uint8).reshape(h, 1 + w * ch)
    out = np.zeros((h, w * ch), np.int32)
    prev = np.zeros(w * ch, np.int32)
    for y in range(h):
        f, line = raw[y, 0], raw[y, 1:].astype(np.int32)
        if f == 0:
            cur = line
        elif f == 2:
            cur = (line + prev) & 255
        elif f == 1:
            cur = (np.cumsum(line.reshape(w, ch), 0) & 255).ravel()
        else:
            cur = np.zeros_like(line)
            for x in range(w * ch):
                a = cur[x - ch] if x >= ch else 0
                up, c = prev[x], (prev[x - ch] if x >= ch else 0)
                if f == 3:
                    p = (a + up) >> 1
                else:
                    pa, pb, pc = abs(up - c), abs(a - c), abs(a + up - 2 * c)
                    p = a if pa <= pb and pa <= pc else (up if pb <= pc else c)
                cur[x] = (line[x] + p) & 255
        out[y] = cur
        prev = cur
    img = out.reshape(h, w, ch)
    if ct == 3:
        img = pal[img[:, :, 0]].astype(np.int32)
    elif ct in (0, 4):
        img = np.repeat(img[:, :, :1], 3, 2)
    return img[:, :, :3].astype(np.float64)


def mip(img, f):
    f = max(1, min(f, img.shape[0], img.shape[1]))
    h, w = img.shape[0] // f * f, img.shape[1] // f * f
    return img[:h, :w].reshape(h // f, f, w // f, f, 3).mean((1, 3))


_images = {}


def texture(path, js, bin_, mat):
    """The material's base colour texture (None without one), its factor
    and its name."""
    if mat is None:
        return None, np.ones(3), "-"
    m = js["materials"][mat]
    pbr = m.get("pbrMetallicRoughness", {})
    fac = np.array(pbr.get("baseColorFactor", [1, 1, 1, 1])[:3], float)
    name = m.get("name", str(mat))
    t = pbr.get("baseColorTexture")
    if t is None:
        return None, fac, name
    src = js["textures"][t["index"]]["source"]
    im = js["images"][src]
    if "uri" in im:
        key = os.path.normpath(os.path.join(os.path.dirname(path), im["uri"]))
        if key not in _images:
            _images[key] = png_decode(open(key, "rb").read())
    else:
        key = (path, src)
        if key not in _images:
            v = js["bufferViews"][im["bufferView"]]
            _images[key] = png_decode(bin_[v.get("byteOffset", 0):v.get("byteOffset", 0) + v["byteLength"]])
    return _images[key], fac, name


def luma(c):
    return 0.299 * c[..., 0] + 0.587 * c[..., 1] + 0.114 * c[..., 2]


def measure(path, seed=1):
    """Per material: (share of the area, mean colour); in all: the mean
    colour and the relative contrast at CLOSE and CAMERA game units."""
    js, bin_ = load(path)
    rng = np.random.default_rng(seed)
    per, far = {}, {CLOSE: [], CAMERA: []}
    for pos, uv, idx, mat in primitives(js, bin_):
        img, fac, name = texture(path, js, bin_, mat)
        tri = idx.reshape(-1, 3)
        P = pos[tri]
        area = 0.5 * np.linalg.norm(np.cross(P[:, 1] - P[:, 0], P[:, 2] - P[:, 0]), axis=1)
        U = uv[tri]
        uv_area = 0.5 * np.abs((U[:, 1, 0] - U[:, 0, 0]) * (U[:, 2, 1] - U[:, 0, 1]) - (U[:, 2, 0] - U[:, 0, 0]) * (U[:, 1, 1] - U[:, 0, 1]))
        # Game units per texel: the area one repeat of the texture covers,
        # over its texels (a non-square texture's texels averaged).
        texel = np.sqrt(area.sum() / max(uv_area.sum() * img.shape[0] * img.shape[1], 1e-12)) if img is not None else 1.0
        r = rng.random((len(tri), SAMPLES, 2))
        flip = r.sum(2) > 1
        r[flip] = 1 - r[flip]
        at = U[:, None, 0] * (1 - r[..., :1] - r[..., 1:]) + U[:, None, 1] * r[..., :1] + U[:, None, 2] * r[..., 1:]
        for f in (CLOSE, CAMERA):
            if img is None:
                c = np.broadcast_to(fac * 255, (len(tri), SAMPLES, 3))
            else:
                # The mip nearest that size, a power of two (the textures'
                # sides divide it, so none is cropped).
                m = mip(img, 2 ** max(0, int(round(np.log2(max(f / max(texel, 1e-9), 1.0))))))
                h, w = m.shape[:2]
                x = np.floor(np.mod(at[..., 0], 1.0) * w).astype(int) % w
                y = np.floor(np.mod(at[..., 1], 1.0) * h).astype(int) % h
                c = m[y, x] * fac
            far[f].append((luma(c).ravel(), np.repeat(area, SAMPLES)))
            if f == CLOSE:
                a = per.setdefault(name, [0.0, np.zeros(3)])
                a[0] += area.sum()
                a[1] += (c.mean(1) * area[:, None]).sum(0)
    total = sum(v[0] for v in per.values())
    mean = sum(v[1] for v in per.values()) / max(total, 1e-12)
    contrast = {}
    for f, parts in far.items():
        lum = np.concatenate([p[0] for p in parts])
        w = np.concatenate([p[1] for p in parts])
        mu = (lum * w).sum() / w.sum()
        contrast[f] = float(np.sqrt(((lum - mu) ** 2 * w).sum() / w.sum()) / max(mu, 1e-9))
    return {k: (v[0] / max(total, 1e-12), v[1] / max(v[0], 1e-12)) for k, v in per.items()}, mean, contrast


def show(orig, new, tolerance, share):
    po, mo, co = measure(orig)
    pn, mn, cn = measure(new)

    def rgb(c):
        return "%3.0f %3.0f %3.0f (luma %3.0f)" % (c[0], c[1], c[2], luma(c))

    ratio = mn / np.maximum(mo, 1e-9)
    off = np.abs(ratio - 1) > tolerance
    print("%s" % os.path.basename(new))
    print("  original %s" % rgb(mo))
    for k, (s, c) in sorted(po.items(), key=lambda kv: -kv[1][0]):
        print("      %5.1f %%  %-24s %s" % (100 * s, k[:24], rgb(c)))
    print("  new      %s" % rgb(mn))
    for k, (s, c) in sorted(pn.items(), key=lambda kv: -kv[1][0]):
        print("      %5.1f %%  %-24s %s" % (100 * s, k[:24], rgb(c)))
    verdict = "OFF (more than %.0f %%)" % (100 * tolerance) if off.any() else "within"
    print("  new / original: %.2f %.2f %.2f %s" % (ratio[0], ratio[1], ratio[2], verdict))
    for f, label in ((CLOSE, "close up"), (CAMERA, "at the camera")):
        low = cn[f] < share * co[f]
        mark = "LOW (under %.0f %% of the original's)" % (100 * share) if low else "ok"
        print("  contrast %s (%g units): original %2.0f %%, new %2.0f %% %s" % (label, f, 100 * co[f], 100 * cn[f], mark))
    return not off.any() and cn[CAMERA] >= share * co[CAMERA]


def main(argv):
    argv = argv[argv.index("--") + 1:] if "--" in argv else argv[1:]
    tolerance, share, paths = 0.10, 0.80, []
    i = 0
    while i < len(argv):
        if argv[i] == "--tolerance":
            tolerance = float(argv[i + 1])
            i += 2
        elif argv[i] == "--contrast":
            share = float(argv[i + 1])
            i += 2
        else:
            paths.append(argv[i])
            i += 1
    if len(paths) != 2:
        print(__doc__)
        return 2
    orig, new = paths
    if os.path.isdir(new):
        pairs = [(os.path.join(orig, f), os.path.join(new, f)) for f in sorted(os.listdir(new)) if f.endswith(".glb") and os.path.isfile(os.path.join(orig, f))]
    else:
        pairs = [(orig, new)]
    ok = all([show(o, n, tolerance, share) for o, n in pairs])
    print("all within" if ok else "not all within")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
