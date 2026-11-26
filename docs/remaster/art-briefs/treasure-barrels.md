# Brief: the treasure barrels

For an art tool that works in Blender (the treasure chest and its heaps
came this way). Two of the game's treasure pickups are barrels full of
gold: a low tub and a tall barrel, each standing in spilled coins. A hero
who walks into one picks it up whole. This brief asks for both, made from
the owner's barrel so they match the breakable barrels already in the
game. The other six treasure pickups (the heaps that broken barrels and
chests drop) get a second brief.

The pickups are in the shared POWERUPS package, so one set serves every
world. Where the levels place them (the level data, read 2026-11-26;
some show only in a game with more players, the level data's player
count for each item):

| Pickup | Object (export label) | The original | Worth | Placed in |
| --- | --- | --- | --- | --- |
| TREAS_BAR1 | TREAS_BAR1LOW_T | a low wooden tub with a handle on each side, heaped with gold coins, coins spilled around it | 500 gold | D1 (two); A4, C1, D3, G2, H3 and I1 (one each); with more players, two more in A4 and one more in C1, D3, G2 and I1, and one in G1 and J6; inside one breakable barrel in A2 |
| TREAS_BAR2 | TREAS_BAR2HIGH | a tall barrel brimming with gold coins, coins spilled around its foot | 1,000 gold | A6, H2, H3 and I3; A4 with four players |

A picture of the original pickups goes with this brief, kept privately in
the art folder (it is the disc's art): Treasure_Pickups_Originals.png, the
two barrels at the top left.

## The look

- **The barrels** are the owner's barrel, the breakable barrel in the game
  now (BAROBJIDLE15F01.glb in the folder): its wood, its staves and its
  bands, made to the sizes below. The tall one is the same barrel a little
  smaller; the low one a short, wide-mouthed tub of the same wood and
  bands, with a handle on each side (iron or rope). Open tops, no lids.
- **The coins:** gold coins heaped above the rim, highest near the middle,
  stacked and slid over each other, a few on edge; and a spill around the
  foot: a few coins piled against the staves, the rest lying flat on the
  ground and thinning out. Warm, bright gold that reads as gold at a
  glance from above, small, in a dark level.
- **The style:** a modern remaster of an early-2000s dark-fantasy arcade
  game, like the chest: rich, chunky, hand-crafted rather than
  photographic.
- **The light:** the game lights the pickups with its sun and ambient
  light, and gives the coins a metal shine of its own (a material list
  beside the files, decisions/0029). Paint bright rims on the coin edges
  and darker gaps between them into the base colour, as the heaps did; no
  strong shadows or a scene's lighting baked in.

## Where they go (the numbers)

Each in the local frame of its original export file (game units; x
across, y up, z front to back; y 0 is the ground the pickup stands on; the
origin is the pickup's place, which the original barrel stands slightly
off; keep the barrel where the original's is):

**TREAS_BAR2HIGH, the tall barrel:**
- The barrel's axis at x -0.18, z -0.01, from the ground (y 0) to the rim
  at y 2.445.
- The radius 0.96 at the foot and at the rim, 1.22 at the belly (y 1.22).
  The owner's barrel is 2.73 tall with a belly radius of about 1.33 and
  4,000 triangles.
- The coins above the rim up to y 2.80 at most, the peak near the axis,
  within the rim.
- The spill: coins against the foot up to y 0.65; flat coins on the ground
  within about 1.65 of the axis (the original's spill covers x -1.60 to
  1.71, z -1.67 to 1.65).

**TREAS_BAR1LOW_T, the low tub:**
- The tub's axis at x -0.22, z 0.04, from the ground (y 0) to the rim at
  y 1.219.
- The radius 0.96 at the foot, widening straight to 1.225 at the rim.
- The handles: one on each side across x (the original's face +x and -x),
  between y 0.63 and 1.15, standing out from the side by no more than
  about 0.15.
- The coins above the rim up to y 1.67 at most, the peak near the axis,
  within the rim.
- The spill: coins against the foot up to y 0.65; flat coins on the ground
  within about 1.65 of the axis (the original's spill covers x -1.70 to
  1.62, z -1.67 to 1.66).

For both: nothing below y 0, and the flat coins just above it (y 0.005 to
0.03), so none sink into an even floor.

## What to hand back

For each barrel:
1. The file under the original's name: `TREAS_BAR1LOW_T.glb` and
   `TREAS_BAR2HIGH.glb` (they go into the model pack's POWERUPS folder).
2. Two renders from the originals picture's camera (from the front and
   above): the new pickup alone, and beside the original.
3. Materials named so the game can tell the coins from the barrel:
   `TREAS_BAR1_coins` and `TREAS_BAR1_barrel`, `TREAS_BAR2_coins` and
   `TREAS_BAR2_barrel` (the two files may share one coin texture).

For the game to read the files (gdl_client::model_pack): one buffer inside
each .glb, base colour textures as PNGs (opaque, no alpha: the spill is
real coins, not a picture on a card), at most about 8,000 triangles per
file (about 4,000 for the barrel, as the owner's, and 4,000 for the
coins), at most two 1024 × 1024 textures per file (the barrel's and the
coins'), no mesh compression (Draco, meshopt), no texture transforms, no
animation. Normal and roughness maps can stay in the art master; the game
uses the base colour.

## In the art folder (Treasure_Barrels)

- `private_export/objects/POWERUPS/TREAS_BAR1LOW_T.glb` and
  `TREAS_BAR2HIGH.glb`: the originals, exported from the owner's disc;
  keep them local.
- `BAROBJIDLE15F01.glb`: the owner's barrel as the game draws it now
  (fitted to G1's breakable barrel; md5 4665a7b1).
- `Treasure_Pickups_Originals.png`: the eight original treasure pickups.

## Next

The second brief (treasure-pickups.md): the other six treasure pickups,
TREAS_GOLD (gold coins around a bag), TREAS_SILVER, TREAS_GOLDSIL (gold
and silver around a bag), TREAS_GOLDGEM (gold with a helmet, a crown and
a sword), TREAS_GEMS (gems around a bag) and TREAS_JUNK (three parts: a
junk pile, a sign, a pipe and a brick, a broken wheel). These are what
broken barrels and chests hold in many levels, and some lie loose.
