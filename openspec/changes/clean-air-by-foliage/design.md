## Context

`lighting_keys.json` blends bleak (0) to bloomed (1) by light quality, and `stubby_world::lighting::CameraLightQuality` already feeds that blend (#145), fixed at 0 until something sets it. `AssetState` carries per-asset `growth` and `light_quality` (presentation uses the latter for bloom). Nothing computes a light-quality value for a place yet.

## Decisions

**D1. Radius around each cleaner, not a grid.** A cleaner `i` at `p_i` with radius `r_i` and current strength `s_i` (0..1) contributes `c_i(x) = s_i * falloff(|x - p_i| / r_i)`, with `falloff` a smoothstep from 1 at the centre to 0 at the radius. Light quality at `x` is `1 - Π(1 - c_i(x))`: one strong tree clears its own ground; overlapping groves add up and saturate at 1; the bare desert is 0. The value is stateless and exact at any point, so the camera, materials and mines all sample the same function.

**D2. Strength eases with life.** `s_i` follows the cleaner's growth and bloom (a dormant oak cleans little, a blooming one fully), eased over several seconds of simulation time so the air clears and fouls gradually. Easing lives on the cleaner, not on places.

**D3. Data, per asset kind.** `air: { radius_m, strength }` in each flora profile; per-instance variation may scale it as other seeded variation does. Starting values: scrub oak 18 m, reeds 6 m, desert brush 8 m (bloomed only); tuned in review.

**D4. A contract in `stubby_core`.** `AirCleaner { radius_m, strength }` on placed assets, written by the owning system (`stubby_life` or the game's glue for flora binders) and read by `stubby_world`'s sampler. Neither side names the other (split-game-into-systems).

**D5. Cost.** Sampling is a loop over cleaners within reach. Hundreds of cleaners are trivial per frame for one camera; mines sample rarely. If cleaners reach thousands, bucket them in a coarse spatial hash, an index only, not stored values.

**Rejected:** the 10 m cubic cell field (`add-desert-opening` 3.5): cell borders need trilinear smoothing, values need storing and easing per cell, and the grid is not what the player acts on. Volumetric fog volumes per grove: costly, and the global lights need one number anyway.

## Decisions from the user (2026-11-16)

- **Foliage only.** Animals do not clean the air; clean water counts only through what it grows.
- **The windstorm follows the basin**: `stubby_world::air::BasinAir` is the basin's mean light quality, integrated over each cleaner's disc (overlaps shared, so groves are not counted twice) and divided by the basin's area (radius 450 m). Until a windstorm exists, its storm (`1 - clean`) thins the sky's dust (`sky::desert_air`), down to a light haze when the basin is clean.

**D6. The data is a rule, not art.** The radii and strengths are gameplay, iterated on with the rules editor, so they live in `assets/recipes/air.json` (a rule type, matched to placed assets by asset ID prefix), not in the flora `asset.json` files the Blender generators rewrite.

**D7. No self-sustaining bloom.** An asset's own cleaning is left out of the light quality written to it, so desert brush (which blooms with light) cannot keep itself bloomed.
