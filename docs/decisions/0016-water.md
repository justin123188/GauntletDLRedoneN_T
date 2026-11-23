# 0016: Water effects — ripples, glints, sheen, soft shores (M2)

The model phase's second step (docs/remaster/models-plan.md M2): a water
shader on the levels' existing water. The owner's go-ahead came on
2026-11-22, with the art direction of decisions/0015 ("an evolution of the
game's core look", the baked light kept as the base).

What the water is (read 2026-11-22, actual disc, GDL_DEBUG_MATERIALS):
- **Animated, translucent textures.** Each level's water is a 30-frame
  texture-modifier sequence: C1's SWATER00–29, G3's and I3's WATER00–29,
  and in A2 WATER, GWATER, HWATER and RAPIDS, 30 frames each. All are
  blended (alpha class 2). RAPIDTAIL, a shared texture, is opaque.
- **Not water.** BASIC_WALL_WATEREDGE (C1) is a wall: its guess is now
  stone.
- **Where it is.** Water collision (flag 0x200) is in A2, A4, C1, C2, C4,
  D1, E1, G3, G5, I3, I4 and J3 (research/collision.md).

Decision (gs.wgsl, gdl_client::material_maps; NATIVE, off in
Settings::baseline() and the Original preset):

- **Marking.** While Settings::water_effects is on at load, every texture
  whose surface type is water (materials.tsv, else its label;
  decisions/0015) marks its material with GsParams depth.z = 1. This
  includes translucent textures (no detail map is made for those) and
  texture-modifier targets without texels, typed by their label. The
  shader's switch is PointLights enhance2.z (live). Turned on in play, it
  applies from the next level.
- **Ripples.** Five travelling waves in UV space (one repeat of the water
  texture is one unit) with a slow warp. They flow with the texture's own
  scroll (options.zw) and tilt the surface normal through the tangent
  frame of decisions/0015 (uv_frame).
- **The texture wobbles.** The ripples bend the water texture's UVs
  (0.012 UV per unit of slope), so its own animated pattern ripples and
  looks less tiled. This is what shows most: the textures are dark, and
  light shading barely registers on them.
- **Light.**
  - The water is lit as material detail is: the live lights by the
    ripples' tilt around their own falloff, and the baked light shaded
    towards the level's light (×(1 + 2.0·tilt·sun), at most 0.55..1.6).
  - Glints: roughness 0.3, the level light's glints ×1.0, times the baked
    light.
- **Sheen.** The level's light colour (PointLights.water.yzw, from LEVL
  +0xfc × +0x108, at most 1 per channel) times a stylised Fresnel
  reflectance, 0.04 + 0.96(1 − n·v)³. The game camera looks down steeply,
  where water's real term (power 5, F0 0.02) reflects almost nothing. It
  is scaled by the baked light, at least 0.3.
- **Soft shores.** For translucent water, where the depth prepass exists
  (ambient occlusion on), the depth to the ground behind the surface
  (opaque parts only) fades the edge over 0.2 units. A faint foam line
  within 0.6 units is broken up by the ripples. Opaque water has no
  ground behind it in the prepass, so it gets no soft edge.
- **The clock.** gdl_play present_water_clock gives updates / 30 plus up
  to one update's time since the last step. It stops when the game pauses
  or freezes (GDL_FREEZE_AT frames stay constant) and moves smoothly
  between 30 Hz updates.
- **The setting.** F10 → Graphics → "Water effects", on in the defaults
  and the Enhanced preset.
- **Unchanged.** The wading height, the collision, the original animated
  frames and their scroll.
- **Debugging.** GDL_DEBUG_MATERIALS now lists every texture prepared
  (blended ones and modifier targets too), with its alpha class.

All values are NATIVE, chosen by eye on frozen frames of C1 and A2:
- C1's channels are dark blue with bright streaks;
- A2's channel is green, lit by its lamps.

The first version (gentle ripples, no wobble) changed the view by at most
3 levels on the dark water, and its glints missed the eye.

Verification (2026-11-22, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS):

- **Unit tests.** The client's 75 pass, among them the guesses for C1's
  water and its edge wall.
- **Off is unchanged.** With material detail and water effects off, the
  frames are pixel-identical to the installed build's predecessor (5b4d6b9)
  in C1 (start view) and G1 (frame 3000 after a freeze at 300). The
  GDL_BASELINE=1 frames of C1 are identical too.
- **On, C1.** The channels' pattern wobbles and breaks its tiling. A
  faint sheen shows toward the far bank, and small glints appear.
- **On, A2.** The green channel has the same effects. White glints
  standing alone read like specks, so the glints were softened (roughness
  0.22 → 0.3, strength 1.5 → 1.0).
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the defaults (water effects on) and with
  GDL_BASELINE=1.
- **Cost.** At 2560×1440 with C1's water over half the view, the median
  uncapped frame is 5.44 ms off and 5.47–5.90 ms on.
- **Not verified.** The motion: the frozen frames show one moment. That
  waits for the owner's playtest.

Not done:
- replacement water meshes with real waves (M2's W2);
- reflections of the scene;
- foam where opaque water meets the shore;
- splashes;
- waterfalls checked on their own;
- lava's glow.
