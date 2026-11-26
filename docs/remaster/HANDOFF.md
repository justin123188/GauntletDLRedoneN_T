# Remaster handoff (for a new Claude Code session)

Written 2026-11-22 at the end of the first (cloud) session, for the next
session running locally on the owner's Windows PC. Read this whole file
before changing anything, then AGENTS.md and the documents it names.

## 1. What this is

- **This repository** is the owner's fork (`justin123188/GauntletDLRedoneN_T`)
  of `jeddhor/GauntletDLRedone`: a native Rust + Bevy reconstruction of the
  PS2 game *Gauntlet: Dark Legacy* (USA, SLUS-20047). Players install it from
  their own disc. The upstream project aims for exact fidelity: its charter
  says extras must not change the PS2 rules.
- **The owner's goal** is an HD remaster on top of it: new and upgraded
  textures, new models, camera changes (done: the free camera), some gameplay
  changes, and upgrades for modern screens and systems.
- **About the owner:**
  - Plays on Windows with a controller and owns the USA PS2 disc. They have
    played the port start to finish, and it also runs well for them in PCSX2.
  - Their programming experience is about 20 years out of date. Explain things
    plainly, avoid jargon, give exact click-by-click steps for anything they
    have to do, and do the coding yourself.
  - They plan to use ChatGPT for art.
  - Their preferences: **ask questions when unsure**, and **research on the
    web freely**.
  - Confirm with them before any outward-facing action (public pull
    requests, releases). Pushing to this fork's branches was authorized.

## 2. Repository, branches, remotes

| Branch | Contents |
| --- | --- |
| `main` | = `remaster` (fast-forwarded 2026-11-22, commit 4da0b88). What visitors see. |
| `remaster` | Integration branch for all remaster work. **Work here** (or topic branches merged into it). |
| `free-camera` | The first free-camera commit only (kept; superseded by `remaster`). |
| `windows-sound-fix` | The sound fix alone, built on upstream 1.3.6 (8cefd32) and worded for upstream. The owner opened a pull request with it on `jeddhor/GauntletDLRedone` (2026-11-22). Check its status; answer review comments only with the owner's go-ahead. |

Suggested remotes for a local clone: `origin` = this fork,
`upstream` = `https://github.com/jeddhor/GauntletDLRedone.git` (for later
upstream releases: merge them into `remaster`, never rebase published
branches).

Commits so far use the author `justin123188 <justin123188@users.noreply.github.com>`.
The trailer convention was `Co-Authored-By:` for Claude.

## 3. What is done (all on `remaster`)

### Free camera: player-controlled turn and zoom
Full rationale: `docs/decisions/0008-free-camera.md`.
- **Where:** `gdl_core/src/game/camera.rs` (`FreeCam`, `FreeView`,
  `CamCtx::free`, `Game::free_cam_input`, `view_yaw`, plus UpdateCam /
  CalcDist / CamUpdate hooks), `gdl_core/src/game/mod.rs` and `setup.rs`
  (`Game::free_cam`), `gdl_client/src/settings.rs` (`camera`,
  `camera_follow_rails`, `camera_invert`; `CameraMode`), `enhance.rs` (F10
  rows "Camera", "Turns with level", "Invert turning"), and `gdl_play.rs`
  (`free_cam_settings` system before `simulate`, `FreeCamZoom` carries the
  zoom between levels).
- **Controls:** the right stick turns (about 100°/s) and zooms (0.55×–2.2×).
  On the keyboard: numpad 4/6 and 8/2. It is ignored in the One Handed and
  Robotron control styles, which use the right stick themselves.
- **It is game logic, not presentation.** Player movement is camera-relative,
  and the camera frustum limits where players can walk. So it lives in the
  simulation, driven only by pad input. It is forced off in network play
  (lockstep would need the same setting on every machine; not exchanged yet).
  Boss and trigger cameras are still the original's.
- **On by default in this fork.** `Settings::baseline()` (GDL_BASELINE) keeps
  `original`. With `original`, `CamCtx::free` is `None` and no arithmetic
  changes, so the upstream fidelity checks (sim hashes, replays) still hold.
- **Status:** unit tests pass, and the owner confirmed in play that it works
  (test build 2).
- **Recentre and tilt** (2026-11-22, decisions/0014): the right stick click
  (numpad 5) swings back to the level's direction, held it returns the zoom
  too; the view tilts with the zoom (lower when close, steeper when far).
- **Open ideas:** mouse control, exchanging the setting in network play,
  free camera in boss fights, limits where turning reveals unfinished
  geometry (no reports yet).

### Windows sound fix
Notes: `docs/research/audio.md`, "Windows output: any device format".
- **The bug:** 1.3.6 only accepted a stereo f32 mode. WASAPI shared mode only
  offers the device's own channel count, so 5.1/7.1 devices were silent, with
  no message.
