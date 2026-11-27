# 0040: G1's banks, the gullies round checked (models-plan M6)

The gullies brief (decisions/0039) came back the same day, 2026-11-27.
The art folder's G1_Ground/review_pack/G1 holds four rebuilt banks
(G1#000, G1#001, G1#028, G1#030), a repainted textures/bank.png, a new
textures/gully.png and a sixth materials.tsv line (G1GROUND_gully
ground). Beside the pack: ChatGPT's checker output and cross-sections
(checks/), renders (renders/) and a hand-off note. The other four
pieces and four textures are byte-identical to the installed fix round
(checked here).

| Piece | Triangles | Colour, new / original | Contrast at the camera, original / new | Gully material | Off the original: median, 90 %, most out |
| --- | ---: | --- | --- | ---: | --- |
| G1#000 | 19,822 | 1.00, 1.03, 0.97 | 24 / 30 % | 30 % | 0.13, 0.29, 0.45 |
| G1#001 | 16,054 | 0.99, 1.00, 0.95 | 26 / 31 % | 31 % | 0.14, 0.34, 0.60 |
| G1#028 | 11,615 | 1.03, 0.97, 1.02 | 13 / 30 % | 29 % | 0.11, 0.22, 0.47 |
| G1#030 | 18,792 | 1.01, 1.02, 0.98 | 26 / 29 % | 28 % | 0.17, 0.37, 0.77 |

The hand-off's own cross-sections give these medians:

| Piece | Gully spacing | Floor width | Depth below the rib |
| --- | ---: | ---: | ---: |
| G1#000 | 2.82 | 0.76 | 0.38 |
| G1#001 | 2.86 | 0.58 | 0.54 |
| G1#028 | 3.28 | 0.58 | 0.32 |
| G1#030 | 3.15 | 0.58 | 0.50 |

Those are its own figures.

Checked here (2026-11-27, the owner's PC):
- **The colour checker** says "all within". Its output is the same as
  the hand-off's, byte for byte.
- **The ground checker:** nothing stands over the path but G1#001's five
  original spots, at the same positions and heights. No bank has walked-on
  ground under it but G1#001's 11 samples, which are the same as its
  original's.
- **The files:**
  - no repeated triangles and no zero-area ones: G1#001's 704 repeats
    and G1#000's 2 zero-area triangles are gone;
  - longest edges 0.536;
  - opaque materials, no extensions and no animation;
  - textures 1024 px, opaque RGB.
- **The direction:** on all four banks the bank and gully textures'
  vertical runs down the fall line: a median of 3 to 5 degrees, 95 to 99
  % within 20. That is closer than the originals (2 to 10 degrees, 87 to
  100 %). Both lay one tile about 10.2 to 10.7 units wide and 8.7 to 11
  tall down the bank, as the original does.
- **The gullies,** by where the gully material lies, in three height
  bands per face (a scratch measure):
  - G1#000 has one every 2.4 units of section, the others every 3.9 to
    4.4. Some gullies count twice, where the gully material splits.
  - The bank faces stand a mean 0.13 to 0.23 above the gully faces.
  - The overall relief is smaller than the fix round's (most out 0.45 to
    0.77 against 1.38 to 1.71), since the ledges and outcrops went, as
    the brief asked.
- **In the game,** with the build of decisions/0037 (65051ea4) in the
  scratch copy:
  - All eight pieces load with the light carried over, and there are no
    warnings.
  - **The mean colour** of the pixels that differ from the original (B /
    A as red, green, blue):

    | View | Upper half | Lower half |
    | --- | --- | --- |
    | Start | 0.98, 1.00, 0.92 | 1.04, 1.07, 1.11 |
    | West side | 1.01, 1.04, 0.94 | 0.98, 1.02, 0.85 |
    | Ramp | 1.25, 1.23, 1.29 | 0.99, 0.97, 0.89 |
    | Higher ground | 1.03, 1.08, 1.11 | 1.19, 1.21, 1.35 |

    These are within 0.06 of the fix round's, except the west side's
    lower half, which comes 0.04 to 0.09 closer to the original.
  - **Frame times:** 8.04 and 8.09 ms (the fix round 8.02 and 8.13).
  - **Loading:** G1's model pack takes 1.14 to 1.17 s.
  - **The simulation hashes** over the 600-update G1 walk equal the
    reference in the defaults and the baseline.
- **The look:** see the art folder's G1_Banks_Gullies_Views.png (the four
  views: the original, the fix round, the gullies round) and
  G1_Banks_Gullies_Closeups.png (three banks at 1.2 times).
  - A mossy lip outlines each bank's top edge, as in the original.
  - Below it, vertical ribs and channels run down to the water.
  - The original's cracks are crisper and darker. The new gullies read
    as softer folds.

Installed 2026-11-27 (the owner's "Yes, install"):
- The seven changed files went into the installed model_pack\G1: the
  four banks, bank.png, gully.png and materials.tsv. Its 15 files now
  have the same hashes as the review pack.
- The exe is unchanged (65051ea4).
- The layered fix round is kept beside the game, outside the model pack,
  as model_pack-G1-layers-backup\G1 (14 files). To go back, put that
  folder in place of model_pack\G1.
- Checked in the installed game, with an isolated APPDATA:
  - The start and the west side load all eight pieces with the light
    carried over, and there are no warnings.
  - The frames are the scratch copy's but for the HUD's "LV 1" (619
    pixels, the owner's font).

The owner, after the install (2026-11-27): "yes the softer gullies are
fine". G1's start area ground is accepted as installed: the fix round's
path and walked-on pieces, and these banks.

Next (models-plan M6 step 4): the rest of G1, then each level's terrain
in its overhaul.
