# Brief: the gold heap in the treasure chest

For an art tool that works in Blender (the treasure chest came this way),
or for ChatGPT and an image-to-3D tool. The heap replaces the coins inside
G1's gold chest, CHESTG2: when the chest opens, the new heap shows instead
of the original's 14 flat triangles and their 64 × 32 coin texture.

## What to make

A heap of gold coins filling the chest's opening, as if the chest were full
to the brim: a low mound, highest in the middle, settling down to the rim
at the walls. Mostly coins, stacked and slid over each other, with a few
standing on edge; one or two small gems or a ring may peek out for colour,
no more (other chests are the gem ones: CHESTG4, CHESTG5).

**The style:** a modern remaster of an early-2000s dark-fantasy arcade game,
like the chest itself: warm, rich gold, chunky coins that read when seen
small from above, semi-realistic and hand-crafted rather than photographic.

**The light:** the game lights the heap with soft diffuse light only (no
reflections, no metal shine), so the gold has to read from its base colour:
paint bright rims on the coins' edges and darker gaps between them into the
colour, the way the original textures do. No strong shadows or a scene's
lighting baked in.

## Where it goes (the numbers)

All in the local frame of the fitted gold-chest base, `CHESTG2CHST_BOT.glb`
from the chest's review pack (game units; x across, y up, z from the hinge
at the back, 0, to the lock at the front, 2.25):

- **The rim** of the base is at y = -0.013; its inside floor at y = -1.30
  (the heap hides it, so nothing below the heap's surface is seen).
- **The opening** is about x -1.30 to +1.30 and z 0.18 to 2.07. Keep the
  heap within x ±1.25 and z 0.22 to 2.03, and let its edge meet the inner
  walls a little below the rim (y -0.03 to -0.08) so no gap shows.
- **Under the shut lid**, which must close over it without any coin poking
  through: at most y 0.38 in the middle (z 0.5 to 1.75, |x| up to 1.2;
  the lid's inside is at 0.42 to 0.43 there), and at most y 0.18 near the
  front and back (z below 0.4 or above 1.85, where the lid's bevels come
  down to 0.20). The original heap's top is 0.35, in the middle.

## What to hand back

1. `CHESTG2CHST_BOT.glb`: the fitted base exactly as it is, with the new
   heap in place of its "Original coin heap" node, in the same frame.
2. The heap on its own, in the same frame (`Gold_Heap.glb`), so it can be
   put into the other chests later.
3. Two renders from the comparison's camera: the chest open with the heap,
   and the chest shut (to show nothing comes through the lid).
4. The heap's triangle count and its bounds.

For the game to read the file (gdl_client::model_pack): one buffer inside
the .glb, a base colour texture as a PNG (opaque, no alpha), at most about
4,000 triangles for the heap and one 1024 × 1024 texture, no mesh
compression (Draco, meshopt), no texture transforms, no animation. Normal
and roughness maps can stay in the art master; the game uses the base
colour.

## Later, the rest of the family

The other treasure chests have the same base with other contents in the
same place: CHESTG0 junk, CHESTG1 silver, CHESTG3 gold, CHESTG4 gold and
gems (up to y 0.51 in the original), CHESTG5 gems (0.51), and the
iron-banded chest's gold (CHESTSG). Of these, G1 places only CHESTG2; the
others appear in other worlds or when the game swaps a chest's look for
what it holds. A heap made this way could be the start of each.
