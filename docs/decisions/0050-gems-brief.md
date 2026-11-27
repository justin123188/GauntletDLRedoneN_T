# 0050: The gems' brief

decisions/0044, step 3: the gems for the art tool (the owner's "Yes, gems
brief", 2026-11-27). The gems are the pickups of decisions/0018's glow:
the crystals, the gargoyle pieces and the runestones.

Found (2026-11-27, the owner's disc, the model export and the level data):
- **The crystals** (POWERUPS, item kind 1, subtype 0xf):
  - eight colours, 1,298 placed in the levels, all in every game whatever
    the number of players;
  - one shape for all eight colours, in two parts of 38 triangles, each
    with a 32 px texture:
    - the shell, `XSNS` (GEMSHEEN_<colour>, opaque);
    - the core, `XP` (GEM_<colour>_IN, blended), slightly smaller inside it;
  - floating from y 0.97 to 4.26 above the pickup's place;
  - a camera-facing glow ring (CFXP_GLOW) and a shadow.
- **How the game draws a crystal** (a scene dump of G1's red crystal at
  updates 242 and 260, the installed build):
  - The shell is drawn solid and unlit, its colour × 1.6. Its texture
    shift changes over time (0.75, 0.92 at update 242; 0.65, 0.22 at
    260), so the sheen slides over the facets.
  - The core is drawn added.
  - The crystal turns between the two updates, at full scale (the
    definition's 0.1 is not a drawing scale).
- **The gargoyle pieces** (POWERUPS, subtype 0x10): the eagle's feather,
  the lion's claw and the serpent's fangs, in one 32 px gold texture
  (GOLD_REF), 202 to 334 triangles. 23 placed: 6, 12 and 5. The feather's
  node holds it 2.0 above the floor; the others sit at the place.
- **The runestones** (each world's ITEMS, subtype 10): 13 stones, one in
  each of 13 levels.
  - Each is a rock (RUNEROCKSHADED, 174 to 368 triangles, its own 128 px
    texture) at its RUNE3D node, 2.036 above the floor.
  - The glyph is cut through the rock's face. A flat card of one colour
    (NSPPLANE, an 8 px texture) sits behind the cut, 0.36 in front of the
    rock's centre: A blue, B red, C yellow, D and E green.
  - Some shadows have rock-like names: RUNEC2RUNE3D, and the #1 files of
    A1, A3, B2 and D2.
- **Out of the brief:** ITEMS_E2's RUNESTONES* and ITEMS_L's rune frames
  (the runestones' other uses, not pickups).

Decision:
- **The brief:** docs/remaster/art-briefs/gems.md, copied to the art
  folder's Gems/Gems_Brief.md with:
  - the originals' renders (Gems_Crystals_Originals.png,
    Gems_Gargoyle_Originals.png, Gems_Runestones_Originals.png);
  - G1's red crystals in the game (Gems_InGame.png);
  - the 77 original objects (private_export, kept local).
- **Three parts, handed back in order,** each checked in the game before
  the next: the crystals, the gargoyle pieces, the runestones.
- **The parts kept:**
  - the crystals keep the two-part shell and core, since the game's node
    draws each as now;
  - the shell's texture tiles both ways, since the game slides it;
  - the shadows, the glow rings and the runestones' glyph cards stay the
    originals'.
- **The limits:**
  - each within the original's bounds + about 10 %;
  - crystals at most 600 triangles and one 512 px texture per part;
  - gargoyle pieces 3,000 triangles and one 1024 px texture each;
  - runestones 4,000 triangles and one 1024 px texture each;
  - opaque PNGs, material names for materials.tsv.

Later, not in the brief: a see-through crystal shell would need a change
in how the game draws that node.

Not yet: the art.
