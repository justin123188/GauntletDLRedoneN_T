# Brief: the treasure pickups

The second brief for the treasure pickups (the first: treasure-barrels.md).
For an art tool that works in Blender, as the chest, its heaps and the
treasure barrels. These six pickups are the treasure a hero picks up off
the floor: the heaps a broken barrel leaves where it stood, and treasure
lying loose in the levels. Each one is the same treasure as one of the
chest heaps ChatGPT made (the art folder's Treasure_Heaps), so the new
pickups should look like those heaps poured out on the ground.

The pickups are in the shared POWERUPS package, so one set serves every
world. What the levels do with them (the level data, read 2026-11-26;
"loose" counts those placed on the floor, some only in a game with more
players; a gold chest names one as its content but shows its own heap and
only gives its worth):

| Pickup | Object (export label) | The original | Worth | Loose | In breakable barrels | The chest it matches |
| --- | --- | --- | --- | --- | --- | --- |
| TREAS_SILVER | TREAS_SILVERL1 | a low pile of silver coins | 100 gold | 35, and 66 more with more players | 64 | CHESTG1 (silver) |
| TREAS_GOLDSIL | TREAS_GOLDSILTR | a small coin bag on a pile of gold and silver coins | 150 | 10, and 47 more | 27 | CHESTG2 (mixed coins) |
| TREAS_GOLD | TREAS_GOLDL1 | a coin bag on a spill of gold coins | 200 | 14, and 42 more | 27 | CHESTG3 (gold) |
| TREAS_GOLDGEM | TREAS_GOLDGEMTR | a winged helmet, a crown and a sword on a heap of gold | 250 | 9, and 30 more | 7 | CHESTG4 (gold and gems) |
| TREAS_GEMS | TREAS_GEMSGEM_P | a large bag on a spread of coloured gems | 300 | 9, and 18 more | 8 | CHESTG5 (gems) |
| TREAS_JUNK | TREAS_JUNKTREAS, TREAS_JUNKGR#0, TREAS_JUNKGR#1 | a grimy junk pile with a wooden sign (red lettering), a bent pipe, a brick corner and a broken wheel | 10 | 16, and 24 more | 83 | CHESTG0 (junk) |

In G1, two of the breakable barrels hold silver, two gold, and two gold
and silver; a gold-and-gems pickup lies loose (more show with more
players). A picture of the original pickups is in the art folder:
Treasure_Pickups_Originals.png (these six are the bottom row and the top
right two).

## The look

- **Match the chest heaps:** the same coins, gems and junk as the heaps in
  Treasure_Heaps (heaps/Heap_CHESTG0.glb to Heap_CHESTG5.glb and their
  textures), so a hero sees the same treasure on the floor as in a chest.
  The textures can be those heaps' textures, or new ones made the same
  way.
- **The heaps:** mounds of the treasure on the ground, highest in the
  middle, thinning out at the edges into a few scattered coins or gems;
  coins stacked and slid over each other, a few on edge; gems cut,
  faceted and chunky.
- **The things on them:** the coin bags (a leather or cloth bag, tied at
  the top, bulging with coins); the winged helmet, the crown and the sword
  of the gold-and-gems pickup; the junk's sign, pipe, brick corner and
  broken wheel. Keep each where the original has it (the numbers below).
- **The style:** a modern remaster of an early-2000s dark-fantasy arcade
  game, like the chest: rich, chunky, hand-crafted rather than
  photographic. They must read apart at a glance from above, small, in a
  dark level: silver cool and pale; gold and silver warm with pale coins
  among them; gold all warm and bright; gold and gems gold with the
  finery and bright colours; gems mostly colour; junk dull and dark.
- **The light:** the game lights the pickups with its sun and ambient
  light, and gives the coins, the metal and the gems a shine of its own (a
  material list beside the files, decisions/0029). Paint bright rims and
  darker gaps into the base colour, as the heaps did; no strong shadows or
  a scene's lighting baked in.

## Where they go (the numbers)

Each in the local frame of its original export file (game units; x
across, y up, z front to back; y 0 is the ground; the origin is the
pickup's place). For all of them: nothing below y 0 but the bags (the
originals' sink up to 0.09), and the flat coins and gems just above it (y
0.005 to 0.03), so none sink into an even floor. The heights are the
originals'; the new ones can be a little higher, not wider.

**TREAS_SILVERL1, silver:** coins within x -1.48 to 1.48 and z -1.38 to
1.25, up to y 0.56.

**TREAS_GOLDSILTR, gold and silver:** the bag at x -0.76 to 0.29, z -0.73
to 0.32 (its centre about x -0.23, z -0.20), up to y 1.27; the coins
within x -1.67 to 1.52 and z -1.79 to 1.79, up to y 0.68.

**TREAS_GOLDL1, gold:** the bag at x -0.62 to 0.84, z -0.62 to 0.58, up to
y 1.15 (its tied top from y 0.92); the coins within x -1.87 to 1.87 and z
-1.52 to 1.38, up to y 0.62.

**TREAS_GOLDGEMTR, gold and gems:** the helmet at x -0.40 to 0.63, z -0.41
to 0.54, y 0.57 to 1.69, its wings up to y 2.07; the crown at x -1.64 to
-0.35, z -0.13 to 1.27, y 0.08 to 1.35; the sword lying on the ground at x
-1.38 to 1.49, z -2.81 to 0.41 (it points out past the heap toward -z);
the coins and gems within x -1.42 to 1.66 and z -1.23 to 1.13, up to y
0.97, and a thin scatter out to x -2.11 to 2.07, z -2.12 to 2.08.

**TREAS_GEMSGEM_P, gems:** the bag at x -1.37 to 1.25, z -0.84 to 1.11, up
to y 2.28 (its tied top from y 1.84); the gems within x -2.12 to 1.89 and
z -1.57 to 1.45, up to y 1.06, and a thin scatter out to x -2.69 to 2.57,
z -2.12 to 2.16.

**TREAS_JUNK, three parts,** each its own file in its own frame. The game
places them by the pickup's tree (POWERUPS ANIM, TREAS_JUNK): GR#0 at the
pickup's place, TREAS at (-0.18, 0, 0) and GR#1 at (0.2511, 1.053,
-0.5625) from GR#0 (the tree nodes' positions). Make the junk as one piece in the
pickup's frame if that is easier, then split it into the three files and
move each into its own frame by those offsets.
- `TREAS_JUNKGR#0` (at the place): the sign at x -0.26 to 1.65, z -0.26
  to 1.02, y 0.34 to 2.91; the pipe at x -0.83 to 1.71, z -0.68 to 1.38, y
  0.44 to 1.88; the brick corner at x -0.41 to 0.70, z -0.01 to 1.09, y
  0.28 to 1.27.
