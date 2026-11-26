# 0032: Items in the colour of their light, and softer glints

The owner's question (2026-11-26), after the treasure pickups
(decisions/0031): "DO you think the lighting of the chest, and treasures
should more better match the environment, and surrounding lighting?" The
answer here: partly. Since decisions/0027 an item's brightness follows the
ground's baked light, but not its colour, and the sharp metals' glints
(decisions/0029) blow out to white. The owner's choice: "Yes, try both".

What was found (2026-11-26, the installed build 0615073b, a scratch copy
of the game with the installed model pack, six spots of the pickups'
in-game picture: K1's silver and junk, D1's gold and silver, I1's gold
and a chest, K2's gems, H3's low tub):
- **The glints.** Shot without POWERUPS's materials.tsv (no detail maps
  on the pickups) and with it: the white patches on the silver, on D1's
  gold and in the middle of H3's tub are the highlights. Without them the
  gold and the tub have none; the silver's paleness is the art itself.
  The metals' line (rough 0.22) gives a highlight of up to about 19 times
  the light (spec_scale), so a broad patch facing the sun saturates to
  white.
- **The light's colour** under the pickups (GDL_DEBUG_GROUND_LIGHT, now
  with the colour; GS scale, 1.0 = 0x80): H3's cave red-orange (0.31,
  0.21, 0.21), K2 green (0.39, 0.56, 0.43 under the gems), D1's dock
  grey-blue (0.43, 0.43, 0.49), K1's dock a little cyan (0.78, 0.93,
  0.93), I1 white (0.93). D1's and K1's green look is in their textures,
  not in their light, so the tint is small there.
- **VU1's cap.** Most items in the open are lit to VU1 program A's cap
  (min(ambient + light, 128)). A first version applied the tint inside
  the cap, and the cap cut it away: H3's tub's mean colour moved by less
  than 1 of 255. The tint is applied after it.

Decision (NATIVE; gdl_client::ground_light::tint, gdl_play
present_ground_light, GsParams ground_tint, gs.wgsl):
- **The colour**, under "Items in shade" (Settings::item_ground_light, off
  in Settings::baseline() and the Original preset). The five rays now give
  the ground light's mean colour (vertex colour × lightmap, as before, no
  longer reduced to its luma). The tint takes its hue at COLOUR 0.5 (each
  channel 1 + 0.5 × (colour / luma − 1)), each channel within 1 ± 0.3
  (TINT_SPREAD), then set back to luma 1, so the factor alone sets how
  bright. It multiplies the item's sun and ambient after VU1's cap, and
  the sun's highlight. Ambient adds (hit flashes, the gem glow) and the
  point lights stay whole. With a tint of 1 this is decisions/0027's
  light. An item in the open under white light still keeps the shared
  material; one under coloured light gets its own.
- **The glints**, under material detail (Settings::material_detail, off in
  the baseline). On lit parts (VU1 program A: items and characters, not
  water) the highlights, the sun's and the point lights', soften towards
  LIT_GLINT_MAX 0.4, measured on their brightest channel (m × (1 −
  e^(−peak / m)), keeping their colour), as water's glints are capped at
  0.3 (decisions/0016). The light-baked world is unchanged.
- **The values**, by eye on the six spots in four rounds: colour 0.5 and
  0.8 inside the cap (no visible change); 0.8 after it (H3's gold read as
  copper); 0.5 after it (firelit gold: the tub's mean colour 95, 63, 21
  before, 107, 59, 20 after); glints at most 0.5, then 0.4.
- **Not now.** Players and enemies keep the level's white light (as in
  decisions/0027; the owner, 2026-11-26: until their new models come, M8);
  no option of its own (it is part of "Items in shade"
  and material detail; the F10 help line now says "the ground's light and
  colour").

Verification (2026-11-26, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS; the build of this change, md5 b5daf56b, against
the installed one, 0615073b, both with the installed model pack):
- **Off** (Items in shade and material detail off): G1 and C1 frozen
  frames identical to the installed build. A first build softened the
  glints of water lit as items are too (785 pixels of C1's moat, max 14
  of 255); water is left out now.
- **The original presentation** (GDL_BASELINE=1): G1 identical.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, in the defaults and the baseline.
- **Items in shade off, material detail on:** G1's start, 965 pixels on
  the enemies' armour and the gravestone beside them (their glints
  softened).
- **The defaults:** G1's start, 36,975 pixels: the two barrels in view
  (the swamp's light) and the enemies' glints. The six spots load the
  pickups with no loader message; the final build's frames equal the
  fourth round's but for D1's water (543 pixels, now left out). The art
  folder's Treasure_Pickups/Treasure_Light_Colour.png: before and after at
  the six spots, a G1 barrel and an I1 chest.
- **Cost.** Frame times the same (vsync off, frozen at update 300, median
  over frames 1000 to 3000: G1 8.74 ms installed, 8.80 ms new; K1 12.16
  and 12.01; a first installed G1 run held at 60 Hz, 16.66, left out).
  Materials: G1 3,897 against 3,735 (items under coloured light get their
  own), K1 4,173 both.
- **Tests.** gdl_client 83 + 5 (ground_light: the tint, the colour read).
  Clippy: nothing new.
- **Installed** 2026-11-26 (the owner's "Yes, install"): this build (md5
  b5daf56b); the previous one, 0615073b, kept beside it as
  gdl_play-parallel-backup.exe; the model pack unchanged. The installed
  game loads K1, D1 and H3 with no loader message (first update 5.4 to
  6.0 s), and draws K1's silver and H3's tub as tested (but for the
  HUD's "LV 1", in the font the installed game uses).
- **Not yet.** The owner's playtest.
