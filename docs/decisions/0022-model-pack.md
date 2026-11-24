# 0022: The model pack and the fitting script (models-plan M5, first step)

The owner's choices (2026-11-23): the first test prop is G1's standing
barrel ("Standing barrel"); its concept picture comes from the owner with
ChatGPT, and the 3D model from an online image-to-3D tool (Meshy or
Tripo) that the owner signs in to ("You, with ChatGPT", "Online: Meshy
or Tripo"). Considered and set aside for now: tools on the owner's PC.
On the RTX 3060's 12 GB, Hunyuan3D-2 needs 6 GB for shape and 16 GB for
shape and texture (a low-VRAM mode exists); Hunyuan3D-2.1 needs 21 GB for
its texture step; Stable Fast 3D and TripoSR need about 6 GB (their
READMEs, 2026-11-23). The local route stays open for many props later.

## Decision

**The model pack** (gdl_client::model_pack; Settings::model_pack_folder,
on by default, off in the baseline and the original presentation; F10 →
Graphics → "Model pack folder", read at startup).
`model_pack\<package>\<LABEL>.glb` beside the game replaces object LABEL of
that package wherever the game draws it: level scenery, items, weapons,
effects. Names follow the model export exactly
(model_export::object_files: the first label's file stem, "#<index>",
"~<index>" for case clashes; level packages by the level's name). The
replacement happens where each package's object parts are built
(build_library, spawn_level), so everything that draws the object draws
the new one. Collision, items, enemies and timing stay the original data.

- **What a file may hold.** glTF 2.0 binary with its data inside;
  triangle primitives with POSITION and NORMAL, TEXCOORD_0 (float or
  normalised unsigned), any unsigned index size; node TRS or matrices
  (a mirrored node flips the triangle order); a base colour texture as an
  embedded PNG and a base colour factor; alphaMode; doubleSided. JPEG, an
  outside buffer, a sparse accessor or another primitive mode is refused
  with a message, and the original stays. At most 2^20 vertices per file
  and 4096² per texture. No new crates (AGENTS.md rule 9).
- **Space.** The model export's: x negated against the original, y up,
  one game unit per metre. The loader negates x back and keeps the
  triangle order (Bevy then mirrors as for every original part).
- **Drawing.** Each primitive becomes one part. Its material is the
  object's first original material (its draw state and node flags still
  apply) with the file's texture (GS alpha, the texture filter's chain
  and sampler), no lightmap, no LOD K and live light: replacements are lit
  as the game lights items (the level's sun and ambient,
  present_scene_light), so G1's barrel is lit as the original. Scenery
  replacements are lit the same way until the baked light is carried over
  (models-plan option A, a later step). An opaque material's alpha is
  ignored, MASK cuts at 0.5, BLEND blends; doubleSided turns culling off.
- **Messages.** Each replacement, or why a file was refused, goes to the
  log ("model pack: ITEMS_G/BAROBJIDLE15F01: 800 triangles in 1 parts
  from …").

**The fitting script** (tools/blender/fit_model.py, run in hidden Blender:
"the owner chooses, scripts do the work"). From the original's export
file and the new model: every mesh joined, transforms applied, an optional
turn about the up axis, scaled to the original's height (or into its box),
centred over the original's centre and stood on its base, reduced to at
most N triangles (default 3,000), every texture but the base colour
removed, the base colour at most N pixels (default 512) and written as
PNG, exported with only what the loader reads; previews of the original
and the fitted model from one camera, and a JSON report. Blender writes
an imported JPEG as JPEG even after scaling, so each texture is copied
into a new image first.

**G1's barrel.** ITEMS_G's BAROBJIDLE15F01 is the standing barrel; the
24 breaking frames and the broken one (decisions/0021) keep the
original until they get files of their own. A replaced barrel shows until
it is hit.

## Verification (2026-11-23, the build of this change)

- **A stand-in barrel, end to end.** A Blender-made barrel (62,396
  triangles, 0.4 units tall, off-centre, a 512² JPEG texture) through
  fit_model.py: scale 6.82, 800 triangles, a 256² PNG, 2.727 units tall
  as the original (2.727), standing on its base; the previews show both
  at one size and place. In G1 (frozen at update 300, frame 1500, the
  defaults with interpolation off) both start barrels draw the stand-in,
  at the original's places and tilt, lit by the level; the log names the
  replacement.
- **Off is unchanged.** With model_pack_folder off, the same frame is
  identical to the exporter's build (decisions/0021, itself identical to
  107c6e8 from a scratch folder).
- **The original presentation.** GDL_BASELINE=1 G1 with the pack folder
  present: identical to 107c6e8, and no replacement is made.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference with the defaults (the stand-in barrel in the pack) and
  with GDL_BASELINE=1.
- **Tests and lints.** cargo test: gdl_client 79 + 5 (new: a minimal .glb
  read back in the original's space with GS alpha, a JPEG refused; the
  overlay row; the baseline); clippy shows only the older warnings
  (audio.rs, wgpu-hal).
- **The owner's playtest:** with the owner's own barrel (below).

## The owner's barrel (2026-11-23)

- **Getting a textured model.** Tripo made a good one from the owner's
  ChatGPT picture but asks for payment to export ("tripo wants money to
  export"); Meshy's own pages say its free tier downloads untextured
  models; Tencent's Hunyuan3D-2 demo on Hugging Face showed "Texture
  Generation (Unavailable)". The owner then used a free Hugging Face demo
  (setting out to use TRELLIS) and downloaded a zip of three files:
  base.glb (shape only), base_basic_shaded.glb (light painted into an
  emissive texture: not used, the game lights the barrel itself) and
  base_basic_pbr.glb (base colour, normal and metallic-roughness PNGs;
  1,000,000 triangles, 618,438 vertices; written by Blender's glTF
  exporter 4.3.47).
- **The first fit tore it.** Reducing the model itself (its UV atlas has
  thousands of islands, split at every seam) stopped at 13,009 triangles
  with holes, splinters and white atlas gaps. fit_model.py now makes a
  copy welded at its seams, reduces that to the triangle budget, gives it
  smart-projected UVs and bakes the full model's base colour onto it
  (Cycles DIFFUSE colour, 4 samples, the GPU through OptiX): 4,000
  triangles and a 1024² texture, clean, in 38 s on the owner's RTX 3060.
- **In G1** (the frame above): both start barrels are the new model, at
  the original's places and tilt. It is warmer and brighter than the
  original: the model's own reddish-orange wood (as in the owner's Tripo
  version).
- **Its breaking frames** followed (decisions/0023), with textures shared
  across a package's files.
- **Installed and playtested (2026-11-24).** The build of this change
  and the fitted barrel (model_pack\ITEMS_G\BAROBJIDLE15F01.glb) in the
  owner's game, the previous exe kept as a backup. The owner kept the
  colours as they are; the owner's playtest in G1: "Looks good".
