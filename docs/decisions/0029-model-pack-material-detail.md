# 0029: Material detail for model pack files

The owner's notice (2026-11-25), on the treasure heaps' review: "the new
heaps lose a bit of the shiny"; the owner's choice: "Real shine" (against
brighter painted highlights in the heap textures).

Why the new heaps were duller (read 2026-11-25):
- **The originals.** With Settings::material_detail on (the remaster's
  default), every original texture gets a detail map whose surface type is
  guessed from its TEXDEF label (material_maps::guess): TREASR_GOLD,
  TREASR_SILVER, TREASR_COINS3 and SILVERCHST_GOLD are metal (GOLD, SILVER,
  COIN), TREASR_GOLD_GEMS and TREASR_GEMS gem (^GEM, checked first),
  TREASR_JUNK wood (TREASR). So in the remaster the original heaps glint
  under the torches, spells and the sun, besides their painted highlights.
- **The model pack** (decisions/0022) drew every replacement without a
  detail map, so the new heaps, the chest's brass and the barrels' iron
  got no glints. Their files cannot say "gold" either: the heaps declare
  metallic 0 and roughness 0.93 (made for the game's diffuse light), and
  their material names (CHESTG3_treasure) would guess wood.

Decision (gdl_client::model_pack, material_maps::make_for and
parse_named; NATIVE, under Settings::material_detail, which is off in
Settings::baseline() and the Original preset, as is the model pack
folder):

- **The list.** `model_pack\<package>\materials.tsv` beside a package's
  files, in the texture pack's line form (`<name> <type> [bump=… rough=…
  metal=…]`, `#` a comment): the name is a glTF material's `name` as the
  files have it (case kept; any word up to 128 bytes). Lines not
  understood are reported in the model pack's messages.
- **The map.** While material detail is on (read as a level loads, as for
  the game's own textures), a listed material's base colour texture gets a
  detail map made from it for that type (material_maps::make_for: the
  generator of decisions/0015 with the line given, no texture pack files),
  once per texture and line in a package. The texture's own texels set
  the bump's scale (no disc record behind it). The material takes it as
  the game's textures do (GsMaterial::detail, GsParams depth.y; depth.z
  for water or plants).
- **Unlisted materials** draw exactly as before. No list: no change.
- **Not now.** A glTF file's own normal and metallic-roughness textures
  (ChatGPT's chest master has them): per-texel shine, the brass bright and
  the wood dull in one baked texture. The chest's and barrels' materials
  are not listed (wood and brass share one texture).

G1's list for the treasure heaps (the art in the owner's folder; the list
beside them in model_pack\ITEMS_G): CHESTG1, G2, G3 and G4's heaps metal
(silver, coins, gold, gold and gems), CHESTG5's gem, CHESTG0's junk stone;
CHESTSG shares CHESTG3's heap.

Verification (2026-11-25, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS; the build of this change, md5 5f847ef8, against
the installed one, 852c95fd):
- **Without a list:** G1 and C1 with the installed layout identical to
  the installed build; with every chest kind's files (the treasure heaps'
  review pack in scratch ITEMS_G, ITEMS_A, ITEMS_K), A1's silver and gem
  chests and K1's coin chest, opened, identical too. (A first shot of the
  gem chest differed across the whole frame; the installed build gives
  the same frame twice there, equal to this build's: that one shot was
  off.)
- **The original presentation** (GDL_BASELINE=1, the list present): G1
  identical.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the list present, in the defaults and the baseline.
- **With the list** (the defaults first: metal and gem as they are): K1's
  coins catch a sheen on the side towards the light (12,537 pixels); the
  gems a few sparkles (817); A1's silver little in that view (214; why
  not measured). Sharper (bump=0.9 rough=0.22 for the metals, bump=0.5
  rough=0.05 for the gems): the coins glint coin by coin across the heap,
  as the originals' do; chosen. A1's burning cart's flames differ from run
  to run (an effect, not a listed material).
- **The list read:** no line reported as not understood.
- **Tests.** gdl_client 82 + 5 (material_maps: a model pack line decides
  the type over the name's guess, no texture pack files, an unknown type
  reported; model_pack: the material's name read).
- **Installed** 2026-11-25 (the owner's "Yes, install"): this build, the
  treasure chests and the list in model_pack\ITEMS_G; checked first in a
  scratch copy that the list with the shared files draws as with the
  embedded ones (A1's gem chest identical; the silver chest's area
  identical, the burning cart's flames apart), and byte-compared after.
- **Not yet.** The owner's playtest; the other worlds' chests.
