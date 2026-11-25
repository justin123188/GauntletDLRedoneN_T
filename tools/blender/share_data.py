"""Let a folder of model pack files share their data on disk (decisions/0026,
models-plan M5): each file's embedded PNG images go to textures/<hash>.png
and the rest of its buffer (the geometry) to data/<hash>.bin beside it,
one file per distinct content, and the .glb keeps only its JSON with
relative uris. The game reads them (gdl_client::model_pack, only plain
relative paths in the file's folder); so does Blender's importer.

    python share_data.py FOLDER [--report FILE.json]
    (Blender's own Python: blender-5.3.0-windows-x64/5.3/python/bin/python.exe,
    or blender --background --factory-startup --python share_data.py -- FOLDER)

G1's barrels: 26 files per barrel embed the same 1.9 MB texture, and the
three barrels' breaking frames have the same geometry, so a package keeps
one texture per barrel and one geometry file per frame. Files already
shared are left as they are; nothing outside FOLDER is written.
"""
import hashlib
import json
import os
import struct
import sys

MAX_BYTES = 512 * 1024 * 1024


def parse_args():
    a = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else sys.argv[1:]
    report = ''
    pos = []
    while a:
        k = a.pop(0)
        if k == '--report' and a:
            report = a.pop(0)
        elif k.startswith('--'):
            raise SystemExit(f'unknown or incomplete option {k}')
        else:
            pos.append(k)
    if len(pos) != 1:
        raise SystemExit(__doc__)
    return pos[0], report


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


def write_json_glb(path, doc):
    js = json.dumps(doc, separators=(',', ':')).encode()
    js += b' ' * (-len(js) % 4)
    with open(path, 'wb') as f:
        f.write(struct.pack('<III', 0x46546C67, 2, 12 + 8 + len(js)))
        f.write(struct.pack('<II', len(js), 0x4E4F534A))
        f.write(js)


def put(folder, sub, ext, data, written):
    """data as <folder>/<sub>/<sha256 prefix>.<ext>, once; its relative uri."""
    name = hashlib.sha256(data).hexdigest()[:20] + ext
    path = os.path.join(folder, sub, name)
    if os.path.exists(path):
        with open(path, 'rb') as f:
            if f.read() != data:
                raise SystemExit(f'{path} exists with other content')
    else:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, 'wb') as f:
            f.write(data)
        written.append(os.path.relpath(path, folder))
    return f'{sub}/{name}'


def share(folder, name, written):
    path = os.path.join(folder, name)
    doc, data = read_glb(path)
    buffers = doc.get('buffers', [])
    if len(buffers) != 1 or 'uri' in buffers[0]:
        return 'left as it is'
    views = doc.get('bufferViews', [])
    if any(v.get('buffer', 0) != 0 for v in views):
        raise SystemExit(f'{name}: a second buffer')

    def view(i):
        v = views[i]
        start = v.get('byteOffset', 0)
        return data[start:start + v['byteLength']]

    drop = set()
    for img in doc.get('images', []):
        if 'bufferView' not in img:
            continue
        if img.get('mimeType') != 'image/png':
            raise SystemExit(f'{name}: an image that is not PNG')
        drop.add(img['bufferView'])
        img['uri'] = put(folder, 'textures', '.png', view(img.pop('bufferView')), written)
    used = {a['bufferView'] for a in doc.get('accessors', []) if 'bufferView' in a}
    if used & drop:
        raise SystemExit(f'{name}: an image view also read as data')
    keep = [i for i in range(len(views)) if i not in drop]
    renumber = {old: new for new, old in enumerate(keep)}
    out = bytearray()
    new_views = []
    for old in keep:
        v = dict(views[old])
        chunk = view(old)
        out.extend(b'\0' * (-len(out) % 4))
        v['byteOffset'] = len(out)
        v['byteLength'] = len(chunk)
        out.extend(chunk)
        new_views.append(v)
    out.extend(b'\0' * (-len(out) % 4))
    for a in doc.get('accessors', []):
        if 'bufferView' in a:
            a['bufferView'] = renumber[a['bufferView']]
    doc['bufferViews'] = new_views
    doc['buffers'] = [{'byteLength': len(out), 'uri': put(folder, 'data', '.bin', bytes(out), written)}]
    write_json_glb(path, doc)
    return 'shared'


def main():
    folder, report_path = parse_args()
    names = sorted(n for n in os.listdir(folder) if n.lower().endswith('.glb'))
    before = sum(os.path.getsize(os.path.join(folder, n)) for n in names)
    written, result = [], {}
    for n in names:
        result[n] = share(folder, n, written)
    after = sum(os.path.getsize(os.path.join(folder, n)) for n in names)
    after += sum(os.path.getsize(os.path.join(folder, w)) for w in written)
    report = {'files': len(names), 'shared': sum(1 for r in result.values() if r == 'shared'),
              'written': sorted(written), 'bytes_before': before, 'bytes_after_new_files': after}
    if report_path:
        with open(report_path, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=1)
    print('share_data:', json.dumps({k: report[k] for k in ('files', 'shared', 'bytes_before', 'bytes_after_new_files')},
                                     ) + f', {len(written)} data files written')


main()
