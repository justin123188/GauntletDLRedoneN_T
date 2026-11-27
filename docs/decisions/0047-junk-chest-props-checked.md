# 0047: The junk chest's props, checked

The brief (art-briefs/junk-chest-props.md, 2026-11-26) came back on
2026-11-27. The art folder's Junk_Chest_Props holds:
- review_pack/ITEMS_G: CHESTG0GROUP3 (the wheel), CHESTG0GROUP4 (the brick
  corner) and CHESTG0GROUP5 (the pipe);
- material_additions.tsv: CHESTG0_wood wood, CHESTG0_metal metal bump=0.9
  rough=0.22, CHESTG0_stone stone;
- renders, ChatGPT's fit and motion checks, its scripts, and a hand-off
  note.

The wheel and the brick reuse the junk pickup's geometry and texture
(decisions/0031). The pipe follows the original's three runs, with
collars.

Checked here (2026-11-27, the owner's PC):
- **The files:**
  - 852, 600 and 1,472 triangles; longest edges 0.30 to 0.65; no
    repeated or zero-area triangles.
  - Opaque, single-sided materials named as the brief asked.
  - One embedded 1024 px PNG each (the pickup's texture).
- **Each inside its original's bounds** (the export's frame):
  - the wheel x -0.75 to 0.69, y -0.73 to 0.73, z -0.40 to 0.46
    (the original -0.88 to 0.86, -0.90 to 0.90, -0.53 to 0.55);
  - the brick x 0.05 to 1.20, y 0.65 to 1.61, z -0.61 to 0.48
    (-0.01 to 1.27, 0.62 to 1.65, -0.64 to 0.52);
  - the pipe x -0.25 to 0.16, y 0.14 to 1.68, z -0.92 to 0.08
    (-0.27 to 0.17, -0.08 to 1.72, -0.94 to 0.10).
- **In the game,** at K1's junk chest (game -28.1, -16.4, -83.6), with
  the build of decisions/0045 (c8d7441e) in the scratch copy. The
  installed ITEMS_K is compared with the same folder plus the three
  props and their materials.tsv lines:
  - All three props load, and there are no messages.
  - **Shut** (update 242 beside it): identical, so nothing comes
    through the lid.
  - **Springing out** (a key, onto the chest; update 218 and 226) and
    **open** (242): 738, 6,577 and 4,080 pixels differ, all at the props.
  - The new brick corner and rusty pipe with iron collars replace the
    original's flat grey brick and plain pipe.
  - The art folder's Junk_Chest_Props/Junk_Chest_InGame.png shows them.
- **The hand-off's motion check** (its own figures, not checked here):
  - The props clear the lid and the shell in every frame of the spring.
  - The pipe passes through the brick on frames 0 to 3 and 10 to 14 of
    52; the original props overlap on 17.

## Installed

2026-11-27 (the owner's "Yes, install"), with the build of
decisions/0049 (md5 6e119bd2; the previous exe, d7ae700a, kept as
gdl_play-loadthreads-backup.exe):
- **The same three files in every world's items:** ITEMS_A, B, D, E, F,
  G, H, I, J, K and S.
  - The 11 worlds' junk chests are the same pack files.
  - The original props have the same geometry in every world, in the
    model export. Only the pipe's texture in D and I differs from the
    others, by one texel.
- **Each folder's materials.tsv** gained the three lines, under a
  comment. The 11 tables are the same file. The previous tables are
  kept as model_pack-ITEMS-tables-backup beside the game. To roll back,
  put them back and delete the 33 props.
- **The model pack:** 1,042 .glb, 1,374 files, 335 MB.

Checked in the installed game (an isolated APPDATA):
- **K1's junk chest,** shut, springing out (update 218 and 226) and
  open, against the frames checked above:
  - the same at the chest and the props;
  - 619 pixels differ in each, all in the HUD's "LV 1" label, which
    the installed game draws in the owner's font and the scratch copy
    does not (seen at decisions/0043's install too).
  - The three props load, and there are no messages.
- **D1 and A1** load all three from their own items with no messages.
- **G1** at its start, a1 and a2 is identical to the previous install's
  frames (decisions/0043): the build's changes leave G1's pack as it
  was.

Not yet: the owner's look in the game (a junk chest opened).