- **The fix:** any channel count and sample format is accepted. A thread owns
  the cpal stream and reopens it on error or when the default device changes.
  Notes go to **`remaster-log.txt`** beside `gdl_play.exe` (named
  `audio-log.txt` on the upstream branch).
- **Status:** the owner confirmed sound now works. Their `remaster-log.txt`
  (read on their PC by the first local session, 2026-11-22) shows the cause:
  the default device, "Speakers (Creative SB X-Fi)", offers only 4-channel
  modes (I32 and F32, 48 kHz) and no stereo mode at all; the build plays
  4 ch F32. The upstream PR's text says 5.1/7.1; a 4.0 device is one more
  case (a comment there only with the owner's go-ahead).

### remaster-log.txt
`gdl_client::audio::note()` writes a line to it (Windows only). Besides
sound, it records the free camera's on/off state and, at most once a second,
the right-stick input that reached the simulation. Use it for any feature that
needs field diagnostics: the game has no console on Windows.

### Stutter report
The owner saw "3 s smooth / 3 s stutter" cycles. It later went away; most
likely the driver shader cache was being rebuilt after the new exe, or HD
textures were being generated. Test build 2 shipped `run-with-frame-log.bat`
(sets `GDL_FRAME_LOG`). Use it if the stutter returns.

## 4. The owner's PC and installation

- **Installed game:** the installer's default is
  `C:\Users\<owner>\GauntletDarkLegacy\`; the owner's is elsewhere (local
  sessions keep the path in their memory, not in this public repo). It holds
  `gdl_play.exe`, `gdl_import.exe` and `data\` (`data\config.json` plus the
  imported disc files).
- **Test builds:** a new `gdl_play.exe` drops into that folder. Keep the
  original as a backup (the owner's is named `gdl_playBU.exe`). Installed
  builds read `data\config.json` next to the exe (`installed_config()`); the
  working directory becomes `data\`.
- **Saves and settings:** `%APPDATA%\gdl-native\SLUS-20047\saves\`, with
  `settings.json` there. A relative `texture_pack` path in settings resolves
  against the working directory, i.e. `data\`.
- **Test builds 1 and 2** were cross-compiled in the cloud and sent as zips.
  They don't need to be kept.
- **Never commit or upload game data:** the ISO, extracted files, texture
  dumps, or packs derived from disc textures. The fork is **public**. Upstream
  keeps such things in the ignored `private/` folder.

## 5. Building on Windows (native build works since 2026-11-22)

The first local session built and checked the game natively on the owner's
PC with the GNU target that upstream cross-builds (`x86_64-pc-windows-gnu`),
not MSVC: no Visual Studio, and the same kind of exe as upstream's Windows
packages. Versions and the check: `docs/toolchain.md`, "Native Windows
build".

1. **Rust:** `rustup-init.exe` for the GNU host, run with
   `-y --default-host x86_64-pc-windows-gnu --default-toolchain 1.93.1 --profile minimal`
   (an MSVC-host rustup would still want Visual Studio's linker for build
   scripts). `rust-toolchain.toml` pins 1.93.1 and adds rustfmt and clippy.
2. **C compiler and linker:** MSYS2 (`winget install MSYS2.MSYS2`), then in
   its shell `pacman -Syu` (twice) and `pacman -S mingw-w64-x86_64-gcc`.
   MINGW64, not UCRT64: Rust's GNU standard library uses msvcrt. Only
   blake3 (a bevy_asset dependency) compiles C/assembly.
3. **HD upscaler weights:** `crates/gdl_client/assets/hd/realesr-general-x4v3-dn05.bin`
   is required at compile time (`include_bytes!`) but git-ignored (`*.bin`):
   SHA-256 `e6626f1f5a94c46423826e44e60a6efe4b92d918aee97877c246fe7c2259ef4b`,
   4,853,208 bytes. Every gdl_play.exe built with it carries it verbatim:
   the first local session cut it out of test build 2 (4,853,208 bytes from
   the first `GDLSRVGG` header: version 1, 32 convolutions, 64 features,
   scale 4) and checked the hash. To regenerate it instead (Python 3 with
   numpy):
   - Download `realesr-general-x4v3.pth` (SHA-256 `8dc7edb9…96292`) and
     `realesr-general-wdn-x4v3.pth` (`1641f8c4…69dca`) from
     `https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/`.
   - Run `python tools/python/hd_weights.py <dir with both> crates/gdl_client/assets/hd/realesr-general-x4v3-dn05.bin`.
   - Check the hash against `realesr-general-x4v3-dn05.json`.
4. **Build:** `tools\windows\build-native.ps1` checks the weights, sets PATH
   for itself and builds `target\release\gdl_play.exe` (`-Tests` also runs
   the tests). A cold build took 18 minutes. Copy the exe into the install
   folder (back up first).
5. **Tests:** `cargo test --locked -p gdl_core` (99 passing on Windows; the
   100th, `save.rs` `a_full_disk_keeps_the_previous_save`, is Linux-only) and
   `cargo test --locked -p gdl_client --lib` (46 passing).
6. **Simulation check:** `tools\windows\sim-hashes.ps1 -Exe … -Config
   <install>\data\config.json -Out FILE [-Baseline]` runs a build in G1 with
   a scripted pad (`tools\windows\g1-walk-dpad.json`) and writes the
   simulation hash after every update, with saves and settings in a scratch
   folder. Two builds, or a feature on and off, agree when their files are
   equal. Use it for every remaster change that must keep the original mode
   bit-identical. The scripted pad has no analog sticks, so the free
   camera's turning cannot be scripted yet (a harness change).
7. **Linux-only parts:** `alsa`. The installer (`installer/`, Qt) is separate
   and not needed for development.

## 6. Diagnostics and test harness (let the local session see for itself)

From `gdl_play.rs` and `docs/research/*.md`; all are environment variables.

| Variable | What it does |
| --- | --- |
| `GDL_TEXTURE_DUMP=DIR` | Every decoded texture saved as `DIR\<name>.png` (alpha doubled), plus `DIR\index.tsv`: name, TEXDEF label, size, flags. One thread while dumping. |
| `GDL_FRAME_LOG=FILE` | One line per frame: frame, ms, entities, sim updates, player action, materials, visible meshes, distinct materials, camera position, Time delta, player body position. |
| `GDL_SHOT_FRAMES=N,M,…` with `GDL_SHOT_DIR=DIR` | Saves rendered frames as PNG. Use it to look at results yourself. |
| `GDL_INPUT_SCRIPT` | Scripted pad input on port 0, as JSON `[first, last, [entries]]`. |
| `GDL_EXIT_AFTER=N` with `GDL_STATE_OUT` | Writes the sim-state SHA-256 after N updates, then exits. |
| `GDL_FREEZE_AT=N` | Freezes the sim and keeps rendering, for A/B renderer captures. Use settings with `"interpolate": false`: with interpolation on, a frozen frame still moves slightly (decisions/0020). |
| `GDL_BASELINE=1` | The original presentation and camera; never writes the player's settings. |
| `GDL_SETTINGS=FILE` | Uses another settings file without writing the player's. |
| `GDL_NO_AUDIO` | No audio output. |
| `GDL_AUDIO_CAPTURE=FILE` | Renders the mix to a file instead of a device. |
| `GDL_SOUND_LOG` | Logs every sound request. |
| `GDL_SCENE_DUMP` | Dumps the scene. |
| F3 / F10 / F11 | Frame-rate counter / settings overlay / fullscreen. |

Commands (no window):
- `gdl_play --export-models CONFIG.json OUT_DIR [NAME ...]`: the model
  exporter (decisions/0021). Names pick levels (G1) or packages
  (ITEMS_G); none means everything (about 3.2 GB, 5 minutes on the
  owner's PC). Never into the repository: the files come from the disc.
- `blender --background --factory-startup --python
  tools/blender/check_glb.py -- REPORT.json [--render DIR] FILE.glb ...`:
  imports exports into empty scenes, reports counts and renders previews.
- `blender --background --factory-startup --python
  tools/blender/fit_model.py -- ORIGINAL.glb NEW.glb OUT.glb [--triangles
  N] [--texture PX] [--yaw DEG] [--fit height|box] [--report F.json]
  [--preview DIR]`: fits a new model to an exported original for the
  model pack (decisions/0022).
- `blender --background --factory-startup --python
  tools/blender/break_frames.py -- ORIGINAL_DIR NEW.glb OUT_DIR FRAME...
  [--thickness T] [--keep-base 0|1] [--inside-from WOOD.glb] [--report
  F.json] [--preview DIR]`: cuts a fitted model into the original's
  breaking pieces and writes a model pack file per flip-book frame
  (decisions/0023; --inside-from for a painted model, decisions/0024).
- `blender --background --factory-startup --python
  tools/blender/recolour.py -- NEW.glb FROM.glb TO.glb OUT.glb [--hue
  to|shift] [--report F.json] [--preview DIR]`: paints a model pack file
  the way original TO differs from original FROM, textures only
  (decisions/0024; --hue shift for another realm's brown, decisions/0026).
- `blender --background --factory-startup --python
  tools/blender/match_colour.py -- NEW.glb REF.glb OUT.glb [--group
  darker|lighter|all] [--ref-group all|darker|lighter] [--image NAME]...
  [--match mean|spread] [--strength S] [--report F.json] [--preview DIR]`:
  gives one colour group of a model pack file's textures (two groups by
  value, saturation and hue: a chest's wood and its brass) REF's coloured
  pixels' mean colour, or one of REF's own groups' (2026-11-25, the
  owner's chest wood in the barrel's). Run on embedded files, before
  share_data.py.
- `model_pack\<package>\materials.tsv` (decisions/0029): lines
  `<glTF material name> <type> [bump=… rough=… metal=…]`, the texture
  pack's form with the files' material names; with Material detail on, a
  listed material's texture shines and takes bumps under the live lights.
  G1's treasure heaps: metal (bump=0.9 rough=0.22) and gem (bump=0.5
  rough=0.05), the junk stone.
- `python tools/blender/share_data.py FOLDER [--report F.json]` (Blender's
  own Python works): moves a package folder's embedded textures and
  geometry to textures/ and data/ beside the files, one file per content
  (decisions/0026). Run it last, on the folder that goes into model_pack.
- `GDL_DEBUG_ITEM_TREES=1`: each item name's tree package, the first
  time the name is posed in a level (stderr: `item tree BAROBJ:
  Some(("ITEMS_G", 20)) in G1`). With `GDL_DEBUG_LEVEL=F2` and
  GDL_TEST_ACTIONS `[{"at": 600, "exit_next": true}]` (each level ends at
  its update 600) a run goes F2 → G1 → the Tower (L1).
- G1's red barrel: GDL_TEST_ACTIONS `[{"at": 200, "teleport": [-2.8,
  25.45, -250.8]}, {"at": 260, "damage_item": [364, 1000]}]` stands the
  player beside two red barrels and breaks 364 (363 follows); freeze at
  updates 250 to 330.
- GDL_DEBUG_GROUND_LIGHT=1 (decisions/0027): the ground read per level
  and each item's place, ground light and factor, once. G1's shaded spots:
  GDL_TEST_ACTIONS `[{"at": 200, "teleport": [-24.0, 17.2, -139.0]}]` (a
  barrel by the gate at the floor, 0.45) or `[76.0, 36.3, -292.0]` (a
  chest at 0.53), freeze at update 260.
- `tools/windows/g1-smash-barrel.json` (GDL_INPUT_SCRIPT): the Sorceress
  breaks the barrel above G1's start at about update 111; freeze at
  updates 113 to 150 to see the break.
- Blender 5.3.0 (portable) is unpacked beside the repository in
  `E:\Emulation\PORTS\GDL PS2 part\tools\blender-5.3.0-windows-x64\`.

The Python tools under `tools/python/` (`frozen_frames.py`, `perf_bench.py`,
`menu_flows.py` …) assume Linux and Xvfb. Treat them as references, not as
ready-to-run Windows tools.

## 7. Current work: the texture phase (agreed with the owner 2026-11-22)

**Why textures first:** the replacement system already exists. It's where the
owner's ChatGPT art fits in. New models will need the same asset pipeline
later. And effects look better mostly through their textures.

### What exists (read on 2026-11-22)
- **`gdl_client/src/texture_pack.rs`**
  - A texture's name is the first 16 hex digits of SHA-256 over width (u16
    LE), height (u16 LE) and the decoded level-0 RGBA with GS alpha (0x80 =
    opaque).
  - `GDL_TEXTURE_DUMP=DIR` writes `<name>.png` with alpha doubled to 0..255,
    and appends `<name>\t<TEXDEF label>\t<w>x<h>\t<flags>` to `index.tsv`.
    Flags 0x4/0x8 mean U/V clamp; otherwise the texture wraps, i.e. tiles.
  - A pack's `PACK\<name>.png` can be any size up to 4096²; its alpha is
    halved back. Only one mip level is used; `texfilter` regenerates the rest
    in "sharp" mode.
  - The user's pack (`Settings::texture_pack`, read once at startup) wins over
    the generated HD pack (`set_generated`).
  - Lightmaps and HUD z-masks are never replaced.
- **Where replacement happens:** at load time, in `gdl_client/src/lib.rs`
  `prepare_texture()` (world, model, item and effect textures; worker
  threads) and `hud_image()` (HUD). `texture_pack::configure()` is called in
  `gdl_play.rs` near the HD setup.
- **Live switching exists for filters:** `gdl_client/src/texfilter.rs` keeps a
  `REGISTRY` of `AssetId<Image>` → `TexKey`, and `apply()` switches every
  registered texture live. Materials have to be prepared again two frames
  later, because a material keeps the texture views it was prepared with (see
  `docs/research/enhancements.md`, "Switching live"). That mechanism is the
  base for an F5 reload.
- **Real dump sizes:** a G1 session dumps about 2,062 textures (11 MB); the HD
  generator saw 13,742 textures in total, mostly 16–64 px.

### Planned tasks (tasks 1–3 done 2026-11-22)
1. **Default pack folder and one-step dump.** Done 2026-11-22 (the owner's
   choices: the folder beside the game, an F10 row;
   `docs/decisions/0009-texture-workflow.md`):
   - `texture_pack\` beside gdl_play.exe is the pack (made at startup with a
     README; Settings::texture_pack_folder, off in the baseline).
   - F10 → Graphics → "Save textures" saves into `texture_dump\` beside the
     game while it runs; off at every start.
   - `texture_dump\index.html`: the textures by source (characters, levels,
     shared packages, HUD) and label, frames together, replacements marked.
   - TEXDEF labels are readable names (YEL_SOR_FACE, BARN_WALL,
     FIRE_POTION08); sources come from `sources.tsv`.
2. **F5 live reload.** Done 2026-11-22 (`gdl_client::texture_reload`,
   decisions/0009 "F5"): F5 in play swaps changed, new and removed
   replacements in place and says what changed where the frame rate is.
   HUD pictures too since 2026-11-22 (the owner is redrawing the pickup
   notices). A big reload costs a hitch (142 textures: 1 s); reading and
   chaining could move to worker threads.
   - Sharper fonts: done 2026-11-22 (decisions/0013). A font file in
     `texture_work\fonts` named after a font's texture (8HIFONTS.ttf: the
     crystal count "1/100" and "LV 1") is drawn into that font's page.
   - Also found and fixed on the way: soft glows sharpened by the HD
     upscaler (decisions/0010, PROCESSING 2), an error box when the
     game quit during HD generation, and the yellow Wizard's magenta hand
     glow (an upstream bug: research/player.md; the owner confirmed it fixed).
   - To do (the owner, 2026-11-22: "not sure that really a problem"):
     small effect sprites kept original by HD (the Red Wizard's sparks) show
     a dark rim when zoomed in: their clear texels are black, which bilinear
     magnification blends into the edge. The fix: give clear texels their
     kept neighbours' colour at load (as hd::prep's fill_clear), NATIVE and
     off in the baseline.
3. **ChatGPT recipe plus a fix-up tool.** Done 2026-11-22 (the owner's
   choice: a drop folder the game processes; decisions/0009 "Fix-up"):
   `texture_work\` beside the game, its README.txt the recipe. An image
   named after its texture is fitted at startup and on F5 (exact multiple,
   the original's proportions and alpha, clear texels coloured) into
   `texture_pack\`. Not done: making repeating textures seamless (the
   recipe asks ChatGPT for it). Not yet tried with a real ChatGPT image.
4. **First targets:** the playable characters and the Tower (the owner's
   art; the tools are ready).
5. **Test build 3.** The builds of a55909e and 81fa091 were installed on
   the owner's PC (2026-11-22); the build with F5 and the fix-up is next.

## 8. Roadmap after textures (owner's priorities, my recommendation)

1. **Quality-of-life for big screens:**
   - HUD layout: done 2026-11-22 (decisions/0011; the owner chose a
     smaller HUD, panels at the edges and no empty panels; F10 → "HUD
     layout..."). Text scaling was offered and not chosen.
   - Controller buttons: done 2026-11-22 (decisions/0012; one layout for
     every controller, in play only; F10 → "Controller buttons...").
   - Camera recentre and tilt: done 2026-11-22 (decisions/0014; the right
     stick click, tilt with the zoom). Maybe mouse camera later.
2. **Models:** a replacement system.
   - Plan: docs/remaster/models-plan.md (2026-11-22). The owner chose
     materials first, the game's light as the base with live light on top,
     "an evolution of the game's core look", and optional extra grass.
   - M1, material detail: built 2026-11-22 (decisions/0015; F10 →
     Graphics → Material detail; surface types in texture_pack\materials.tsv,
     guessed from the labels, gems included).
   - M2, water effects: built 2026-11-22 (decisions/0016; F10 →
     Graphics → Water effects; C1's channels show it well).
   - M3, plant effects: built 2026-11-22 (decisions/0017; F10 →
     Graphics → Plant effects; G1's moss and grass).
   - The owner's idea, a gem glow: built 2026-11-22 (decisions/0018; F10
     → Graphics → Gem glow). Crystals, runes and gargoyle items glow in
     their colour and flare while their pickup sparkle plays. Part of
     item 3, effects.
   - The owner's request, the same glow for magic potions: built
     2026-11-22 (decisions/0019; F10 → Graphics → Gem and potion glow).
     Potions glow in their bottle's colour; with no pickup sparkle of
     their own, they flare from the pickup for 0.75 s (the owner's
     playtest: one second was a little long).
   - Frames the same from run to run: fixed 2026-11-23 (decisions/0020).
     With ambient occlusion and sharpening on, Bevy ran SMAA and
     sharpening in either order; SMAA now runs first. Comparisons of
     frozen frames are exact again.
   - M4, the exporter: built 2026-11-23 (decisions/0021; the owner: "Yes,
     start it", the exporter and a test barrel before effects).
     `gdl_play --export-models CONFIG.json OUT_DIR [NAME ...]` writes
     every object of every level and shared package as `.glb`, and each
     level's scene with its COLLISION and WATER layers. The owner's
     Blender experience is basic, so the owner chooses and our scripts do
     the work (models-plan.md "Tools for the owner").
   - M5, first step: the model pack and the fitting script, built
     2026-11-23 (decisions/0022). `model_pack\<package>\<LABEL>.glb`
     beside the game replaces that object; tools/blender/fit_model.py
     fits any model to the original. Tested with a Blender-made stand-in
     barrel in G1.
   - The owner's barrel (a ChatGPT picture, a free TRELLIS demo on
     Hugging Face, fit_model.py's bake): in the owner's game 2026-11-24,
     the owner's playtest "Looks good". G1's barrel is a flip-book of 26
     objects (models-plan.md M4).
   - Its breaking frames (decisions/0023, 2026-11-24):
     tools/blender/break_frames.py cuts the fitted barrel into the
     original's ten pieces and moves each as the original's piece moves;
     the model pack shares one texture across the 26 files.
   - The red and poison barrels (decisions/0024, 2026-11-24): the owner's
     barrel painted as the originals are (tools/blender/recolour.py),
     bare wood inside, the same break; checked with G1's red barrel; the
     owner's playtest 2026-11-25, "look like they work right".
   - Every realm's barrels (decisions/0026, 2026-11-24): ten item
     packages, 29 barrels, 754 files sharing their textures and geometry
     (74 MB); the loader reads data beside a .glb.
   - G1's chests: the art brief written 2026-11-25
     (docs/remaster/art-briefs/chest.md; the owner makes the picture and
     the model). One chest shape serves every world but C (desert) and the
     Tower (none); the lid is its own part, hinged at the back edge.
   - The owner's chest arrived 2026-11-25, made in Blender by ChatGPT, with
     a fitted review pack for CHEST, CHESTEXP and CHESTG2 (base and lid
     each, 3,500 and 2,749 triangles, the original heap kept in the G2
     base), kept in the owner's art folder outside the repository. Checked
     in a scratch copy: the six files load, fit the original frames within
     about 0.1 unit, and a CHESTEXP chest opens on its hinge with its time
     bomb rising as the original's. Its look is lighter and plainer than
     the original and the brief (pale wood, a thin dull trim, small
     rivets); the owner decides on a revision before it goes in.
   - The owner's "Can we make the wood on the chest closer to barrel wood
     color?" (2026-11-25): tools/blender/match_colour.py on the six files
     (the wood group only: 78 % of a base texture, 65 % of a lid's; the
     brass and the G2 coins unchanged; the three bases and the three lids
     still share one texture each). Tried in a scratch G1 beside a barrel:
     matched to the barrel's wood as a whole (hue 18.4°, value 0.38) the
     chest is a darker red-brown; to its staves alone (--ref-group lighter:
     19.2°, saturation 0.82, value 0.56) the chest's wood is the barrel's
     orange-brown, the trim then standing out less; matching the spread
     too (--match spread) was pale and blotchy, not used. The owner's
     choice: "let's keep left for now" (the wood as delivered); the tool
     stays for later.
   - The chest installed as delivered (decisions/0028, 2026-11-25, the
     owner's "Install both"), shared like the barrels.
   - The gold heap: the owner's "we should probably make a new gold heap"
     (2026-11-25), then "Let's do all the gold/treasure heaps next": brief
     docs/remaster/art-briefs/treasure-heaps.md for every treasure chest
     (CHESTG0 junk, G1 silver, G2 coins, G3 gold, G4 gold and gems, G5
     gems, CHESTSG gold in the iron-banded chest) and the iron-banded
     chest's textures (CHESTS, CHESTSG), with the room under the fitted lid
     measured (y 0.38 in the middle, 0.18 at the bevels). Found with the
     project's ANIM parser (gdl_formats::animation, ITEMS_G): every
     treasure tree's base and lid nodes are CHESTG2's (base y 1.427, z
     -1.261; lid y 1.426, z -1.260), CHESTS's are CHEST's (lid z -1.342),
     as the review pack's fit read them; every lid is CHESTLID's shape, the
     treasure bases the plain base without its inside. The original gem
     heaps (G4, G5) rise to 0.51, above the fitted lid's inside. HealItem
     (one hero class's healing attack on a chest holding a little gold)
     swaps a chest's tree to CHESTG1 (level 25) or CHESTG3 (level 50) in
     any level, so those show the original chest until their files exist.
     Offered stand-ins (the fitted base and lid with the original heaps,
     not possible for G4 and G5); the owner's choice: "Wait for the art".
   - The heaps delivered 2026-11-25 (ChatGPT in Blender, the owner's art
     folder, outside the repository): seven heaps (1,948 to 4,478
     triangles, peaks 0.23 to 0.345, a buried band at y -0.079 to -0.058
     meeting the walls) in full bases for CHESTG0..G5 and CHESTSG, and the
     iron-banded CHESTS base with iron lids CHESTSLID and CHESTSGLID.
     Checked here: shells, lids and the brass texture identical to the
     installed chest's; every heap within the brief's limits; the least
     gap under the shut lid 0.092 (G1; ChatGPT's own 0.081, sampled
     otherwise); A1's and K1's ITEMS_A and ITEMS_K have the same tree
     nodes; in a scratch copy A1's silver and gem chests and K1's junk and
     coin chests open with their new heaps (the junk's original props
     standing out of it), and the shared layout draws identically. The
     brass treasure lids are copies of CHESTG2LID under each tree's label.
     Not installed: the owner's review of the comparison chart first.
   - The owner on the review: "the new heaps lose a bit of the shiny".
     The original heap textures get material detail's metal and gem maps
     from their labels; model pack files got none. The owner's choice
     "Real shine": decisions/0029, a package's materials.tsv gives listed
     materials detail maps (the heaps metal and gem).
   - Installed 2026-11-25 (the owner's "Yes, install"): the build of
     5930731 and, in model_pack\ITEMS_G, every treasure chest kind (the
     heaps' files, the iron-banded chest, the brass lids as CHESTG2LID's
     copies; shared) with the heaps' materials.tsv. The old coin heap's
     two data files stay in the folder, unused.
   - Every world's chests (decisions/0030, the owner's "Yes, every
     world"): the same 20 files and list in ITEMS_A, B, D, E, F, H, I, J,
     K and S (the same objects, nodes and art as ITEMS_G's; not C's desert
     chests). They made a level's load 2.3 s longer; the model pack loader
     now makes the textures' mipmaps and detail maps on the machine's
     threads (about 0.6 s longer). GDL_FRAME_LOG prints PERF model_pack per
     package.
   - Items in shade (decisions/0027, 2026-11-25, the owner's "Yes, try
     it"): items take the baked light of the ground under them, darker in
     shade, torches and spells still lighting them; F10 → Graphics →
     "Items in shade". For original and replaced items alike.
   - Next (models-plan.md M5): the owner's chest, F5 reload. Light carried
     over from the baked scenery (option A) is for replaced scenery (M6):
     items have no baked light (models-plan.md "Lighting new models").
   - Characters' skeletons come with M8.
   - Load replacements keyed to the original model packages.
   - Start with props and terrain (static; collision is separate data), then
     characters (they must fit the original rigs and animations). Image-to-3D
     tools (Meshy, Tripo, Hunyuan3D) can make rough models from ChatGPT
     concept art, but Blender cleanup and rigging is unavoidable; scripts can
     automate much of it.
3. **Effects and particles:** mostly through textures first, then more or
   softer particles.
   - The owner's wish (2026-11-24, "we don't have to do that now"): the
     poison and red barrels' effects upgraded, now "kinda a flat mist",
     to something more volumetric. Not started; the effects are the
     game's particle systems (gdl_core psys), so a NATIVE presentation
     upgrade of how they draw, the original kept in the baseline.
   - Later, the owner's wishes (2026-11-24, "we can add physics later down
     the road"): debris that answers the hit (the pieces of a replaced
     breakable pushed away from the spell or blow that broke it, falling
     and settling on the level's collision), remastered death animations
     and bodies, and decals: blood of the right colour, scarring and
     scorch marks from attacks and spells. All presentation only (NATIVE,
     off in the baseline; the simulation never sees them). Not started.
     Open: where the game keeps what hit an item and from where (DamageItem
     callers); a physics library (Avian or Rapier; Bevy 0.18 support not
     checked) or a small own solver (AGENTS.md rule 9: the owner decides a
     new dependency); the cheap first step is the original's break turned
     away from the hit and widened by its strength.
   - The owner's wish (2026-11-25, "Also need to add water interaction to
     the list"): water interaction. My reading, to confirm with the owner:
     ripples, splashes and wakes where players, enemies, missiles and
     debris meet water (water effects, decisions/0016, already ripple the
     surfaces). Presentation only. Not started.
4. **Gameplay changes:** behind a "Remastered rules" option, with original
   rules kept bit-identical. Note: network play needs identical settings on
   every machine.

**PCSX2 was discussed and set aside.** It already has a widescreen patch for
SLUS-20047 (`PCSX2/pcsx2_patches`, `SLUS-20047_15030637.pnach`), upscaling
and texture replacement. A camera mod there would mean pnach machine-code
patches or a PINE companion program. New models are impossible within PS2
memory. The port stays the base.

## 9. Conventions to keep

- **AGENTS.md applies.** In this fork, remaster features are *NATIVE
  additions*:
  - label them,
  - keep them out of `Settings::baseline()`,
  - keep the original mode bit-identical,
  - record decisions in `docs/decisions/` and update the research docs.
- **The simulation stays deterministic** and independent of presentation.
  Anything that changes game logic must come from pad input plus a setting
  that every network machine shares (or be off in network play).
- **Be honest about verification.** Say which checks are synthetic tests,
  which ran on the real disc, and what only the owner's playtest showed.
- **Keep the owner's test loop simple:** one zip or exe, a READ-ME-FIRST, and
  a backup-and-undo step. Better still, run things yourself locally.

## 10. Open items checklist

- [x] Native Windows build works (§5): GNU toolchain, 2026-11-22; the
      same simulation hashes as test build 2.
- [x] Read `remaster-log.txt` (2026-11-22): a 4-channel device with no
      stereo mode (§3). Mention it on the upstream PR if useful (owner
      approves any comment).
- [x] Check the upstream PR's status (`windows-sound-fix`):
      jeddhor/GauntletDLRedone#1, open, no comments or reviews on
      2026-11-22. Check again later.
- [ ] Texture phase (§7): tasks 1–3 done 2026-11-22; the owner's art next.
- [x] HUD layout (§8, decisions/0011): the owner's playtest 2026-11-22,
      "Everything looks good so far".
- [x] Controller buttons (§8, decisions/0012): the owner uses them with an
      Xbox One controller (2026-11-22).
- [x] Camera recentre and tilt (decisions/0014): the owner's playtest
      2026-11-22, "that's pretty good!".
- [x] Sharper fonts (decisions/0013): the owner's playtest 2026-11-22 with
      Palatino Bold as 8HIFONTS, "Confirmed working and I like it".
- [x] Material detail (decisions/0015): the owner's playtest 2026-11-22,
      "Looks good".
- [x] Water effects (decisions/0016): the owner's playtest 2026-11-22,
      "Looks good". G1's swamp fixed afterwards (decisions/0017): it
      needs a look in play.
- [x] Potion glow (decisions/0019): the owner's playtest 2026-11-23 of
      the 0.75 s flare, "yes .75 feels much better".
- [x] Barrels missing, chests and portal spirals still (the owner's
      playtest 2026-11-23, G1, one session): fixed 2026-11-24
      (decisions/0025). Item trees were looked up once per name for the
      whole run, so a level could pose its items with the previous
      world's trees; reproduced with G1 then the Tower. The owner's
      playtest through the Tower next.
- [x] Frames that varied from run to run (decisions/0020): fixed
      2026-11-23, SMAA before sharpening.
- [ ] Interpolation while the simulation waits (research/unknowns.md):
      a hypothesis that what moved can shake while a scroll's message box
      is up; look at it in play.
- [x] Gem glow (decisions/0018): the owner's playtest 2026-11-22, "Looks
      good".
- [x] Plant effects (decisions/0017): the owner's playtest 2026-11-22.
      Moss, bushes and corn sway; G1's ground grass clumps do not, and
      the owner wants them replaced in the vegetation overhaul instead.
- [ ] Camera follow-ups (§3) as the owner asks.
- [x] Model exporter (decisions/0021): built and checked in Blender 5.3.0
      2026-11-23.
- [x] Model pack (decisions/0022): the owner's barrel installed
      2026-11-24; the owner's playtest in G1, "Looks good".
- [ ] Breaking frames (decisions/0023): built and checked in G1
      2026-11-24; the owner's playtest next.
- [x] Red and poison barrels (decisions/0024): built 2026-11-24, the red
      one checked in G1; the owner's playtest 2026-11-25 with every
      realm's barrels installed, "Poison and red barrels look like they
      work right".
- [ ] Barrels in every realm (decisions/0026): built 2026-11-24, checked
      in G1 and A1; the owner's playtest next.
- [ ] Items in shade (decisions/0027): built 2026-11-25, checked in G1,
      A1 and D1, installed 2026-11-25; the owner's playtest next.
- [ ] The owner's treasure chest (decisions/0028): installed 2026-11-25;
      the owner's playtest next (open one with a key; the gold chest in
      the swamp).
- [ ] The treasure heaps and their shine (decisions/0029): installed
      2026-11-25 in G1; the owner's playtest next.
