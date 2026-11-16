## Why

The opening uses abrupt LOD swaps and one 1%-of-screen cutoff for unrelated assets. The shell's copied threshold is absent from the tuning tool. Begin measured tuning of the existing model-owned LODs in the sunrise opening.

## What Changes

- Add an opt-in bounded LOD review to the Jackdaw game adapter: deterministic near/transition/far/cull shots with level/geometry evidence, without changing normal gameplay or inventing a second LOD selector.
- Make handover, final-cull and crossfade tuning reproducible from an authoring policy, including the placed shell's scale.
- Review the oak, reeds, burbler and shell; keep current authored geometry and generated simplification ratios in this pass.
- Record before/after evidence, including growth and harvested states, scale and FOV checks.

## Capabilities

### New Capabilities
- `model-lod-tuning`: reproducible tuning and bounded review of existing model-owned levels.

### Modified Capabilities
None; retains the model-owned import contract from organize-model-owned-lod-assets.

## Impact

Model .glb.meta settings, tools/jackdaw_lods.py and its authoring policy, the Jackdaw adapter's opt-in review, documentation and focused checks. No engine upgrade or mesh re-export. Final settings remain in each model's metadata; no placement-owned LodGroup.
