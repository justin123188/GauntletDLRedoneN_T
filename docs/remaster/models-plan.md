# Plan: the model phase (draft for the owner, 2026-11-22)

Roadmap item 2 (HANDOFF §8). The owner asked to plan models together with
water, terrain, vegetation and material types. Nothing here is built yet.
Every feature below is NATIVE: a setting that is off in
`Settings::baseline()`, with the original mode bit-identical.

## What the game has (read from the code and research, 2026-11-22)

- **Scenery is named objects placed by the level.** Each level's OBJECTS
  package holds objects with 16-character labels (the labels the game itself
  looks up). Each object is made of parts: one texture each, plus an
  optional lightmap. The WORLDS file places them (G1: 930 placed objects,
  1,706 parts; research/geometry.md).
- **Collision is separate data** in WORLDS (G1: 3,461 records;
  research/collision.md). Changing what is drawn does not change where
  anyone walks. A new model must still follow the collision shape, or feet
  float or sink.
- **The scenery's light is baked.** Lightmaps (16 shades, they only darken)
  and vertex colours hold all of it (research/rendering.md). Characters and
  items are lit live by one sun direction and an ambient level. There is no
  list of the lights the lightmaps were made from.
- **Water** is ordinary scenery with animated textures: frame swaps (G3's
  WATER, JET, BIGSPLASH) and scrolling. Collision objects with flag 0x200
  set the wading height (C1WATER19; collision.md "Water"). The flag also
  marks some rock piles and flame holders. There is no water shader.
- **Plants** are cut-out textured cards: alpha-tested, with hard edges.
  Some nodes turn to face the camera (the CameraFace flags).
- **Characters** are rigid pieces, one per skeleton node (HEAD, the hands
  and so on). The original clips move each piece (gdl_play spawn_body,
  RigNode). The held weapon (WEAP_HOLD) hangs under the right hand. Enemies
  are built the same way, plus flip-book nodes (OANIM).
- **One material**, the GS-style GsMaterial. It has no normal, roughness or
  metal maps. The enhancements add point lights, torch lights, a sun shadow
  for characters, SSAO and bloom.
- **Tools at hand.** A research glTF exporter (tools/python, Linux,
  diagnostic materials; Blender imported its G1 scene). Bevy's tangent
  generator is already in the build. Bevy's glTF loader is not: adding it
  means new pinned dependencies (the `gltf` crates), explained when added
  (AGENTS.md rule 9).

## How replacement works

