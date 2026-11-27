# 0038: G1's start ground, the fix round checked (models-plan M6)

The fix brief (decisions/0037; art-briefs/g1-start-ground.md, "Fix
round") came back the same day, 2026-11-27. It is in the art folder's
G1_Ground (outside the repository): the pack in review_pack/G1, ChatGPT's
checker output in checks/, renders in renders/, its build scripts in
source/, and a hand-off note.

What changed (the hand-off, and the files checked here):
- **The five textures are repainted**, at the same sizes (path 2048 px,
  bank, stone, root and moss 1024 px, opaque RGB). The colour factor is
  the default (1).
- **The five free pieces are rebuilt** (G1#000, G1#001, G1#028, G1#029,
  G1#030), with deeper strata, thicker roots and larger outcrops. The
  hand-off counts 49 outcrops, a median of 2.23 to 2.41 units across
  and 0.62 to 0.88 out from the bank. Those are its own figures.
- **The three walked-on pieces are unchanged** from the first round. Their
  file dates, triangle counts and checker figures are the same.
- **materials.tsv is unchanged** (five entries). No recess material was
  added.

| Piece | Triangles | Colour, new / original | Contrast at the camera, original / new | Off the original's surface: median, 90 %, most |
| --- | ---: | --- | --- | --- |
| G1#005 (the start ground) | 13,333 | 0.96, 0.99, 0.96 | 11 / 15 % | 0.02, 0.04, 0.05 |
| G1FLOOR#02 (the ramp) | 3,085 | 1.05, 1.01, 1.05 | 11 / 13 % | walked on |
| G1#004 (the higher ground) | 10,458 | 1.02, 1.05, 1.02 | 8 / 15 % | walked on |
| G1#000 (the side into the swamp) | 19,879 | 0.97, 0.98, 0.97 | 24 / 29 % | 0.24, 0.59, 1.38 |
| G1#001 (the side up) | 15,037 | 1.01, 1.01, 0.99 | 26 / 27 % | 0.21, 0.61, 1.46 |
| G1#028 (the slope) | 12,112 | 1.04, 0.97, 1.05 | 13 / 27 % | 0.27, 0.76, 1.71 |
| G1#029 (the low ground) | 4,764 | 1.01, 0.99, 0.95 | 10 / 14 % | 0.15, 0.56, 1.25 |
| G1#030 (the far side) | 18,294 | 1.00, 1.01, 1.01 | 26 / 28 % | 0.25, 0.64, 1.40 |

The first round stood off the original's surface by a median of 0.06 to
0.16 units, 90 % within 0.25 to 0.42, at most about 0.7.

Checked here (2026-11-27, the owner's PC):
- **The colour checker** (tools/blender/check_colour.py, Blender's
  Python, 3 s) says "all within". The table's colour and contrast come
  from it, and its output is the same, byte for byte, as the hand-off's
  checks/check_colour.txt.
- **The ground checker** (tools/blender/check_ground.py) gives what the
  first round gave:
  - G1#005, G1FLOOR#02 and G1#004 are 100 % within the tolerance over
    14,043, 3,307 and 9,272 samples (at most 0.055 off), with no holes.
  - Nothing stands over the path but G1#001's five spots, at the same
    positions and heights as its original's (up to 1.962 above).
- **The relief** (the table's last column) comes from Blender, a BVH of
  the original, per face of the new piece: about two to three times the
  first round.
- **The bank texture's layers run level:** the direction of its u axis is
  a median of 4 to 5 degrees from level on all four banks, 100 % within
  20. In the game's views the bands curve with the banks themselves.
- **The files:**
  - Longest edges are 0.515 to 0.536.
  - The materials are opaque, single-sided, with repeating samplers.
  - There are no extensions and no animation.
  - G1#000 has 2 zero-area triangles.
  - G1#001 has 704 exact repeats: the same positions, winding, normals,
    UVs and material, all on the bank in one stretch (local x 14 to 22).
    They draw the same pixels, so nothing shows. They are worth dropping
    in a later pass.
- **In the game,** with the build of decisions/0037 (md5 65051ea4) in a
  scratch copy of the game with the installed model pack, the pack in
  model_pack\G1, isolated APPDATA and GDL_NO_GAMEPADS:
  - All eight files load, each with "the original's light carried over",
    and there are no warnings or errors.
  - The original start on this build is the same file as the 0035
    build's.
  - **The mean colour** of the pixels that differ from the original
    (frozen at update 300, frame 1500; ingame-mean, the upper and lower
    halves). B / A is the new ground's mean over the original's, as
    red, green, blue:

    | View | Fix round, upper half | Fix round, lower half | First delivery (same build) |
    | --- | --- | --- | --- |
    | Start | 0.96, 0.94, 0.88 | 1.02, 1.03, 1.08 | 1.46 to 1.51; 2.4 |
    | West side | 1.00, 1.03, 0.92 | 0.94, 0.93, 0.77 | 1.48 to 1.61; 2.4 to 2.5 |
    | Ramp | 1.31, 1.17, 1.34 | 0.97, 0.94, 0.87 | 1.44 to 1.93; 2.5 to 3.5 |
    | Higher ground | 1.04, 1.08, 1.11 | 1.17, 1.15, 1.33 | 1.63 to 1.69; 2.7 to 3.0 |

    The values are dark (4 to 32 of 255), so 1.3 is 1 to 5 levels.

    The hand-off's own screen figure, "within 0.05 of the original on
    the 0–255 scale" at the start, is not what this measure gives: it
    gives 1 to 2 levels.
  - **Frame times:** vsync off, frozen at update 300, the median over
    frames 1000 to 3000. With the pack: 8.02 and 8.13 ms. Without it:
    8.77 ms. A first run without it was held at 16.7 ms and is not
    counted.
  - **Loading:** G1's model pack takes 1.17 to 1.20 s (read and decode
    0.43 to 0.47 s, mipmaps and detail maps 0.58 s). The first update
    comes 1.0 to 1.4 s later than without the pack.
  - **The simulation hashes** over 600 updates of the G1 walk equal the
    reference with the ground present, in the defaults and the baseline.
- **The look:** see the art folder's G1_Start_Ground_FixRound_Views.png
  (the four views: the original, the first delivery, the fix round) and
  G1_Start_Ground_FixRound_Closeups.png (full-size crops).
  - The path and banks are now G1's dark olive and red-brown.
  - The banks read as layered earth with ledges and outcrops.
  - Differences from the original that remain:
    - The original's path is more mottled at mid scale, with patches of
      green moss.
    - The original's sides have vertical gullies. The new banks have
      level strata.
    - The ramp's upper half and the higher ground's lower half are a
      little lighter.

Not yet: the owner's look at the pictures, the install (the build of
0037 with this pack), the owner's playtest.
