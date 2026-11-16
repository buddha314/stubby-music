# Bevy and Jackdaw LOD asset organization

Research: 2026-11-16. Subsequently implemented in `organize-model-owned-lod-assets`; see [validation](lod-asset-layout-validation.md).

## Evidence and version scope

Godot reference is `../wait-for-stubs`, commit `d14edf0066ae75fdea1a430e415ecd3cec462841`: `assets/README.md`, `docs/godot-asset-guide.md`, and `tools/import/asset_lod_import.gd`. It co-locates each master scene, profile and models; variants sit beneath their family. Authored `_LOD0`/`_LOD1` nodes become visibility ranges at import, alongside Godot automatic mesh LOD. Transfer the asset ownership pattern, not Godot scripts or import files.

This project pins Jackdaw `82bfd6e18da95667bd7dadbea62e19e2a9330331` and Bevy `c9c397ba1a73e6cfffe957318e0359330b701865` in `compatibility.json`. The local Jackdaw checkout matches that revision. Public `latest` documentation is supplementary; it may describe a newer Bevy.

## What Bevy provides

[VisibilityRange documentation](https://docs.rs/bevy/latest/bevy/camera/visibility/struct.VisibilityRange.html) describes distance-based HLOD with dithered transitions. Matching transition margins join adjacent levels. Ranges must reach each render entity; they do not propagate from a parent. These are visibility controls, not asset streaming or an automatic mesh simplifier.

The [official visibility-range example](https://bevy.org/examples/3d-rendering/visibility-range/) loads alternate glTF scenes and assigns ranges after scene spawning. The pinned Bevy source (`examples/3d/visibility_range.rs`) follows the same approach. Neither the example nor the API prescribes a project folder hierarchy or makes `_LOD` names an automatic glTF import convention. Consequently, the layout recommended here is a project decision, not a Bevy requirement. Do not assume Godot's automatic import decimation transfers to Bevy.

## What the pinned Jackdaw actually requires

Primary source: [model_import.rs](https://github.com/buddha314/jackdaw/blob/82bfd6e18da95667bd7dadbea62e19e2a9330331/crates/jackdaw_scene_types/src/model_import.rs), [runtime lod.rs](https://github.com/buddha314/jackdaw/blob/82bfd6e18da95667bd7dadbea62e19e2a9330331/crates/jackdaw_runtime/src/lod.rs), and [editor model_lod.rs](https://github.com/buddha314/jackdaw/blob/82bfd6e18da95667bd7dadbea62e19e2a9330331/editor/jackdaw_editor_core/src/model_lod.rs).

- A model's adjacent `<model>.glb.meta` selects `ModelLoader` and carries `ModelSettings.lod`. Without that selection, ordinary glTF loading does not establish this model-owned LOD configuration.
- Import supports named `_LOD<n>` node groups and sibling level files. The project will continue choosing embedded node groups; sibling files are supported by Jackdaw but unnecessary here.
- `model.lod.import`, `model.lod.generate`, then `model.lod.apply` expose the import pipeline. Apply persists metadata; generated geometry has a cache that is distinct from authoring inputs.
- `GltfSource` placements derive `ModelLevels`; normal prefabs do not need `LodGroup`. `LodOverride` is an optional placement exception. Runtime switching belongs to Jackdaw.
- Levels use descending screen-height thresholds. Size and perspective field of view determine distances; the last threshold also controls final disappearance. Godot's fixed 45 m handover is not identical for every instance scale and FOV.
- No `models/` or `prefabs/` directory requirement was found in these implementations. Relative sibling references must still resolve after a move. Keep metadata beside its model, preserve scene selectors, and include the runtime loader/plugins and metadata in packaged builds.

## Recommended game folder

Use one family folder, one placeable prefab, and model-owned LODs:

```text
assets/flora/scrub_oak/
  scrub_oak.bsn
  asset.json
  colliders.json
  README.md
  scrub_oak_0/
    scrub_oak_0.glb       # embedded authored LOD nodes
    scrub_oak_0.glb.meta  # Jackdaw settings
  scrub_oak_1/...
assets/machines/mudpit_burbler/
  mudpit_burbler.bsn
  mudpit_burbler.glb
  mudpit_burbler.glb.meta
  asset.json
```

Keep `assets-src/blender/`, `tools/blender/` and `previews/` outside runtime assets. Keep shared rules, audio, lighting, UI and shared textures in their current shared folders. References in profiles and prefabs use paths relative to `assets/`, never Godot `res://` paths. Do not duplicate materials or textures merely to fit the tree.

A layout change alone improves discoverability, not rendering performance. Prefer authored far silhouettes for flora; use Jackdaw simplification for suitable rigid models. Preserve state-part names, animation bindings, colliders and seeded identities across all levels. Simplification should be reviewed for animated/stateful assets rather than assumed safe.

## Gaps to resolve during implementation

`tools/jackdaw_lods.py` currently scans `assets/models`, keys generation and switch-distance tables beneath that root, and writes screen thresholds using a 45-degree reference FOV. Migrate discovery and policy together; do not blindly replace distances with screen fractions or discard existing metadata. Audit path literals in exporters, profiles, prefabs, code, tests, documentation and package tooling. `sync-blender-asset-contracts` currently requires `assets/models` paths and must be reconciled explicitly.

Validate a copied oak family and burbler first, including a clean-cache standalone load with committed metadata, nondefault placement scale/FOV, transitions and final culling. Verify both LODs preserve growth/harvest state and animation. Any graphical review must build first and use only `tools/jackdaw_timed_test.py` with its independent 20-second watchdog. This research did not launch graphics or establish performance gains.
