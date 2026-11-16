## Why

Track shared designs that are not yet implemented upstream. The sibling Godot project has diverged; the user requested feature sync on 2026-11-15. See [the full audit](../../../docs/research/wait-for-stubs-feature-sync.md). Priority: P2. This change proposes work; it does not implement it.

## What Changes

- Design parity is distinct from shipped parity.
- Preserve native architecture when adopting designs.
- Future design work has explicit gates.

## Capabilities

### New Capabilities
- `plan-cross-engine-design-parity`: the parity contract described below, extending the owning in-flight changes rather than replacing their implementation.

### Modified Capabilities
None in accepted specs; reconcile the related in-flight requirements before implementation.

## Impact

Source: wait-for-stubs `add-acoustic-asset-profiles, add-air-quality-field, add-builder-hud, add-module-library, add-tree-inlets-and-pulse, add-weather-and-dust-storm; add-rack-audio-engine` at `d14edf0066ae75fdea1a430e415ecd3cec462841`. Reuse the existing Rust core, Bevy plugins, Jackdaw prefabs and Blender pipeline. New code is limited to missing domain schema, adapters and behavior. Godot nodes, shaders, addons and path layout are references, not dependencies. Only documentation is changed by this proposal.

## Ownership after the crate split

Where this lands now that the game is independent system crates (`split-game-into-systems`, AGENTS.md):

- Each future scoped change names its owning crate under the layering in AGENTS.md (cores, services, systems, game) and puts cross-system rules in the game's glue; `cargo deny check` enforces the layers.
