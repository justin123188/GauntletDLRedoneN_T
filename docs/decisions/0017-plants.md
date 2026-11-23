# 0017: Plant effects — sway and light through leaves (M3)

The model phase's third step (docs/remaster/models-plan.md M3), on the
levels' existing plants. The owner's go-ahead came on 2026-11-22.

What the plants are (G1, read 2026-11-22, GDL_DEBUG_MATERIALS):
- **Mostly see-through textures** (alpha class 2): hanging moss
  (MOSS_BRANCH, MOSS_WAD_02), grass tufts (GRASS_TUFFS), vines (VINEGATE),
  corn (CORN_SIDE, CORN_TOP), branches and leaves (BRANCH_01, STUN_LEAF).
- **Some opaque ones:** GROWTH, the corn stalk, grass ground (GRASS_LAND)
  and hay and straw bales.
- **Guesses fixed.** CORN, STALK, WILLOW, CYPRESS and HAYSTACK were added.
  PINE now matches only the whole word, so SKEL2_SPINE is no plant.

Decision (gs.wgsl, gdl_client::material_maps; NATIVE, off in
Settings::baseline() and the Original preset):

- **Marking.** While Settings::plant_effects is on at load, GsParams depth.z
  marks the plants:
  - 2 for foliage (any alpha class);
  - 3 for cut-out or see-through grass. Opaque grass, the grass ground and
    the bales, stays still.

  The switch is PointLights enhance3.x (new, after the water's vec4; the
  buffer is 32 × 32 + 80 bytes). Turned on in play, it applies from the
  next level.
- **Sway by the texture.** The UVs move, so a cut-out's outline moves with
  it and the geometry stays. Nothing can crack apart, and plants made of
  several pieces keep together.
  - A wave along the texture's v makes leaves bend rather than slide.
  - A slow gust travels across the level (from the render-space position).
  - Amplitude: foliage 0.012 UV, grass and moss 0.007.
  - The clock is the water's (decisions/0016): it stops with the game.
- **Light through leaves.** For plants, a point light (spell, torch,
  effect) behind the leaf lights it at 0.6 of its front light. The light is
  coloured by the texture, so a torch behind moss glows green. Plants with
  a detail map get this in material detail's light loop (its `back`
  factor); other objects pass 0 and are lit exactly as before.
- **The setting.** F10 → Graphics → "Plant effects", on in the defaults
  and the Enhanced preset.
- **Not done: softer cut-out edges.** Alpha to coverage needs MSAA, and
  ambient occlusion (on by default) turns MSAA off. SMAA then smooths
  those edges anyway. With ambient occlusion off and MSAA on it would be
  possible; left for later.

All values are NATIVE, chosen by eye.

**A water fix, made while testing this** (decisions/0016, after the
owner's water playtest in C1). G1's shallow, near-black swamp (SWAMP_03,
opaque) showed big white blotches:
- the ripples were in UV space, and that texture spans a large area per
  repeat;
- a constant part of the sheen greyed the swamp.

The fix:
- The ripples are now in game units over the ground plane, the same size
  in every level. Level water tilts; falls keep their normal. The
  texture's wobble is converted to UV by the surface's UV scale.
- The glints come from three finer waves, sharper (roughness 0.06), at
  most +0.3.
- The sheen is (1 − n·v)⁴, with no constant part.

G1's swamp mean went from 1.0/4.8/1.9 (water off) to 3.0/7.6/4.6 (fine,
dim sparkle; with the blotches it had been 9.6/15.5/12.4). C1 and A2 look
as the owner saw them, subtler in A2.

Verification (2026-11-22, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS):

- **Unit tests.** The client's 76 pass, among them the plant guesses and
  the marking codes (special_surfaces_follow_their_switches).
- **Off is unchanged.** With material detail, water and plant effects off,
  frames are pixel-identical to the build of 5b4d6b9 in C1 (start) and G1
  (frame 3000 after a freeze at 300). The GDL_BASELINE=1 frames of C1 are
  identical too.
- **On, G1.** The moss, grass tufts and vines are offset by the sway (77k
  pixels changed), with no gaps or artefacts at cut-out edges.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the defaults (all three effects on) and with
  GDL_BASELINE=1.
- **Motion.** Frozen frames could not show it: the camera still moves
  between the moments the harness can freeze. The owner judges it in play.
