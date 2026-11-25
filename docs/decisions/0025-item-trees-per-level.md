# 0025: Item trees looked up per level (a fix in both presentations)

The owner's report (2026-11-23, research/unknowns.md "Item trees after
another level"): G1 reached through the title screen and the Tower showed
its start barrels missing (the gold and silver inside them showing) and its
chests and portal spirals not animating; G1 started directly was fine. The
owner's go-ahead (2026-11-24): "Fix the bug", before more model pack props,
since replaced items are drawn from their level's own packages.

## The cause

present_item_trees (gdl_play, from upstream) found each item name's tree
(the package's animation tree with that label, searched in the level's item
packages first, then POWERUPS, then every other loaded package) once, and
kept the answer in a map that lived for the whole run. Every world has its
own item package with the same names (EXIT_PORTAL, BRIDGEPAD, BAROBJ,
CHEST ...: ITEMS_F, ITEMS_G, ITEMS_L ...), and packages stay loaded, so the
next level posed its items with the first level's trees.

Shown with GDL_DEBUG_ITEM_TREES (new, DEBUG: each name's package, the
first time it is posed in a level), the installed build's code:

| Run | The Tower's EXIT_PORTAL and BRIDGEPAD |
| --- | --- |
| The Tower (L1) started directly | ITEMS_L (its own) |
| F2 → G1 → the Tower | ITEMS_G |
| F1 → the Tower | ITEMS_F |

The owner's path, the Tower then G1, is the same map read the other way
(the harness has no way into G1's portal, so it was not run).

## Decision

The map is cleared when the level changes (Session::current), so each
level looks its names up in its own packages, as the original's levels do
with their own packages. This is a fix of the original presentation as
well as the remaster's: it changes nothing in a level started fresh, and
after a level change it draws the level's own trees.

## Verification (2026-11-24, the build 07628ba0 of this change)

- With the fix: F2 → G1 → the Tower and F1 → the Tower both pose the
  Tower's EXIT_PORTAL and BRIDGEPAD from ITEMS_L, and G1's items from
  ITEMS_G.
- Levels started directly are unchanged: G1 with the model pack (78
  files), G1 with the pack off, C1, and G1 with GDL_BASELINE=1 (frozen at
  update 300, frame 1500, the defaults with interpolation off) are
  identical to the installed build (167bb28a), both from scratch folders
  of one layout.
- Simulation hashes over 600 updates of the G1 walk equal the reference
  with the defaults and with GDL_BASELINE=1.
- cargo test: gdl_client 79 + 5; clippy shows only the older warnings.
- Not yet: the owner's playtest through the Tower.
