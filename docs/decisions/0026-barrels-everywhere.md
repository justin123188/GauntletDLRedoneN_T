# 0026: The owner's barrels in every realm, and model pack files that share their data

The owner's choice (2026-11-24): "Let's do barrels everywhere and yes use
the script", after the red and poison barrels (decisions/0024), with the
data-sharing step first so the files do not multiply on disk.

## What the originals are

The model export's item packages with barrels (checked: each object's
positions against ITEMS_G's, frame by frame):

| Package | Brown | Red | Poison |
| --- | --- | --- | --- |
| ITEMS_A, B, D, E, F, G, H, K, S | yes | yes | yes |
| ITEMS_J | yes | no | yes (BARPOIIDLE18F01, its frames numbered 19 and 20) |
| ITEMS_C, ITEMS_I | no (only the BAREXP0 / BARPOI0 decals) | | |

Every barrel there has G's shape and G's breaking pieces. Their textures
differ in bytes, but their colour statistics (recolour.py's coloured
pixels) agree with G's to within 0.1 %: brown hue 32.4°, red 0.3°, green
102.3°.

## Decision

**Shared data** (gdl_client::model_pack, NATIVE): a .glb may name its
buffer and its PNG images as files beside it (`uri`: plain relative paths
of letters, digits, `_`, `-`, `.` in the file's folder or below; never
`..`, a drive, a scheme or an encoded name; the 64 MB read limit). A
package's .glb files read and decode each such file once, and the
texture's identity is hashed once per image (before: once per part, about
1,500 hashes of a 1.9 MB PNG for G1's 78 files).

**tools/blender/share_data.py FOLDER** (plain Python; Blender's own works):
each .glb's embedded PNGs to `textures/<sha256 prefix>.png` and the rest
of its buffer to `data/<sha256 prefix>.bin`, one file per content, the
.glb keeping its JSON. A breaking frame's pieces are the same meshes in
every frame (only the nodes move), so a barrel's 25 frames and all three
barrels share one geometry file.

**recolour.py --hue shift**: the hue turned by TO's mean minus FROM's (one
material in another realm's colour), so the owner's warmer brown keeps
its tone; `--hue to` (the default) stays for paint.

## The realms' barrels (2026-11-24)

From the owner's fitted brown barrel: recolour.py to each package's brown
(--hue shift, from G's brown), red and poison (--hue to), break_frames.py
with --inside-from the brown barrel, share_data.py per package (tools/blender/make_realm_barrels.sh). 29 barrels, 754 .glb
files; each package 78 files in 7.7 MB (4 textures, 2 geometry files;
ITEMS_J 52 files, 3 textures), 74 MB in all, against 186 MB for G's 78
embedded files before.

## Verification (2026-11-24, the build d41840c9 of this change)

- **Unchanged:** G1 with G's 78 files, G1 with the pack off, C1, and G1
  with GDL_BASELINE=1 (frozen at update 300, frame 1500) identical to the
  installed build (07628ba0, G's files embedded); G1's brown barrel
  breaking (g1-smash-barrel.json, frozen at update 116) identical with the
  shared files as with the embedded ones. The red barrel's fireball
  (update 280) differs by one level in two pixels between any two runs,
  shared or embedded alike.
- **Other realms:** A1, D1 and K1 started directly pose their barrels from
  ITEMS_A, ITEMS_D and ITEMS_K (GDL_DEBUG_ITEM_TREES) and the log names
  those packages' replacements; A1's start shows two of the new barrels.
- **Loading:** G1 to its first frames 6.0 and 6.1 s with every realm's
  shared files, 7.1 s (twice) with the installed build and G's embedded
  files, 5.5 and 5.6 s with the pack off.
- **Game logic:** simulation hashes over 600 updates of the G1 walk equal
  the reference, with the defaults and with GDL_BASELINE=1.
- **Tests:** cargo test gdl_client 79 + 5 (the minimal .glb read again
  with its buffer and image beside it, sharing the embedded image's
  texture; paths out of the folder refused); clippy shows only the older
  warnings.
- **Not yet:** the owner's playtest.
