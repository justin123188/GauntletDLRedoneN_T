# 0030: The new chests in every world (models-plan M5)

The owner's "Yes, every world" (2026-11-25), after G1's chests
(decisions/0028) and treasure heaps with their shine (decisions/0029).

What the worlds' item packages hold (read 2026-11-25):
- **The same chests.** ITEMS_A, B, D, E, F, H, I, J, K and S have the same
  28 chest objects as ITEMS_G (the model export's labels), with the same
  geometry (positions within 1e-4) and the same tree nodes (the project's
  ANIM parser: every tree's base and lid nodes as ITEMS_G's).
- **The same art.** Their textures differ from ITEMS_G's only by
  quantisation: on the treasure heaps a mean difference of at most 2.3 per
  channel out of 255 (H's coins) with equal mean colours, the iron chest's
  inside and B's chest back 0.02 (the same pictures in each package's own
  texture data).
- **Not the same:** the desert world C (CHESTDES*: another chest, CHESTEXP
  and CHESTG* as two-part DESCH objects) and the Tower (no chests). C's
  chests wait for the desert world's own overhaul (the owner, 2026-11-26;
  HANDOFF section 8, World overhauls).
- ITEMS_I has chests and no barrels: a model pack folder of its own.

Decision: each of those ten packages' model_pack folder gets G1's 20
chest files (every kind's base and lid: decisions/0028 and 0029, the brass
treasure lids as CHESTG2LID's copies) and its materials.tsv, as they are,
shared per folder by tools/blender/share_data.py (the loader reads only a
package's own folder, so each keeps its copy: about 16 MB a world). The
files stay out of the repository.

**The load time, and the loader on the machine's threads.** With every
world's chests, a level's model pack took about 2.3 s longer to load (K1's
first update after 4.6 s against 2.3 s without the chests; A1 4.6 against
3.1). GDL_FRAME_LOG's new line per package (PERF model_pack ...) gave K1's
2.8 s: reading and decoding 0.6 s, the sharp filter's mipmap chains of the
1024 px textures 1.24 s (0.02 s with the original filter), the detail maps
0.86 s, one texture after another. model_pack::apply now reads and
decodes the files in order, makes every texture's mipmap chain and detail
map once on the machine's threads (run_parallel: std::thread::scope, the
work pure, the results in the jobs' order), then builds the parts in
order, adding each image where it was first used before. On the owner's
PC (an i7-2600K, 4 cores, 8 threads) K1's mipmaps and detail maps take
0.4 s instead of 2.1, its model pack 1.2 s instead of 2.8: the chests add
about 0.6 s to a level's load. (Reading and decoding stay in order; a
later step.)

Verification (2026-11-25, the owner's PC, scratch copies, isolated
APPDATA):
- **Every world loads its chests.** With the installed build 5f847ef8:
  A1, B1, D1, E1, F1, H1, I1, J1, K1 and S1 each load all 20 chest files,
  with no loader message.
- **The parallel loader** (the build of this change, md5 0615073b, against
  5f847ef8, both with every world's chests): G1 and C1, A1's gem chest and
  K1's coin and junk chests opened, and G1 in the original presentation,
  identical; simulation hashes over 600 updates of the G1 walk equal the
  reference, in the defaults and the baseline; every world's first level
  (and G1) loads with no loader message. The model pack per level, files
  in the disk cache: 1.1 to 1.2 s (A1, G1, K1; I1 1.05 s, chests only);
  read from the disk the first time: 1.9 s (B1, D1, E1, F1, H1, S1; J1
  1.6 s), the reading and decoding then 1.4 s.
- **Tests.** gdl_client 82 + 5.
- **Installed** 2026-11-26 (the owner's "Yes, install"): the build of this
  change and the ten folders' files, the installed model pack byte for
  byte the tested one; the installed game loads I1, G1 and K1 with no
  loader message (the model pack 0.8 s in I1, 1.2 s in G1 and K1).
- **Not yet.** The owner's playtest.
