# Brief: the gems

For an art tool that works in Blender, as for the chest, the treasure and
the junk chest's props. The gems are three kinds of pickup that float in
the levels and are picked up by walking into them:
- **the crystals,** in eight colours;
- **the gargoyle pieces:** a feather, a claw and a pair of fangs, in gold;
- **the runestones,** thirteen carved stones, each with its own glyph.

They are still the original PS2 models, about 40 to 370 triangles each with
32 to 128 px textures. Remake them so they stand out as the prizes they
are, in the style of the new chest and treasure.

**Hand them back in three parts, in this order,** each checked here before
the next:
1. the crystals;
2. the gargoyle pieces;
3. the runestones.

## Where the levels have them

The level data, read 2026-11-27. Every one shows in every game, whatever
the number of players.

| Gem | Placed | In the levels |
| --- | --- | --- |
| Red crystal | 125 | G1 to G4 |
| Blue crystal | 160 | A1 to A4, A6, B1 to B5, G1 to G4 |
| Yellow crystal | 203 | A1 to A4, A6, D1 to D4, G1 to G4, K1 to K4 |
| Green crystal | 180 | A1 to A4, A6, B1 to B5, G2 to G4, K1 to K4 |
| Orange crystal | 15 | L1 |
| White crystal | 225 | B1 to B5, C1 to C4, D1 to D4, K1 to K4 |
| Black crystal | 250 | A1 to A4, A6, C1 to C4, D1 to D4, I1 to I4 |
| Purple crystal | 140 | B1 to B5, G1 to G4 |
| Gargoyle feather (eagle) | 6 | A1 to A4, D1, K4 |
| Gargoyle claw (lion) | 12 | C1 to C3, I1 to I4, J2 to J4, J6 |
| Gargoyle fangs (serpent) | 5 | B3 to B5, G1, G2 |
| Runestones | 13, one each | see part 3 |

G1, the first level, has 40 red crystals, 7 purple, 4 blue, 1 yellow,
the serpent's fangs and runestone C2.

In the art folder (this folder, Gems):
- **`Gems_Crystals_Originals.png`, `Gems_Gargoyle_Originals.png`,
  `Gems_Runestones_Originals.png`:** the originals rendered from the front
  (shadows and glow rings left out).
- **`Gems_InGame.png`:** G1's red crystals in the game.
- **`private_export/objects/`:** the original objects, exported from the
  owner's disc. Keep them local.

## For all three

- **The style:** a modern remaster of an early-2000s dark-fantasy arcade
  game, like the chest and the treasure: rich, chunky and hand-crafted
  rather than photographic. They must read at a glance from the game's
  camera, high above and small on screen, in dark levels.
- **The game moves them.** Each pickup's tree moves it (its ACTIVE loop):
  the crystals turn and bob. So the files have no animation.
