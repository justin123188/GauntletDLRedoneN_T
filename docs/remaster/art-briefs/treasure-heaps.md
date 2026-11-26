# Brief: the treasure heaps

For an art tool that works in Blender (the treasure chest came this way).
Every treasure chest in the game is the same chest with something else
inside: when it opens, the heap shows. This brief asks for a new heap for
each of them, and for the chest with iron bands instead of brass, so every
chest the game can show matches the new one.

The chests (the item package's trees; the original's look in brackets):

| Chest | Inside | Seen in (A1, D1, G1 and K1 checked) |
| --- | --- | --- |
| CHESTG0 | junk (a grimy heap, with a broken cart wheel, a brick corner and a metal pipe sticking out) | K1 |
| CHESTG1 | silver (silver coins) | A1; and in any level when one class of hero hits a chest holding a little gold with a healing attack, from character level 25 |
| CHESTG2 | coins (gold and silver coins mixed) | G1 (one, in the swamp), K1 |
| CHESTG3 | gold (gold coins) | that healing upgrade from character level 50 |
| CHESTG4 | gold and gems (gold coins, coloured gems, a few gems standing up) | not in the four levels checked |
| CHESTG5 | gems (a heap of gems of every colour, a few standing up) | A1 |
| CHESTSG | gold, in the iron-banded chest | when a random chest (the iron-banded kind) turns out to hold gold |
| CHESTS | nothing (the plain iron-banded chest) | K1 |

A picture of the original heaps goes with this brief, kept privately with
the chest's review pack (it is the disc's art): from the left, top row
CHESTG0, G1, G2, G3; bottom row G4, G5, CHESTSG.

## The heaps

Each heap fills the chest's opening, as if the chest were full to the brim:
a low mound, highest in the middle, settling down to the rim at the walls,
made of the things the table names. They should read apart at a glance
from above, small, in a dark game: junk dull and dark; silver cool and
pale; the mixed coins warm with pale coins among them; gold all warm and
bright; gold and gems gold with bright colours through it; gems mostly
colour. Coins stacked and slid over each other, a few on edge; gems cut,
faceted and chunky. The junk heap keeps the original's three props
standing out of it (they are their own parts and stay as they are for
now).

**The style:** a modern remaster of an early-2000s dark-fantasy arcade
game, like the chest itself: rich, chunky, hand-crafted rather than
photographic.

**The light:** the game lights the heaps with soft diffuse light only (no
reflections, no metal shine), so the metal and the gems have to read from
their base colour: paint bright rims on coin edges and gem facets and
darker gaps between them into the colour, the way the original textures
do. No strong shadows or a scene's lighting baked in.

## Where they go (the numbers)

Every treasure chest's base and lid have the same shape and the same
hinge as the gold chest CHESTG2 (the item package's tree nodes: the base
at y 1.427, z -1.261, the lid at y 1.426, z -1.260), so all the heaps are
made in the local frame of the fitted gold-chest base,
`CHESTG2CHST_BOT.glb` from the chest's review pack (game units; x across,
y up, z from the hinge at the back, 0, to the lock at the front, 2.25):

- **The rim** of the base is at y = -0.013; its inside floor at y = -1.30
  (the heap hides it).
- **The opening:** the inner walls stand straight from the rim down at
  x ±1.389 and z 0.116 to 2.117 (corrected 2026-11-25; this brief first
  gave a rougher x ±1.30, z 0.18 to 2.07).
- **The visible pieces** (coins, gems, anything standing up) stay within x
  ±1.25 and z 0.22 to 2.03: near the walls the shut lid's side walls and
  bevels come down to a few hundredths above the rim.
- **To the walls**, a shallow layer of buried treasure just below the rim
  (y -0.03 to -0.08) fills the band between those limits and the walls, so
  no gap shows; nothing outside the visible pieces' limits rises above
  y -0.01.
- **Under the shut lid**, which must close over every heap without
  anything poking through: at most y 0.38 in the middle (z 0.5 to 1.75,
  |x| up to 1.2; the lid's inside is at 0.42 to 0.43 there), and at most y
  0.18 near the front and back (z below 0.4 or above 1.85, where the lid's
  bevels come down to 0.20). This holds for the gems too: the original gem
  heaps (CHESTG4, CHESTG5) stood up to 0.51, which the new, thicker lid
  has no room for.

## The iron-banded chest

CHESTS and CHESTSG are the chest with dark iron instead of brass: the bands,
rivets, corners, crest and lock plate in dark, slightly rusty iron (the
original's are grey-green iron), the wood as the chest's. The same model,
the same UVs: only the two baked base colour textures change, the base's
and the lid's. CHESTS has the plain chest's hinge (lid at y 1.411, z
-1.342, as CHEST), so it is the delivered CHESTCHST_BOTM and CHESTLID
with the iron textures; CHESTSG has the gold chest's (as CHESTG2) and its
gold heap.

## What to hand back

For each heap, in the frame above:
1. The heap on its own (for example `Heap_CHESTG3.glb`).
2. The full base file, the fitted base exactly as it is with the heap in
   place of the original's: `CHESTG0CHST_BOT.glb`, `CHESTG1CHST_BOT.glb`,
   `CHESTG2CHST_BOT.glb`, `CHESTG3CHST_BOT.glb`, `CHESTG4CHST_BOT.glb`,
   `CHESTG5CHST_BOT.glb` and `CHESTSGCHST_BOT.glb` (iron-banded).
3. Two renders from the comparison's camera: open with the heap, and shut
   (to show nothing comes through the lid).

For the iron-banded chest: `CHESTSCHST_BOTM.glb`, `CHESTSLID.glb` and
`CHESTSGLID.glb` with the iron textures, and the two textures as PNGs.
The other chests' lids are the delivered `CHESTG2LID.glb` as it is (the
same hinge); no new lid files are needed for them.

For the game to read the files (gdl_client::model_pack): one buffer inside
each .glb, base colour textures as PNGs (opaque, no alpha), at most about
4,000 triangles per heap (6,000 for the gem heaps) and one 1024 × 1024
texture per heap, no mesh compression (Draco, meshopt), no texture
transforms, no animation. Normal and roughness maps can stay in the art
master; the game uses the base colour.

## Later

The junk chest's three props (CHESTG0GROUP3, GROUP4, GROUP5: the wheel,
the brick corner and the pipe) could be remade like the chest; the chest
that bursts in a blast leaves broken remains (CHESTGEXP0, CHESTSEXP0); and
the same chests in every other world but the desert world C, done the way
the barrels were.
