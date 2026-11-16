## Why

Repair and synchronize Blender asset exports. The sibling Godot project has diverged; the user requested feature sync on 2026-11-15. See [the full audit](../../../docs/research/wait-for-stubs-feature-sync.md). Priority: P0. This change proposes work; it does not implement it.

## What Changes

- State parts retain their child transforms.
- Portable assets keep native paths.

## Capabilities

### New Capabilities
- `sync-blender-asset-contracts`: the parity contract described below, extending the owning in-flight changes rather than replacing their implementation.

### Modified Capabilities
None in accepted specs; reconcile the related in-flight requirements before implementation.

## Impact

Source: wait-for-stubs `archived migrate-off-stubby-music and add-scrub-oak; docs/upstream/zero-scale-exports.md` at `d14edf0066ae75fdea1a430e415ecd3cec462841`. Reuse the existing Rust core, Bevy plugins, Jackdaw prefabs and Blender pipeline. New code is limited to missing domain schema, adapters and behavior. Godot nodes, shaders, addons and path layout are references, not dependencies. Only documentation is changed by this proposal.

## Ownership after the crate split

Where this lands now that the game is independent system crates (`split-game-into-systems`, AGENTS.md):

- Generators and exports: `tools/blender/` (`export_gltf.py`, `module_kit.py`, `collider_shapes.py`), sources in `assets-src/blender/`, models and profiles in `assets/` (art's area).
- Restoring a part's hidden state (`start_scale`) at load: `stubby_world` (`prefab_presentation.rs`, the placed-asset presenter) and the adapter's model bindings (`stubby_jackdaw/src/model_part_bindings.rs`); `stubby_core::AssetPart` carries the extras.
