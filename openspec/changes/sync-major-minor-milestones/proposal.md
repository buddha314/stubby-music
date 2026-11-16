## Why

Synchronize milestone hierarchy and opening goal data. The sibling Godot project has diverged; the user requested feature sync on 2026-11-15. See [the full audit](../../../docs/research/wait-for-stubs-feature-sync.md). Priority: P1. This change proposes work; it does not implement it.

## What Changes

- Major goals own ordered minor steps.
- Conditions latch once.
- Opening milestone rows and validation.

## Capabilities

### New Capabilities
- `sync-major-minor-milestones`: the parity contract described below, extending the owning in-flight changes rather than replacing their implementation.

### Modified Capabilities
None in accepted specs; reconcile the related in-flight requirements before implementation.

## Impact

Source: wait-for-stubs `add-goals-hub and assets/recipes/goals.json` at `d14edf0066ae75fdea1a430e415ecd3cec462841`. Reuse the existing Rust core, Bevy plugins, Jackdaw prefabs and Blender pipeline. New code is limited to missing domain schema, adapters and behavior. Godot nodes, shaders, addons and path layout are references, not dependencies. Only documentation is changed by this proposal.
