# 0052: The crystals, checked; the original's texture slide carried to replacements

Part 1 of the gems brief (decisions/0050, art-briefs/gems.md) came back
on 2026-11-27: the art folder's Gems/review_pack/POWERUPS, with renders,
ChatGPT's checks, its source scripts and a hand-off note.
- **The crystals:** 16 files, the eight colours' shells (XSNS) and cores
  (XP), one new cut shared by all eight.
- **The hand-off's report:** the shell's sheen stays fixed in the game,
  where the original's slides.

Checked here (2026-11-27, the owner's PC):
- **The files:**
  - 256 triangles each;
  - each exactly its original's bounds, to the thousandth (the shells x
    -0.438 to 0.477, y 0.969 to 4.258, z -0.383 to 0.461);
  - one buffer, no extensions, no animation;
  - the materials named as asked (GEM<colour>_shell opaque, GEM<colour>_
    core blended);
  - one embedded 512 px RGB texture each;
  - the shells' textures wrap: the edge columns and rows match.
- **The report is right.** The game slides a texture (DoTexScroll2Sub,
  present_texmods) by finding the materials whose base image is the
  original's. A model pack replacement starts as a copy of the
  original's material, but with an image of its own, so no slide reached
  it. With the build of decisions/0049 and the crystals, the new red
  shell's shift was 0, 0 at updates 242 and 260. The original's at those
  updates is 0.75, 0.9167 and 0.65, 0.2167 (decisions/0050).

Decision (NATIVE in what it changes; nothing without a model pack):
- **GsMaterial::scroll_from:** the original texture a model pack
  replacement stands in for. model_pack sets it from the original's
  material (its base image, or its own scroll_from). The game's own
  materials have none.
- **present_texmods** also indexes the materials by scroll_from. A
  scroll of the original's texture sets the replacement's shift too.
  Frame animations are unchanged: a replacement keeps its own texture.
- **The limit:** the replacement follows its original object's first
  part, whose material the model pack copies. An original with several
  parts, only some of them scrolled, gets that first part's slide.

Verification (the scratch copy; the build of this change, md5 4986cfa7,
against 6e119bd2):
- **Tests and clippy:** gdl_client 89 + 5. Clippy has nothing new.
- **Without the crystals, identical frames:**
  - G1's crystal spot and b1, with the installed G1 pack;
  - C1's start;
  - G1 with GDL_BASELINE=1.
- **With the crystals,** the scene dumps of G1's red crystal:
  - the new shell's shift is 0.75, 0.9167 at update 242 and 0.65, 0.2167
    at 260, the original's;
  - against the build before, the frames differ by 7,869 pixels, at the
    crystals.
- **The crystals in the game** (with the defaults, the original's against
  the new, the art folder's Gems/Gems_Crystals_Checked_InGame.png):
  - G1 (red, at updates 242 and 260), B1 (purple), C1 (white), D1 (black
    and white) and L1 (orange) load all 16 files, with no messages;
  - the new crystals show their cut facets, and the sheen moves over
    them;
  - L1's view is behind the Tower's welcome scroll, so its orange
    crystals are not seen there.
- **The simulation hashes** over the 600-update G1 walk, with the
  crystals, equal the reference in the defaults and the baseline.

## Installed

2026-11-27 (the owner's "Yes, install"):
- **The build of this change** (md5 4986cfa7). The previous one
  (6e119bd2) is kept as gdl_play-texslide-backup.exe.
- **The 16 crystal files** in model_pack\POWERUPS, byte-identical to the
  review pack. No crystal file was there before, so to roll back, delete
  them. No materials.tsv lines.
- **The model pack:** 1,066 .glb, 1,400 files, 353 MB.

Checked in the installed game (an isolated APPDATA):
- **C1 and D1** are identical to the checked frames above, but for the
  HUD's "LV 1" label (619 pixels, the owner's font; decisions/0047).
- **G1:** the installed game has G1's new ground (decisions/0051), which
  the checked frames above did not. Against the scratch copy's G1 frame
  with that ground and the original crystals, 34,624 pixels differ (the
  crystals and the label), as the crystals alone differ (34,835).
- **G1's red crystal** draws as checked.
- **All 16 files load** in each level, with no messages.

Not yet:
- the owner's look in the game;
- the gems brief's parts 2 and 3, the gargoyle pieces and the runestones.
