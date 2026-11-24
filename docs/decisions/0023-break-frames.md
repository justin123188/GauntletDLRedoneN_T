# 0023: The breaking frames of a replaced object (models-plan M5)

The owner's choice (2026-11-24): after the barrel's playtest ("Looks
good"), the next step for the model pack is "Barrel breaking frames"
(over more props). Until now a replaced barrel turned back into the
original's pieces when hit.

## What the original does (its frames, read from the model export)

G1's brown barrel (ITEMS_G) is a flip-book: BAROBJIDLE15F01 standing,
BAROBJACTI16F01 ... F24 breaking, BAROBJDONE17F24 broken (decisions/0021).
Read exactly from the export (tools/blender/break_frames.py, no importer):

- F01 is the standing barrel (48 triangles, a ten-sided barrel).
- F02 is the rest frame: the barrel as a six-sided one in pieces, in
  place (sides at radius 1.06 top and bottom, 1.36 in the middle; the lid
  as two triangles at the top).
- F03 to DONE share one vertex layout: ten pieces, each drawn twice (both
  faces): four full-height side panels, four half panels (two sides split
  at the middle) and two lid triangles. Each moves rigidly: the worst
  distance left by a rigid fit over F03..DONE is 0.012 units (the export's
  rounding).
- DONE is F24's shape.
- BARPOIACTI22 and BAREXPACTI19 (the poison and red barrels) have the
  same positions as BAROBJACTI16 in F03, F12 and F24 (checked): the same
  break with other textures.

## Decision

**tools/blender/break_frames.py** (hidden Blender, as fit_model.py):
`-- ORIGINAL_DIR NEW.glb OUT_DIR FRAME... [--thickness T] [--keep-base 0|1]
[--report F.json] [--preview DIR]`. From the original's frames and the
fitted standing model it writes one model pack file per frame:

- the frames before the rest frame get NEW.glb itself;
- each piece's place at rest is found by stepping its F03 -> F04 motion
  back once from F03 and matching it to F02's corners (for G1's barrel:
  at most 0.0065 units left, the step back within 0.04 for the sides and
  0.5 for the lid halves);
- the new model is cut where the pieces meet at rest (planes through the
  shared edges and the centre: the six corners and the middle), and
  between the two lid halves (two pieces of one material with a long
  border on the new model but no shared edge at rest);
- each face goes to the part of the object it is (the nearest surface of
  F01: side or top, the normals counting) and then to that part's nearest
  piece; every part between the cuts goes whole to one piece, so the
  pieces end at the cuts; each piece keeps its largest connected part and
  stray bits go to their neighbour;
- the base and what only it held are removed (the original's pieces have
  none; the new barrel has a bottom ring and two battens there);
- each piece is thickened inwards (3 % of the height) with plain wood
  inside and on its broken edges: a second material, from the new
  texture's middle tones, darker, with streaks and stave seams;
- in each frame every piece is placed as its original piece is (rest ->
  frame rigid transform, one glTF node per piece), never lower than the
  original's piece or the ground.

The same files always give the same frames (checked: a second run wrote
byte-identical files).

**The model pack shares textures** (gdl_client::model_pack, NATIVE,
decisions/0022): a package's files that embed the same image (the frames
all embed the barrel's texture) decode it once (model_pack::Textures, by a
hash of the image bytes, the colour factor and the alpha class), upload
one image, and per object share one material for parts of one texture.
G1's 26 barrel files load one barrel texture and one inside texture.

## G1's barrel (2026-11-24)

The owner's fitted barrel (decisions/0022) through break_frames.py: ten
pieces, 12,304 triangles per frame in 20 parts (ten nodes, two
materials), 2.5 MB per file, 61 MB for the 25 frames (each embeds the
barrel's 1.9 MB PNG: the loader decodes it once). 8 s in Blender.

## Verification (2026-11-24, the build 167bb28a of this change)

- **In G1.** tools/windows/g1-smash-barrel.json (d-pad entry 2 for 15
  updates, then attacks): the Sorceress's bolt reaches the barrel above
  the start at about update 111. Frozen at updates 113, 116, 119, 122,
  126, 132, 140 and 150: standing, bursting (the staves with their hoops,
  the lid halves lifting), spreading (the dark wood inside showing),
  landed, and lying broken afterwards, at the original's places; the log
  names all 26 replacements. The same moments with the pack off show the
  original's break.
- **Off is unchanged.** With model_pack_folder off, G1 (frozen at update
  300, frame 1500, the defaults with interpolation off) is identical to
  the installed build (4508f35), both run from scratch folders of one
  layout.
- **One-part files are unchanged.** With only the standing barrel in the
  pack (as installed), the same frame is identical to the installed build.
- **The original presentation.** GDL_BASELINE=1 G1 with all 26 files in
  the pack: identical to the installed build.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk
  equal the reference with the defaults (all 26 files in the pack) and
  with GDL_BASELINE=1.
- **Tests and lints.** cargo test: gdl_client 79 + 5 (the minimal .glb
  test now also reads one file twice through one Textures and gets one
  shared texture); clippy shows only the older warnings (audio.rs,
  wgpu-hal).
- **Not yet.** The owner's playtest.

## Not done

- The poison and red barrels (BARPOI, BAREXP): the same script fits them
  once they have standing models. The game fades their break trees out
  (research/rendering.md "Sequence texture modifiers"); whether that fade
  reaches model pack parts is not checked (research/unknowns.md).
- Barrels of other worlds' item packages (ITEMS_F2 and others): each
  package has its own files.
