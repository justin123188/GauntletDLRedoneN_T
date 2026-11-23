# 0021: The model exporter (models-plan M4)

The owner's go-ahead (2026-11-23): "Yes, start it", the exporter and a
test barrel before effects, after the owner pointed out that their
"blender experience is limited to make a cube of water smash into a
sphere". The plan changes with that: the owner chooses, scripts do the
work, and Blender is used by hand only for touch-ups (models-plan.md
"Tools for the owner").

## Decision

`gdl_play --export-models CONFIG.json OUT_DIR [NAME ...]`
(gdl_client::model_export). No window: it reads the installed data and
writes new files, so it is no setting and nothing in the game changes.
A command rather than the F10 button the plan offered: our scripts run
it; a button can follow if the owner wants one. Names pick levels (G1)
and packages (ITEMS_G); none means every level and every package of the
configuration. It refuses a folder that holds other files (anything
without an earlier export's manifest.json).

- **Files.** glTF 2.0 binary (.glb): `objects/<package>/<LABEL>.glb`,
  one per object, and `levels/<LEVEL>.glb`, one per level with three root
  nodes: SCENERY (the placed objects as spawn_level draws them, instanced),
  COLLISION and WATER (never drawn). `manifest.json` lists every file with
  its SHA-256, part and triangle counts, and the SHA-256 of every source
  file read; `README.txt` says what the files are and that they stay
  private. A writer of our own (serde_json and png, already dependencies):
  no new crates (AGENTS.md rule 9).
- **Space.** The original coordinates with x negated, as the renderer's
  MIRROR does, so the files are right-handed with y up, as glTF wants; one
  game unit is one metre. Triangle order is kept: Bevy draws these same
  mirrored positions with counter-clockwise front faces, glTF's rule
  (bevy_pbr 0.18.1 uses the model matrix's determinant sign only for
  tangents, not for culling).
- **Geometry.** Each part as the renderer builds it: mesh_and_colours now
  takes its streams from `gdl_client::part_geometry`, which the exporter
  shares. One primitive per part.
- **Materials.** Base colour = level 0 of the part's texture as PNG, alpha
  doubled to 0..255 as the texture dump writes it; MASK (cutoff 0.5) or
  BLEND by materials_for's alpha classes; the sampler clamps U/V by record
  flags 0x4/0x8. A record without texels of its own (a texture modifier's
  target) gets the game's magenta placeholder colour. Each material's
  extras give the record, the TEXDEF label and the texture's name as the
  texture pack keys it (16 hex digits), so a model's textures link back to
  `texture_pack\`.
- **Light.** The lightmap becomes the occlusion texture on TEXCOORD_1,
  grey index / 16 (the lightmap pass multiplies by 8 × index / 128).
  Vertex colours become COLOR_0 (RGB, the GS 0x80 = 1.0 scale) and the GS
  vertex alpha a custom attribute _GDL_VERTEX_ALPHA.
- **Collision.** Each object's triangles, from its world node's range
  (+0x38, count +0x36) and the static grid's entries for it, through
  `gdl_core::collision::Triangle::corners` (new: the origin and the
  local-plane vertices through WorldVectorNorm, as BTriLineCol turns its
  hit back). Dynamic objects (0x1000, 0x1000000) at their rest
  translation. Water objects (0x200) in WATER. Corners go out as c0, c2,
  c1: BTriLineCol hits only triangles whose (c1 − c0) × (c2 − c0) points
  along the normal, and mirrored, that order keeps the normal's side the
  front face.
- **Names.** An object's first definition label; a frame object of a
  definition with frames "<LABEL>+k"; otherwise "#<index>". Characters
  Windows refuses become '_', device names (AUX, CON …) get '_', and a
  name that clashes ignoring case gets "~<index>".
- **Not yet.** The player characters (their packages load from the
  catalog, not from the configuration's packages; skeletons with M8) and
  the items' own collision (counted; research/unknowns.md "Items' own
  collision in the model export").
- **Privacy.** The exports come from the disc. They go beside the game
  (`model_work\export\` for the owner's copies) or into scratch folders,
  never into the repository.

## What the first exports show (2026-11-23, the owner's PC)

- **The whole game:** 61 level scenes and every object of the levels and
  the 107 shared packages, 90,303 files, 3.2 GB, 282 s; no part failed to
  decode.
- **G1:** 835 placed objects, 944 level objects, 278 ITEMS_G objects;
  the scene file is 4.9 MB.
- **Faces.** Every one of G1.glb's 11,379 triangles (scenery meshes and
  collision) has its counter-clockwise face normal on the side of its
  vertex normals; the standing barrel 48 of 48; a chest lid 16 of 20 (the
  4 others presumably its inner side).
- **Collision accounting.** In A1, B4, F1, G1, J3 and S7 every triangle
  record is in a world node's range or an item instance's own range (G1:
  3,351 exported + 110 item triangles = the 3,461 records of
  research/collision.md); the static grid lists no triangle outside its
  object's range.
- **The barrel is a flip-book.** ITEMS_G's BAROBJIDLE15F01 (standing),
  BAROBJACTI16F01..F24 (breaking) and BAROBJDONE17F24 (broken); the
  poison barrel (BARPOI…) and the exploding one (BAREXP…) likewise. A
  replacement of the standing frame alone shows until the barrel is hit.

## Verification

- **Tests and lints.** cargo test: gdl_client 78 + 5 (new: the .glb
  layout, Windows-safe names), gdl_core 102 (new: a sloped triangle's
  corners lie on its plane and BTriLineCol hits the line through their
  centroid there); clippy shows only the older warnings (audio.rs,
  wgpu-hal).
- **The game unchanged** (the build of this change, md5 315ADAF0, against
  the installed 107c6e8, both run from scratch folders):
  - G1 and C1 frozen at update 300 with the defaults and interpolation
    off (s-defnointerp), frame 1500: identical. Run from the game's own
    folder, the installed build draws "LV 1" with the owner's font
    (texture_work\fonts, decisions/0013), which a scratch folder lacks:
    619 pixels there, nothing else.
  - The original presentation (GDL_BASELINE=1, G1): identical.
  - Simulation hashes over 600 updates of the G1 walk equal the
    reference, with the defaults and with GDL_BASELINE=1.
- **Blender** (tools/blender/check_glb.py, Blender 5.3.0 hidden, on the
  G1 and ITEMS_G export of this build): G1.glb and four ITEMS_G objects
  (the standing barrel, a breaking frame, a chest lid and bottom) import
  without an error or warning. Blender's counts equal the exporter's (G1:
  1,157 mesh objects, 835 placed plus 322 collision, 11,379 triangles, 98
  of 98 textures with pixels, the colour attribute and up to two UV sets);
  every object keeps its "gdl" extras as custom properties. The previews
  show the barrel textured and standing on the ground (2.5 × 2.7 units),
  and G1's collision on the level's paths and floors. The 37 world nodes
  that are both drawn and colliding have their two objects' centres at
  most 0.0055 units apart (median 0.0019).
- **The Blender used.** blender-5.3.0-windows-x64.zip (425,299,259 bytes,
  SHA-256 3ac567a63d59d9934f7b9213bfb27ba05760e1fb366f8f55c381573ec3eabfa8),
  downloaded by the owner from blender.org (download.blender.org refuses
  scripted downloads and shows browsers a bot check, so the published
  checksum was not compared); blender.exe's Authenticode signature is
  valid, signed by the Blender Foundation (Amsterdam). Unpacked beside the
  repository in `tools\blender-5.3.0-windows-x64\` (not in it).
