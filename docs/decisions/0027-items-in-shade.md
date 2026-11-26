# 0027: Items in shade — items take the light of the ground they stand on

The owner's choice (2026-11-25): "Yes, try it", to a new option where
items take the light of the ground they stand on, offered after the
planned light transfer turned out not to apply to items (models-plan
"Lighting new models": no item carries vertex colours or a lightmap).

What the game does (gdl_play present_scene_light, gs.wgsl, read
2026-11-25):
- **Items and characters** are lit by VU1 program A: the level's ambient
  plus its one directional light (Level +0xf0.. direction, +0xfc..
  colour × intensity), capped, plus the point lights. The same light
  wherever they stand.
- **The scenery** carries its light baked: vertex colours (GS 0x80 = 1.0)
  times the lightmap's CLUT alpha / 0x80, plus the point lights.
- So in the original a barrel in a dark corner is as bright as one in the
  open. Replaced items are lit the same way (decisions/0022).

Decision (gdl_client::ground_light, gdl_play present_ground_light; NATIVE,
Settings::item_ground_light, off in Settings::baseline() and the Original
preset, on in the defaults and the Enhanced preset; F10 → Graphics →
"Items in shade"):

- **The ground.** Whenever world nodes are spawned (a level loads), the
  ground is read from the world parts as they are drawn: opaque parts lit
  by baked light (not VU1-lit, not blended, cut-out or additive), visible
  at that moment, their triangles in render space with their vertex
  colours and, when lightmapped, their lightmap UVs and lightmap texels.
  Triangles steeper than |normal.y| 0.35 are dropped; the rest go into a
  4-unit grid over x/z.
- **The open ground's light.** The light of each ground triangle at its
  centre (Rec. 601 luma of colour × lightmap), weighted by its area; the
  open ground is the light that 75 % of the ground area is darker than
  (G1 0.591, A1 0.603, D1 0.374).
- **An item's ground light.** Five straight-down rays, at the item's
  origin and 0.8 units out on each side, each meeting the highest ground
  between 0.75 above and 3 below the origin (a bridge over ground: the
  bridge); the mean of those that meet ground. No ground: unchanged.
- **The factor.** 1 − 0.8 × (1 − light / open), between 0.45 and 1: an
  item in the open or brighter keeps the original's look, one in shade is
  dimmed, one in deep shade keeps 45 % so it still reads (readability
  first, models-plan's owner's choices). Measured again only when an item
  moves 0.05 units.
- **What it dims.** The level's sun and ambient on the item (GsParams
  depth.w, unused before; gs.wgsl's VU1 program A path), and the sun's
  highlight with material detail. Not the ambient adds (hit flashes,
  MBTreeSetAmbientAdd, the gem glow) and not the point lights: a torch or
  a spell beside a shaded barrel lights it fully. Only lit, non-additive
  materials of drawn parts of items (portal glows stay). With depth.w 0
  the shader takes the original's expression, and the sun's highlight is
  multiplied by exactly 1 (the frames are identical, below).
- **How.** A MatReq `ground` field: like `glow`, it keeps the part's own
  alpha mode. Items at 0.995 or more get no request (no material of their
  own).
- **Not now.** Players and enemies (they move every frame; another step
  if wanted); a coloured ground light (only the luma is taken, so an item
  keeps its colours; since decisions/0032 an item takes half the light's
  hue too).

All values are NATIVE, chosen by eye on G1 (a barrel by the gate behind
the first fence: ground 0.183, factor 0.45; a chest at 0.53) and A1's
start.

GDL_DEBUG_GROUND_LIGHT=1 prints the ground read per level (triangles, the
open ground, the read time) and each item's place, ground light and
factor, once.

Verification (2026-11-25, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS; the build of this change, md5 852c95fd, against
the installed one, d41840c9, both with every realm's barrels):
- **Off** (item_ground_light false): G1 and C1 frozen frames, and G1's two
  shaded spots below, identical to the installed build.
- **The original presentation** (GDL_BASELINE=1): G1 identical.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the option on (the defaults) and in the baseline.
- **On.** The barrel by the gate (teleport [-24.0, 17.2, -139.0], frozen
  at update 260) darkens into the corridor's shade and keeps the light of
  the yellow crystal beside it (35,056 pixels differ); the chest at
  [76.0, 36.3, -292.0] dims a little (4,315). G1's start view: identical
  (no item in shade in view). A1's start: the barrel by the back wall
  darkens, the red barrel a little, the barrels in the open not.
- **Levels.** G1: 2,838 ground triangles, open ground 0.591. A1: 4,914,
  0.603, 130 of 209 items dimmed (mean 0.79). D1: 4,232, 0.374, an evenly
  lit level: 9 of 222 dimmed, mean 0.98.
- **Cost.** Reading the ground: 4.3 ms per level. Frame times the same
  (G1, 600 frames: median 16.7 ms on and off, vsync); loading the same.
  Materials: 3,775 with the option on against 3,529 off. A first version
  requested the ground light on every part under an item, the hidden ones
  too (a barrel's 26 frames): 12,126 materials; only the parts drawn are
  requested now.
- **Tests.** gdl_client 81 + 5 (ground_light: the open ground, shade, the
  floor, a bridge over ground, steep walls; lightmap sampling).
- **Not yet.** The owner's playtest.
