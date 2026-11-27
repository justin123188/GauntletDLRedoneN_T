# 0036: G1's start ground, the delivered art checked (models-plan M6)

The brief (art-briefs/g1-start-ground.md, 2026-11-26) came back the same
day: ChatGPT's eight pieces in the art folder's
G1_Ground/review_pack/G1 (outside the repository), with five shared
textures beside them (textures/: path 2048 px, bank, stone, root and moss
1024 px, opaque RGB PNGs), a materials.tsv (G1GROUND_path and _bank
ground, _stone stone, _root wood, _moss grass), the checker's reports and
maps, and two comparisons of its own.

| Piece | Triangles | Materials |
| --- | ---: | --- |
| G1#005 (the start ground) | 13,333 | path |
| G1FLOOR#02 (the ramp) | 3,085 | path |
| G1#004 (the higher ground) | 10,458 | path |
| G1#000 (the side into the swamp) | 19,764 | bank, stone, moss, root |
| G1#001 (the side up) | 14,932 | bank, stone, moss, root |
| G1#028 (the slope) | 11,680 | bank, stone, moss, root |
| G1#029 (the low ground) | 6,047 | moss, stone, root |
| G1#030 (the far side) | 16,447 | bank, stone, moss, root |

Checked here (2026-11-26):
- **What the loader reads.** Each file one buffer inside it for the
  geometry, the textures beside it by relative path (textures/, as
  share_data.py writes), triangles with positions, normals and UVs,
  repeating samplers, opaque materials, no extensions, animation or
  texture options. Longest edges 0.515 to 0.536 (the brief: about 0.5).
- **The fit.** Each piece in its original's frame and footprint (x and z
  bounds equal to 0.001, but G1#030 0.117 further in z); the walked-on
  pieces within y ±0.056 of their flat originals; G1#029 0.37 higher (its
  stones and roots); the sides into the swamp 0.6 lower than their
  originals (game y -9.7, under the water).
- **The checker** (tools/blender/check_ground.py, the same script as the
  art folder's copy): G1#005, G1FLOOR#02 and G1#004 100 % within the
  tolerance over 14,043, 3,307 and 9,272 samples (at most 0.055 off), no
  holes; nothing standing over the path but G1#001's five spots (up to
  1.962 above), the same positions and heights as its original's.
- **In the game** (the build of decisions/0035, 9d157a00, a scratch copy
  with the installed model pack and the pack in model_pack\G1): all eight
  files load, each "the original's light carried over", no loader
  message; G1's model pack 1.1 to 1.5 s (read and decode 0.35 to 0.45 s,
  mipmaps and detail maps 0.6 s); the first update 0.5 to 1.5 s later
  than without it; frame times the same (vsync off, frozen at update 300,
  median over frames 1000 to 3000: 8.44 and 8.34 ms against 8.45 and
  8.78); simulation hashes over 600 updates of the G1 walk equal the
  reference with the ground present, in the defaults and the baseline.
- **The look** (the art folder's G1_Start_Ground_InGame_Check.png: the
  start, the ramp, the west side, the ground by the hero): sharper and
  more detailed close up; but the new ground is lighter and greyer than
  G1's (the dirt by the hero a mean of 55, 47, 27 of 255 against the
  original's 31, 29, 7, under the same carried light: the textures'
  colour), and the sides read as smooth bands from the game's camera
  (the modelled roots and outcrops hardly show at that distance; the
  originals' sides have a rocky texture).

Not yet: the owner's review (a fix round, or a playtest as it is), the
install (with the build of decisions/0035).

Later (2026-11-27, the owner's "Yes, write the fix"): part of the
difference was the game's (the carried light about 5 / 255 too bright on
dark ground, fixed in decisions/0037); the rest is the textures (the
colour checker, tools/blender/check_colour.py). The fix brief:
art-briefs/g1-start-ground.md, "Fix round".
