## Why

Synchronize night sky motion and environment lighting timing. The sibling Godot project has diverged; the user requested feature sync on 2026-11-15. See [the full audit](../../../docs/research/wait-for-stubs-feature-sync.md). Priority: P1. This change proposes work; it does not implement it.

## What Changes

- One solar-time mapping.
- Night sky travels coherently.
- Environment lighting follows visible time.
- Simulation controls remain separate.

## Capabilities

### New Capabilities
- `sync-calendar-and-night-sky`: the parity contract described below, extending the owning in-flight changes rather than replacing their implementation.

### Modified Capabilities
None in accepted specs; reconcile the related in-flight requirements before implementation.

## Impact

Source: wait-for-stubs `archived add-sky-motion-and-mountains, add-compass-bar, add-time-speed-controls; autoload/calendar.gd` at `d14edf0066ae75fdea1a430e415ecd3cec462841`. Reuse the existing Rust core, Bevy plugins, Jackdaw prefabs and Blender pipeline. New code is limited to missing domain schema, adapters and behavior. Godot nodes, shaders, addons and path layout are references, not dependencies. Only documentation is changed by this proposal.

## Ownership after the crate split

Where this lands now that the game is independent system crates (`split-game-into-systems`, AGENTS.md):

- Solar time, opening time and simulation speed: `stubby_world` (`calendar.rs`).
- Moon, stars, clouds and environment lighting: `stubby_world` (`sky.rs` and the desert setup); lighting keys stay art-authored in `assets/lighting/`.
