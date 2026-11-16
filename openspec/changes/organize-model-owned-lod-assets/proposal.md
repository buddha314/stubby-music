## Why

The Godot sibling organizes assets around one master scene and model-owned LODs. Bevy already has Jackdaw's equivalent import/runtime mechanism, but this game's models, profiles and prefabs are spread across separate trees. Make each family discoverable in one place without introducing another LOD system.

## What Changes

- Group runtime models, adjacent Jackdaw metadata, profiles, colliders and variant master prefabs under `assets/<category>/<id>/`; keep variants underneath their family.
- Retain embedded authored LODs and Jackdaw-generated alternatives; preserve import settings and migrate every reference atomically.
- Adapt export/import tooling and add focused path, metadata and runtime coverage.
- Reconcile AGENTS.md, the asset guide and the in-flight `sync-blender-asset-contracts` native-path requirement, which currently mandates `assets/models`. This proposal deliberately replaces that path convention as part of this migration.

## Capabilities

### New Capabilities
- `asset-layout`: asset-family organization and model-owned LOD import contract.

### Modified Capabilities
None in accepted specs. Coordinate overlapping in-flight asset-contract and module-binding changes before implementation.

## Impact

Affects assets, Blender exporters (art ownership), `tools/jackdaw_lods.py`, Jackdaw prefab tests, game asset paths and docs. No new dependency or engine upgrade. Sources and evidence: [research](../../../docs/research/bevy-jackdaw-lod-assets.md). Asset paths change; geometry, import settings, gameplay IDs and authored variants are preserved.
