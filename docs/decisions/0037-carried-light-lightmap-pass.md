# 0037: Carried light through the original's lightmap pass; the colour checker

The owner chose "Yes, write the fix" (2026-11-27): a fix round for
ChatGPT's G1 start ground (decisions/0036), which came out lighter and
greyer than G1. Setting the brief's colour targets meant first finding
out how much of that came from the textures and how much from the game.

What was found (2026-11-27, the build of decisions/0035, 9d157a00; G1's
start, frozen at update 300, the mean over the pixels that differ from
the original):
- **ChatGPT's ground in the game** was 1.64 to 2.20 times the original
  in red and green and 2.9 to 4.4 in blue, over four views. Its textures,
  as they lie on the surface, are only 1.30 to 1.45 and 1.66 to 1.90
  (below). Material detail off and ambient occlusion off left this
  unchanged.
- **The control:** G1's eight original pieces, from their own exports,
  subdivided to edges of at most 0.55 (4,672 to 30,656 triangles, so the
  light has the new ground's density) and drawn through the model pack.
  They came out 1.16 to 1.22 in red and green and 1.48 to 1.50 in blue:
  about 5 / 255 brighter on G1's dark ground.
- **0035's figure was not per pixel.** Its "mean 1.4 to 1.6 brighter" was
  averaged over the whole frame. Per ground pixel the difference was 4 to
  8, as its own counts show (403,534 pixels by 4 to 8).
- **The cause:** an original lightmapped part draws its base pass into
  the 16-bit frame first. That write averages 4.5 / 255 down at full
  colour (gs.wgsl, the lightmap fold), and the lightmap then multiplies
  what was written. The carried light put the lightmap into the vertex
  colour and drew one pass, so that write was missing.
- **Two more differences:**
  - The part's point lights were not dimmed by the lightmap.
  - With options.y 0, the part had no ambient occlusion, since SSAO
    shades only the light-baked world.

Decision (NATIVE; light_transfer and model_pack::apply; under the model
pack folder, so off in Settings::baseline() and the Original preset):
- **The carried light draws through the original's own path.**
  - Each new vertex takes the nearest point of the original's surface:
    min(colour, 1) there as its vertex colour.
  - Where that triangle is lightmapped, the lightmap value there becomes
    a coordinate into a ramp, held in UV_1 as ((value × 128 + 0.5) / 256,
    0.5).
  - The ramp (light_transfer::ramp_image) is a 256 × 1 R8 texture whose
    texel i holds i, sampled linearly like the disc's lightmaps, so the
    sample is the value, interpolated between vertices. The smooth
    lightmaps' B-spline keeps a straight ramp.
  - The part keeps the original's lightmap flag. So the base pass's frame
    write, the lightmap over the point lights, and ambient occlusion are
    the original's. The shaders are unchanged.
  - A vertex whose nearest triangle has no lightmap takes 1. A part
    without any lightmapped vertex draws as before, with no ramp.
- **The colour checker,** tools/blender/check_colour.py (Blender's
  Python, or Blender), compares a replacement with its original's export,
  or a folder of them:
  - **The mean colour** the textures lay on the surface (area-weighted
    samples of the base colour texture at the 1/8 mip), per material and
    overall.
  - **The relative contrast** (the luma's standard deviation over its
    mean) at the 1/8 and 1/32 mips.
  - **Marks:** a channel more than 10 % off, and a contrast at the camera
    (1/32) under 80 % of the original's.
  - The light is now the original's, so these texture figures set the
    colour in the game.

Verification (2026-11-27, the owner's PC, a scratch copy of the game with
the installed model pack, isolated APPDATA, GDL_NO_GAMEPADS; the build of
this change, md5 65051ea4):
- **The control again:**
  - The mean is 0.98 to 1.00 of the original in every channel.
  - Material detail off: 143,309 pixels differ at all (at most 29 / 255).
    Only 2 to 3 % of each half of the frame differs by 3 or more (the
    channels' sum).
  - Material detail on: 626,273 pixels differ (at most 65), with the same
    mean. The exports' materials are unlisted, so they get no detail map,
    while the original's texture gets one from its name (as in 0035).
- **ChatGPT's ground:** 1.44 to 1.69 in red and green and 2.4 to 3.0 in
  blue over the four views. The texture-colour differences above account
  for this, since the frame write's fixed step weighs more on dark ground.
  Its banks look a little darker and more shaded than before.
- **The checker:** the originals against themselves give 1.00 and the
  same contrast. Against their subdivided copies, 0.98 to 1.04 (sampling).
  On ChatGPT's pack it says "not all within":

  | | Colour, new / original | Contrast at the camera |
  | --- | --- | --- |
  | Walked-on pieces | 1.30 to 1.45 in red and green, 1.66 to 1.77 in blue | 5 to 8 % against 8 to 11 % |
  | Banks | 1.35 to 1.46, 1.71 to 1.90 | 16 % against 24 to 26 % |

  The relief: the banks stand off the original's surface by a median of
  0.06 to 0.16, at most about 0.7 (Blender, a BVH of the original).
- **Tests:** gdl_client 87 + 5. The new one (light_transfer): the ramp
  samples back as the lightmap's value, and linearly between two
  vertices. baked_at gives the colour and the lightmap apart. Clippy:
  nothing new.
- **Nothing else changes.** There is no level folder in the installed
  model pack. Against 0035's build (9d157a00), these are identical:
  - G1 in the original presentation (GDL_BASELINE=1);
  - G1 and C1 with Items in shade and material detail off;
  - G1 with Items in shade off, and with the defaults.

  The simulation hashes over 600 updates of the G1 walk equal the
  reference, in the defaults and the baseline.

The fix brief (art-briefs/g1-start-ground.md, "Fix round"; the art
folder's G1_Ground/G1_Start_Ground_Fix_Brief.md with check_colour.py and
G1_Start_Ground_Fix_Views.png) asks for three changes:
- the colour toward the originals' texture means (path 50, 47, 23; banks
  54, 40, 21; slope 53, 42, 19);
- the banks' large-scale light and dark, with the relief's own shading
  painted in;
- bolder relief: ledges and undercuts, outcrops 1.5 to 3 units across,
  thick roots.

The rules stay the same.

Not yet: the art, the install (this build with it), the owner's playtest.

Later (2026-11-27): the fix round came back the same day and passes both
checkers; in the game its colour is close to the original's
(decisions/0038).
