## Context

See [versioned research](../../../docs/research/bevy-jackdaw-lod-assets.md). Bevy supplies VisibilityRange; the pinned Jackdaw supplies model import, generation and runtime switching. The Godot organization is a reference, not a runtime dependency.

## Decisions

1. Co-locate a family's profile, collider data, models and model metadata in `assets/<category>/<id>/`. Put variant models in variant subfolders. Preserve one existing master prefab per authored variant beside its model; the current game does not implement family-prefab variant selection. Do not add that behavior in a path migration or create a prefab per LOD.
2. Preserve embedded `<id>_LOD<n>` nodes. Use Jackdaw import/generate/apply and commit adjacent `.glb.meta`. No replacement game LOD scheduler, Godot import files or placement-owned LodGroup.
3. Keep threshold policy model-owned. Retain existing authored distances as reference-camera tuning inputs, document FOV and size conversion, and distinguish final culling from level handover. Avoid conflicting independent profile and metadata settings.
4. Keep sources, previews and shared services outside family runtime folders as described in the research. Preserve asset-relative paths and any `#Scene0` selectors.
5. Migrate an oak family and burbler as a pilot, then all remaining families in one coordinated reference update. Audit stem-based binders, hidden parts, materials, animations and IDs. Move existing metadata together with models; do not regenerate authored settings indiscriminately.

## Alternatives

Keeping the split trees already works with Jackdaw but leaves the organization requested here unresolved. A central LOD folder splits import ownership. Sibling LOD files are supported by Jackdaw, but embedded levels already match this art pipeline. Godot automatic mesh LOD is not a portable implementation.

## Risks and rollout

Broken paths and missing metadata are the main migration risks. The migration reconciles `sync-blender-asset-contracts` and updates AGENTS.md's target layout/export convention. The implementation claim in AGENTS.md covers path-only exporter edits; no geometry is regenerated. Preserve a path manifest for review and make the asset/reference migration atomic. Do not rename gameplay IDs or change seed draw order. Rollback restores paths and references together.

## Validation

Headless checks resolve every profile/prefab/model reference, parse metadata and verify named levels. Exercise a malformed/missing-level fixture. A clean-cache standalone smoke check verifies the same import/runtime pipeline as editor Play. For this path-only migration, preserve scale/FOV policy and final-culling values byte-for-byte, and review near/far state changes and return. Separate visual FOV/scale/final-culling calibration is required when these settings change. Build first; graphical checks use only the independent 20-second timed harness. Performance changes require measured evidence; folder moves alone make no performance claim.
