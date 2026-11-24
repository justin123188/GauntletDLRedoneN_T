# 0024: The red and poison barrels (models-plan M5)

The owner's choice (2026-11-24): "Red & poison barrels" next, after the
brown barrel's breaking frames (decisions/0023).

## What the originals are

ITEMS_G's BAREXP (the red, exploding barrel) and BARPOI (the poison
barrel) are the brown barrel's shape painted: BAREXPIDLE18F01 and
BARPOIIDLE21F01 have BAROBJIDLE15F01's positions (checked), with
RED_BARREL_* and GREEN_BARREL_* textures (the red one with a bung hole in
its lid, the green one with lighter streaks). Their breaking flip-books
(BAREXPACTI19F01..F24, BAREXPDONE20F24; BARPOIACTI22F01..F24,
BARPOIDONE23F24) have BAROBJACTI16's positions (decisions/0023). Their
leftovers BAREXP0 (SCORCHED_EARTH) and BARPOI0 (GREEN_BARREL_UNDERSPLAT)
are ground decals (research/rendering.md) and stay the original's.

## Decision

**tools/blender/recolour.py** (hidden Blender): `-- NEW.glb FROM.glb
TO.glb OUT.glb [--report F.json] [--preview DIR]` paints a model pack
file the way one original differs from another, changing only its
textures (the file's other bytes stay; nothing is re-exported). In HSV on
the coloured pixels (saturated and not dark, weighted so edges blend): the
mean hue turned to TO's, the saturation and value scaled by TO's over
FROM's means. For the owner's barrel: red, hue −18.2°, saturation × 1.25,
value × 1.01; poison, hue +83.9°, saturation × 1.21, value × 1.04 (the
originals' mean hues 32.4° brown, 0.3° red, 102.3° green; the owner's
barrel 18.4°).

**break_frames.py --inside-from WOOD.glb**: the pieces' inside wood from
another model's texture. A painted barrel is bare wood inside, so the red
and poison barrels' pieces take the brown barrel's (the same inside
texture as decisions/0023's frames, shared by the loader).

## G1's red and poison barrels (2026-11-24)

recolour.py on the owner's fitted barrel, then break_frames.py with
--inside-from the brown one: 26 files each (the standing barrel, 24
breaking frames, the broken one), the same ten pieces and 12,304
triangles per frame as the brown barrel's. ITEMS_G then has 78 files,
about 190 MB.

## Verification (2026-11-24, the installed build 167bb28a; no code changed)

- **In G1, the red barrel.** GDL_TEST_ACTIONS: the player teleported to
  (−2.8, 25.45, −250.8) at update 200 and DEBUG damage_item 364 at 260
  (research/rendering.md's barrel 364): frozen at 250, the two red
  barrels standing as the new model; at 264 and 270 the fireball and the
  chain to 363; at 280 the pieces flying; at 296 the pieces faint, as the
  original's staves are at that moment with the pack off. The game's fade
  of the red and poison barrels' break trees (ProcessItems +0xda,
  present_tree_alpha's per-entity alpha) reaches model pack parts.
- **The poison barrel.** Its 26 files load (the log names them); its
  break is the same pieces and motion as the red one's, checked in
  Blender previews. Not seen in play: no poison barrel was found in a
  test run.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk with
  all 78 files in the pack equal the reference.
- **Loading.** G1 to its first frames: 6.6 s with the 78 files against
  5.6 s with the pack off (twice each; reading the files).
- **Not yet.** The owner's playtest.
