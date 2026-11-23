# 0019: Potion glow — the gem glow for magic potions

The owner's request (2026-11-22), after the gem glow (decisions/0018):
"I'd like to add the same glow effect to magic potions". Like the gems:
a soft glow while a potion lies in the level, a flare when it is picked
up.

What the game has (gdl_core game/pickups.rs and effects.rs, read
2026-11-22):
- **The potions.** Pickups (item kind 1) of subtype 4. GrabPowerup stores
  the definition's +0x3c word in the player's potion slots; start_magic
  later reads its low four bits as the potion's colour (1..4; 0 = no
  colour, chosen in turn when the potion is used, randpottype).
- **The magic's colour.** start_magic lights its blast, shield and thrown
  flask with light_color[mclr_idx[colour mod 5]] (ELF tables, loaded into
  EffectTables). Read in G1 (GDL_DEBUG_GLOW, 2026-11-22): mclr_idx
  [6, 0, 6, 3, 1] over the rows red, green, blue, yellow, magenta, cyan,
  white (2.0 each; white 1.0). Colour 1 is lit red, 3 (POT_YEL) yellow,
  4 green, and both 0 and 2 (POT_BLU) white.
- **The potions in G1.** POT_YEL (item 367, +0x3c 3) and POT_BLU (371,
  372, +0x3c 2) on the ground; chest 31 by the start gives a random pick
  (item 305, a POT_BLU, in the scripted run).
- **No pickup sparkle.** Taking a potion plays a sound and the "got it"
  icon, and expires the item (ExpireItem: +0xd8 0x100 and a count at
  +0xda; DeleteItem once that has run down). No effect starts, unlike the
  gems' StartGemFX.

Decision (gdl_play present_gem_glow, extended; NATIVE, the same setting
as the gems, Settings::gem_glow, off in Settings::baseline() and the
Original preset):

- **While a potion lies in the level:** as a gem (decisions/0018): a
  point light half a unit above it (reach 5, attenuation 0.35, its colour
  × 1.2) and the potion brightened by +0.6 (MatReq `glow`, which keeps
  the part's alpha mode). The same drawn test: active, not 0x8900, not
  +0xe1, its slot not hidden.
- **The colour.** The potion's textures' colour, as a gem's (the most
  saturated of its textures' means, brightest channel 1): POT_YEL
  (0.95, 1, 0), POT_BLU (0.34, 0.45, 1). Not the magic's light colour,
  tried first: it lights a blue potion white.
- **The flare.** It starts when the potion's record starts expiring
  (+0xd8 0x100: GrabPowerup's ExpireItem, so only a pickup), at the place
  it was last drawn, once per pickup. It lasts as long as the rune's
  pickup sparkle (EffectHeaders 0x4f, timed as StartFXTree times a tree:
  frames × timing × 0x3a91a2aa, about 1/900; one second if the header is
  missing), on ATime, so it stops when the game pauses. Its strength and
  reach follow the gems' envelope (gem_flare: up over the first quarter,
  down to nothing at the end). The headers read in G1: 0x4f 30 × 30, one
  second; the crystals' 0x46..0x4c 45 × 30 (1.5 s), 0x45 7 × 30, the
  gargoyle items' 0x4e 20 × 30.
- **The setting.** F10 → Graphics → "Gem and potion glow" (the same
  switch, renamed). The settings file keeps `gem_glow`.
- **Unchanged.** The simulation, which is only read. A potion destroyed
  by a shot (DamageItem) does not expire through GrabPowerup and gets no
  flare: its magic blast has the game's own light.

Debug aid: GDL_DEBUG_GLOW=1 prints each glowing item (name, place,
subtype, +0x3c, colour) once, and each potion flare as it starts.

Verification (2026-11-22, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS):

- **Unit tests.** potions_glow_as_gems_do (subtype 4 is a potion, not a
  gem), with the gems' tests and the presets; clippy shows no new
  warnings.
- **The glow.** G1, the player placed by POT_YEL (frame 3000 after a
  freeze at 300): the bottle brighter and the leaves and ground around it
  lit yellow; off, only the game's own halo on the bottle.
- **The flare.** G1, chest-g1's teleports (the key, then chest 31): the
  log shows "potion item 305 taken at ATime 82.33: a flare of 1.00 s".
  Not looked at in frames.
- **Off is unchanged.** With material detail, water, plant effects and
  gem glow off, frames are pixel-identical to the build of 5b4d6b9 in G1
  (frame 3000 after a freeze at 300) and C1 (start). The GDL_BASELINE=1
  G1 frames are identical too. (C1's start frame differs between two runs
  of the same build, 24148 pixels; the new build equals the second.)
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the defaults and with GDL_BASELINE=1.
- **The owner's playtest:** next.
