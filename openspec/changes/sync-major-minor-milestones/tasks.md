## Implementation (claimed: claude, 2026-11-17)

- [x] 1.1 Extend existing add-inventory-and-goals-hud via stubby_recipes schema and Bevy plugin, with backward-format/migration handling and stable completion state. (`goals.json` has its own format, 2 (`GOALS_FORMAT`); a format-1 file fails with a format error. Goal IDs are unchanged, so no migration rows; no saves exist.)
- [x] 2.1 Port format-2 goal semantics and opening rows; regenerate schemas and ids.lock using rules-fmt, never hand edit generated files. (Godot `d14edf0` rows verbatim: `build_a_camp` with five steps.)
- [x] 3.1 Connect held-item and growth observations to the existing message boundary; display active major, step and progress without ending play. (Done: `item_held` via `Refresh` on inventory change, `machine_configured` from the rack, the hub's Goal/Step bands and 4 s `Complete:` notice. `tree_watered`/`tree_grown` come from `stubby_game::watering`, `sync-stream-growth-and-dismantling` 3.1.)
- [x] 4.1 Test validation, ordered progression, prerequisite cycles, existing held items and exactly-once completion; run cargo test -p stubby_recipes. (Fixture goals only; the shipped list is only checked to load.)
