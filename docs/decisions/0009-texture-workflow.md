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
600 per-update simulation hashes equal the reference in every run. The
owner's playtest (2026-11-22, test build of a55909e): "It works good and
looks ok".

## F5: reload while the game runs (2026-11-22)

- **What.** F5 in play (gdl_client::texture_reload; not while the overlay
  is open; no F key is bindable) asks texture_pack again for every base
  texture materials_for made: the user's pack, then the generated HD pack.
  Where the answer changed (another file, a file written since: its path,
  modification time and length, or no file) the same image takes the new
  texels: a replacement's one level, or the original's levels, taken from
  the image the first time a reload replaces it (no copy is held before:
  no memory until the feature is used). texfilter's chain and sampler
  follow the new level count; the materials are prepared again two frames
  later (texfilter's live switch). The result is shown for 4 s where the
  frame rate is ("Textures reloaded: 3 replaced"); a saving texture_dump's
  index.html is written again.
- **Limits.** Texture-modifier targets keep copying their frames (the
  frames are reloaded). HUD images and lightmaps are not reloaded (the next
  start). A texture replaced when its level loaded was never in memory as
  the original: removing its file takes effect at the next level. A
  replacement whose alpha class (opaque, cut-out, blended) differs from the
  original's keeps the material's until the level loads again (counted in
  the message).

Verification (2026-11-22, the owner's PC, the actual disc's import, a
scratch copy with APPDATA redirected): unit tests (a replacement, an
unchanged and a changed file, the original's levels back, a borrowed
target left alone, a texture replaced at load, an alpha class change, a
removed image, the messages). A G1 TEST run with GDL_TEST_ACTIONS
`{"at": u, "reload_textures": true}` (the same path as F5): 134
red/blue-swapped G1 level textures copied into texture_pack at update 153,
a reload at update 300 ("142 replaced": some textures serve several
records), the files deleted at update 403, a reload at 500 ("142 back to
the original"); screenshots show the level's surfaces swapped two frames
after the first reload and the originals after the second; the 600
per-update simulation hashes equal the reference. The first reload's frame
took 1.04 s (142 textures read and their chains built at once), the second
0.14 s. Not checked: F5 from the keyboard in the owner's game.

## Fix-up: images made with ChatGPT (2026-11-22)

The owner's choice (over a separate drag-and-drop tool): a drop folder the
game processes itself, so there is nothing extra to run.

- **texture_work\ beside the game** (gdl_client::texture_fix), made at
  startup with README.txt: the ChatGPT recipe (save textures, find one on
  index.html, the prompt, the extra lines for repeating and see-through
  textures, where to save the image, which textures suit AI). An image
  named after its texture (`<name>.png`; words after the name allowed, as
  ChatGPT's downloads get renamed) is fitted at startup, before anything
  loads, and on F5, before the reload; then it moves to texture_work\done.
- **The fit.** The original from texture_dump (`<name>.png`) and its record
  flags (index.tsv). Size: an exact multiple of the original, the image's
  own scale rounded (1..8, at most 2048 texels a side), in the original's
  proportions (ChatGPT's square or 3:2 images are squeezed back), resampled
  with a Catmull-Rom kernel widened when shrinking. Alpha: the original's,
  upscaled bilinearly (wrapped or clamped like the record), a cut-out's
  thresholded at half, an opaque original opaque: an AI image's own alpha
  is ignored (cut-out masks do not survive it, and the geometry expects the
  original's). Clear texels within four rings of kept ones take the kept
  neighbours' colour (hd::prep fill_clear_rings; a full fill at fitted
  sizes took about a second per image and does not reach a filtered sample
  past the first ring). PNG only; up to 4 images at once.
- **Not done:** making repeating textures seamless (the recipe asks
  ChatGPT for it); HUD images still need a restart after F5.

Verification (2026-11-22, the owner's PC, the actual disc's import, a
scratch copy, isolated APPDATA): unit tests (names, scales, resampling,
the mask restored and the clear texels' colour, the folder pass with a
missing original and a non-PNG). No ChatGPT image: 134 stand-ins for G1's
level textures (each blown up to a 1024×1024 square, no alpha channel,
red and blue swapped) in texture_work at startup and 2 for the barrels
dropped in during play: "134 new from texture_work" at startup, "2 new
from texture_work; Textures reloaded: 2 replaced" on F5; screenshots show
the swapped level with its foliage cut-outs intact and then the barrels;
the fitted files have the originals' proportions (CORN_SIDE 64×128 →
512×1024) and masks. Startup with 134 waiting images: 9.1 s to the first
update (2.6 s with none; 128.5 s before the fill was bounded). 40
regenerated HD textures are byte-identical to the p2 pack (hd::prep's
fill_clear unchanged). The simulation hashes equal the reference. Not
checked: a real ChatGPT image, and the owner's playtest.
