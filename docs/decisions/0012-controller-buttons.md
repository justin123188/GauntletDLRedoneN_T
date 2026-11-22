# 0012: Controller buttons — one layout for every controller, in play

Roadmap item 1 of this fork (docs/remaster/HANDOFF.md §8): the keyboard
keys of player 1 could be rebound (research/enhancements.md "Key
bindings"), the controllers' buttons could not. gdl_play's player_input
pressed fixed joyGetStatus entries for each gamepad button (d-pad 0–3,
South 0xd cross, East 0xb circle, North 0xc triangle, West 0xa square,
LeftTrigger 8 L1, LeftTrigger2 9 L2, RightTrigger 0x12 R1, RightTrigger2
0x13 R2, Start 0x14, Select 0x15).

The owner's choices (2026-11-22): one layout shared by every controller
(over a layout per player), and menus keep the standard buttons (over
menus following the layout).

Decision:

- **Settings::pad** (NATIVE; action name → button names, only the
  actions that differ; empty: the standard layout). Like the keys it is
  a control, not presentation: the original presentation keeps it and
  Settings::baseline() has none. gdl_client::controls::ACTIONS gives each
  action its standard buttons (`pad`, exactly the former fixed mapping);
  Bindings holds the layout beside the keys.
- **Where it applies**: while a level plays (Sim::in_play: no select
  screen, shop, map, movie or scroll message — the screens that read the
  same PlayerControl ports keep the standard buttons, as do the native
  menus and the settings overlay, which read the controllers directly).
  Network play samples the local controllers the same way; the layout
  changes only this machine's input, so no setting has to match.
- **Binding** (F10 → "Controller buttons..."): choose an action, press
  Enter or A, then the controller button. Bindable: the face buttons,
  shoulders and triggers, Select, Start, the stick clicks, the d-pad and
  C/Z; not the Mode (guide) button. A button moved onto an action leaves
  the action that had it, which takes the moved action's previous
  buttons (a swap), so no action loses its only button. Escape, or ten
  seconds without a button, cancels the wait; Backspace gives the action
  its standard button (swapped in); "Standard layout" restores all. The
  sticks are not rebound (the left stick moves, the right stick is the
  free camera's and the game's own entries 0xe–0x11); a button can also
  press a right-stick direction.

Limits: the game's button prompts (gdl_client::glyphs) still show the
standard buttons. Player 2's fixed keyboard keys are unchanged.

Verification (2026-11-22, the owner's PC): unit tests (controls: a swap,
a free button, the settings round trip, unknown and duplicated names,
the standard layout; the overlay page: binding, waiting, cancelling,
Backspace, Standard layout, Back; gdl_play's player_input with a
synthetic gamepad: each of the 14 standard buttons presses the entry the
fixed mapping did, in play and outside it; with attack on RB, RB presses
cross and A strafe in play, the standard entries outside it; a muted
controller presses nothing). The scripted TEST pad bypasses the
controllers, so the simulation hashes check only that nothing else
changed. Not checked: a real controller (the owner's playtest).
