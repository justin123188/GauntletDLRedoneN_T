# Brief: the junk chest's props

For an art tool that works in Blender, as the chest, its heaps and the
treasure pickups. The junk chest (CHESTG0) holds three props of its own
besides its heap: a broken cart wheel, a brick corner and a bent metal
pipe. They are still the original's low-poly props (the heaps brief kept
them). Remake them with the junk ChatGPT made for the junk pickup, so the
chest and the pickup show the same junk.

When the chest opens, its animation (CHESTG0's tree in the item package)
makes the props spring out of it: the wheel rises 1.29 units, the brick
0.61, and the pipe rises 1.29 while it tips forward about 90°, ending
sticking out toward the front. Shut, all three lie inside the chest, under
the lid.

## The look

- **The same junk as the pickup's:** the pipe and the brick corner of
  `TREAS_JUNKGR#0.glb`, the broken wheel of `TREAS_JUNKGR#1.glb` (the art
  folder's Treasure_Pickups/review_pack/POWERUPS), with their materials and
  texture, rebuilt to the chest props' shapes below.
- **The style** as before: a modern remaster of an early-2000s dark-fantasy
  arcade game, rich, chunky and hand-crafted; junk dull and dark. Bright
  rims and darker gaps painted into the base colour; no strong shadows or
  a scene's light baked in.

## Where they go (the numbers)

Each prop in the local frame of its original export file (game units; x
across, y up, z front to back; the export's frame, which negates the game's
x). The files: `private_export/objects/ITEMS_G/CHESTG0GROUP3.glb`,
`CHESTG0GROUP4.glb` and `CHESTG0GROUP5.glb` in the art folder's
Junk_Chest_Props, exported from the owner's disc (keep them local).

**CHESTG0GROUP3, the broken wheel:** a broken wheel standing upright, made
of two crossed planes (x -0.844 to 0.844, y -0.898 to 0.898, z -0.453 to
0.453) with pieces of its rim (x -0.875 to 0.859, y -0.578 to 0.656, z
-0.531 to 0.547).

**CHESTG0GROUP4, the brick corner:** three faces of a brick corner, within
x -0.008 to 1.266, y 0.617 to 1.648, z -0.641 to 0.516.

**CHESTG0GROUP5, the pipe:** a pipe in three straight runs, within x -0.266
to 0.172, y -0.078 to 1.719, z -0.938 to 0.102: the foot from y -0.08 to
0.44 (z -0.67 to 0.10), the middle from y 0.35 to 1.09 (z -0.94 to -0.51),
the top from y 1.06 to 1.72 (z -0.94 to -0.66), its open end at the top.

**Keep each prop within its original's bounds** (thinner or shorter is
fine, not bigger): then the chest still closes over them. Shut (the tree's
CLOSED pose), in the frame of the gold-chest base the heaps brief used (x
across, y up from the rim, z from the hinge, 0, to the lock, 2.25), the
originals lie at:
- the wheel: x -1.47 to 0.26, y -1.45 to 0.35, z 0.73 to 1.81;
- the brick: x -0.01 to 1.27, y -0.81 to 0.22, z 0.62 to 1.78;
- the pipe: x 0.59 to 1.02, y -1.42 to 0.38, z 1.01 to 2.05;
and the new lid leaves at most y 0.38 in the middle (z 0.5 to 1.75) and
0.18 near the front and back (z below 0.4 or above 1.85).

For a preview of the chest open: the props' nodes move from the chest's
tree root (the base sits at (0, 1.427, -1.261) from it). Shut: the wheel
at (-0.597, 0.878, 0), the brick at (0, 0, 0), the pipe at (0.851, 0.084,
0.682). Open: the wheel at (-0.686, 2.172, 0.087), the brick at (0, 0.611,
0), the pipe at (0.851, 1.374, 0.500), turned so that its own x, y and z
axes point along (0.943, -0.130, -0.306), (0.303, -0.043, 0.952) and
(-0.137, -0.991, -0.001).

## What to hand back

1. The three files under the originals' names: `CHESTG0GROUP3.glb`,
   `CHESTG0GROUP4.glb` and `CHESTG0GROUP5.glb` (they go into the model
   pack's ITEMS folders, every world's).
2. Renders: the three beside their originals, and the junk chest (the
   base with its new heap, Treasure_Heaps/review_pack/ITEMS_G/
   CHESTG0CHST_BOT.glb, and the chest's lid) open with the props in their
   open pose, and shut (nothing through the lid).
3. Each material named `CHESTG0_<surface>`: `CHESTG0_wood` (the wheel),
   `CHESTG0_metal` (the pipe, the wheel's iron) and `CHESTG0_stone` (the
   brick), so the game can give each its shine.

For the game to read the files (gdl_client::model_pack): one buffer inside
each .glb, base colour textures as PNGs (opaque, no alpha), at most about
1,500 triangles per file, at most two 1024 × 1024 textures per file (the
pickup's junk texture can serve all three), no mesh compression (Draco,
meshopt), no texture transforms, no animation (the chest's tree moves
them).
