# 0041: The rest of G1's ground, the brief (models-plan M6)

After the start area was accepted (decisions/0040), the owner chose "Yes,
the rest of G1" (2026-11-27): a brief for the rest of G1's ground, in the
start area's way.

What was found (2026-11-27, the model export's levels/G1.glb and objects/G1,
scratch measurements):
- **G1's pieces:** the scene has 114 named pieces (G1#nn and G1FLOORnn).
  - 53 are terrain: the start area's 8, and 45 more.
  - The rest are walls, buildings, the barn, fences, planks, platforms
    and doors; the corn; and the swamp and the rapids.
- **The 45 terrain pieces** use GROUND_01, FIELD_02, GRASS_LAND,
  HILL_GROUND, SLOPE_01, GROUNDSIDE_01, RIVER_BANK_03 and ROCK_WALL_02.
  - Together they cover about 113,800 square units, against the start
    area's 5,200.
  - All are opaque, alpha class 0, with no vertex alpha.
- **Walked on** (the checker run with each original as the new piece):
  - 20 have the walkable collision on their surface (1,296 to 184,698
    samples each).
  - 8 have a little of it under them (3 to 811 samples): G1#038, G1#040,
    G1#046, G1#078, G1#016, G1#048, G1#049 and G1#051. G1#038 and
    G1#016 lie on it there (100 % within); the others are where the
    collision passes by.
  - 4 have none under or beside them: G1#042, G1#041, G1#083 and G1#059,
    backdrop that heroes cannot reach.
  - The other 13 are free.
  - Some originals already depart from the collision: G1FLOOR#36 80.5 %
    within, G1#020 92.5 %, G1#054 88.2 %, G1#055 89.5 %.
  - A few already stand in the way: G1#020 at 5 samples, G1#051 at 3,
    G1#048 at 1.
- **The lightmaps** have 0.44 to 1.15 units a texel, so a vertex spacing
  of 0.5 follows them, as in the start area. That is about 10 triangles a
  square unit, about 1.05 million for the 45 pieces: G1#065 about
  134,000, G1#018 about 107,000.
- **The game's material types by name** (material_maps::guess):
  - GROUND_01, GROUNDSIDE_01 and HILL_GROUND are ground.
  - GRASS_LAND is grass, and ROCK_WALL_02 is stone.
  - FIELD_02 and SLOPE_01 match no word, so they are auto.
  - RIVER_BANK_03 is water, from the word RIVER: no bump and a wet shine
    on the original river banks with material detail on.

    That is likely a misreading, since the texture is mostly earth (its
    foot shows the water). It is not changed here: a possible follow-up
    for the name rules (NATIVE; baseline unaffected).

The test of the cost (2026-11-27, the build of decisions/0037, 65051ea4, in
the scratch copy of the game):
- **The setup:** the 45 originals subdivided to edges of at most 1.0
  (791,000 triangles, about what an even 0.5 mesh takes), with the
  installed start area pack (95,000). That is 53 files in model_pack\G1.
- **The views:** twelve spots across G1, teleported at update 201,
  frozen at 300, frame 1500: four in the fields and the west, two on the
  grassland, four on the farm, two on the hills.
- **Every file loads** with the light carried over, and there are no
  warnings.
- **The control:** the mean of the pixels that differ from the original
  is 0.97 to 1.00 in most views and channels. The exceptions:
  - The field under the corn (the lower half at a1): 0.87 to 0.88 in red
    and green, 0.75 in blue, the same with the field pieces subdivided
    to 0.5. With material detail off, only 1 to 3 % of the pixels differ
    there at all (1.03 to 1.07 of a few levels). So the gap is the
    material detail: the test's materials are unlisted and get none,
    while the original FIELD_02 gets the auto type's.
  - Blue on the grassland: 0.83 to 0.88, about 1 level of 7 to 9, the
    same at 0.5 spacing (red and green 0.96 to 0.98).
- **Frame times** (vsync off, the median over frames 1000 to 3000):

  | Spot | Without the pieces | With them |
  | --- | --- | --- |
  | a1 | 9.03 ms | 9.33 ms |
  | c1 | held at 60 Hz, not counted | 8.66 ms |
  | c5 | 8.71 ms | 8.68 ms |

- **Loading:** the first update comes 2.4 to 3.0 s later (6.7 to 7.5 s
  against 4.3 to 4.5). G1's model pack takes 2.05 to 2.33 s:
  - read and decode 0.83 to 0.91 s;
  - mipmaps and detail maps 0.59 to 0.75 s;
  - parts 0.63 to 0.68 s.

  The art's larger textures will add some.

The brief (docs/remaster/art-briefs/g1-rest-ground.md; the art folder's
G1_Ground/G1_Rest_Ground_Brief.md):
- **The pictures:** G1_Rest_Area_A.png to G1_Rest_Area_D.png (each
  area's map by kind, textured, and the original at its spots) and
  G1_Rest_Map.png.
- **The originals:** the 45 added to private_export/objects/G1, as the
  start area's. They are the same files as the full export of
  decisions/0021.
- **Four areas,** handed back one at a time, A first, each checked in the
  game before the next:
  - A: the fields and the west, 17 pieces, about 24,600 square units;
  - B: the grassland and the river, 8 pieces, about 30,800;
  - C: the farm, 7 pieces, about 33,200;
  - D: the hills, 13 pieces, about 25,200.
- **The surfaces:**
  - The walked-on dirt keeps the start area's path; the farmyard may get
    a variant.
  - New materials: field, grass, hill, river bank, rock.
  - The slopes, sides and river banks get the start area's gullies; the
    rock walls fractured plates, not level strata.
  - The types are listed for materials.tsv.
- **The rules:** as the start area's, and:
  - keep the ground's height to the original's within about 1 unit of
    walls, buildings, fences, platforms and doors;
  - seams with the installed start area;
  - the originals' stand-in-the-way spots no worse;
  - the river banks at least 1 unit below the rapids;
  - vertices at most 0.5 apart, 1.0 on the unreachable backdrop;
  - no per-piece triangle cap.
- **Not in this brief:** the walls, buildings, barn, fences, planks,
  platforms and doors; the corn; the swamp and the rapids.

Not yet: the art, area by area; its check; the install; the owner's
playtest.

Later (2026-11-27): RIVER_BANK_03 is now typed as ground
(decisions/0042).
