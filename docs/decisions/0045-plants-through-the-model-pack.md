# 0045: Plants through the model pack; CORN as a word of its own

The game's side of plants (decisions/0044, step 3; models-plan M7), begun
2026-11-27 on the owner's "Yes, start now".

What the model pack already did (model_pack.rs):
- **Cut-out and see-through materials:** glTF MASK (cut at 0.5) and
  BLEND, with the texture's alpha as GS alpha.
- **Double-sided materials.**
- **A materials.tsv type** gives plants the plant effects
  (decisions/0017): depth.z 2 for foliage, 3 for cut-out or see-through
  grass.

What the plant effects are: the sway moves the texture's UVs, not the
shape. That suits the game's own plants, which are see-through cards; a
cut-out's outline moves with the texture. On a modelled, opaque stem the
texture would slide. So replacement plants keep the leaves as cut-out or
see-through cards typed foliage or grass, and the stems typed wood. A
sway that moves the shape needs a vertex shader of the game's own
material, with its prepass and shadows; that is left for later.

The control (2026-11-27, the build of decisions/0042, d7ae700a, the scratch
copy):
- **The setup:** G1's 77 plant objects through the model pack, their own
  exports typed as the game types the originals:
  - the G1XS, G1XL, G1XSNC and G1XLNC families: moss, grass tufts, marsh
    and corn sides;
  - the corn pieces;
  - 12 materials: BRANCH_01, CORN_* and MOSS_BRANCH foliage; GRASS_TUFFS
    and MOSS_WAD_02 grass; MARSH_02 auto; BARN_BROKEN and FENCE_02 wood;
    ROPE cloth.
- **Every file loads,** with no message.
- **With the defaults:** at the start, 204,973 pixels differ (at most
  48). Of them, 74,714 differ by 1, 40,577 by 2 and 26,354 by 3; the
  mean is 0.08 to 0.19 levels darker. The differences cover every plant
  card, the wood fence too, so they come from all of them, not from the
  sway:
  - the see-through value's rounding through the export;
  - the bump map, which a pack material makes from its texture alone,
    while the original's comes from the disc record.
- **With material detail off and plant effects on:** 208,670 pixels
  (at most 62), with the pack's plants still. The loader read
  materials.tsv only while material detail was on, so its plants were
  never marked, while the game's own plants sway with any of the three
  effects on.

Decision (NATIVE; model_pack.rs, material_maps.rs):
- **The model pack's types mark plants and water** whenever material
  detail, water effects or plant effects are on (material_maps::typing),
  as the game's own textures' types do. The detail maps are still made
  only with material detail on.
- **CORN only as a word of its own** (`=CORN`, with CORNFIELD and ACORN)
  in the foliage words. Before, CORN matched inside a run, so with plant
  effects on these textures slid like leaves:
  - BRICKCORNER (the junk chest's brick, every world's items) is now
    stone;
  - ROCK_CORNER and ROCK_CORNER2 (B4) are stone;
  - CORNER_ROOF_RUIN (A1) is stone;
  - CASTLE_WALL_CORNICE2 (A5) is stone;
  - BLACKCORNERTILE and WHITECORNERTILE (J4) are stone;
  - CORNERFLESH (J6) is skin;
  - PATHWAY_RIGHTTOPCORNER (G3) is ground;
  - H_GOTH_CORNR (J2) is auto.

  G1's CORN_* and S7's CORNFIELD stay foliage. That is every label with
  CORN in the full model export of decisions/0021 (4,876 labels).
  CRATE_TOP_BURNT_CORNER and H_CORNR_TABLE* were wood already.
- **DIAGNOSTIC:** the loader's PERF line (with GDL_FRAME_LOG) now also
  gives the PNG decoding, the longest texture job and the light carried.
- **The test:** labels_suggest_types adds the corner labels and the corn.

Verification (2026-11-27, the owner's PC, the scratch copy; the build of
this change, md5 c8d7441e, against d7ae700a):
- **Tests:** gdl_client 87 + 5.
- **The plant control, material detail off:**
  - At the start, 135,369 pixels differ (at most 46). Of them, 77,662
    differ by 1, 24,415 by 2 and 10,541 by 3; the mean is 0.09 to 0.19
    levels darker.
  - At a1, 98,221 (at most 46); at a2, 48,467 (at most 14).
  - Before: 208,670, 157,407 and 55,365, at most 62 to 82. The pack's
    plants now sway with the game's own.
  - With the defaults: as before (204,973 at the start).
- **G1 with the installed ground** (the start area and area A) is
  identical to the old build at the start (defaults) and at a1 (material
  detail off, and GDL_BASELINE=1).
- **The corners,** at each level's start with the defaults, with
  GDL_DEBUG_MATERIALS:
  - B4: 122,255 pixels differ (at most 30), in the corner of the view
    where its rock corners stand still now.
  - A1: 8 pixels. J4: identical.
  - The listing gives each corner label's new type (foliage before).
  - B4 with GDL_BASELINE=1: identical.
- **The simulation hashes** over 600 updates of the G1 walk equal the
  reference in the defaults and the baseline.
- **Where G1's load goes** (the new PERF line, the start area and area A,
  two runs at a1):

  | Phase | Time |
  | --- | --- |
  | Read and decode | 464 to 807 ms (PNG decoding 145 ms; the first run with cold files) |
  | Mipmaps and detail maps | 662 to 706 ms (16 jobs, the longest 522 to 529 ms) |
  | Parts | 532 to 546 ms (the light carried 454 to 463 ms) |
  | In all | 1,673 to 2,045 ms |

  The next steps: the light carried on the machine's threads, the
  longest texture job (one large texture) split, and the files read and
  parsed in parallel.

Not yet: the install; the load-time work.
