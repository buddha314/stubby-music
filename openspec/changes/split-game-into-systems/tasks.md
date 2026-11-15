## 1. Unwind the source includes

- [x] 1.1 `stubby_jackdaw` depends on `stubby_game` as an ordinary dependency; remove the `#[path]` `desert` module and repoint `prefab_bridge.rs` and `model_part_bindings.rs`
- [x] 1.2 `stubby_game` depends on `bevy_stubby_recipes` normally; remove `embedded_recipes_plugin` from both crates
- [x] 1.3 With the shared editor idle, rewrite `stubby_jackdaw::desert::` to `stubby_game::` in `assets/prefabs/**/*.bsn` and the two scenes; `cargo test -p stubby_jackdaw --test prefabs` passes

## 2. Shared core

- [x] 2.1 Create `stubby_core`: `ScreenMode`/`Screen`, key bindings, `HudCamera` (was `RackHudCamera`), `InteractTarget`, shared system-set labels (`PrefabBinding`, `MachineInteractionSet`). `GroundHeight` waits for 4.5, the first step where a crate boundary needs it
- [x] 2.2 Repoint every importer. `pulse` ↔ `machine_config` is gone; the other three carry feature types, not contracts, and end with their moves: `rack` ↔ `machine_config` become one crate (4.1), `prefabs` ↔ `resources` (4.2: `PlayerInventory`, `PrefabError`), `prefabs` ↔ `pulse` (4.4: the prefab lifecycle calls `bind_placed_bug`)

## 3. Services

- [x] 3.1 Create `bevy_stubby_audio` from `audio.rs`, `module_ids.rs` and the pulse bus (with `PulseFollower`); rewrite its saved type path

## 4. Systems (one PR each)

- [ ] 4.1 `bevy_stubby_rack` from `rack.rs`, `synth_modules.rs` and the generic rack HUD, over `stubby_rack`; inventory reaches it only through `RackOffers` and `MountRequested` in glue
- [ ] 4.2 Move `ResourceNode`, `MineNode` and harvesting into `bevy_stubby_recipes`; rewrite their saved type paths
- [ ] 4.3 `stubby_player` from the player half of `controls.rs`, `hand.rs` and `interaction.rs`
- [ ] 4.4 `stubby_life` from the lightning bug, its voice and the flora presentation
- [ ] 4.5 `stubby_world` from terrain, sky, ground, world generation, export, the prefab lifecycle and the calendar; the `ResourceTime` conversion moves to glue (schedule with Codex, `add-hybrid-world-placement`)
- [ ] 4.6 `stubby_streams` from `streams.rs`; reed cost through `BuildBudget` and `SegmentLaid` in glue (schedule with the stream owner)

## 5. Glue and enforcement

- [ ] 5.1 `stubby_game` holds only plugin assembly, the translations between systems, the burbler composition, the inventory screen and review drivers
- [ ] 5.2 `tools/check_layers.py` over `cargo metadata`, failing with both crate names; run it in CI
- [ ] 5.3 After each step: `cargo build -p stubby_game -p stubby_jackdaw` and the `prefabs` test; after the last, one bounded smoke test under `tools/jackdaw_timed_test.py`

## 6. Docs

- [ ] 6.1 AGENTS.md: repo map with the layers, the "systems never depend on each other" rule, where cross-system rules go, and the type-path rule replacing "keep the adapter package/type paths"
