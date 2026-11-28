# 0054: G1's area C (the farm), checked (models-plan M6)

Area C of the rest of G1's brief (decisions/0041) came back on
2026-11-27. The art folder's G1_Ground/review_pack/G1 adds:
- 7 pieces: the farmyard (G1#065), the ground west of the barn (G1#019),
  the rise to the hills (G1#020), the rock slope west of the barn
  (G1#016), and the rock walls west, north and east (G1#051, G1#057,
  G1#060);
- textures/yard.png (2048 px) and textures/rock_farm.png (1024 px);
- one materials.tsv line, G1GROUND_yard ground. The rock takes area B's
  G1GROUND_rock stone.

ChatGPT's checks, maps, renders and notes are beside the pack.

Checked here (2026-11-28, the owner's PC):
- **The rest of the pack is unchanged:** the 44 installed files (the
  start area, areas A and B) are byte-identical to the pack's.
  materials.tsv differs by the one line.
- **The colour checker** (the corrected one, decisions/0048; the art
  folder's copy is identical to tools/blender/check_colour.py): its
  output over the 40 pieces is byte-identical to the hand-off's.
  - Area C's pieces are all within: the colour 0.93 to 1.08 of the
    originals'.
  - The contrast at the camera: the dirt 14 % against the originals' 12
    to 15 %, the rock 29 to 35 % against 29 to 34 %.
  - It marks 11 pieces of the start area and areas A and B, accepted
    before the measure changed. The brief keeps them as they are.
- **The ground checker:**
  - G1#065 is 99.6 % within and G1#019 96.4 %, the same as their
    originals. The rest is collision above them (per the hand-off, the
    ramps and fixtures over the dirt). G1#065 is never above the
    collision by more than 0.068.
  - G1#020 is 100 % within, where its original was 92.5 %. G1#016 is 100
    %. G1#051 is 100 %, where its original was 9.3 %.
  - No holes.
  - Standing in the way: G1#020 at 5 spots, its original's own (the
    original run through the checker stands in the same 5), as the brief
    allows. G1#051 at none, where its original stood in 3. The others at
    none.
- **The files:**
  - 526,739 triangles (G1#065 159,229; G1#051 123,315).
  - Longest edges 0.508 to 0.515; no zero-area triangles; no extensions
    or animation; opaque RGB textures.
  - G1#051 repeats 54 triangles (21 groups, in the same material). That
    is harmless, a note for the next hand-off.
- **The textures' colours** (a scratch measure of the texels' red minus
  green):
  - The disc's GROUND_01 is green turf with patches of bare earth, pink-
    brown and red-brown. 24 % of its texels are clearly green and 40 %
    clearly brown.
  - yard.png is in the start area's path family, as the brief asks, and
    drier: 5 % green and 33 % brown (path.png: 18 and 23 %). The means
    are the same, 49, 47, 23.
- **In the game,** with the installed build (5b8aaffd) in the scratch
  copy, the original ground against the pack (the start area and areas A
  to C), at c1 to c4 (decisions/0041):
  - All 40 pieces load with the light carried over, and there are no
    messages.
  - **The mean colour** of the pixels that differ is 0.96 to 1.17 of the
    original's at c2 to c4 and the top of c1. The lower half of c1 is
    0.85, 0.90, 0.78: the original's red-brown earth there, about 8
    levels.
  - **The green:** of the pixels that differ, 11 % are green at c2
    against the original's 37 %, 1.6 against 8 % at c1, 7 against 14 % at
    c4. The brightness contrast is the same or higher (44 to 50 % against
    36 to 49 %).
  - **Frame times** at c1 (vsync off, the median over frames 1000 to
    3000): 8.46 and 10.30 ms, against 9.16 and 9.84 with the installed
    G1 pack and 9.96 and 10.40 without one. No difference beyond the
    spread.
  - **Loading:** the first update comes at 6.92 to 6.95 s, against 6.51
    to 6.65 with the installed pack and 4.21 to 4.48 without. G1's model
    pack takes 2.50 to 2.66 s (the installed one 2.07 to 2.14).
  - **The simulation hashes** over the 600-update G1 walk, with the pack
    (and the gargoyle pieces, decisions/0055), equal the reference in
    the defaults and the baseline.
- **The look** (the art folder's G1_Ground/G1_Rest_Area_C_Views.png,
  G1_Rest_Area_C_Closeups.png and G1_Area_C_Yard_Texture.png):
  - The farmyard is sharper: grass tufts, straw and worn bare patches.
    It is olive where the original's is green turf with red-brown earth.
    The wheel ruts are faint from the camera.
  - The rock walls have modelled plates with moss along the top. From
    the camera the western wall (G1#051) reads softer than the
    original's crisp crack lines.

Not yet: the owner's choice (the install, or a fix round), area D.
