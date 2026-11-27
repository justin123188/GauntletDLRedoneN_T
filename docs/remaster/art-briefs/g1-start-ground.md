# Brief: G1's start area ground (the M6 test)

For an art tool that works in Blender, as the chest, the heaps and the
treasure pickups. The first terrain of the remaster: new ground for the
area around the hero's start in G1 (the swamp), its shape as well as its
textures. It is a test of the whole way of doing it; after it come the
rest of G1 and, later, each level in its own overhaul.

The game keeps its own collision, the invisible floor heroes walk on. So
where heroes walk, the new ground must lie on that floor; everywhere else
it can take real shape. A checker (check_ground.py, in the art folder's
G1_Ground) measures both, and should run on every piece before hand-off.

## The pieces

Eight of the level's ground objects, around the start (the art folder's
G1_Ground: `G1_Start_Area_Map.png`, green the walked-on pieces, orange the
others, grey the rest of the level; `G1_Start_Area_Textured.png`; and
`G1_Start_InGame.png`, the game's own view at the start). Game
coordinates (x across, y up, z front to back; the map has x to the right
and z down):

| Piece | What it is | Triangles now | Texture now | Walked on | Where (game x, y, z) |
| --- | --- | ---: | --- | --- | --- |
| G1#005 | the ground around the start | 47 | GROUND_01 | yes | x -8.9 to 38.8, y 0, z -26.3 to 13.8 |
| G1FLOOR#02 | the ramp up to the higher ground | 10 | GROUND_01 | yes | x 17.7 to 49.9, y 0 to 10, z -32.3 to -16.1 |
| G1#004 | the higher ground | 19 | GROUND_01 | yes | x -18.2 to 57.8, y 10, z -39.7 to -23.1 |
| G1#000 | the side dropping from the start ground into the swamp | 40 | GROUNDSIDE_01 | no | x -14.2 to 44.4, y -9.1 to 0, z -28.2 to 20.2 |
| G1#001 | the side rising from the start ground to the higher ground | 26 | GROUNDSIDE_01 | no | x -25.9 to 45.3, y 0 to 10, z -32.3 to -23.1 |
| G1#028 | the slope above the low ground | 15 | SLOPE_01 | no | x 19.8 to 78.9, y -0.7 to 10, z -27.8 to -16.1 |
| G1#029 | the low ground by the swamp | 15 | GROUND_01 | no | x 19.8 to 79.5, y -0.7 to 1.9, z -27.7 to -14.5 |
| G1#030 | the side from the low ground into the swamp | 16 | GROUNDSIDE_01 | no | x 19.8 to 80.9, y -9.1 to 1.9, z -27.2 to -12.9 |

The files: `private_export/objects/G1/<piece>.glb`, each in its own frame;
`private_export/levels/G1.glb`, the whole level as placed, with the
collision as COL_ nodes (exported from the owner's disc: keep them local).
Work in each piece's own frame, or in the level scene and bring each piece
back to its own frame (the scene's node of that name places it). The
export's frame mirrors the game's x; the checker takes care of that.

Not in this round: the swamp water (G1#063, G1#079: the level's whole
swamp surface at y -9.2, which has its own water effect), and the plants
(the corn, moss and marsh cards stay the original's for now).

## The look

- **A swamp-edge island** of a modern remaster of an early-2000s
  dark-fantasy arcade game, rich, chunky and hand-crafted, in G1's palette
  (dark greens and browns, `G1_Start_InGame.png`).
- **The walked-on ground:** packed dirt and moss, worn paths, roots and
  stones pressed flat into it, damp patches; the relief there in the
  texture (the game also gives it bumps, below), the shape only gently.
- **The sides and banks:** real shape: layered mud and earth, roots
  breaking out, rocks and stones, an undercut edge or two, slumped mud
  where they meet the swamp; the side up to the higher ground an earthen
  wall with roots and rocks.
- **The light:** the game puts G1's own baked light on the new ground (each
  new vertex takes the light of the original's surface there: shadows,
  torchlight and darker corners stay where they are). So paint no light
  or shadows into the textures: base colour only, darker in crevices at
  most.

## Where heroes walk: G1#005, G1FLOOR#02, G1#004

- **On the collision:** the collision lies exactly on these pieces'
  surfaces now. The new surface stays within 0.10 of it wherever heroes
  walk (the checker's tolerance): ruts, stones and roots no higher or
  deeper than that; the rest of the relief in the texture.
- **The edges:** where these meet each other, the sides and the rest of
  the level, keep their border where the original's is, so no gap opens.

## Where nobody walks: G1#000, G1#001, G1#028, G1#029, G1#030

- **Free to shape**, within about the original's footprint.
- **Not over the path:** nothing of them may stand over the walked-on
  ground beside them (the checker's "standing in the way": 0), and their
  tops meet the walked-on pieces' edges.
- **G1#001's foot** already stands up to 1.43 above the path at a few spots
  (the checker counts 11 samples): keep it no worse.
- **Into the swamp:** the sides end at y -9.1, at the waterline (the swamp
  surface is at about y -9.2); carry them on down below it, so no gap
  shows at the water.

## Density, size and textures

- **Vertices at most about 0.5 units apart** on every surface seen in
  play: the original's light is carried to the vertices, and its lightmap
  has about one texel a unit, so the light follows it at that spacing.
- **Up to about 20,000 triangles a piece** (tell us if a piece needs more).
- **Tiling base colour textures**, PNG, opaque, repeating, 1024 or 2048
  px, shared between the pieces where they are the same surface.
- **Materials named `G1GROUND_<surface>`:** `G1GROUND_path` (the walked-on
  dirt and moss), `G1GROUND_bank` (the earth and mud of the sides),
  `G1GROUND_stone`, `G1GROUND_root` and `G1GROUND_moss`, so the game can
  give each its bumps and shine (path and bank as ground, stone as stone,
  root as wood, moss as grass).

## The checker

In Blender's own Python, or with Blender (`blender --background
--factory-startup --python check_ground.py -- ...`):

    python check_ground.py private_export/levels/G1.glb G1#005 new/G1#005.glb --map G1#005_map.png --report G1#005_check.json

It samples the walkable collision every 0.25 units. Under a piece it
reports where the new surface is above the collision (feet would sink),
below it (they would float) or missing (a hole), against the original
measured the same way; beside it, where the new surface stands over
walkable ground. The map: green within the tolerance, red above, blue
below, black a hole, orange standing in the way, dark green walkable
ground beside it. The goal: the walked-on pieces 100 % within, no holes;
every piece 0 standing in the way (G1#001's foot as the original's).

## What to hand back

1. The eight files under the originals' names: `G1#000.glb`, `G1#001.glb`,
   `G1#004.glb`, `G1#005.glb`, `G1FLOOR#02.glb`, `G1#028.glb`, `G1#029.glb`
   and `G1#030.glb` (they go into the model pack's G1 folder).
2. The checker's report and map for each.
3. Renders: the area from above (as `G1_Start_Area_Map.png`) and from the
   game's camera at the start (as `G1_Start_InGame.png`), the new beside
   the original.
4. The materials named as above.

For the game to read the files (gdl_client::model_pack): one buffer inside
each .glb, triangle primitives with positions, normals and UVs, base
colour textures as PNGs (opaque), no mesh compression (Draco, meshopt), no
texture transforms, no animation.


## Delivered

2026-11-26: ChatGPT's eight pieces in the art folder's
G1_Ground/review_pack/G1; checked in decisions/0036 (sound for the game:
the collision, the light, the hashes and frame times; lighter and greyer
than G1, the sides plainer from the game's camera).