- **The gem glow** (this remaster's own effect) lights a soft point light
  half a unit above each gem, in the most saturated colour of the gem's
  textures, and flares when it is picked up. Keep each gem's colour clear and
  saturated, so the glow takes the right colour.
- **Their shadows and glow rings stay as they are:** the `SHADOW` files,
  and the crystals' `CFXP_GLOW` cards (a camera-facing halo).
- **The frame:** each file in the local frame of its original export file
  (game units; x across, y up, z front to back, the front facing +z; the
  export's frame, which negates the game's x). y 0 is the floor under the
  pickup.
- **No bigger than the original's bounds by more than about 10 %:** the
  heroes run into them, and the floating heights must stay clear of the
  floor.

## Part 1: the crystals (POWERUPS)

One package (POWERUPS) serves every level, so one set of crystals serves
the whole game.

**What the original is:** a long, roughly faceted crystal, pointed at both
ends, floating upright with its lower tip about 1 unit above the floor. All
eight colours share one shape, in two parts:
- **The shell, `XSNS`** (material `GEMSHEEN_<colour>`, 38 triangles, a
  32 px texture): the crystal's surface, x -0.44 to 0.48, y 0.97 to 4.26,
  z -0.38 to 0.46.
  - The game draws it solid and unlit, at full brightness and a little
    over (its colour × 1.6), so it glows in the dark levels.
  - **It slides its texture across the shell all the time,** so the light
    seems to move over the facets.
- **The core, `XP`** (material `GEM_<colour>_IN`, 38 triangles, a 32 px
  texture): a slightly smaller crystal inside the shell, x -0.40 to 0.44,
  y 1.10 to 4.13, z -0.36 to 0.42. The game draws it added, as light: dark
  adds nothing, bright glows.

**The new crystals:**
- **One new shape for all eight colours,** a cut crystal with crisp
  facets, pointed at both ends, a little more detailed than the
  original's: a few more facets, a bevel on the edges, perhaps a smaller
  crystal growing from its side. Keep it long and upright, within the
  bounds above (+10 %).
- **The same two parts:** the shell and, inside it, the core, with the
  core a little smaller as now.
- **The shell's texture must tile seamlessly both ways,** since the game
  slides it. Paint the colour's facets, glints and inner veins into it.
  The light is in the texture: the game does not light the shell.
- **The colours:**
  - red: ruby, deep crimson;
  - blue: sapphire;
  - yellow: citrine;
  - green: emerald;
  - orange: amber to fire opal;
  - white: pale, icy, with blue-white glints;
  - black: dark smoky crystal, with blue-grey glints so it still reads in
    the dark;
  - purple: amethyst.
- **At most 600 triangles per part.**
- **At most one 512 × 512 texture per part.**

**The files,** named as the originals (the names are cut at 15
characters), into `review_pack/POWERUPS/`:

| Colour | The shell | The core |
| --- | --- | --- |
| Red | `GEMREDXSNS_CRST.glb` | `GEMREDXP_CRYSTA.glb` |
| Blue | `GEMBLUEXSNS_CRS.glb` | `GEMBLUEXP_CRYST.glb` |
| Yellow | `GEMYELLOWXSNS_C.glb` | `GEMYELLOWXP_CRY.glb` |
| Green | `GEMGREENXSNS_CR.glb` | `GEMGREENXP_CRYS.glb` |
| Orange | `GEMORANGEXSNS_C.glb` | `GEMORANGEXP_CRY.glb` |
| White | `GEMWHITEXSNS_CR.glb` | `GEMWHITEXP_CRYS.glb` |
| Black | `GEMBLACKXSNS_CR.glb` | `GEMBLACKXP_CRYS.glb` |
| Purple | `GEMPURPLEXSNS_C.glb` | `GEMPURPLEXP_CRY.glb` |

The materials: `GEM<colour>_shell` and `GEM<colour>_core`, for example
`GEMRED_shell`.

## Part 2: the gargoyle pieces (POWERUPS)

**What the originals are:** three golden pieces of the gargoyles, each
floating, in one gold texture (`GOLD_REF`, 32 px, with its
reflections painted in):

| Piece | File | Triangles | Bounds |
| --- | --- | --- | --- |
| The eagle's feather | `GARGEAGLGROUP1.glb` | 202 | x -0.36 to 0.36, y -1.71 to 1.71, z -0.10 to 0.10 |
| The lion's claw | `GARGLIONGROUP1.glb` | 334 | x -0.52 to 0.52, y 0.73 to 3.41, z -0.77 to 0.81 |
| The serpent's fangs | `GARGSERPGROUP1.glb` | 313 | x -0.62 to 0.62, y 0.70 to 2.93, z -1.06 to 0.84 |

- The feather's file is centred on its own node, which the tree holds 2.0
  units above the floor (and 0.012 forward), so it floats from about 0.3
  to 3.7.
- The claw and the fangs are at the pickup's place.

**The new pieces:**
- **The same three things,** each recognisable at a glance: a long
  feather with its vane and barbs, a curved lion's claw, a serpent's two
  curved fangs.
- **Cast in polished gold,** as relics: worn hollows darker, raised edges
  bright.
  - Fine engraving on each fits: the feather's barbs, scales near the
    fangs' roots, the claw's sheath.
  - Paint the gold's bright rims and darker hollows into the base colour.
    The game gives metal its shine from a material list beside the files
    (`materials.tsv`).
- **At most 3,000 triangles each.**
- **At most one 1024 × 1024 texture each.**
- **The files:** `GARGEAGLGROUP1.glb`, `GARGLIONGROUP1.glb` and
  `GARGSERPGROUP1.glb`, into `review_pack/POWERUPS/`.
- **The materials:** `GARGEAGL_gold`, `GARGLION_gold`, `GARGSERP_gold`.

## Part 3: the runestones (each world's items)

**What the originals are:** thirteen carved stones, each with its own
glyph.
- The stone floats upright, its centre 2.036 units above the floor, at
  its `RUNE3D` node. The stone is the `RUNEROCKSHADED` material, with its
  own 128 px texture.
- **The glyph is cut through the stone's face.** Behind the cut, inside
  the stone, sits a flat card of one colour, the glyph card (`NSPPLANE`,
  an 8 px texture of one colour). The colour shows through the cut.
- The world gives the colour: A blue, B red, C yellow, D and E green.

**The new runestones:**
- **Each stone with the same glyph,** at the same size and place on its
  face, cut deep with clean edges, so the card behind still shows through
  it.
- **The stone:** weathered and chipped standing stones, with moss or
  lichen in the hollows and worn edges.
  - A little more worked than now: a bevelled rim around the face, and
    the edges of the cut catching the light.
  - Paint the light and dark into the base colour.
- **The glyph cards stay as they are** (the `NSPPLANE` files). Keep the
  stone's cut open over the card, and clear of it: the card sits 0.36
  units in front of the stone's centre, so the stone's face must stay in
  front of that.
- **At most 4,000 triangles each.**
- **At most one 1024 × 1024 texture each.**
- **The material:** `RUNE<id>_stone`, for example `RUNEC2_stone`.

The stones, each in its stone file's own frame, centred on the stone:

| Rune | Package | Placed in | Glyph colour | The stone's file | Its bounds | The card: centre, half size |
| --- | --- | --- | --- | --- | --- | --- |
| A1 | ITEMS_A | A3 | blue | `RUNEA1RUNE3D#0.glb` | x -1.17 to 0.95, y -1.23 to 1.16, z -0.45 to 0.45 | (0.16, 0.01, 0.36), 0.39 × 0.76 |
| A2 | ITEMS_K | K3 | blue | `RUNEA2RUNE3D.glb` | x -1.09 to 1.16, y -1.27 to 1.06, z -0.45 to 0.45 | (0.17, -0.11, 0.36), 0.66 × 0.79 |
| A3 | ITEMS_K | K1 | blue | `RUNEA3RUNE3D#0.glb` | x -1.05 to 1.25, y -1.38 to 1.09, z -0.45 to 0.45 | (-0.11, -0.17, 0.36), 0.63 × 0.77 |
| B1 | ITEMS_B | B1 | red | `RUNEB1RUNE3D.glb` | x -1.34 to 1.06, y -1.34 to 1.08, z -0.45 to 0.46 | (0.17, -0.16, 0.36), 0.62 × 0.81 |
| B2 | ITEMS_I | I2 | red | `RUNEB2RUNE3D#0.glb` | x -1.07 to 1.15, y -1.34 to 1.11, z -0.45 to 0.45 | (0.06, -0.15, 0.36), 0.64 × 0.53 |
| B3 | ITEMS_I | I3 | red | `RUNEB3RUNE3D.glb` | x -0.95 to 1.11, y -1.23 to 1.17, z -0.45 to 0.45 | (-0.17, 0.01, 0.36), 0.66 × 0.85 |
| C1 | ITEMS_C | C1 | yellow | `RUNEC1RUNE3D_D1.glb` | x -1.16 to 1.09, y -1.41 to 1.05, z -0.45 to 0.47 | (0.08, -0.12, 0.36), 0.52 × 0.71 |
| C2 | ITEMS_G | G1 | yellow | `RUNEC2RUNE3D_D2.glb` | x -1.05 to 1.38, y -1.38 to 1.05, z -0.45 to 0.49 | (-0.01, -0.28, 0.36), 0.56 × 0.88 |
| C3 | ITEMS_G | G4 | yellow | `RUNEC3RUNE3D.glb` | x -1.11 to 1.10, y -1.23 to 1.23, z -0.45 to 0.45 | (0.01, -0.17, 0.36), 0.70 × 0.77 |
| D1 | ITEMS_D | D2 | green | `RUNED1RUNE3D.glb` | x -1.16 to 1.16, y -1.20 to 1.20, z -0.45 to 0.45 | (0.02, 0.04, 0.36), 0.52 × 0.64 |
| D2 | ITEMS_J | J6 | green | `RUNED2RUNE3D#0.glb` | x -1.29 to 0.98, y -1.36 to 1.14, z -0.45 to 0.45 | (0.11, -0.10, 0.36), 0.67 × 0.90 |
| D3 | ITEMS_J | J4 | green | `RUNED3RUNE3D.glb` | x -1.07 to 1.27, y -1.42 to 1.06, z -0.45 to 0.45 | (-0.01, -0.21, 0.36), 0.62 × 0.73 |
| E1 | ITEMS_H | H3 | green | `RUNEE1RUNE3D.glb` | x -2.32 to 2.32, y -1.32 to 1.32, z -0.45 to 0.45 | (0.10, 0.03, 0.36), 1.12 × 0.93 |

- **Mind the names.** Some shadows have stone-like names:
  - `RUNEC2RUNE3D.glb` is C2's shadow; its stone is `RUNEC2RUNE3D_D2.glb`.
  - The `#1` files (A1, A3, B2, D2) are shadows; their stones are the
    `#0` files.
- **E1 is a wide, low stone:** about twice as wide as the others.
- **Each stone goes into its package's folder,** for example
  `review_pack/ITEMS_G/RUNEC2RUNE3D_D2.glb`.

## What to hand back, for each part

1. **The files,** under the originals' names, in `review_pack/<package>/`.
2. **Renders** from the originals pictures' cameras: the new ones alone,
   and each beside its original.
3. **A short note** of what was made, and anything that did not fit.

**For the game to read the files** (gdl_client::model_pack):
- one buffer inside each .glb;
- base colour textures as PNGs, opaque (no alpha);
- no mesh compression (Draco, meshopt), no texture transforms, no
  animation.

Normal and roughness maps can stay in the art master; the game uses the
base colour.

We will check each part here, in the game, before the next.
