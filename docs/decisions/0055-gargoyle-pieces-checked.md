# 0055: The gargoyle pieces, checked

Part 2 of the gems brief (decisions/0050, art-briefs/gems.md) came back
on 2026-11-27. The art folder's Gems/review_pack/POWERUPS adds
GARGEAGLGROUP1, GARGLIONGROUP1 and GARGSERPGROUP1 (the eagle's feather,
the lion's claw, the serpent's fangs). Beside them:
- three materials.tsv lines (Gems/Gargoyle_material_additions.tsv:
  GARG*_gold metal bump=0.9 rough=0.22, as the treasure's coins);
- renders, ChatGPT's checks, its source script and a hand-off note.

Checked here (2026-11-28, the owner's PC):
- **The files:**
  - 2,736, 2,784 and 2,628 triangles (the originals 202, 334 and 313;
    the brief's limit 3,000);
  - each exactly its original's bounds, to the thousandth;
  - the same single node as its original, with no transform, so the
    feather stays centred on its node;
  - closed surfaces: no open edges, and no edge on three triangles;
  - one buffer, no extensions, no animation;
  - one opaque material each, named as asked, with one embedded 1024 px
    RGB texture.
- **The original is drawn by its UVs:** the client has no reflection
  mapping, and GOLD_REF is opaque (alpha class 0). The game slides or
  animates none of the gold (decisions/0050's texture modifier check).
- **In the game,** with the installed build (5b8aaffd) in the scratch
  copy and the three lines merged into its POWERUPS table, at update
  242 from the hand-off's close-up cameras (GDL_DEBUG_CAMERA):
  - A1's feather, C1's claw and G1's fangs: the three files load in each
    level, and there are no messages.
  - Each piece is at its original's place and size. The feather's barbs
    and shaft, the claw's sheath and the fangs' scales show (the art
    folder's Gems/Gems_Gargoyle_Checked_InGame.png).
  - **The glow changes colour a little.** Gem glow lights a pickup's
    surroundings in its texture's colour (gem_colour: the mean, its
    largest channel 1). The new gold is less saturated:
    - the original GOLD_REF's light is 1.00, 0.75, 0.12;
    - the new pieces' 1.00, 0.72, 0.27 (the feather), 1.00, 0.68, 0.20
      (the claw) and 1.00, 0.73, 0.26 (the fangs), as the hand-off's
      telemetry gives.
    - The walls and floors near the pieces are 1 to 3 levels bluer. At
      A1, 470,839 pixels differ, 402,432 of them by 1 to 3. With bloom
      off it is 426,372, so it is not the bloom.
  - **The simulation hashes:** decisions/0054's run had the pieces too.

Not yet: the owner's choice on the install; part 3, the runestones.
