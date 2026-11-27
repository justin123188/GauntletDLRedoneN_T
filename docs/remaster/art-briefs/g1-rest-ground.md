# Brief: the rest of G1's ground

The start area is done and in the game (`review_pack/G1`, the gullies
round). The owner liked it, so the same approach now covers the rest of
G1: **45 more ground pieces**, from the swamp's edge north through the
fields, the grassland and the river, the farm, and up to the hills.

The work goes **area by area**, in four areas, A first. Hand each area
back on its own. We check it in the game, and any notes on it carry into
the next area.

## What carries over from the start area

- **The light:** the game puts G1's own baked light on the new ground
  through the original's lightmap pass, with ambient occlusion. Paint the
  shading of your own shapes into the textures: dark in recesses and
  cracks, lighter on tops and edges. Paint no light of the level's own:
  no cast shadows or torch glow.
- **The colour:** each piece's mean colour within 10 % of its original's
  per channel, and the contrast at the camera at least 80 % of the
  original's. `check_colour.py` measures both. It takes the whole folder,
  so new surface kinds are checked against their own originals with no
  extra targets.
- **The walked-on ground:** within 0.10 of the collision, with no holes
  (`check_ground.py`). The relief lives in the texture; the shape stays
  gentle.
- **The banks and sides:** the start area's style, vertical gullies down
  the fall line with a mossy, overhanging lip over a dark undercut (the
  owner's choice). Keep level layers for another realm.
- **The files:** each piece in its original's frame and file name, the
  shared `textures/` folder, opaque PNGs (1024 or 2048), the base colour
  factor at 1, no vertex colours, no repeated or zero-area triangles, and
  `G1GROUND_<surface>` material names in `materials.tsv`.

## The four areas

Pictures: `G1_Rest_Area_A.png` to `G1_Rest_Area_D.png`. Each holds:
- the area's map: green walked on, orange free, blue the start area,
  grey not in this brief;
- the same map textured;
- the original in the game at a few spots in the area.

`G1_Rest_Map.png` shows the whole level. The originals are in
`private_export/objects/G1/` (all 53 pieces now), and the level scene
with the collision is `private_export/levels/G1.glb`.

Game coordinates: x across, y up, and z more negative toward the hills.
In the pictures and the tables, "north" is toward the hills (up on the
maps) and "east" is toward positive x (right on the maps).

What the tables mean:
- **Walked on:** the collision lies on the piece's surface, so the
  checker's tolerance applies there.
- **Free:** nobody walks on it, so it can take real shape.
- **Within:** the checker's share of samples within the tolerance on the
  original itself. Where the original already departs from the
  collision, the collision is the truth: keep to it.

### Area A: the fields and the west (z +15 to -140), 17 pieces, about 24,600 square units

| Piece | What it is | Texture now | Walked on | Where: game x, y, z |
| --- | --- | --- | --- | --- |
| G1#017 | the ground north of the start | GROUND_01 | yes (27,386 samples, 100 % within) | -6.5 to 36.1, 10.0 to 10.0, -94.8 to -39.7 |
| G1FLOOR#10 | the ground at the top of the ramp, east | GROUND_01 | yes (7,613 samples, 100 % within) | 45.3 to 79.5, 10.0 to 10.0, -39.7 to -21.2 |
| G1FLOOR#37 | the higher ground, west end | GROUND_01 | yes (5,901 samples, 100 % within) | -48.8 to -6.5, 10.0 to 10.0, -39.7 to -26.1 |
| G1#007 | the west ground, by the river | GROUND_01 | yes (45,782 samples, 99.5 % within) | -53.2 to -6.5, 10.0 to 10.0, -136.0 to -39.7 |
| G1#035 | the ground west of the big field, toward the grassland | GROUND_01 | yes (16,247 samples, 100 % within) | -6.5 to 57.8, 10.0 to 10.0, -136.0 to -94.8 |
| G1FLOOR#36 | the strip west of the field | GROUND_01 | yes (1,328 samples, 80.5 % within) | 36.1 to 57.8, 10.0 to 10.0, -94.8 to -75.7 |
| G1FLOOR#17 | the small ramp down to the wooden platform | SLOPE_01 | yes (1,296 samples, 100 % within) | -0.0 to 10.6, 7.3 to 10.0, -120.1 to -109.3 |
| G1#006 | the big ploughed field, under the corn | FIELD_02 | yes (46,925 samples, 100 % within) | 57.8 to 118.9, 10.0 to 10.0, -122.1 to -63.7 |
| G1#064 | the field, south part, under the corn | FIELD_02 | yes (13,909 samples, 100 % within) | 57.8 to 118.9, 10.0 to 10.0, -63.7 to -39.7 |
| G1FLOOR#03 | the field strip west of the corn | FIELD_02 | yes (9,729 samples, 100 % within) | 36.1 to 57.8, 10.0 to 10.0, -75.8 to -39.7 |
| G1#033 | the slope east of the low ground (carries G1#028 on) | SLOPE_01 | no | 76.9 to 156.3, -0.1 to 10.0, -28.4 to -26.1 |
| G1#034 | the side into the swamp, east (carries G1#030 on) | GROUNDSIDE_01 | no | 78.9 to 155.8, -9.0 to 0.0, -27.7 to -25.0 |
| G1#040 | the slope from the west ground down to the river | SLOPE_01 | a little (42 samples, 11.9 % within) | -55.6 to -47.1, 0.4 to 10.0, -106.5 to -30.2 |
| G1#082 | the river bank, near side | RIVER_BANK_03 | no | -59.4 to -52.7, -2.2 to 6.0, -130.5 to -30.2 |
| G1#083 | the river bank, far side | RIVER_BANK_03 | no, and none beside it | -69.1 to -63.4, -2.2 to 6.0, -130.5 to -30.2 |
| G1#041 | the slope up from the far bank | SLOPE_01 | no, and none beside it | -75.6 to -68.0, 0.4 to 10.0, -130.6 to -30.2 |
| G1#042 | the backdrop ground beyond the river | GROUND_01 | no, and none beside it | -126.1 to -68.0, 10.0 to 10.0, -130.6 to -30.2 |

### Area B: the grassland and the river (z -105 to -275), 8 pieces, about 30,800 square units

| Piece | What it is | Texture now | Walked on | Where: game x, y, z |
| --- | --- | --- | --- | --- |
| G1#018 | the grassland | GRASS_LAND | yes (184,698 samples, 98.2 % within) | -30.6 to 134.6, 17.1 to 25.6, -212.2 to -110.5 |
| G1#038 | the ground east of the grassland | GROUND_01 | a little (521 samples, 100 % within) | 115.9 to 169.1, 17.1 to 25.6, -214.4 to -125.8 |
| G1#046 | the slope down to the river | SLOPE_01 | a little (295 samples, 6.4 % within) | -69.3 to 166.4, 11.0 to 25.5, -251.3 to -136.0 |
| G1#078 | the side down to the river | GROUNDSIDE_01 | a little (160 samples, 5 % within) | -3.3 to 94.0, 16.2 to 25.2, -217.5 to -183.6 |
| G1#080 | the river bank, by the side | RIVER_BANK_03 | no | -3.3 to 94.0, 12.9 to 19.7, -217.5 to -186.4 |
| G1#081 | the river bank, long | RIVER_BANK_03 | no | -69.3 to 166.4, 5.9 to 18.1, -245.6 to -133.7 |
| G1#056 | the backdrop ground west of the river | GROUND_01 | no | -121.4 to -23.5, 10.0 to 25.5, -271.9 to -130.6 |
| G1#059 | the rock wall at the far west | ROCK_WALL_02 | no, and none beside it | -121.4 to -104.6, 17.6 to 49.0, -271.9 to -162.0 |

### Area C: the farm (z -200 to -345), 7 pieces, about 33,200 square units

| Piece | What it is | Texture now | Walked on | Where: game x, y, z |
| --- | --- | --- | --- | --- |
| G1#065 | the farmyard | GROUND_01 | yes (112,028 samples, 99.6 % within) | -58.7 to 168.3, 25.5 to 26.9, -324.9 to -202.7 |
| G1#019 | the ground west of the barn | GROUND_01 | yes (42,041 samples, 96.4 % within) | -68.4 to 60.3, 28.8 to 35.5, -340.7 to -263.9 |
| G1#020 | the rise to the hills | GROUND_01 | yes (49,436 samples, 92.5 % within) | -70.3 to 40.3, 29.0 to 42.7, -381.1 to -272.4 |
| G1#016 | the rock slope west of the barn (its foot walked on) | ROCK_WALL_02 | a little (811 samples, 100 % within) | -62.9 to 37.0, 29.0 to 42.7, -344.5 to -283.1 |
| G1#051 | the rock wall along the farm, west | ROCK_WALL_02 | a little (43 samples, 9.3 % within) | -107.1 to -47.2, 20.9 to 63.4, -350.6 to -262.4 |
| G1#057 | the rock wall north of the barn | ROCK_WALL_02 | no | 34.1 to 70.9, 29.0 to 65.4, -344.5 to -306.5 |
| G1#060 | the rock wall along the farm, east | ROCK_WALL_02 | no | 117.6 to 166.4, 25.5 to 46.2, -324.9 to -251.3 |

### Area D: the hills (z -345 to -505), 13 pieces, about 25,200 square units

| Piece | What it is | Texture now | Walked on | Where: game x, y, z |
| --- | --- | --- | --- | --- |
| G1#047 | the hill | HILL_GROUND | yes (45,710 samples, 95.7 % within) | -71.6 to 37.0, 47.0 to 63.4, -410.3 to -349.7 |
| G1#054 | the path from the hill up toward the hilltop | HILL_GROUND | yes (8,542 samples, 88.2 % within) | -50.8 to -1.1, 63.4 to 74.8, -432.0 to -391.2 |
| G1#055 | the hilltop | HILL_GROUND | yes (43,890 samples, 89.5 % within) | 10.4 to 123.6, 74.8 to 80.4, -503.0 to -425.9 |
| G1FLOOR#38 | a hilltop step | HILL_GROUND | yes (2,535 samples, 97.7 % within) | 98.9 to 120.5, 80.4 to 84.7, -459.0 to -449.7 |
| G1FLOOR#39 | a hilltop step, highest | HILL_GROUND | yes (2,221 samples, 100 % within) | 111.0 to 123.6, 86.2 to 86.2, -481.3 to -465.5 |
| G1FLOOR#40 | a hilltop step | HILL_GROUND | yes (4,070 samples, 100 % within) | 73.4 to 86.3, 80.4 to 83.7, -484.3 to -452.8 |
| G1#052 | the high ledge above the hill (not walked on) | GROUND_01 | no | -16.1 to 60.6, 75.4 to 85.7, -408.4 to -364.8 |
| G1#058 | the low ledge under the hilltop (not walked on) | GROUND_01 | no | 14.0 to 122.0, 48.4 to 54.0, -462.7 to -421.8 |
| G1#048 | the rock wall west and north of the hill | ROCK_WALL_02 | a little (60 samples, 10 % within) | -75.2 to -4.0, 63.4 to 96.4, -434.9 to -345.8 |
| G1#049 | the rock wall from the low ledge up to the hilltop | ROCK_WALL_02 | a little (3 samples, 66.7 % within) | 10.4 to 119.9, 48.4 to 80.4, -465.7 to -423.9 |
| G1#050 | the rock wall from the hill up to the high ledge | ROCK_WALL_02 | no | -16.1 to 37.0, 47.7 to 79.2, -402.9 to -364.8 |
| G1#053 | the rock face from the rise up to the hill | ROCK_WALL_02 | no | -52.6 to 27.9, 42.5 to 63.4, -380.4 to -349.7 |
| G1#066 | the rock wall behind the hilltop | ROCK_WALL_02 | no | 10.4 to 81.9, 74.8 to 102.3, -503.0 to -435.2 |

## The surfaces

Reuse the start area's textures where the surface is the same. Add a new
`G1GROUND_<surface>` material and texture for each new kind. In
`materials.tsv`, use `ground` for earth and dirt, `grass` for turf, and
`stone` for rock:

| Material | Type |
| --- | --- |
| `G1GROUND_yard`, `G1GROUND_field`, `G1GROUND_riverbank` | ground |
| `G1GROUND_grass`, `G1GROUND_hill` | grass |
| `G1GROUND_rock` | stone |

- **Dirt** (GROUND_01 walked on: G1#007, G1#017, G1#035, G1FLOOR#10,
  G1FLOOR#36, G1FLOOR#37, G1#019, G1#020, G1#065 and G1#038):
  `G1GROUND_path`, the start area's
  path texture and colour, which the owner liked.
  - The farmyard (G1#065, G1#019, G1#020) may get its own variant,
    `G1GROUND_yard`: more trodden, with wheel ruts and straw in the
    texture, in the same colour family.
- **The ploughed field** (FIELD_02: G1#006, G1#064, G1FLOOR#03):
  `G1GROUND_field`.
  - Furrows of dark, turned earth with clods and small stones, running
    the same way as the original's.
  - The fields are walked on, so the furrows live in the texture
    (painted shading), not the shape.
  - The corn stays the original's (plants come later).
- **The grassland** (GRASS_LAND: G1#018): `G1GROUND_grass`.
  - Rough swamp meadow over olive earth, with damp patches and worn
    tracks.
  - The original's low ridges and terraces go in the texture.
  - It is walked on, so keep its rolling shape to the collision.
- **The hills** (HILL_GROUND: G1#047, G1#054, G1#055, G1FLOOR#38 to #40):
  `G1GROUND_hill`. Mossy turf with grey stone breaking through, as in the
  original. Walked on.
- **The slopes** (SLOPE_01: G1#033, G1#040, G1#041, G1#046, and
  G1FLOOR#17, which is walked on): as G1#028 in the start area, mossy
  earth with shallow gullies, roots and vines running down it.
- **The earth sides** (GROUNDSIDE_01: G1#034 into the swamp, G1#078
  down to the river): the start area's gullies banks, with the same bank
  and gully textures. G1#034 carries G1#030 on to the east; match it at
  the seam.
- **The river banks** (RIVER_BANK_03: G1#080, G1#081, G1#082, G1#083):
  `G1GROUND_riverbank`.
  - Soft earth banks cut down to the river, with gullies, roots and a
    mossy lip, as the swamp sides.
  - At its foot, under the rapids' surface, the original's texture turns
    into the water's dark blue. Do the same: the earth goes dark and wet,
    then dark blue-black, below the waterline.
  - Carry the banks at least 1 unit below the river's surface. The
    rapids (G1#075 to G1#077) stay the original's.
- **The rock walls** (ROCK_WALL_02: G1#016, G1#048 to G1#051, G1#053,
  G1#057, G1#059, G1#060, G1#066): `G1GROUND_rock`.
  - Real shape: big fractured plates of grey-green rock split by deep
    cracks, with moss along the top edge and in the cracks, as in the
    original. Not level strata.
  - G1#016 is partly walked on, at its foot: keep that part to the
    collision.
- **The backdrop ground** (GROUND_01 that nobody reaches: G1#042,
  G1#056, G1#052, G1#058): `G1GROUND_path` or the grass, matching what
  each borders. Free to shape, for example gentle mounds and banks.

## Rules

- **Next to walls, buildings, fences, platforms and doors** (all stay
  the original's): within about 1 unit of them, keep the ground's height
  to the original's (within 0.10). Then nothing pokes through a wall or
  floor, and no gap opens at a foot.
- **Seams:** pieces meet each other, and the installed start area pieces,
  with no gaps. Keep each border where the original's is.
- **Nothing over the path:** the checker's "standing in the way" stays
  at 0, apart from spots where the original already stands in the way.
  Keep those no worse: G1#020's 5, G1#051's 3 and G1#048's 1.
- **Into the water:** G1#034 ends below the swamp's surface (y -9.2).
  The river banks and G1#078 go at least 1 unit below the rapids.
- **Density:**
  - Vertices at most about 0.5 units apart. That follows the lightmap,
    which has 0.44 to 1.15 units a texel on these pieces.
  - That is about 10 triangles a square unit, so the big pieces take
    many: G1#065 about 135,000 and G1#018 about 105,000. That is fine.
    We measured it with G1's own pieces subdivided to about that count,
    890,000 triangles in all: the frame times stayed the same, and G1
    took about 3 s longer to load.
  - The pieces heroes cannot reach may go to 1.0 apart: G1#042, G1#041,
    G1#083 and G1#059, which have no collision under or beside them.
- **Not in this brief:**
  - the walls, buildings, barn, fences, planks, platforms and doors;
  - the corn;
  - the swamp and the rapids.

## The checkers

As in the start area, run them on each area before hand-off:

    python check_colour.py private_export/objects/G1 review_pack/G1
    python check_ground.py private_export/levels/G1.glb <piece> review_pack/G1/<piece>.glb --map <piece>_map.png --report <piece>_check.json

## What to hand back, per area

- **The area's files**, added to `review_pack/G1` (the start area's stay
  as they are), with any new textures and `materials.tsv` lines.
- **The checkers' output:**
  - `check_colour.py` on the whole pack ("all within");
  - `check_ground.py`'s reports and maps for the area's pieces.
- **Renders** from about the game's camera at the area's spots, beside
  the original, as in the area picture.
- **A short note** of what changed.

We will check each area in the game here.
