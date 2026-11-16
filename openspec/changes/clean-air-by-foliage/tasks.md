## Implementation (claimed: claude, 2026-11-16)

- [x] 1.1 Cleaning data: `assets/recipes/air.json`, a new rule type in `stubby_recipes` (`air::AirCleanerDefinition`: asset, radius, strength, what it follows, harvest), editable as rules; validated on load. (Not in the flora `asset.json` files, which the Blender generators rewrite.)
- [x] 1.2 `stubby_core::AirCleaner`; the game's glue attaches it from the rules; `stubby_world::air` eases its strength with growth, water/light or harvest.
- [x] 2.1 `stubby_world::air::light_quality_at` (D1); each placed asset's `AssetState.light_quality` (excluding its own cleaning) and `CameraLightQuality` follow it. Tests: locality, saturation, a dying grove fouling the air.
- [ ] 2.2 Mine productivity (`ProductivityFactor::LightQuality`) samples it at the mine. (Waits for scrub-oak production, `add-scrub-oak-production`.)
- [x] 2.3 The basin's clean air (`BasinAir`) drives the windstorm; for now the sky's dust thins with it (user decision 2026-11-16).
- [ ] 3.1 Ground tint and flora emission sample it per place (was `add-desert-opening` 3.7). Flora already follows via `AssetState.light_quality`; the ground does not yet.
- [ ] 4.1 Bounded review: grow a grove, walk from bleak desert into it at noon and at night; no visible steps. (Prefab review: blooming the oak visibly clears the camera's air within ~3 s; the reeds beside it green only ~20% by then, as cleaning eases over 6 s. A longer walk-in review is still to do.)
