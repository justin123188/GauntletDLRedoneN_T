# 0028: The owner's treasure chest (models-plan M5)

The owner's art (2026-11-25): after the chest brief
(docs/remaster/art-briefs/chest.md), the owner had the chest made in
Blender by ChatGPT, which also fitted it to G1's chests and handed back a
review pack in the owner's art folder outside the repository: an art
master with separate base and lid, renders, a comparison with the
original in the closed and open poses, and six model pack files. The
owner's "Install both" (2026-11-25), after keeping the wood as delivered
("let's keep left for now").

What the six files are (checked here, 2026-11-25):
- **The parts.** CHESTCHST_BOTM and CHESTLID (CHEST), CHESTEXPCHST_BO and
  CHESTEXPLID (CHESTEXP), CHESTG2CHST_BOT and CHESTG2LID (CHESTG2): each
  tree's base and lid, in the original objects' local frames (the base
  from y -1.43 to the rim at 0, the lid hinged at its origin on the back
  edge, z 0). The tree nodes (positions, the lid's swing of about 1.74
  radians) stay the game's; CHEST's NULL1 content node and CHESTEXP's
  explosion frame are not replaced.
- **The model.** 3,500 triangles per base and 2,749 per lid, one 1024 ×
  1024 base colour PNG per piece, shared by the three trees (one texture
  for the bases, one for the lids). The G2 base keeps the original's coin
  heap: its 14 triangles, UVs and 64 × 32 texture.
- **The form the game reads.** One embedded buffer, PNG textures, no
  extensions, no animations, opaque materials; bounds within about 0.1
  unit of the originals'.

Decision: the six files go into model_pack/ITEMS_G as delivered, in the
shared layout (tools/blender/share_data.py: three textures and six
geometry files beside them, the G1 folder 16 to 12 MB). They stay out of
the repository (a fit to the disc's objects, and the owner's art).

Verification (2026-11-25, the build of dd4b985, scratch copies, isolated
APPDATA):
- **Loading.** The model pack reports all six (3,500, 2,749 and 3,514
  triangles in two parts for the G2 base).
- **In play.** A CHESTEXP chest in G1 (slot 462) shut, and opened with a
  key (GDL_TEST_ACTIONS player_or [6676, 1], keys at player +0x1a14, then
  a teleport onto it): the lid swings up on its hinge as the original's,
  and the chest's time bomb rises out of it as with the original chest.
  G1's gold chest (slot 460) stands half in the swamp water; its heap was
  checked in the review pack's renders and by measure (it stays under the
  shut lid).
- **The shared layout.** The chest shut and open from the shared files:
  identical to the delivered files.
- **The installed folder.** Byte-equal to the tested one; the other
  realms' folders unchanged.

The review (for the owner, 2026-11-25): the fit and the swing are right;
the look is lighter and plainer than the original and the brief (pale
wood, a thin dull trim, small rivets). tools/blender/match_colour.py can
put the wood in the barrel's colour (two looks tried, HANDOFF); the owner
kept the wood as delivered for now.

Not yet: the owner's playtest; a new gold heap (the brief
docs/remaster/art-briefs/gold-heap.md); the other worlds' chests (the same
shape everywhere but the desert world C, as the barrels were done); the
iron-banded chests (CHESTS) and the burst chest's remains (CHESTGEXP0,
CHESTSEXP0).
