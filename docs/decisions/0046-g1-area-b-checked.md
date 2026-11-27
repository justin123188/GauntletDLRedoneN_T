# 0046: G1's area B (the grassland and the river), checked (models-plan M6)

Area B of the rest of G1's brief (decisions/0041) came back on
2026-11-27. The art folder's G1_Ground/review_pack/G1 adds:
- 8 pieces: the meadow and the eastern dirt (G1#018, G1#038), the river
  slope and the cut earth (G1#046, G1#078), the banks (G1#080, G1#081),
  and the western backdrop and rock wall (G1#056, G1#059);
- textures/grass.png and textures/rock.png;
- two materials.tsv lines, G1GROUND_grass grass and G1GROUND_rock stone.

ChatGPT's checks, renders and notes are beside the pack. The installed
start area and area A are byte-identical to the pack's copies (checked
here).

Checked here (2026-11-27, the owner's PC):
- **The colour checker:** "all within" over 33 pieces, the same output as
  the hand-off's.
  - Area B's pieces come out 0.93 to 1.07 of their originals.
  - The contrast at the camera is higher than the originals': the meadow
    20 % against 6 %, the river banks 38 to 40 % against 28 to 30 %.
- **The ground checker:**
  - The meadow (G1#018) is 98.2 % within, the same as the original, and
    never above the collision (at most 0.001). The rest is collision
    above it: per the hand-off, a raised walkway and two ember mounds.
  - G1#038 is 100 % within. G1#046 and G1#078 are 100 % within where
    their originals were 6.4 and 5.0 %.
  - No holes, and nothing in the way.
- **The files:**
  - 434,948 triangles (G1#018 127,212; G1#056 86,760).
  - Longest edges 0.504 to 0.518, and 0.98 on G1#059, the unreachable
    rock wall.
  - No repeated or zero-area triangles; opaque materials; textures 1024
    px.
- **In the game,** with the build of decisions/0045 (c8d7441e) in the
  scratch copy, the original against the pack (the start area, area A
  and area B), at b1 and b2 (decisions/0041), b3 (the boardwalk, game
  -10, 18.42, -150) and b4 (by the river, 60, 23.62, -195):
  - All 33 pieces load with the light carried over, and there are no
    warnings.
  - **The mean colour** of the pixels that differ is 0.92 to 1.07 of the
    original's.
  - **Frame times** at b1: 8.53 ms, against 8.54 and 8.69.
  - **Loading:** the first update comes 3.0 s later (7.25 to 7.32 s
    against 4.29). G1's model pack takes 2.70 to 2.72 s:
    - read and decode 0.90 s (PNG decoding 0.17);
    - mipmaps and detail maps 0.71 to 0.73 s (the longest job 0.53);
    - parts 1.08 to 1.09 s (the light carried 0.97 to 0.98).
  - **The simulation hashes** over the 600-update G1 walk, with the pack,
    equal the reference in the defaults and the baseline.
- **The look** (the art folder's G1_Rest_Area_B_Views.png and
  G1_Rest_Area_B_Closeups.png):
  - The banks, the bridge approaches and the rock wall sit well.
  - **The meadow has lost the original's terraces.** The original
    GRASS_LAND shows distinct steps: dark edges, mossy tops and red-brown
    earth between them. The new grass is even and fine-grained, with
    only faint wide bands, and a little browner.
  - The checker passes it, because the contrast it measures is fine
    grain, not large-scale structure.

Not yet: the owner's choice on the meadow (as it is, or a fix round for
G1#018's texture), the install, area C.