1. **Export.** A command (`gdl_play --export-models`, decisions/0021;
   built 2026-11-23) writes the originals to a folder (for the owner's
   copies, `model_work\export\` beside the game):
   - one `.glb` per object;
   - one scene per level, holding every placed object, the collision as a
     separate layer that is never drawn, and the water surfaces.

   New models are made in place against these. Characters get their
   skeleton (node names, rest pose), their pieces and a few clips for
   preview, with M8. Exports are disc-derived, so they are never uploaded.
2. **Model pack.** `model_pack\<source>\<LABEL>.glb` beside the game
   replaces object LABEL of that package, like `texture_pack\` does for
   textures. A missing or broken file falls back to the original model, with
   a message.
3. **Loading.** At level load, and again on F5. The glTF's own textures and
   material settings come with it.
4. **Gameplay never changes.** Collision, items, enemies and timing all
   stay the original data. The simulation hashes are the check.
5. **A checker for walkable ground.** It reports where a replacement floor
   is more than a set distance above or below the collision.

## Lighting new models (a choice for the owner)

- **A. Keep the game's light (recommended).** At load, each new vertex
  takes the original's baked light (lightmap × vertex colour) from the
  nearest original surface. Everything stays in the same mood with no
  baking work. Torches and the sun still add live highlights, through
  normal maps.
- **B. Modern live lighting.** The sun, point lights, ambient and AO light
  everything. This looks more modern, but the mood shifts from the
  original. It needs more renderer work, and there is no light list to
  start from.
- **C. Bake in Blender.** The best quality, but a lot of work per level,
  with the lights guessed.

The owner chose A, with live light on top (see the owner's choices below).

**Items have no baked light** (checked 2026-11-25 in the model export): none
of ITEMS_G's 278 objects carries vertex colours or a lightmap, against 849
of G1's 944 scenery objects with vertex colours. The game lights items with
the level's sun and ambient only (present_scene_light), wherever they stand,
and replaced items are lit the same way (decisions/0022). So option A is for
replaced scenery (M6); an item in a dark corner is as bright as one in the
open, in the original and in the remaster alike.

## Material types

There would be a small library of surface types: stone, wood, metal,
ground, grass, foliage, water, lava, ice, cloth and skin.
- **Assigning types.** Each texture gets a type in `materials.tsv`
  (texture name → type and maps). Types are guessed from the labels (WATER*,
  LAVA*, GRASS*, BARK*) and shown in the texture dump page, where you
  confirm or change them.
- **Extra maps beside a texture** in the pack: `<name>_n.png` (normal) and
  `<name>_r.png` (roughness), with optional `_m` (metal) and `_h` (height).
  These work on today's models, before any new model exists.
- **Type behaviour:**
  - stone, wood and ground: bumps and shine under torches;
  - metal: highlights;
  - ice: gloss;
  - lava: glow and heat shimmer;
  - foliage: sway in the wind and let light through;
  - water: its own shader (below).
- **Free sources.** Poly Haven and ambientCG (CC0, no attribution needed)
  for realistic maps. Normal maps can also be generated from your ChatGPT
  textures.

## Water

- **W1, on today's water.** A water material with:
  - moving ripples, flowing the way the original texture scrolls;
  - a sky-coloured sheen at low angles;
  - glints from the sun and torches;
  - softer edges where it meets the shore (from the depth pass).

  The original animated texture stays visible underneath. Waterfalls and
  rapids get the same flow.
- **W2, replacement water meshes.** Finer meshes with visual waves. The
  wading height stays the original's.

## Terrain

- **T1, on today's ground.** Material types and maps, plus a detail texture
  that tiles fine grain over the ground near the camera. It hides big
  texels at high resolution: the largest gain for the least work.
- **T2, replacement ground for a level.** Exported together with its
  collision. Sculpted in Blender, with the walkable surface kept within a
  tolerance (the checker above). Rocks, ledges and smoother slopes go where
  the original is flat.

## Vegetation

- **V1, today's plants.** Wind sway (by height on the card), softer cut-out
  edges (alpha to coverage) and light shining through.
- **V2, replacement plants and trees.** 3D models keyed by label, like
  props. First targets, the owner (2026-11-22): G1's ground clumps of
  grass and reeds, which the owner finds weak.
- **V3, extra scatter (optional).** Grass and flowers on grass-type ground:
  drawn only, no collision, the same layout every time (seeded per level),
  with a density setting.

## Characters (after the scenery)

- **Rigid pieces.** One replacement per piece, with the same pivot, so the
  original animations just work. The joints show seams, as the original
  does.
- **Skinned.** One continuous mesh with bones named after the nodes; the
  original animation drives the bones. It looks better, but needs weight
  painting (Blender's automatic weights, then cleanup).
- **Either way:** hit sizes, hand and head attachment points and the
  animations stay the original. New animations would be a separate step.

## Tools for the owner

**The owner's Blender experience is basic** (2026-11-23: "blender
experience is limited to make a cube of water smash into a sphere"). So
the owner chooses and scripts do the work: concept art, then an
image-to-3D tool, then our script fits the model to the original (size,
place, facing, polygon count, ground contact) and saves it into
`model_pack\`, then F5 in the game, and the owner says yes or "try
again". Blender by hand only for touch-ups, with click-by-click steps for
that one job.

- **Blender** (free): the current release, pinned once installed. Our
  scripts run it hidden (`--background`); tools/blender/check_glb.py
  imports exports and renders previews (decisions/0021).
- **A small Blender script or add-on from us.** It imports an export,
  checks size, pivot and polygon count, and saves into `model_pack\` under
  the right name.
- **Image-to-3D for rough models from ChatGPT concept art:** Meshy, Tripo,
  Hunyuan3D, TRELLIS. Expect cleanup: fewer polygons, UVs, textures. Check
  each tool's terms before sharing anything made with it.

## Proposed order

1. **M1** Material types on today's textures: the table, the guesses, the
   review page, normal and roughness maps. No modelling needed, and it sets
   the look direction early. Built 2026-11-22 (decisions/0015), with a gem
   type the owner asked for. The owner's playtest: "Looks good". The owner
   (2026-11-22): tune it further once better models and textures exist
   (strengths, types, hand-made maps), not before.
2. **M2** Water shader on today's water. Built 2026-11-22
   (decisions/0016): ripples, a wobbling texture, glints, a sheen and soft
   shores. The owner's playtest: "Looks good".
3. **M3** Foliage on today's plants: sway, edges, light through. Built
   2026-11-22 (decisions/0017): sway and light through leaves. Softer
   edges wait for MSAA, which ambient occlusion turns off. The owner's
   playtest: the moss, bushes and corn sway. G1's ground clumps of grass
   and reeds stay still; the owner wants them replaced in V2 rather than
   tuned.
4. **M4** Exporter: objects, level scenes and the collision layer, checked
   in Blender on Windows. Built 2026-11-23 (decisions/0021): every
   object of every level and shared package, and each level's scene with
   its COLLISION and WATER layers. Not yet: the player characters'
   models and skeletons (M8) and the items' own collision (counted).
   Checked in Blender 5.3.0: the counts agree, the textures load, and
   G1's collision lies on its paths and floors.
   The first test prop: G1's barrel is a flip-book (ITEMS_G
   BAROBJIDLE15F01 standing, BAROBJACTI16F01..F24 breaking,
   BAROBJDONE17F24 broken), so a replacement of the standing frame alone
   shows until the barrel is hit.
5. **M5** Model pack loader for props, with light transfer, F5 and the
   fallback. A handful of props on the first test level. First step
   built 2026-11-23 (decisions/0022): the model pack (by package and
   label, the fallback, live light) and tools/blender/fit_model.py,
   tested with a stand-in barrel in G1. The owner's barrel in the game
   2026-11-24 (the owner's playtest: "Looks good"); its breaking frames
   2026-11-24 (decisions/0023, tools/blender/break_frames.py: the fitted
   model cut into the original's pieces, each moved as the original's
   piece moves). Still to do: F5 and more of the owner's props (G1's
   chests: art brief 2026-11-25, docs/remaster/art-briefs/chest.md); light
   taken over from the baked scenery (option A) comes with replaced
   scenery, M6, since items have no baked light. The red and poison
   barrels 2026-11-24 (decisions/0024: the owner's barrel painted as the
   originals, tools/blender/recolour.py). Every realm's barrels 2026-11-24
   (decisions/0026: ten packages, files sharing their textures and
   geometry through tools/blender/share_data.py).
6. **M6** Replacement ground for one level, with the collision checker.
7. **M7** Replacement plants and trees, and the optional scatter.
8. **M8** Characters: rigid or skinned, your choice then.

**Each step gets:**
- an F10 row;
- a decision record;
- frozen-frame comparisons;
- the 600-update simulation hashes equal to the reference;
- your playtest.

## Owner's choices (2026-11-22)

- **Start with materials (M1).**
- **Extra grass and flowers:** yes, as an option (V3).
- **Art style, in the owner's words:** "an evolution of the game's core
  look", the way its designers might have made it with modern hardware.
  The working rules this gives:
  - Colours and palette come from the original textures, or from the
    owner's repaints in the same palette. Each realm keeps its colour theme.
  - Detail comes from the material maps (bumps, shine, depth), not from
    photo textures. CC0 photo maps may serve as bump and roughness sources
    only.
  - Shapes keep the original silhouettes and proportions, with rounder
    curves, bevelled edges and more polygons where the PS2 had to cut
    corners.
  - Readability from the game camera comes first: items, enemies, exits and
    danger must still stand out from the ground.
- **Lighting: keep the game's light as the base** (option A), with live
  light on top. The owner asked whether this still works once spells and
  effects get more particles and active lighting. It does: the baked light
  is the levels' fixed light, and the effect lights add to it, as the game's
  own spell lights and the torch lights already do (rendering.md, "Point
  lights"; enhancements.md "Lighting"). Normal maps make those live lights
  show bumps and shine. This mix of baked and live light is the usual way
  games combine them. The limits:
  - Baked shadows do not move. Moving shadows from effect lights would need
    shadow-casting point lights, which are costly, so only a few at a time.
  - The live light count (32 today) may need raising for particle-heavy
    effects.
  - Where a replacement model's shape is very different from the original,
    the transferred light is approximate. Such spots can be baked in Blender
    one by one (option C).

## M1 in detail: material types on today's textures

1. **The table.** `texture_pack\materials.tsv`: one line per texture name
   (the pack's 16-hex names) giving a type and options. The game writes the
   first version from guesses: label words (WATER, LAVA, GRASS, BARK, ROCK,
   WOOD, METAL, ICE), with the remaining textures marked "unsure". The
   texture dump page shows each texture's type, so the owner can check and
   correct them.
2. **Maps.** In the pack, beside a texture: `<name>_n.png` (normal),
   `<name>_r.png` (roughness), and optionally `<name>_m.png` (metal) and
   `<name>_h.png` (height). F5 reloads them. A generator makes first
   versions from the texture itself (height from brightness, then normal and
   roughness from that, shaped by the type). It is a starting point that can
   be replaced by hand-made or CC0-derived maps.
3. **Shading.** For parts whose texture has maps:
   - the mesh gets tangents, generated by Bevy's mikktspace;
   - GsMaterial reads the maps;
   - the live lights (spells, torches, the sun) use the bumped normal and
     the roughness for diffuse light and highlights;
   - the baked light gets a gentle bump shading from the sun's direction, so
     bumps show even away from torches.

   Parts without maps draw exactly as today.
4. **The setting.** F10 → Graphics, "Material detail", on by default in
   the Enhanced preset and off in the baseline and the original
   presentation.
5. **Checks:**
   - frozen frames of G1 and the Tower, with the setting on and off (off
     must be pixel-identical to the build before);
   - the simulation hashes;
   - the frame time on the owner's PC;
   - the owner's playtest.

Water (M2) and foliage (M3) build on the same table: their types choose
their shaders.
