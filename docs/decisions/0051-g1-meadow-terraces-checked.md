# 0051: G1's meadow terraces, checked

The meadow's fix round (decisions/0048) came back on 2026-11-27: a new
textures/grass.png in the art folder's G1_Ground/review_pack/G1 (1024 px,
RGB, SHA-256 69d13a75…a046c), with ChatGPT's colour check, a comparison
render and a note.

Checked here (2026-11-27, the owner's PC):
- **Only grass.png changed:**
  - The pack's 33 start-area and area A files are byte-identical to the
    installed model_pack\G1. materials.tsv differs by area B's two lines,
    as before.
  - Area B's eight pieces and rock.png keep the delivery's file times
    (12:04), before their check (decisions/0046). grass.png alone is
    newer than the brief.
- **The corrected colour checker** (tools/blender/check_colour.py) on
  G1#018 gives the hand-off's figures:
  - the colour 1.01, 1.01, 1.01 of the original's, within;
  - the contrast close up 29 % against 28 %;
  - at the camera 15 % against 14 %, ok (the mark is 11.2 %; the delivery
    before the fix had 11 %).
- **The terraces in the texture** (a scratch measure: the luma's row and
  column means; the rows run along u, east to west on G1#018):
  - the bands along u are 13.9 % of the mean against the original's
    12.8 %, and the bands across 5.3 % against 5.7 %;
  - the light and dark rows follow the original's almost row for row
    (the smoothed row means down the texture: 58 57 59 50 58 55 55 64 66
    64 51 41 … against the original's 59 55 60 52 59 56 60 63 66 64 51
    43 …);
  - 9.4 % of the texels are darker than half the mean, against 10.9 %
    (the delivery before the fix: 4 %).
- **In the game,** with the build of decisions/0049 (6e119bd2, the
  installed build) in the scratch copy, at b1 to b4 (decisions/0041,
  0046):
  - **The control:** b1 without a G1 pack is identical to decisions/0046's
    frame, so this build's frames compare with those of the build of
    decisions/0045.
  - **The pieces:** all 33 load with the light carried over, and there
    are no messages.
  - **Against the delivery before the fix:** the frames differ over the
    meadow. At b4 the differences stay at the left (x 0 to 821), where
    the meadow is: the river banks are unchanged.
  - **The mean colour** of the pixels that differ from the original is
    0.99 to 1.07 of the original's at b1 to b3, and 0.93 to 1.00 at b4.
  - **The simulation hashes** over the 600-update G1 walk, with the pack,
    equal the reference in the defaults and the baseline.
- **The look** (the art folder's G1_Ground/G1_Meadow_Terraces_InGame.png):
  - The terraces are back. From the camera, the wavy step lines run as
    the original's do at b1 to b3.
  - The fix is a little more mottled, with red-brown and bright green
    patches, where the original is a more even brown-olive. Its step
    lines are softer than the original's crisp ones.

Not yet: the owner's look and choice to install area B (the start area,
areas A and B in model_pack\G1).
