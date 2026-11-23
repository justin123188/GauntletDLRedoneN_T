# 0020: SMAA before sharpening — frames the same from run to run

The owner's go-ahead (2026-11-23): "Yes, look into it", after the potion
glow's checks found a frozen frame that came out in one of two fixed
states from run to run (research/unknowns.md "Frozen frames in two
states", decisions/0019).

What was found (2026-11-23, G1 frozen at update 300, frame 1500, the
fork's four new effects off, the rest of the Enhanced preset on:
scratch settings o-alloff3):
- **The two states.** A debug build (34bb9bf with a light-list print)
  gave state A (md5 91E74BB6, the 5b4d6b9 frame) in four runs of five and
  state B (BD7CF796) in one: 14174 pixels differ, 8122 of them by one
  level, on polygon edges and fine texture detail (characters, barrels,
  plants); nothing moved. The point-light list was empty in every run
  (no light reaches that view), so the lights were not the cause.
- **Bisected by settings.** Ambient occlusion off (MSAA 4 instead of
  SMAA): six runs, one frame. Sharpening off (SMAA, no CAS): six runs,
  one frame. Only with both on does the frame vary.
- **The cause (Bevy 0.18.1, read).** SMAA's node runs after tonemapping
  and before EndMainPassPostProcessing; so does contrast-adaptive
  sharpening (CAS), which is also ordered after FXAA's. No edge orders
  SMAA and CAS. Both write the next post-process texture, so the frame
  depends on which runs first. The graph runner queues unordered nodes in
  the order of RenderGraph's node map, a HashMap keyed by interned labels,
  and those hash by type id and pointer address (bevy_ecs label.rs
  ref_hash). The addresses change from run to run, and so does the order.

Decision (gdl_play PostProcessOrder; NATIVE, a fix to two NATIVE
ENHANCEMENTS, so no setting): a render graph edge Node3d::Smaa →
Node3d::ContrastAdaptiveSharpening. Sharpening comes after anti-aliasing,
as Bevy already orders it after FXAA. Neither SMAA nor sharpening is
active in the original presentation, so it is unchanged. Bevy's later
versions replace the render graph with systems (bevy PR 22144); this edge
is for 0.18.1.

Verification (2026-11-23, the build of this change, md5 2a7d98ae):
- **The fixed order.** Eight runs: every one state A, identical to the
  5b4d6b9 frame.
- **The reverse order.** A test build with CAS → SMAA instead: three runs,
  every one state B, identical to the debug build's variant. The order of
  the two passes is the whole difference.
- **The full defaults.** With interpolation off (s-defnointerp: the four
  new effects and the free camera on): four runs, one frame, and frames
  1500 and 2500 of each run identical.
- **With interpolation on** (the defaults, no settings file): frames 1500
  and 2500 of one run differ in 536126 pixels (451217 by one level). A
  frozen frame still moves while interpolation is on; see
  research/unknowns.md "Interpolation while the simulation waits". Not
  caused by this change.
- **C1's start** (the four new effects off, where 5b4d6b9 gave two states
  in two runs): two runs, one frame, identical to one of 5b4d6b9's.
- **The original presentation.** The GDL_BASELINE=1 G1 frame is identical
  to 5b4d6b9's.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, with the defaults and with GDL_BASELINE=1.
- **Tests and lints.** cargo test: 76 + 5 pass; clippy shows only the
  older warnings (audio.rs, wgpu-hal).
