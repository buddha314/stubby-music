## Why

Port the snail shell resource and four-step rack sequencer. The sibling Godot project has diverged; the user requested feature sync on 2026-11-15. See [the full audit](../../../docs/research/wait-for-stubs-feature-sync.md). Priority: P1. This change proposes work; it does not implement it.

## What Changes

- Shell collection and mounting.
- Four sample-timed steps.
- Visible routing completes the rack.

## Capabilities

### New Capabilities
- `sync-snail-shell-sequencer`: the parity contract described below, extending the owning in-flight changes rather than replacing their implementation.

### Modified Capabilities
None in accepted specs; reconcile the related in-flight requirements before implementation.

## Impact

Source: wait-for-stubs `add-snail-shell-sequencer` at `d14edf0066ae75fdea1a430e415ecd3cec462841`. Reuse the existing Rust core, Bevy plugins, Jackdaw prefabs and Blender pipeline. New code is limited to missing domain schema, adapters and behavior. Godot nodes, shaders, addons and path layout are references, not dependencies. Only documentation is changed by this proposal.

## Ownership after the crate split

Where this lands now that the game is independent system crates (`split-game-into-systems`, AGENTS.md):

- The shell's item and resource rows: `assets/recipes/` (a Data PR through `rules-fmt`, unless a new field is needed).
- The sequencer module: a `module.json` beside its panel (`stubby_rack`), and a new `ModuleKind` in `stubby_audio` built from fundsp/Firewheel parts where they fit.
- Optional rack places, buttons and route completion: `bevy_stubby_rack` (rack configuration is a rule type there).
- The shell as a placed pickup: an `AssetBinder` in the game's glue (`stubby_game/src/glue.rs`); seeded placement in `stubby_world`.
