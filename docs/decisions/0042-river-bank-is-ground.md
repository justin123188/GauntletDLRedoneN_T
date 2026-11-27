# 0042: G1's river bank is ground, not water (material types by name)

Found while writing the rest of G1's brief (decisions/0041). The game's
name rules (material_maps::guess, decisions/0015) typed RIVER_BANK_03, the
texture of G1's river banks, as water, from the word RIVER. The owner
chose "Yes, fix it" (2026-11-27).

What the water type did to those banks (both NATIVE, off in
Settings::baseline()):
- **Material detail:** water's values, no bump and a wet shine.
- **Water effects** (decisions/0016): the water shader's sheen and glints,
  and its ripples bending the texture's UVs. In the game this showed as a
  bright, glinting streak along the earth bank above the rapids.

The texture is a bank of earth and roots; only its foot shows the water.
The same kind of mistake was fixed for C1's BASIC_WALL_WATEREDGE in
decisions/0016.

The labels (2026-11-27, every material label in the full model export of
decisions/0021: 4,876 labels from 90,303 files):
- **BANK** appears in 3 labels:
  - RIVER_BANK_03 (G1), with BANK as a run of its own;
  - MUDBANK (D1) and SANDBANK (C3), run together, which are already
    ground (MUD, SAND).
- **RIVER** appears in RIVER_BANK_03 and in K2's RIVERNEW, RIVERNEW2 and
  RIVERNEW3, which are water.
- Labels of textures that no model uses (the HUD's, for example) are not
  in that export.

Decision (NATIVE; material_maps.rs):
- **The rule:** a ground rule for the whole run BANK (`=BANK`), tried
  after the effects and the WATEREDGE stone rule and before water.
- **Its reach:** of the labels above, it changes only RIVER_BANK_03. A
  materials.tsv line still overrides any guess.
- **The test:** labels_suggest_types adds RIVER_BANK_03 ground, MUDBANK
  and SANDBANK ground, and RIVERNEW and RAPIDS00 water.

Verification (2026-11-27, the owner's PC, the scratch copy of the game
with no G1 level folder; the build of this change, md5 d7ae700a, against
the installed build of decisions/0037, 65051ea4):
- **Tests and clippy:** gdl_client 87 + 5; clippy has nothing new.
- **The listing:** GDL_DEBUG_MATERIALS gives "RIVER_BANK_03 water guess"
  in the old build and "ground guess" in the new.
- **G1, at the west river bank** (the spot a2 of decisions/0041):
  - The original presentation (GDL_BASELINE=1): identical.
  - The defaults: 59,457 pixels differ, all in the box (1266, 450) to
    (1599, 899), which holds the bank. The streak is gone, and the bank
    reads as the earth beside it.
  - Material detail off, with water effects on: 59,614 pixels differ in
    the same box, from the water effect alone.
- **Identical with the defaults:** G1 at its start and at two other spots
  (a4, b2); C1 (BASIC_WALL_WATEREDGE); C3 (SANDBANK); D1 (MUDBANK).
- **The simulation hashes** over 600 updates of the G1 walk equal the
  reference, in the defaults and the baseline.
- **The picture:** the art folder's G1_Ground/G1_RiverBank_Type_Fix.png
  (the bank before and after, with the defaults and with material detail
  off).

Not yet: the install.
