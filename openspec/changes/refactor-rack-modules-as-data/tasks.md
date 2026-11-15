## 1. Engine interface

- [ ] 1.1 Add `ModuleKind::ports()` (name, direction) and `ModuleKind::params()` per variant in `stubby_audio`, from the documented interface, and a constructor from a kind name plus parameter values
- [ ] 1.2 Offline test: every variant's constructor accepts exactly the parameters `params()` lists

## 2. The `stubby_rack` crate

- [ ] 2.1 Create `crates/stubby_rack` (no Bevy; `stubby_audio`, `serde`, `serde_json`) and add it to the workspace members
- [ ] 2.2 Definition types with `deny_unknown_fields`: parts, internal wiring, jacks, knobs, defaults (number or `{ "source": ... }`)
- [ ] 2.3 `DefaultSource` enum (`machine_clock_bpm`, `lightning_bug_pitch`) and `DefaultContext`
- [ ] 2.4 Loader and validator reporting every error with file, module ID and field: unknown kinds, ports, params and sources; duplicate IDs; defaults out of range
- [ ] 2.5 Cross-check against `asset.json`: every panel port and knob bound, nothing extra, jack direction matches the engine port's
- [ ] 2.6 Move `modules`, `Plan` and `plan` from `rack.rs` into `stubby_rack` as a pure planner over definitions, mounted modules, knob values and patched cables
- [ ] 2.7 Tests: shipped definitions load and cross-check; one report holds two errors; the burbler's partly and fully patched plans

## 3. Data

- [ ] 3.1 Write `module.json` for `burbler_clock`, `square_vco` and `volume`, reproducing today's values exactly (volume gain 0.25, VCO pitch from the bug, clock from the machine)
- [ ] 3.2 Write `module.json` for `sine_vco` and `clock_lamp` (engine parts exist); leave saw and triangle without one
- [ ] 3.3 Fix the stale built-in envelope note in `assets/modules/volume/asset.json` and in `tools/blender/` where it is generated
- [ ] 3.4 Document `module.json` in `assets/modules/README.md`

## 4. Switch the game over

- [ ] 4.1 `rack.rs` reads definitions through `stubby_rack` and supplies `DefaultContext`; delete `parts`, `knob_default`, `knob_target`, `internal`, `jack`, `is_output`
- [ ] 4.2 `synth_modules.rs` builds engine modules from definitions; delete `engine_kind`
- [ ] 4.3 Add `stubby_rack` to `stubby_game` and `stubby_jackdaw` dependencies
- [ ] 4.4 Check: `cargo test -p stubby_rack`, `cargo build -p stubby_game -p stubby_jackdaw`, `cargo test -p stubby_jackdaw --test prefabs`
- [ ] 4.5 Brief smoke test under `tools/jackdaw_timed_test.py`: the burbler patches and sounds as before

## 5. Docs

- [ ] 5.1 AGENTS.md: a rack module is a `module.json` row, not an entry in `rack.rs`; add `stubby_rack` to the repo map
