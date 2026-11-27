# 0039: G1's banks get vertical gullies (models-plan M6)

After playing with the fix round installed (decisions/0038), the owner
liked the path's new colour. The banks looked good on their own, but the
owner could not choose between them and the original's style. The fix
round's banks have level layers: strata, ledges and outcrops. The
original's sides have vertical gullies.

The recommendation (2026-11-27) was gullies for G1:
- they suit a swamp, where runoff cuts soft, wet earth into channels down
  the slope; level layers suit harder, drier rock;
- they are the original's look for this realm;
- the layered style can go to a drier realm later, which also helps the
  realms differ.

The owner chose "Brief the gullies".

What the original does (2026-11-27, the eight pieces' exports; scratch
measurements):
- **GROUNDSIDE_01** (G1#000, G1#001, G1#030) is a 128 px tile:
  - A grass and moss lip runs along its top. A dark undercut sits under
    it.
  - Below are rounded earth ribs between dark vertical cracks, 3 or 4 to
    a tile.
  - The pieces lay it about 9 to 11.5 units wide and 10 to 10.7 tall,
    with the image's top at the top of the bank. So a gully comes about
    every 2 to 4 units.
  - Its vertical runs down the fall line: a median of 10 degrees on
    G1#000 and 2 on G1#001 and G1#030 (87 to 100 % within 20).
- **SLOPE_01** (G1#028) is mossy earth with roots, vines and ferns
  running down it. Its vertical is a median of 3 degrees from the fall
  line.
- **The fix round lays its bank texture the same way**: its vertical is a
  median of 11 to 16 degrees from the fall line on G1#000 and G1#030.
  Its layers run across the image, so they lie level on the bank
  (decisions/0038). The layout can stay; the image's content turns.

The gullies brief (the art folder's G1_Ground/G1_Banks_Gullies_Brief.md,
with G1_Banks_Gullies_Reference.png; art-briefs/g1-start-ground.md,
"Gullies round"):
- **Changes:** the shape of G1#000, G1#001, G1#030 and G1#028, and the
  bank texture.
  - Gullies run down the fall line, every 2 to 4 units: 0.4 to 1.0 wide
    and 0.3 to 0.8 deep, between rounded ribs.
  - An overhanging lip with hanging roots runs over a dark undercut.
  - Slumped fans of mud sit at the waterline.
  - The level ledges and strata go.
  - G1#028 gets shallower gullies, with roots and vines down its face.
  - The texture's vertical runs down the slope.
  - A dark gully material (G1GROUND_gully, ground) may be added, so the
    shading follows the shape.
- **Stays:**
  - the walked-on pieces and the path texture;
  - G1#029, unless a seam needs it;
  - the colours, within 10 % of the originals as now;
  - the contrast at the camera, at least 80 % of the original's;
  - the rules.
- **Clean-ups:** G1#001's 704 repeated triangles and G1#000's 2
  zero-area ones go.

The installed fix round stays in the game until the new art is checked.

Not yet: the art, its check, the install, the owner's playtest.
