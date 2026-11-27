# 0035: Replaced scenery keeps the level's light; the ground checker

The owner's "Yes, start them" (2026-11-26): the first two steps of the M6
test (models-plan M6; the owner's choices: G1, its start area first): the
original's baked light carried over to replaced scenery, and a checker of
new ground against the collision.

What a replaced level object looked like (read 2026-11-26): the model
pack drew every replacement with white vertex colours and no lightmap,
marked lit (decisions/0022). The scenery's light is baked (vertex colours
× the lightmap, VU1 program B and the lightmap pass, gs.wgsl), so a
replaced piece of scenery drew at full brightness: K1's platform in
decisions/0033, and G1's start ground below.

Decision (NATIVE; gdl_client::light_transfer, model_pack::apply; under the
model pack folder, off in Settings::baseline() and the Original preset):
- **Which.** A replacement whose original part is drawn with baked light
  (not lit, light_dir.w 0, not additive): the level's scenery. Items and
  characters stay lit live.
- **The light.** The original object's parts give its surface with its
  light: their triangles, vertex colours and, where the material draws a
  lightmap, its UVs and texels. Each new vertex takes the light of the
  nearest point of that surface (a grid of 2-unit cells, nearest triangle
  by Ericson's closest point), min(colour, 1) × lightmap / 0x80, as its
  vertex colour; the part draws as the scenery does (program B: that
  colour × the texture, the point lights, material detail when listed),
  without a lightmap. The message says "the original's light carried
  over".
- **Why it is the same light.** Program B clamps the vertex colour at
  0x80 before the lightmap pass multiplies, and the disc's lightmaps are
  4-bit (CLUT alpha index × 8, at most 120 / 128): they only darken. What
  the single pass does not have: the two-pass draw's model of the 16-bit
  frame write (4.5 / 255 off before the lightmap pass, 0.5 after), about
  1.5 / 255 brighter on average. The light follows the lightmap as finely
  as the new mesh's vertices: new ground is dense; a coarse mesh smears it.
- **The checker**, tools/blender/check_ground.py (Blender, or Blender's own
  Python): a replacement against the level scene of the model export (its
  COL_ collision layer). Under the original's drawn surface, the walkable
  collision (|normal.y| at least 0.35) sampled on a level-wide grid every
  0.25 units: the new surface's height nearest the collision (within 1.5),
  above it (feet sink), below it (they float) or missing (a hole), against
  a tolerance (0.10 by default); the original measured the same way, for
  reference. Beside it, over walkable collision it did not draw, the new
  surface must not stand up to 2.0 above it (a hero would walk through
  it). A summary, a JSON report with the worst spots in game coordinates,
  and a top-down map.

What G1's start area holds (the checker on the ten ground pieces' own
exports): the walkable ground is G1#005 and G1FLOOR#02, on which the
collision lies exactly on the drawn surface (0.000 over 14,043 and 3,307
samples); the banks G1#000, G1#028, G1#029 and G1#030 and the swamp bed
G1#063 and G1#079 have no walkable collision under them (free for new
shapes that keep out of the path).

Verification (2026-11-26, the owner's PC, a scratch copy of the game with
the installed model pack, isolated APPDATA, GDL_NO_GAMEPADS; the build of
this change, md5 9d157a00):
- **G1's start with G1#005 replaced by its own export** (frozen at update
  300, the hero idle): lit live (the installed build) the dirt is plainly
  too bright, 525,602 pixels differ from the original, 307,034 by 9 to 32
  of 255 and 18,743 by more; with the light carried over 487,238, 66,582
  by 9 to 32 and 192 by more. With material detail off (the original's
  texture gets a ground detail map from its name, the file none, which
  looks softer): the export subdivided (47 to 2,303 triangles, the same
  surface) 466,647 pixels, 403,534 of them by 4 to 8, 4,578 by 9 to 32 and
  115 by more, the mean 1.4 to 1.6 brighter (the two-pass write above).
- **The checker:** G1#005's export and the subdivided one 100 % within;
  the subdivided one with a 5 × 5 patch raised by 0.5 and faces cut out:
  the patch found (+0.500, 97.2 % within) and 84 hole samples, all at the
  cut; the bank G1#029 with a 3 × 3 block on the path at the start: 144
  samples standing in the way (the block, +0.600).
- **Tests.** gdl_client 86 + 5 (light_transfer: the nearest point of a
  triangle; a vertex takes the nearest baked light, interpolated, clamped
  at 0x80, a lightmap's). Clippy: nothing new.
- **Nothing else changes** (no level folder in the installed model pack):
  against the installed build (bc0cda39), G1 in the original presentation
  (GDL_BASELINE=1), G1 and C1 with Items in shade and material detail off,
  G1 with Items in shade off and with the defaults: identical. Simulation
  hashes over 600 updates of the G1 walk equal the reference, in the
  defaults and the baseline.
- **Not yet.** The art (the brief: art-briefs/g1-start-ground.md), the
  install with it, the owner's playtest.

Later (decisions/0037): the "mean 1.4 to 1.6 brighter" above is over the
whole frame; per ground pixel it was 4 to 8 / 255 (the base pass's 16-bit
frame write, which the single pass left out). The carried light now draws
through the original's lightmap pass (a ramp lightmap): 0.98 to 1.00 of
the original.
