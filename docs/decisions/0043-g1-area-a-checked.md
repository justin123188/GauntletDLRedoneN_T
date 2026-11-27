# 0043: G1's area A (the fields and the west), checked (models-plan M6)

Area A of the rest of G1's brief (decisions/0041) came back on
2026-11-27. The art folder's G1_Ground/review_pack/G1 adds:
- 17 pieces: the dirt (G1#007, G1#017, G1#035, G1FLOOR#10, #36, #37), the
  fields (G1#006, G1#064, G1FLOOR#03), the ramp, slopes and swamp side
  (G1FLOOR#17, G1#033, G1#034, G1#040, G1#041), and the river banks and
  the far ground (G1#082, G1#083, G1#042);
- two textures, field.png and riverbank.png;
- two materials.tsv lines, G1GROUND_field ground and G1GROUND_riverbank
  ground.

Beside the pack are ChatGPT's checker output and reports (checks/area_a/),
renders, scripts and a hand-off note. The start area's files are
byte-identical to the installed ones (checked here).

Checked here (2026-11-27, the owner's PC):
- **The colour checker:** "all within" over the 25 pieces, and its output
  is the same as the hand-off's, byte for byte.
  - Area A's pieces come out 0.91 to 1.08 of their originals per
    channel.
  - The contrast at the camera is higher than the originals': the fields
    27 % against 7 %, the slopes 25 to 30 % against 12 to 14 %, the river
    banks 39 % against 27 to 29 %, the dirt 14 to 15 % against 8 to 11 %.
- **The ground checker:**
  - Every walked-on piece is 100 % within the tolerance, with no holes,
    at most 0.073 off. That includes G1FLOOR#36 (the original 80.5 %) and
    G1#040's samples (the original 11.9 %).
  - The fields are flat on the collision (0.000 off): their furrows are
    in the texture.
  - Nothing stands in the way. G1#041, G1#042 and G1#083 have no
    collision under or beside them.
- **The files:**
  - 332,904 triangles for area A. G1#007 has 58,080 and G1#006 44,855.
  - Longest edges 0.498 to 0.515, and 0.99 on G1#042 and G1#083 (the
    backdrop, as allowed).
  - No repeated or zero-area triangles.
  - Opaque materials with the default colour factor; no extensions or
    animation; textures 1024 and 2048 px, opaque RGB.
- **The direction:** on the slopes, the swamp side and the river banks,
  the textures' vertical runs down the fall line: a median of 3 to 10
  degrees, 76 to 100 % within 20.
- **The relief** off the originals: a median of 0.08 to 0.17, most out
  0.45 to 1.17, like the start area's gullies.
- **In the game,** with the installed build of decisions/0042 (d7ae700a)
  in the scratch copy, the original against the pack (the start area and
  area A):
  - All 25 pieces load with the light carried over, and there are no
    warnings.
  - **The mean colour** of the pixels that differ from the original (B /
    A as red, green, blue):

    | View | Upper half | Lower half |
    | --- | --- | --- |
    | a1, the field under the corn | 1.01, 0.99, 0.99 | 0.92, 0.92, 0.83 |
    | a2, the west ground | 0.99, 1.05, 1.01 | 0.95, 0.94, 0.95 |
    | a3, west of the big field | 1.20, 1.04, 1.09 | 1.02, 1.01, 1.01 |
    | a4, west, by the river | 0.88, 0.97, 0.87 | 0.99, 1.00, 1.00 |
    | Start | as decisions/0040 | |

    The values are dark: a3's 1.20 is 6 to 7 of 255, and a4's 0.88 is
    about 2 levels.
  - **Frame times** at a1 (vsync off, the median over frames 1000 to
    3000): 8.34 and 8.44 ms, against 8.47 and 8.31.
  - **Loading:** the first update comes 2.0 s later (6.29 s against 4.30
    to 4.33). G1's model pack takes 1.84 to 1.86 s (read and decode 0.70
    s, mipmaps and detail maps 0.61 to 0.65, parts 0.51 to 0.52). The
    start area alone added 1.2 s.
  - **The simulation hashes** over 600 updates of the G1 walk, with the
    pack present, equal the reference in the defaults and the baseline.
- **The look** (the art folder's G1_Rest_Area_A_Views.png and
  G1_Rest_Area_A_Closeups.png):
  - The fields read as diagonal furrows, softer and a little darker than
    the original's bright clods.
  - The river banks have rounded gully ribs, a mossy lip, and a dark toe
    at the water, with no glint and no gap.
  - The dirt is the start area's path. At a4 it is more evenly mossy and
    a little darker than the original's lighter, brown worn patches.

Installed 2026-11-27 (the owner's "Yes, install"):
- Area A's 17 pieces, field.png, riverbank.png and materials.tsv went
  into the installed model_pack\G1. Its 34 files now have the same
  hashes as the review pack.
- The exe is unchanged (d7ae700a). model_pack now holds 1,009 .glb,
  1,341 files, 291 MB.
- The start area alone is kept beside the game as
  model_pack-G1-startarea-backup\G1 (15 files). To go back, put that
  folder in place of model_pack\G1.
- Checked in the installed game, with an isolated APPDATA, at a1, a2
  and the start: all 25 pieces load with the light carried over and no
  warnings. The frames are the scratch copy's but for the HUD's "LV 1"
  (619 pixels, the owner's font).

Not yet: the owner's playtest, area B.
