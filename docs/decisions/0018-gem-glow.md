# 0018: Gem glow — a soft glow, flaring at pickup

The owner's idea (2026-11-22): "a soft glow to the gemstones, that
brightens as picked up when the particles form and then dims as the
particles fade". The owner's choices:
- which pickups: crystals, runestones and gargoyle items;
- a steady soft glow while they lie in the level;
- lighting the gem and its surroundings.

This is a first piece of roadmap item 3 (effects; HANDOFF §8).

What the game has (gdl_core game/pickups.rs, read 2026-11-22):
- **The gems.** Pickups (item kind 1) of subtype 0xf are the crystals:
  CrystalDesc and MAX_CRYSTALS have 8 kinds, and the record's +0xf4
  amount is the kind. Subtype 10 is a runestone, 0x10 a gargoyle item.
- **The sparkle.** Taking one calls StartGemFX(position, n), which starts
  an EffectHeaders tree at the gem's place: 0x45 + n for a crystal, 0x4f
  for a rune, 0x4e for a gargoyle item. The tree plays for its frames ×
  timing (fx::DURATION).
- **The original glow.** G1's red crystals (SM_CRYSTAL_RED) already have
  an additive halo sprite. That stays as it is.

Decision (gdl_play present_gem_glow, GemGlow; NATIVE,
Settings::gem_glow, off in Settings::baseline() and the Original preset):

- **While a gem lies in the level** (drawn: active, not 0x8900, not
  +0xe1, its slot not hidden — a chest's closed content and a hidden Tower
  crystal are not lit):
  - a point light half a unit above it: reach 5, attenuation 0.35, its
    colour × 1.2;
  - the gem itself brightened by +0.6 ambient.
- **The colour.** The most saturated of the gem's textures' colours (each
  alpha-weighted mean, brightest channel 1, as the torch lights take
  theirs), cached per item slot and definition.
- **While a gem sparkle plays** (an effect node with header 0x45..0x4f):
  - the light flares: strength 1.2 + 3.0·k, reach 5 × (1 + k);
  - k rises smoothly over the first quarter of the effect's time (the
    particles forming) and falls smoothly to 0 at its end (as they fade);
  - the colour is that of the gem last seen within 3 units of the effect.
- **The lights' place in the list.** They go into present_point_lights'
  list after the game's own lights and before the torch lights (up to 32,
  with Dynamic lights; in view, nearest first).
- **A brightening that keeps the draw mode.** MatReq has a new `glow`
  field: a brightening like `ambient`, which keeps the part's alpha mode.
  `ambient` (MBTreeSetAmbientAdd, the game's own) switches an opaque part
  to blending: a crystal brightened that way lost its crisp bright edge.
  The game's own path is unchanged.
- **The setting.** F10 → Graphics → "Gem glow", on in the defaults and the
  Enhanced preset. It switches live: the lights and the brightening are
  made every frame.
- **Unchanged.** The simulation, which is only read.

All values are NATIVE, chosen by eye on G1's two red crystals behind the
first fence:
- first try: light 0.6, ambient 0.35, the crystals +5 levels;
- now: the crystal's body mean 155 → 169, and the ground around lit red
  (16.6 → 21.9 in red).

Verification (2026-11-22, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS):

- **Unit tests.** The flare's envelope (gem_flare_rises_then_fades: 0 at
  the start, 1 at a quarter, falling steadily to 0 at the end), and the
  presets.
- **Off is unchanged.** With material detail, water, plant effects and
  gem glow off, frames are pixel-identical to the build of 5b4d6b9 in C1
  (start) and G1 (frame 3000 after a freeze at 300). The GDL_BASELINE=1 G1
  frames are identical too.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the defaults and with GDL_BASELINE=1.
- **The pickup flare in frames:** not seen, as the scripted walk takes no
  gem.
- **The owner's playtest** (2026-11-22, the build of 6ef588c, the glow and
  picking gems up): "Looks good".
