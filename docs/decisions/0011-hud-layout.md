# 0011: HUD layout — smaller panels, panels at the edges, no empty panels

Roadmap item 1 of this fork (docs/remaster/HANDOFF.md §8, quality of life
for big screens): the original HUD is a fixed 512×384 layout. Its four
player panels are one strip, 128 units per box at box_lx 0/0x80/0x100/0x180,
along the bottom rows (y 0x130 to 0x180); the 3D view is the frame's rows
above InitCamera's 64-row inset, so the strip's lower part is the frame's
black. With the widescreen view the strip stays in the frame's middle with
empty sides (research/enhancements.md "Picture shape and widescreen").

The owner's choice (2026-11-22, after a screenshot of their settings at
1600×900; bigger menu text was offered and not chosen): a smaller HUD,
the panels at the screen's edges and no panels for players not in the
game. All of it is presentation, labelled NATIVE, and off in
Settings::baseline() and after the overlay's Original presentation.

Decision:

- **Tags in gdl_core** (hud::PanelPart, presentation only; the simulation
  never reads them and the state hash does not cover the HUD): a blit or
  font message records the box it belongs to. Panel(p):
  reset_players_sub's blits (backgrounds, runes, keys, abilities, the
  turbo meter, the level, RUNE13 and quest blits), WritePlayerInfo's temp
  blits and text for the box (write_health_and_items, write_gold,
  write_health, show_crystals), the It marker and do_got_it's notices.
  Over(p): a help box NewHelp placed over a box (player given, no world
  point) and its text. Glyph blits take their message's tag.
- **gdl_client::hud_layout** places what is tagged. HUD size (50..100 %)
  scales each panel toward the screen's bottom; without the edges option
  the four stay together in the middle. Panels at the edges: the boxes
  spread over the HUD camera's width (in widescreen, 512 × window width /
  frame width, centred), player 1's at the left edge and player 4's at
  the right, the others evenly between; in a 4:3 view that is the
  original placement. Hide empty panels: no panel for a box whose player
  is not in active_players; it shows again when the player joins. A help
  box keeps its size and follows its box's centre and the panels' top. A
  tagged blit above or below the screen (the level blit at y −0x143, a
  notice sliding out below 0x180) stays off it.
- **The 3D view under the HUD.** When the layout differs from the
  original (any panel smaller, moved or hidden), ps2_viewport makes the
  3D viewport the frame's full height and Ps2Projection continues the
  frustum below (`below`, the rows drawn / the original rows: an
  off-centre frustum with the original angle per row, so the original
  picture keeps its place and the extra rows show what lies below it).
  Under each panel drawn, a black backing covers what the original left
  black (from the original 3D view's last row to the screen's bottom,
  scaled and moved with the panel), so a panel looks as it did; elsewhere
  the game shows. Not while a full-frame 2D screen or a movie is up
  (tally, shop, realm map, select screen, full-screen inventory, front
  end): those keep the original layout. With the original layout (100 %,
  four players, or the options off) nothing changes: the viewport, the
  projection (bit-identical: `below` 1) and the HUD are as before.
- **Settings and the overlay.** Settings::hud_size (default 100),
  hud_edges and hud_hide_empty (default on: the owner's choice). F10 →
  "HUD layout..." (a page of its own): HUD size (50 to 100 % in steps of
  10; Left smaller, Right and Enter larger), Panels at the edges, Hide
  empty panels. Changes show at once.

Limits: the simulation's visibility tests (MBWorldSphereVisible,
ItemVisible, PointVisible) use the original camera, so the rows added
below are off screen to the game, like the widescreen margins: an enemy
there may wait and an item may not animate until it enters the original
view; players are still kept within the original view. Text not tied to
a box (captions, scroll messages, the boss and timer meters) stays where
it was. The panels' own art is not redrawn at the smaller sizes (scaled,
bilinear).

Verification (2026-11-22, the owner's PC, the actual disc's import, a
scratch copy of the build with APPDATA redirected): unit tests (gdl_core
100: the tags reach the draw list, glyphs included; gdl_client 63: the
placements, smaller panels toward the bottom, the edges over a 1920×1080
span, hidden boxes, off-screen blits kept off, the help box, the overlay
page, the defaults, the baseline and the clamp). Frozen G1 frames
(GDL_FREEZE_AT 300, frame 1500, 1600×900 widescreen, the original camera
and no blending so the frame repeats): with the options at the original
layout the build is pixel-identical to the previous one (3538bfd; two runs
of the previous build differed by 11471 pixels of run-to-run variation in
the 3D view, and the new build equals one of them exactly). With the
defaults, player 1's panel is at the window's left edge, the three empty
panels are gone and the 3D view reaches the bottom with the original
picture unmoved; a key in the grass, under the strip before, shows. 70 %
at the edge and 50 % with the four panels together in the middle look as
designed. Per-update simulation hashes over 600 updates of the G1 walk
equal the reference with the defaults (1280×896), with 50 % at 1600×900
widescreen and with GDL_BASELINE=1. Clippy: only the existing audio.rs
warnings. Not checked: a help box over a panel, pickup notices, the It
marker, a second player joining, and the owner's playtest.
