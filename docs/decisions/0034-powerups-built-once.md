# 0034: The pickups package built once

The owner's "Let's start F5 reload, the pickups change, and the junk props"
(2026-11-26), the change decisions/0031 offered: the game built the
shared POWERUPS package twice, once for the front end's arrow icon and
screen saver (setup_menu_icon, at start-up) and once for the game
(setup's libraries), 0.75 to 0.95 s each with the treasure pickups' model
pack.

What stood in the way: the front end builds before the title menu's
options can change. A first version kept the front end's library while
the options that shape a build were the same, the texture filter among
them; it still built twice. The filter is set after setup_menu_icon at
every start (apply_graphics on the first frame), so the key always
differed (logged: sharp off at the front end's build, on at the game's).
The filter needs no rebuild: a switch changes every texture made so far in
place (texfilter::apply, the front end's included), as when the player
switches it in F10.

Decision (NATIVE; gdl_play setup_menu_icon, setup, BuildKey):
setup_menu_icon puts its POWERUPS library into the game's libraries with
the state it was built under: material detail, the texture pack, the
generated HD pack, a texture dump and the model pack. The game's setup
takes it while that state is the same, and builds it again, as before,
when the title menu's options changed it ("POWERUPS built again: ..." in
the log). Not the original presentation's concern: the same library
either way.

Verification (2026-11-26, the owner's PC, scratch copies, isolated
APPDATA, GDL_NO_GAMEPADS; the build of this change, md5 bc0cda39, against the
installed one, b5daf56b):
- **Built once.** K1 and D1 in the TEST start, three rounds alternating:
  POWERUPS built once (0.78 to 1.07 s) against twice; the first update
  after 4.5 to 6.2 s against 5.7 to 8.0 s (medians K1 5.7 s against
  6.1 s, D1 4.8 s against 6.0 s; the first rounds read the files cold).
- **The same pictures.** The six treasure spots of decisions/0032
  identical to the installed build's (K1's junk 16 pixels in a 10 × 4
  patch, as an earlier build of decisions/0032 differed from itself there).
- **The original presentation** (GDL_BASELINE=1): G1 identical to the
  installed build. With Items in shade and material detail off, G1 and
  C1 identical; with the defaults, G1's start identical.
- **Game logic.** Simulation hashes over 600 updates of the G1 walk equal
  the reference, in the defaults and the baseline.
- **Tests.** gdl_client 84 + 5. Clippy: nothing new.
- **Installed** 2026-11-26 (the owner's "Yes, install"): the installed
  game builds POWERUPS once in K1, D1 and H3, with no loader message.
- **Not yet.** A session's first level in the owner's play.
