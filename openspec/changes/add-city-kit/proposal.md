## Why

Phase 4, the City, is the game's reward: a dense, lush solarpunk city that is lunarpunk at
night, grown from the restored mudflat (`docs/design/roadmap.md`, `add-settlement-phase-reviews`).
The user supplied a production brief (`docs/art/prompts/city-kit.md`) for a reusable modular
kit and a reference district, so that Town and City scenes are assembled from shared pieces
rather than modelled as one-off scenes.

## What Changes

- Add the City art direction and modular contract: `docs/art/city-kit.md`, plus regeneration,
  snapping and the manifest in `docs/art/city-kit/`.
- Add reproducible Blender generators (`tools/blender/city_kit.py`, `city_build_p0.py`, `_p1`,
  `_p2`, `create_city_kit.py`, `check_city_kit.py`, `render_city_previews.py`,
  `create_city_district.py`) and their sources under `assets-src/blender/structures/city/`.
- Export 86 placeable pieces as `assets/structures/city/<id>/` (glb, Jackdaw prefab, profile;
  LOD metadata for plants and the monument), with a family index and collision proxies.
- Add a reference district and test strip (`city_district.blend`: day, dusk and night share
  its geometry; five review cameras) and the same layout as a Jackdaw review scene
  (`assets/world/asset_reviews/city.bsn`).
- Add City colours to the style guide palette.

## Capabilities

### New Capabilities
- `city-kit-assets`: the modular contract, light and material rules, and the asset pipeline
  for the City kit.

### Modified Capabilities
None.

## Impact

Art and asset-pipeline only: no crate changes. The game loads the pieces as plain Jackdaw
models until the crate tasks in `tasks.md` (section 5) bind them: the prefab bridge, a scenery
binder, and model-part bindings for acoustic extras. Walkable decks wait on
`add-prefab-colliders`. Canals need a ground cut or a city podium, which is level work.
