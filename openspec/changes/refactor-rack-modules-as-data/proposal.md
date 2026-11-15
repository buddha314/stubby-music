## Why

What a rack module *is*, which engine parts it is built from, where each jack and knob
lands in the engine, and what its knobs start at, is written as six `match` statements
in `crates/stubby_game/src/rack.rs` (`parts`, `knob_default`, `knob_target`, `internal`,
`jack`, `is_output`). A second copy lives in `synth_modules::engine_kind`, and a third,
partial one (ports, directions, knob ranges) in each module's `asset.json`. The copies
already disagree: the volume's gain starts at 0.25 on the rack and 0.7 in the module
showcase, and `volume/asset.json` still describes a built-in envelope that the rack
rule ("a rack plays only what it shows") removed. Adding a module means editing six
functions in one 2,174-line file, which breaks the AGENTS.md promise that new machines
add data, not systems. Now is cheap: there are only three rack modules.

## What Changes

- Each rack module gets one **module definition**, `assets/modules/<id>/module.json`,
  beside its art-generated `asset.json`: its engine parts, internal wiring, each panel
  jack's engine port, each knob's engine parameter, and each knob's default.
- A Bevy-free crate, `stubby_rack`, loads and validates the definitions (strict, every
  error reported with file, module and field, as `stubby_recipes` does), cross-checks
  them against `asset.json`, and plans a rack's engine patch from its mounted modules
  and patched cables. The pure planning that is now inside `rack.rs` moves there.
- `rack.rs` and `synth_modules.rs` read module behaviour only through `stubby_rack`.
  The six `match` functions and `engine_kind` are deleted.
- `stubby_rack` is an **independent system**: it depends only on the audio engine and
  never names items, creatures, machines, the world or Bevy. Starting values that come
  from the game (the burbler's seeded BPM, the lightning bug's pitch) are passed in by
  the game's rack glue, which is where systems are joined.
- **BREAKING (sound):** the module showcase takes its defaults from the definitions, so
  its volume gain becomes 0.25 and the rack and showcase can no longer disagree.
- Fix `volume/asset.json`'s stale built-in envelope note.

## Capabilities

### New Capabilities
- `rack-module-definitions`: rack modules defined as validated data: engine parts,
  wiring, jack and knob bindings, defaults, and the patch planned from them.

### Modified Capabilities
None in `openspec/specs/` (it is empty). The in-flight `add-rack-configurations` and
`add-module-asset-binding` changes are unaffected at the requirement level; this change
replaces how both resolve a module's engine behaviour.

## Impact

- New crate `crates/stubby_rack` (depends on `stubby_audio` and `serde`; no Bevy, ever).
- `crates/stubby_game/src/rack.rs`: planning and the six match functions move out
  (roughly 200 lines); the Bevy systems stay.
- `crates/stubby_game/src/synth_modules.rs`: `engine_kind` replaced.
- `crates/stubby_jackdaw`: compiles `stubby_game` by `#[path]`, so it gains the
  `stubby_rack` dependency.
- New data: `assets/modules/{burbler_clock,square_vco,volume}/module.json`. The sine,
  saw and triangle VCOs and the clock lamp get definitions where the engine has the part.
- Root `Cargo.toml` workspace members, and `Cargo.lock` (shared file: keep the edit small).
- No change to saved prefabs, the rack screen, or what the burbler sounds like.
