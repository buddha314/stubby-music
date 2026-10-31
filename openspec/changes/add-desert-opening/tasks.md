## 1. Asset pipeline

- [ ] 1.1 Create `assets/`, `assets-src/blender/`, `tools/blender/` (in progress on `add-lightning-bug-pulse`)
- [ ] 1.2 Write `tools/blender/export_gltf.py` (apply transforms, +Y up, metres, .glb) (in progress on `add-lightning-bug-pulse`)
- [x] 1.3 Blockout `square_vco.blend` and `clock_lamp.blend` (emissive lamp); export and check in the game's asset test (`tools/blender/create_vco.py`, `tools/blender/create_clock_lamp.py`, `crates/stubby_game/tests/module_assets.rs`). Placing them in the world is task 2.5
- [x] 1.3a Saw, triangle and sine VCO assets from the same generator, for later discoveries (`assets/modules/README.md`)
- [x] 1.4 Decide on Git LFS for `.blend` files: not yet; see design.md "Asset files and Git LFS"

## 2. World generation

- [ ] 2.1 Seeded per-chunk RNG keyed by (seed, cx, cz); determinism test
- [ ] 2.2 Voronoi crack heightfield + low-amplitude fBm; flatness test for spawn area
- [ ] 2.3 Seamless chunk borders (shared edge vertices and normals); seam test
- [ ] 2.4 Async chunk streaming and despawn around the player
- [ ] 2.5 Placement of Square VCO (40–120 m, outside its 35 m hearing radius) and Clock (farther, ≤300 m); placement tests across 1,000 seeds

## 3. Atmosphere

- [ ] 3.1 Seeded `StormIntensity` gust curve with guaranteed troughs every 20–40 s
- [ ] 3.2 Volumetric fog bound to storm intensity
- [ ] 3.3 Camera-following sand particles (`bevy_hanabi`, CPU fallback if incompatible)
- [ ] 3.4 `Mood` resource, authored curves asset, eased mapping to grading/fog/light/storm

## 4. Player and discovery

- [ ] 4.1 Resolve camera question (first- vs third-person) in design.md
- [ ] 4.2 Player controller and camera
- [ ] 4.3 Square emission: spatialized, audible only within 35 m, follows camera yaw; visual ripple as fallback; discovery adds it to the patch without a gap
- [ ] 4.4 Clock gating (not collectable before the square) and cue toward the square
- [ ] 4.5 Clock light driven by audio tick display-time prediction; verify ≤10 ms offset
  Progress: the clock module's light flashes are scheduled from each tick's predicted instant (`add-module-asset-binding`, PR #17). Open: the ≤10 ms on-screen measurement (`add-module-asset-binding` 2.4).
- [ ] 4.6 Mood increases on each discovery

## 5. Initial visual scene increment

- [x] 5.1 Add standalone Bevy 0.20 game and seeded inspection camera
- [x] 5.2 Generate clipped mud plates, rim relief, fine-crack albedo and normals
- [x] 5.3 Generate desert HDR cubemap and matching lighting, haze and ridgelines
- [x] 5.4 Validate build, determinism tests and runtime preview

## 6. Jackdaw opening integration

- [x] 6.1 Isolate the editor's pinned Bevy development revision from the game workspace
- [x] 6.2 Share procedural scene source; add runtime plugin and windowless Play support
- [x] 6.3 Export reproducible glTF floor preview and load through Jackdaw's GltfSource
- [x] 6.4 Validate project doctor and schema extraction with the matching CLI
- [x] 6.5 Verify shared editor Play after fixing process-local environment-map projection