- `TREAS_JUNKTREAS` (the pile): within x -1.91 to 1.73 and z -1.88 to
  1.88 of its own frame, up to y 0.92.
- `TREAS_JUNKGR#1` (the broken wheel): x -0.72 to 0.75, y -0.92 to 0.90,
  z -0.79 to 0.79 of its own frame (about y 0.13 to 1.95 above the
  ground), standing up.

## What to hand back

For each pickup:
1. The files under the originals' names: `TREAS_SILVERL1.glb`,
   `TREAS_GOLDSILTR.glb`, `TREAS_GOLDL1.glb`, `TREAS_GOLDGEMTR.glb`,
   `TREAS_GEMSGEM_P.glb`, and for the junk `TREAS_JUNKGR#0.glb`,
   `TREAS_JUNKTREAS.glb` and `TREAS_JUNKGR#1.glb` (they go into the model
   pack's POWERUPS folder).
2. Two renders from the originals picture's camera (from the front and
   above): the new pickup alone, and beside the original (the junk with
   its three parts together).
3. Each material named `<pickup>_<surface>`, the surface one of coins,
   gems, bag, metal (the helmet, crown, sword, pipe and the wheel's iron),
   wood (the sign, the wheel), stone (the brick) or junk (the pile), for
   example `TREAS_GOLD_coins`, `TREAS_GOLD_bag`, `TREAS_GOLDGEM_metal`, so
   the game can give each surface its shine.

For the game to read the files (gdl_client::model_pack): one buffer inside
each .glb, base colour textures as PNGs (opaque, no alpha: scatters and
the sword are real geometry, not pictures on cards), at most about 4,000
triangles per file (6,000 for the gems), at most two 1024 × 1024 textures
per file (the treasure's, and one for a bag or the finery), no mesh
compression (Draco, meshopt), no texture transforms, no animation. Normal
and roughness maps can stay in the art master; the game uses the base
colour.

## In the art folder (Treasure_Pickups)

- `private_export/objects/POWERUPS/`: the eight original objects of these
  six pickups, exported from the owner's disc; keep them local.
- `Treasure_Pickups_Originals.png`: the eight original treasure pickups.
- The look to match: `Treasure_Heaps/heaps/` and `Treasure_Heaps/textures/`
  (ChatGPT's chest heaps).

## Delivered

2026-11-26: ChatGPT's eight files in the art folder's
Treasure_Pickups/review_pack/POWERUPS; checked in decisions/0031 (the
junk sign's lettering is on its front only; the original's is on both
faces, and the levels show its back).

## Later

The junk props remade here (the sign, the pipe, the brick corner, the
wheel) may also suit the junk chest's three props (ITEMS_G CHESTG0GROUP3,
GROUP4 and GROUP5), which kept the original's (treasure-heaps.md, Later).
