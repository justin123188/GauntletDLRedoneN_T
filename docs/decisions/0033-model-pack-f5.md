# 0033: F5 reloads the model pack

The owner's "Let's start F5 reload, the pickups change, and the junk props"
(2026-11-26), the step models-plan M5 still listed ("F5 and more of the
owner's props"). F5 in play already reloads the texture pack
(decisions/0009); now it reads the model pack again too, so a new or
changed model shows without restarting the game.

How the game holds replaced models (read 2026-11-26): every package's
objects are built once into a library (gdl_play Libraries; the level's
own objects as LEVEL_LIB), model_pack::apply replacing the parts of the
objects that have a file. Mesh handles live only in the libraries and in
the drawn entities. Items, enemies, effects, held effects, familiars,
powerup trees, hand objects and critters remember which model they show
(a key) and spawn its parts again from the libraries when it changes; the
level's world nodes are spawned once when the level loads.

Decision (NATIVE; gdl_client::model_pack::signature, gdl_play
reload_models; under the model pack folder, which is off in
Settings::baseline() and the Original preset):
- **What is read again.** On F5 (reload_textures, then reload_models), a
  package whose folder changed since it was read gets the model pack
  again: its folder's state (every file below it: its path there, length
  and modification time, hashed) against the state when it was read. The
  library keeps the original parts of the objects it replaced, so the
  objects go back to their originals first and the files are read again
  from scratch (a removed file brings back the original). Unchanged
  folders are not touched; nothing changed: no message.
- **What is shown again.** The level's changed world nodes are spawned
  again in place with their current visibility (spawn_world_parts, the
  function spawn_level now uses, with spawn_level's material variants kept
  as a resource, WorldVariants), and the ground light reads the ground
  again. Everything that picks its model by name forgets its key, so it
  takes the new parts from the libraries on the next frame (their
  animations start again).
- **The message**, with the texture reload's: "Models reloaded: POWERUPS
  10 replaced, 1 back to the original", and how many files did not load.
- **Not covered**: what else holds a package's parts until the next start
  (the players' shadows, the front end's arrow icon and screen saver) and
  the heroes' own models (not in the model pack). The level folder is the
  level's name, as at load.
- GDL_RELOAD_FRAMES=N,... (DIAGNOSTIC) presses F5 at those rendered
  frames, also while the level is frozen (GDL_FREEZE_AT).

Verification (2026-11-26, the owner's PC, the build of this change, md5
a5534567, and the three POWERUPS cases again with the final build of
decisions/0034, bc0cda39: the same; a scratch copy of the game with the
installed model pack, isolated APPDATA, GDL_NO_GAMEPADS; F5 at frame
2500 by GDL_RELOAD_FRAMES, the folder changed after frame 2000, K1 frozen
at update 242 with the hero beside the silver pickup):
- **A replacement added** (TREAS_SILVERL1.glb put back): "POWERUPS 10
  replaced"; the frame after the reload is identical to a run started with
  the file.
- **Removed:** "9 replaced, 1 back to the original"; identical to a run
  started without it.
- **One part to two** (TREAS_GOLDL1.glb under the silver's name): the
  silver pickup draws the gold heap; identical to a run started so.
- **The level's own objects** (K1's platform under the silver, K1PAD10,
  its exported original put in model_pack\K1, drawn lit live instead of
  with its baked light), added and removed: "K1 1 replaced" and "K1 0
  replaced, 1 back to the original"; the platform changes (16,902 pixels)
  and the frame equals a fresh start but for 1,299 pixels by 1 of 255 on
  the silver. A fresh start reads the ground light once, as the level
  loads; F5 reads it again at update 242, when the game has hidden some
  of the level's parts (3,565 ground triangles against 3,569), so the
  silver's factor is 0.9080 against 0.9077. Every drawn mesh, its
  transform, textures and material otherwise the same (GDL_SCENE_DUMP of
  both runs).
- **Tests.** gdl_client 84 + 5 (model_pack: the folder's state follows
  its files).
- **Installed** 2026-11-26 (the owner's "Yes, install"): the build of
  decisions/0034 (bc0cda39); the previous one, b5daf56b, kept beside it as
  gdl_play-lightcolour-backup.exe; the model pack's README.txt replaced
  with the current text. The installed game loads K1, D1 and H3 with no
  loader message.
- **Not yet.** F5 from the keyboard in the owner's game.
