## Why

Synchronize pickup outlines and compass guidance. The sibling Godot project has diverged; the user requested feature sync on 2026-11-15. See [the full audit](../../../docs/research/wait-for-stubs-feature-sync.md). Priority: P1. This change proposes work; it does not implement it.

## What Changes

- Aim ranks physics-query candidates.
- One readable pickup outline.
- Compass follows view.

## Capabilities

### New Capabilities
- `sync-targeting-and-compass`: the parity contract described below, extending the owning in-flight changes rather than replacing their implementation.

### Modified Capabilities
None in accepted specs; reconcile the related in-flight requirements before implementation.

## Impact

Source: wait-for-stubs `add-collectible-outline and add-compass-bar` at `d14edf0066ae75fdea1a430e415ecd3cec462841`. Reuse the existing Rust core, Bevy plugins, Jackdaw prefabs and Blender pipeline. New code is limited to missing domain schema, adapters and behavior. Godot nodes, shaders, addons and path layout are references, not dependencies. Only documentation is changed by this proposal.

## Ownership after the crate split

Where this lands now that the game is independent system crates (`split-game-into-systems`, AGENTS.md):

- Target ranking: `stubby_player` (`interaction.rs`), still avian3d queries over `stubby_core::Interactable`.
- The outline: an established Bevy crate (spike first, as tasks say), wired in `stubby_player` or `stubby_ui`.
- The compass: `stubby_ui`, reading the camera heading.
