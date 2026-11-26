# 0031: The treasure pickups (models-plan M5)

The owner's "Might as well do the pickup heaps for barrels now"
(2026-11-26), in two briefs (the owner: "start with the two barrels for
the first brief and then do the others on a 2nd"):
art-briefs/treasure-barrels.md and art-briefs/treasure-pickups.md.
ChatGPT's art came back the same day (the owner's art folder,
Treasure_Barrels and Treasure_Pickups, review_pack/POWERUPS; outside the
repository).

What came back (ten files for the shared POWERUPS package, one model
pack folder for every world):

| File | Pickup | Triangles | Parts (materials) |
| --- | --- | ---: | --- |
| TREAS_BAR1LOW_T | the low tub of gold (500) | 7,437 | barrel, coins |
| TREAS_BAR2HIGH | the tall barrel of gold (1000) | 7,935 | barrel, coins |
| TREAS_SILVERL1 | silver (100) | 3,310 | coins |
| TREAS_GOLDSILTR | gold and silver (150) | 3,606 | coins, bag |
| TREAS_GOLDL1 | gold (200) | 3,766 | coins, bag |
| TREAS_GOLDGEMTR | gold and finery (250) | 3,800 | coins, gems, metal |
| TREAS_GEMSGEM_P | gems (300) | 4,702 | gems, bag |
| TREAS_JUNKTREAS | the junk pile (10) | 1,766 | junk |
| TREAS_JUNKGR#0 | the junk's sign, pipe and brick | 1,980 | wood, metal, stone |
| TREAS_JUNKGR#1 | the junk's broken wheel | 852 | wood, metal |

The barrels are the owner's barrel (BAROBJIDLE15F01) cut open and filled
with the gold chest's coins; the six heaps use the chest heaps' own
textures (Treasure_Heaps, CHESTG0..G5), byte for byte, and one new
texture for the bags and the junk's props.

Checked here (2026-11-26, a scratch reader of the files and the original
exports):
- **What the loader reads.** Each file one buffer inside it, one or two
  opaque RGB 1024 × 1024 PNGs, triangles only, no extensions, animation
  or texture options; within the briefs' limits (the barrels about 8,000
  triangles, the pickups 4,000, the gems 6,000).
- **The fit.** Each file in its original's frame, within the original's
  bounds to 0.004 (the bags 0.055 to 0.075 below the ground, as the
  originals' 0.06 to 0.09), but for the barrels' spilled coins: 0.12
  (the tub) and 0.09 (the tall barrel) past the original's spill toward
  -x, the spill centred on the barrel where the original's is a little
  off it. The junk's three parts in their own frames (the tree's node
  offsets), each within its original to 0.003.
- **The junk sign has its lettering on one face.** The original sign
  (TREAS_JUNKGR#0) has the red "JUNK" on both faces; the new one on its
  front only, its back plain planks. The levels place 289 of their 341
  treasure pickups (30 of the 40 junk) turned half a turn, so the game
  shows mostly the back: in K1 the original sign's lettering faces the
  camera, the new sign's blank back (renders of both from the front and
  from behind agree).

Decision: model_pack\POWERUPS gets the ten files, shared by
tools/blender/share_data.py (24.2 MB to 13.6 MB: eight textures, ten
geometry files), and a materials.tsv of the hand-offs' 16 lines: coins
and metal `metal bump=0.9 rough=0.22` (as the chest heaps' coins), gems
`gem bump=0.5 rough=0.05` (as CHESTG5's), the bags cloth, the junk's
wood wood, its brick and pile stone; the barrels' wood unlisted (it
draws as the breakable barrels). No code changes (the tests not
rerun). The files stay out of the repository.

**The load time.** The game builds the POWERUPS package twice: at
start-up for the front end's arrow icon and screen saver
(setup_menu_icon), and once a session for the game (the Libraries cache
keeps it for later levels). With the new files each build takes 0.75 to
0.95 s (reading and decoding 0.3 s, the mipmaps and detail maps 0.4 to
0.6 s). In the TEST start (both builds in one start), K1's first update
came about 2.5 s later: 5.4 to 5.6 s from the process start, against 2.9
to 3.0 s without the folder (three rounds, alternating); 1.7 s of it is
the two builds, the rest not found yet. Later levels of a session do
not build it again (the cache, read in the code; not timed). Using the
front end's build for the game would save one, but the front end builds
before the title menu's options can change (material detail, the
texture filter), so that is a change of its own (offered, not made).

Verification (2026-11-26, the owner's PC, scratch copies, isolated
APPDATA, the installed build 0615073b and model pack):
- **Every file loads.** K1, D1, I1, H3 and K2: all ten files, with no
  loader message.
- **In the game** (frozen frames, the hero teleported beside each
  pickup, without and with the folder): silver in K1, the junk in K1,
  gold and silver lying on D1's dock, gold in I1, gems in K2 and the low
  tub in H3 and D3 draw new (D1's pair, mapped: the pixels that differ
  on the two pickups, a short streak on the dock beside them and a faint
  scatter of 1 to 3 levels; the same scene and the hero's health). Not
  in view in any try (20 spots in 13 levels): gold and silver, gold and
  finery (only its heap's edge behind D1's totem) and the tall barrel
  (H3's in deep shade: a tall patch changed by at most 30 of 255).
- **The original presentation** (GDL_BASELINE, which reads no model
  pack): G1 identical with and without the folder, and identical to the
  run of 2026-11-25; simulation hashes over 600 updates of the G1 walk
  equal the reference, in the defaults and the baseline, with the folder.
- **Not yet.** The owner's review (the in-game pictures and ChatGPT's
  comparisons), the install and the playtest (a barrel broken open, the
  pickups in the levels, the load time).
