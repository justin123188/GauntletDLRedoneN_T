# 0009: Texture workflow — a pack folder, saving from the menu, a browser page

This fork's texture phase (the owner's plan, 2026-11-22): the owner makes
replacement textures, with ChatGPT for the art, and should need neither
hand-edited files nor environment variables. texture_pack (M19,
research/enhancements.md "Texture replacements") already replaces a texture
by its name from Settings::texture_pack and dumps the decoded textures with
GDL_TEXTURE_DUMP; this adds the steps around it. All of it is presentation
(no simulation code reads it), labelled NATIVE, and off in
Settings::baseline().

Decision:

- **The pack folder beside the game.** The owner's choice (2026-11-22, over
  a folder beside the saves): `texture_pack\` beside gdl_play.exe, made at
  startup with a README.txt, is the pack when Settings::texture_pack names
  no path. Settings::texture_pack_folder (default on; off in the baseline
  and after the overlay's Original presentation; the Graphics page's
  "Texture pack folder" row) switches it. It is read at startup, like the
  path.
- **Saving from the overlay.** The owner's choice (over a launcher .bat):
  the Graphics page's "Save textures" row starts and stops the dump into
  `texture_dump\` beside the game (texture_pack::set_dump). Textures decoded
  from then on are saved: the select screen's, each level's, the HUD's. It
  is not a setting and is off at every start, so it never runs unnoticed
  (a dump prepares textures on one thread: loads are slower).
  GDL_TEXTURE_DUMP still starts it for research runs.
- **Where each texture came from.** gdl_play names the source before it
  prepares a package's textures (texture_pack::set_source): the level
  ("level", e.g. "G1 The Poison Fields"), the players' model and effect
  packages ("character", PLAYERS/SOR/YEL00 and PLAYERS/SOR/SFXYEL), the
  shared packages ("package": WEAPONS, POWERUPS, ITEMS_G, the monsters) and
  the HUD sets ("hud": STATIC, SELECT, a player's model). The dump writes
  `sources.tsv`, one line per new (texture, source) pair, so a texture
  shared by several packages is listed under each. index.tsv keeps
  upstream's four columns (upscale_compare.py and gdl_play --hd-check read
  exactly four).
- **index.html.** gdl_client::texture_browser writes a static page into
  texture_dump when saving starts, after new index or sources lines (every
  2 s at most) and when saving stops: by kind (characters, levels, shared
  packages, HUD and menus) and source, then by label, a texture's frames
  together (labels that differ in a trailing number), each with its file
  name, size, wrap mode (record flags 0x4/0x8) and a "replaced" mark when
  the pack has its file. It opens from the folder with a double click and
  is never uploaded: the textures are the disc's.

Verification (2026-11-22, on the owner's PC with the actual disc's import;
the builds ran from scratch copies with APPDATA redirected): unit tests (the
dump's lines, the page's grouping, the overlay rows, the baseline
settings). TEST runs in G1 (tools/windows/sim-hashes.ps1): with the remaster
defaults texture_pack and its README appear beside the exe, with
GDL_BASELINE=1 nothing; the dump's 2063 textures all have a source
(distinct textures: the level 134, the character 105, the shared packages
1826, e.g. WEAPONS 827 and POWERUPS 406, the HUD 16);
Edge (headless, local) renders index.html with the groups and thumbnails;
134 red/blue-swapped copies of G1's level textures in texture_pack changed
exactly the level's surfaces in a frame-700 screenshot (the character,
items, enemies and HUD unchanged) and the page counted 134 replaced. The
600 per-update simulation hashes equal the reference in every run. Not
checked: the overlay rows with a keyboard or pad in the running game (the
owner's test).

Not done yet: reloading the pack while the game runs (F5), the fix-up tool
for AI images and the recipe.
