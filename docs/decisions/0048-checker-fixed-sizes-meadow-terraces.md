# 0048: The colour checker at fixed sizes; the meadow's terraces

After area B's check (decisions/0046), the owner chose "Ask for
terraces" (2026-11-27): a fix round for the meadow's texture.

What was found while writing it:
- **check_colour.py compared contrast at different sizes.** It took each
  texture's mips at fractions of its own size: 1/8 "close up" and 1/32
  "at the camera" (decisions/0037).
  - The originals are 128 px and the new textures 1024 or 2048 px. So "at
    the camera" looked at patches about 2.5 to 3.7 units across on an
    original, and about 0.2 to 0.5 units on a new texture.
  - Fine grain on a new texture passed for an original's large-scale
    light and dark. That is how the meadow showed 20 % against 6 % while
    it plainly had less large-scale structure.
- **The figures recorded in decisions/0037, 0038, 0040, 0043 and 0046**
  for the contrast at the camera are therefore not like for like. The
  owner accepted those pieces by eye, in the game, and they stay
  accepted.

Decision (tools/blender/check_colour.py):
- **The contrast at fixed sizes in game units:** 0.5 units close up and 2
  units at the camera.
  - Each primitive's texel size comes from its UVs and its texture's
    size: the area one repeat covers, over its texels, so a non-square
    texture's texels are averaged.
  - The mip is the power of two nearest that size, so the textures'
    sides divide it and none is cropped.
- **The mean colour** is taken at the 0.5-unit mip.
- **The marks are unchanged:** each channel within 10 %, and at the
  camera at least 80 % of the original's.

The new measure (2026-11-27, the whole review pack):
- **The self-checks:** the originals against themselves give "all within".
  The meadow's original against its own subdivided copy gives 1.02 in
  colour, with the same contrast (26 and 28 % close up, 14 and 14 % at the
  camera).
- **At the camera:**
  - The walked-on dirt is 11 to 14 % against 11 to 15 %.
  - The slopes are 15 to 16 % against 12 to 14 %.
  - The start area's gully banks are 16 % against 24 to 26 %.
  - The fields are 10 % against 17 to 18 %.
  - The low ground (G1#029) is 2 % against 12 %.
  - The river banks are 25 to 26 % against 27 to 35 %.
  - The rock wall (G1#059) is 24 % against 39 %.
  - The meadow (G1#018) is 11 % against 14 %, 79 % of it, just under the
    mark.
- **So many new textures are smoother at the camera** than their
  originals. That matches the softer look the owner saw and accepted.
- **G1#059's colour** is 0.90 of the original in each channel, at the
  tolerance's edge.

The meadow (G1#018, measured here):
- **The layout:** the original and the delivery lay their textures the
  same way: one repeat every 14.7 units each way, the texture's
  horizontal along game x.
- **The original GRASS_LAND's terraces** run along that horizontal, so
  east to west in the game.
  - Two strong and two lighter per repeat, a step every 3.5 to 5.5 units,
    wavy.
  - Each has a near-black shadow under its lip, a mossy top, and
    red-brown earth between.
  - 11 % of its texels are darker than half its mean.
- **The delivered grass.png** has 4 % such texels. Its soft bands run
  diagonally in the texture, so diagonally in the game.

The brief (the art folder's G1_Ground/G1_Meadow_Terraces_Brief.md, with
G1_Meadow_Terraces_Reference.png and the corrected check_colour.py):
- **Change grass.png only** (used by G1#018 alone): the terraces as the
  original's, along the texture's horizontal, with the dark lines, the
  mossy tops and the red-brown earth.
- **The mesh stays** as it is (it is walked on).
- **The checks:** the colour within 10 %, and at the camera about the
  original's 14 % (at least 11.2 %).
- **The new checker for areas C and D.** Pieces already accepted stay as
  they are.

Not yet: the art, its check, area B's install.
