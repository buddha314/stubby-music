## Why

The user asked for a saved game that opens into the City. Saving and loading is the roadmap's
first foundation (`docs/design/roadmap.md`), and nothing saves yet. The user chose a minimal
first slice: a save file holding the scene, where the player stands and faces, and the time
of day, which opens from the main menu or the command line. It ships with one slot that
opens standing in the City neighbourhood.

## What Changes

- Add `stubby_game::saves`: a versioned JSON save (`version`, `name`, `scene`, `seed`,
  `seeded_layout`, `player` feet position and facing, `calendar_seconds`, `saved_at`) in the
  OS's data folder, plus slots shipped in `assets/saves/`.
- `--load=<slot>` starts the game in a save: its scene and seed, the seeded layout on or off
  as saved, and the loading screen held until the player stands where the save says.
- The main menu's **Continue** (shown when a save exists) relaunches into the newest save.
  The pause menu's **Save Game** writes the slot `quicksave`.
- Ship `assets/saves/city_neighbourhood.json`, which opens on the City neighbourhood's
  promenade at golden hour, and its scene `world/asset_reviews/city_neighbourhood.bsn`
  (cut from the reference district by `create_city_district.py`).
- City collision: walkable City shapes (paving, decks, bridges; stairs as tilted slabs) are
  solid too, since the player's body stands on colliders, and the canal bed has a floor.

## Capabilities

### New Capabilities
- `save-slots`: the save file, loading at launch, Continue and Save Game.

### Modified Capabilities
None.

## Impact

`stubby_game` (new `saves` module; menus and pause menu buttons), `stubby_jackdaw` (`--load`
in the binary, the seeded layout honouring a save, City colliders now include walkable
shapes), `stubby_player` (`movement::body_centre`). Data: `assets/saves/`, a new review scene.
Laid streams, racks, inventory, goals and Dancers are not saved yet.
