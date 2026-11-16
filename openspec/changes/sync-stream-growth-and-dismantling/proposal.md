## Why

Synchronize stream dismantling and clarity-limited tree growth. The sibling Godot project has diverged; the user requested feature sync on 2026-11-15. See [the full audit](../../../docs/research/wait-for-stubs-feature-sync.md). Priority: P1. This change proposes work; it does not implement it.

## What Changes

- Dismantle refunds and breaks flow.
- Water clarity caps growth.
- Watering reports real flow.

## Capabilities

### New Capabilities
- `sync-stream-growth-and-dismantling`: the parity contract described below, extending the owning in-flight changes rather than replacing their implementation.

### Modified Capabilities
None in accepted specs; reconcile the related in-flight requirements before implementation.

## Impact

Source: wait-for-stubs `add-stream-building; archived add-scrub-oak; planned add-tree-inlets-and-pulse` at `d14edf0066ae75fdea1a430e415ecd3cec462841`. Reuse the existing Rust core, Bevy plugins, Jackdaw prefabs and Blender pipeline. New code is limited to missing domain schema, adapters and behavior. Godot nodes, shaders, addons and path layout are references, not dependencies. Only documentation is changed by this proposal.

## Ownership after the crate split

Where this lands now that the game is independent system crates (`split-game-into-systems`, AGENTS.md):

- F dismantle, splitting and refunds: `stubby_streams`, refunding through `stubby_core::Stock`.
- Growth levels and their validation: `stubby_recipes` (a rule type).
- Joining stream clarity to tree growth and goal observations: the game's glue; the oak's growth presentation is data the `stubby_world` presenter shows.
