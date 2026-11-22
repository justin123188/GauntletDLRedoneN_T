# 0014: Free camera — recentre on the stick click, tilt with the zoom

Roadmap item 1 of this fork (docs/remaster/HANDOFF.md §8): the free camera
(decisions/0008) turns and zooms; it had no recentre and no tilt. The
owner's choices (2026-11-22, playing with an Xbox One controller): the
right stick click recentres — a quick press the direction only, a long
press everything — and the tilt follows the zoom (one control, no extra
buttons).

Decision:

- **Recentre button.** A NATIVE input beside the PS2 entries:
  PadStatus::camera_recentre (not a joyGetStatus entry; network frames
  carry it in the flags byte, bit 3), copied per port by ControlsUpdate
  (Controls::camera_recentre) and read only by the free camera's input
  (free_cam_input: any present player whose control style leaves the right
  stick free, as the turn and zoom). In the client it is the action
  "camera_recentre" (controls::RECENTRE, F10's key and controller pages):
  numpad 5 and the right stick click (RightThumb) by default, rebindable
  like the others. The TEST input script presses it with entry 0x16.
- **The swing.** A press starts a swing back to the rails' direction (the
  yaw offset to 0, 0.35 radians per update: half a turn in about 0.3 s);
  held 18 updates (0.6 s), the zoom returns to the original distance too
  (by 12 % per update) and with it the tilt. The stick takes over: any
  turn or zoom input ends a swing. The state (held updates, swing) is the
  FreeCam's, in the simulation, from the ports only.
- **Tilt with the zoom.** FreeView::pitch = 0.3 × ln(zoom) radians
  (FreeCam::tilt): at the closest zoom (0.55) the view looks about 10°
  less steeply down, towards the players' backs; at the farthest (2.2)
  about 14° more steeply; none at the original distance. CamUpdate's look
  direction takes the record's pitch minus the tilt (pitches below 0 look
  down), kept between −0.25 (14° down) and −1.4 (80° down) unless the
  rails' own pitch is already beyond. The eye follows the look direction
  at the follow distance, and the players' limit planes (CalcDist's
  frustum normals from the look) follow the tilted view.
- **Unchanged.** With the camera "original" (and in network play) nothing
  of this runs. At the original zoom the tilt is 0 and the pitch path is
  the previous one exactly.

Verification (2026-11-22, the owner's PC, the actual disc's import, a
scratch copy, isolated APPDATA): unit tests (the tilt's sign and range,
a quick press swinging the direction back with the zoom kept, a long
press returning the zoom, the stick ending a swing, the pitch limits; the
client: the right stick click sets camera_recentre and no PS2 entry).
Frozen G1 frames at 1600×900 with scripted right-stick and recentre
input: zoomed in, the view is lower behind the sorceress; zoomed out,
steeper; turned and then recentred by a quick press, and turned and
zoomed out and then held, the level's geometry is where it is without
any input (the enemies, which kept moving, differ). Simulation hashes
over 600 updates of the G1 walk equal the reference with the free camera
and with GDL_BASELINE=1. The owner's playtest (2026-11-22, the build of
5b4d6b9, Xbox One controller): "that's pretty good!".
