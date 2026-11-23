# 0015: Material detail — surface types, bump and shine maps (M1)

The model phase's first step (docs/remaster/models-plan.md M1). The
owner's choices (2026-11-22):
- materials first;
- the game's baked light kept as the base, with live light on top;
- "an evolution of the game's core look".

The owner also asked that gems and crystals get a type of their own, to
be upgraded later.

Decision (gdl_client::material_maps, gs.wgsl; NATIVE, off in
Settings::baseline() and the Original preset):

- **Surface types.** There are 14: none, auto, stone, wood, metal, ground,
  grass, foliage, water, lava, ice, gem, cloth and skin. Each has a bump
  strength, a roughness and a metalness, NATIVE values chosen by eye.
  - A texture's type comes from `texture_pack\materials.tsv`, one line per
    texture: `<name> <type> [bump=… rough=… metal=…]`, where `#` starts a
    comment.
  - Without a line, the type is guessed from the texture's TEXDEF label
    words, tried in order (effects and skies first). The label is split into
    runs of letters. A word of four or more letters matches anywhere in a
    run; a shorter one only at its start. Marks: `^` start or end only, `=`
    the whole run, `*` anywhere, effects too.
  - In the Tower's 815 textures: auto 256, none 223, stone 78, skin 64,
    metal 54, wood 48, cloth 37, ground 20, foliage 14, gem 15, ice 10,
    lava 7, water 3, grass 1. The gems are SOR_GEMA, GEMSTONES-NA,
    PINK_GEM, the hand gems and the SM_CRYSTAL colours.
- **Detail map.** One RGBA8 image on the base texture's UVs:
  - R and G the tangent-space normal (OpenGL convention, green up);
  - B the roughness;
  - A the metalness;
  - a mip chain made with 2×2 box filtering; trilinear sampling, 16×
    anisotropic, the record's clamp modes.
- **Where the map comes from.** The owner's files beside the texture in the
  pack (`_n` normal, `_h` height, `_r` roughness, `_m` metal; any size) are
  used first. Whatever is missing is generated from the texture:
  - The height comes from the brightness, scaled by the texture's own
    spread (mean at 0.5, ±2 standard deviations to 0 and 1, at least 0.04).
    Without that scaling, G1's dark swamp textures got almost no relief.
  - Sobel slopes on that height give the normal. The slope is 4 × bump at
    the disc's texel size, steeper for finer HD or replaced texels.
  - Clear texels of cut-outs are flat, and no slope is taken across their
    edge.
  - The roughness is the type's, ±0.15 with the height on bumpy types.
  - The map is at most 1024 on a side.
- **Which textures.** Opaque and cut-out textures only. Blended ones (glows,
  smoke, mist, translucent moss) get none, and type none is a flat 1×1 map.
  The map is made as a texture loads (materials_for's worker threads), so
  it applies from the next level when turned on in play.
- **Shading** (gs.wgsl; GsParams depth.y marks a bound map, PointLights
  enhance2.y switches it live). Not on additive draws, chrome, pbBlit
  sprites or hit flashes.
  - **Tangent frame.** It comes from the screen-space derivatives of
    position and UV (T = ∂p/∂u, B = ∂p/∂v; the determinant's sign keeps the
    mirrored render root right), so the meshes are unchanged.
  - **Bumps keep the mean light.** They shade by the detail normal's tilt
    along the surface: dot(n − n_geo·(n·n_geo), l). Slopes facing a light
    brighten and the others darken by as much. An earlier version shaded by
    dot(n, l) − dot(n_geo, l), which also dimmed lit walls, through the
    normal's shorter perpendicular part.
  - **Baked light.** Scenery is shaded ×(1 + 0.8 · tilt·sun), clamped to
    0.55..1.45. The sun is the level's light direction (LEVL +0xf0, as the
    shadows; lib.rs write_sun).
  - **Live lights.** The game's point lights, torch lights and the
    characters' VU1 light keep their original falloff on the surface
    normal; the tilt adds ±, clamped to 1.5.
  - **Highlights.** Blinn-Phong from the roughness, with exponent
    2^(1 + 11(1 − r)) and strength (1 − r)² × (s + 8)/8π, skipped below
    0.05 (matte types). They come from every point light and the level
    light, the latter times the baked light there, so dark corners do not
    glint. A metal's highlights take its texture's colour. They are added
    after the texture and before the lightmap.
- **The setting.** F10 → Graphics → "Material detail", on in the defaults
  and the Enhanced preset (Settings::material_detail).
- **F5.** It re-reads materials.tsv and remakes the maps whose table line,
  map files or texture file changed.
- **The texture page.** index.html shows each texture's surface type
  ("(guess)" when guessed), and searching for a type finds its textures.
  The pack's README.txt explains materials.tsv and the map files.
- **Debugging.** GDL_DEBUG_MATERIALS=1 lists each texture's type on
  stderr.

Verification (2026-11-22, the owner's PC, the actual disc's import,
scratch copies, isolated APPDATA, GDL_NO_GAMEPADS):

- **Unit tests.** Label guesses, including run-together labels and the
  gems; table lines; generated normals following the slope (OpenGL
  convention); type none and clear texels flat; resampling; the page's
  surface column; the settings presets.
- **Off is unchanged.** Frozen G1 frames with the setting off (enhanced
  settings, the original camera, frame 3000 after a freeze at update 300)
  are pixel-identical to the installed build of 5b4d6b9, and so are
  GDL_BASELINE=1 frames (both checked again with the final build). One
  earlier run of the new build differed from the others by 14,174 pixels
  (at most 89 levels); its repeat matched the old build exactly, and two
  old-build runs matched each other.
- **The freeze needs late frames.** On this PC, frames 500 and 700 come
  before update 400, so earlier "frozen" shots compared different moments.
- **On, in the Tower.** The stone walls by the braziers show relief and
  sheen. Mean brightness is kept: a torch-lit wall 142.3/98.1/23.8 against
  141.5/98.8/24.0, the whole frame 32.5 against 33.0.
- **On, in G1.** The effect is faint there (dark, mostly blended foliage
  and lightmapped ground).
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the defaults (material detail on) and with
  GDL_BASELINE=1.
- **Cost.** At 2560×1440, uncapped, the median frame is 5.40 ms off and
  5.56 ms on. Windows capped some runs at 60 Hz; those are left out.
  Loading G1 took 1.43 s against 1.33 s (materials_for, 15 packages: 173 ms
  against 97 ms).

Not done:
- the owner's playtest;
- water ripples and flow (M2), foliage sway and light through leaves
  (M3), lava glow, gem sparkle;
- a global strength setting;
- choosing types on the texture page itself (materials.tsv is edited by
  hand).
