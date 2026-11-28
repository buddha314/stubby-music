## 1. Save slots

- [x] 1.1 (claimed: Claude) `stubby_game::saves`: the versioned save file, slots by name, the saves folder, newest save.
- [x] 1.2 (claimed: Claude) `--load=<slot>` in the game binary: the save's scene, seed and seeded layout, the loading screen held until the player is placed.
- [x] 1.3 (claimed: Claude) Main menu Continue (relaunch into the newest save); pause menu Save Game (`quicksave`).
- [x] 1.4 (claimed: Claude) Tests: a save round-trips and names its world; the shipped City save reads.

## 2. The City save

- [x] 2.1 (claimed: Claude) The neighbourhood scene and `assets/saves/city_neighbourhood.json`, generated with the district.
- [x] 2.2 (claimed: Claude) Walkable City shapes solid (stood on); the canal bed a floor.
- [x] 2.3 (claimed: Claude) Open the save in the game under the 20 s watchdog and look.

## 3. Later slices

- [ ] 3.1 Save laid streams, racks, inventory, goals and world time's checkpoints.
- [ ] 3.2 A load screen listing every save.
