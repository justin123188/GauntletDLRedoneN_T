# 0008: Free camera — player-controlled turn and zoom

This fork is a remaster: its owner asked (2026-11-22) for a camera the
player controls, because the original gives no zoom and turns only "on
rails" (the level's transmitters, research/camera.md). Unlike the M19
enhancements this is a gameplay option, not presentation: PlayerMotion
moves the players relative to the camera's forward vector (MboxWinPtr
+0x90/+0x98), NewCamLimitPlayerDpos keeps them inside the camera's frustum,
and CalcDist fits several players into it. So the camera state lives in the
simulation (Game::free_cam), driven only by the controller ports' input.

Decision:

- **An offset on top of StdCam, not a new camera.** UpdateCam still runs as
  the original does: target history, transmitter choice, rotation steps,
  distance smoothing. The free camera adds a yaw offset to the record's yaw
  where the view is built (CamUpdate's look direction, CalcFrustrumNormals)
  and a factor on the follow distance (CalcDist) and on the players' limit
  radius (UpdateCam's max radius). The players' limit planes therefore
  follow the view the player sees, and several players are still fitted.
- **Input.** The right stick of every present player whose control style
  leaves it free (not One Handed, mode 2, nor Robotron, mode 3), read from
  the controls module's analog_right after ControlsUpdate: left/right
  turns (about 100°/s at full deflection), up zooms in, down zooms out
  (0.55× to 2.2× the original distance). Keyboard: the right-stick keys
  (numpad 4/6/8/2 by default). Dead zone 0.3 of the magnitude.
- **Rails.** "Turns with level" on (the default): the transmitters still
  turn the camera and the offset turns with them. Off: the rails choose
  no new direction (a new transmitter's rotation asks for the record's own
  yaw and pitch, so LimitYawPitch steps by zero); the level start's
  direction plus the player's offset stays.
- **Scope.** The gameplay camera (StdCam) only. Trigger cameras, the boss
  camera, the attract flybys and the stage-start zoom's own framing are the
  original's (the offset applies to the stage-start zoom too, since it
  draws through CamUpdate). The yaw offset starts at zero each level; the
  front end carries the zoom to the next level.
- **Off by default and off in network play.** Settings::camera
  ("original" / "free", the overlay's Camera row), camera_follow_rails,
  camera_invert. With the camera "original" every computation is the
  original's (CamCtx::free is None and no branch changes arithmetic), so
  reference replays and state hashes are unaffected. Network play forces
  the original camera: the lockstep needs every machine on the same
  setting, and the setting is not exchanged yet.

Arithmetic: the offset and zoom integrate in ordinary f32 (native, not the
guest's rounding subset); the stick decomposition uses the port's own
sinf/cosf. Inputs come from the ports only, so a run is reproducible from
its inputs.

Not done: mouse control, exchanging the setting in network play,
per-level limits for views that show unfinished geometry. (Recentre and a
tilt with the zoom: decisions/0014.) Verification: synthetic unit tests (FreeCam bounds, the overlay
rows); the owner's playtest on Windows (2026-11-22, test build 2): the
right stick turns and zooms the camera in play.
