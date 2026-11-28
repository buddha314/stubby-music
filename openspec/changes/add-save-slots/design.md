## Context

A save must reopen a world the binary builds at launch: the scene and seed come from
`WorldConfig::from_args` before the app exists. Exit to Main Menu already restarts the process
to reset the world.

## Decisions

- **Loaded at launch, not hot-swapped.** `--load` reads the save before the app is built and
  feeds its scene and seed through the same arguments `--scene` and `--seed` use. Continue
  relaunches the process with `--load` (as Exit to Main Menu restarts), so a different world
  never has to be torn down in place.
- **Placed after the world settles.** The player's body spawns on the terrain under its
  camera. Once the scene is up and its colliders exist (0.75 s), the restore moves the body,
  its physics position, velocity and camera to the saved feet and facing, and sets the
  calendar. The loading screen holds until then, so the player never sees the swap.
- **Slots by name.** A slot is a path, else `<slot>.json` in the player's saves folder, else
  in `assets/saves/`. Continue opens the newest by `saved_at`. A shipped slot has `saved_at`
  0, so any save of the player's own comes first.
- **Versioned for growth.** `version` 1 holds only the place and time. Later slices add laid
  streams, racks, inventory and the settlement; loading refuses a newer version.

## Risks / Trade-offs

- Saving the player's feet as the camera's eye height minus 1.65 m is off while crouched;
  acceptable for a first slice.
- A shipped slot depends on its scene staying where it is; the asset layout check resolves the
  scene path in the save file.
