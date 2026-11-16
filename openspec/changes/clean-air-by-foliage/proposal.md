## Why

Cleaning the air through foliage is a key mechanic (user, 2026-11-16): the desert opens under dust and smog, and plants the player grows with music clear it. The current plan (`add-desert-opening` 3.5) stores light quality in sparse 10 m cubic cells, eased toward targets set from plant density. That models a grid, not the thing that cleans: the user's direction is that the clean air should be a **radius around each cleaning asset**, such as a tree or plant.

## What Changes

- **Light quality is how clean the air is**, and it comes from foliage: each air-cleaning asset (scrub oaks, reeds, desert brush, future plants) cleans the air within a radius, more as it grows and blooms.
- Light quality at any point is computed from the cleaners around it, overlapping groves adding up without exceeding 1. No grid, no cells, nothing stored per place.
- Each cleaner's radius and strength are **data** in its asset profile; its output eases with its growth, so the air clears gradually as a grove blooms and fouls again when it dies.
- The global lights sample light quality at the camera (`CameraLightQuality`, which already blends the lighting keys); ground tint, flora emission and mine productivity sample it where they are.
- **Supersedes** `add-desert-opening`'s "Light quality field" requirement and task 3.5 (10 m cells); its other light-quality requirements (bleak-to-bloomed look, storm easing) stand and sample this instead.

## Capabilities

### New Capabilities
- `air-cleaning`: foliage cleans the air within a radius; light quality as the result.

### Modified Capabilities
None in accepted specs. In-flight: `add-desert-opening` (atmosphere spec, task 3.5) and the AGENTS.md and style-guide wording on 10 m cells are updated to point here.

## Impact

- Data: an `air` block (`radius_m`, `strength`) in flora asset profiles (`assets/flora/*/asset.json`, `assets/props/desert/asset.json`), art-owned.
- `stubby_core`: an `AirCleaner` contract (radius, current strength) placed assets carry.
- `stubby_world`: the sampler (`light_quality_at(point)`), `CameraLightQuality` fed from it.
- `stubby_life` / glue: cleaners' strength follows growth and bloom.
- `stubby_recipes`: `ProductivityFactor::LightQuality` reads the sample at the mine.
