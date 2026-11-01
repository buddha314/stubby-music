## 1. Asset pipeline

- [ ] 1.1 Create `assets/`, `assets-src/blender/`, `tools/blender/` (in progress on `add-lightning-bug-pulse`)
- [ ] 1.2 Write `tools/blender/export_gltf.py` (apply transforms, +Y up, metres, .glb) (in progress on `add-lightning-bug-pulse`)
- [x] 1.3 Blockout `square_vco.blend` and `clock_lamp.blend` (emissive lamp); export and check in the game's asset test (`tools/blender/create_vco.py`, `tools/blender/create_clock_lamp.py`, `crates/stubby_game/tests/module_assets.rs`). Placing them in the world is task 2.5
- [x] 1.3a Saw, triangle and sine VCO assets from the same generator, for later discoveries (`assets/modules/README.md`)
- [x] 1.4 Decide on Git LFS for `.blend` files: not yet; see design.md "Asset files and Git LFS"
- [ ] 1.5 Standalone game (Codex): levels of detail from a model's `_LOD<n>` nodes (Bevy `VisibilityRange` per level; screen heights from the Jackdaw `.meta` when present); make sure Jackdaw's `.meta` files (naming its `ModelLoader`) do not break the standalone asset loader

## 2. World generation

- [ ] 2.1 Seeded per-chunk RNG keyed by (seed, cx, cz); determinism test
- [ ] 2.2 Voronoi crack heightfield + low-amplitude fBm; flatness test for spawn area
- [ ] 2.3 Seamless chunk borders (shared edge vertices and normals); seam test
- [ ] 2.4 Async chunk streaming and despawn around the player
- [ ] 2.5 Placement of Square VCO (40–120 m, outside its 35 m hearing radius) and Clock (farther, ≤300 m); placement tests across 1,000 seeds
- [ ] 2.6 Scatter desert props (`assets/props/desert/asset.json`: brush, stumps, mud debris) from the seed, sparse and silhouetted in the haze; bloom each from local light quality

## 3. Atmosphere

- [ ] 3.1 Seeded `StormIntensity` gust curve with guaranteed troughs every 20–40 s
- [ ] 3.2 Volumetric fog bound to storm intensity
- [ ] 3.3 Camera-following sand particles (`bevy_hanabi`, CPU fallback if incompatible)
- [ ] 3.4 `TimeOfDay` resource and cycle; load and hot-reload `assets/lighting/lighting_keys.json`; interpolate the time keys (wrapping night → dawn) into sun, ambient, fog, sky, exposure and emissive gain
- [ ] 3.5 Light quality field: sparse 10 m cubic cells, value and target in [0, 1], eased; trilinear sample at a point; tests for locality and easing
- [ ] 3.6 Blend bleak and bloomed keys by the light quality sampled at the camera; storm intensity falls as light quality rises
- [ ] 3.7 Per-cell light quality in materials: ground tint and flora emission (the `accents` in the keys file)

## 4. Player and discovery

- [ ] 4.1 Resolve camera question (first- vs third-person) in design.md
- [ ] 4.2 Player controller and camera: the long-term stack (user, 2026-11-01; see `docs/research/player-controller-and-physics.md`)
- [ ] 4.2a Physics: add `avian3d` 0.8 (Jackdaw's physics engine); the terrain as a heightfield collider from the same seeded generator as its mesh
- [ ] 4.2b Player on `bevy-tnua` 0.33 + `bevy-tnua-avian3d` 0.13: capsule body, walk/sprint/jump/crouch, camera at 1.65 m (1.0 m crouched), our mouse look; replace the hand-written movement in `controls.rs`, keeping its keys, speeds and tests as the acceptance bar
- [ ] 4.2c Input on `bevy_enhanced_input` 0.27: actions for movement and the AGENTS.md control map (E, O, Q, F, Tab, B, WASD, Shift, Space, C), a gameplay context and a rack-screen context (cursor free, movement off)
- [ ] 4.2d Jackdaw physics module: `[modules] physics = true` in `jackdaw/jackdaw.toml` with `jackdaw_avian_integration`; static colliders (`AvianCollider`, `RigidBody::Static`) on the prefabs the player must not walk through: oak trunks, the burbler's banks, stumps, modules. Reeds, brush, debris and bugs get none. No editor change
- [ ] 4.2e Debug camera: Bevy's built-in `free_camera` (`bevy_camera_controller` feature), dev builds only
- [ ] 4.3 Square emission: spatialized, audible only within 35 m, follows camera yaw; visual ripple as fallback; discovery adds it to the patch without a gap
- [ ] 4.4 Clock gating (not collectable before the square) and cue toward the square
- [ ] 4.5 Clock light driven by audio tick display-time prediction; verify ≤10 ms offset
  Progress: the clock module's light flashes are scheduled from each tick's predicted instant (`add-module-asset-binding`, PR #17). Open: the ≤10 ms on-screen measurement (`add-module-asset-binding` 2.4).
- [ ] 4.6 Plant and animal density sets light quality cell targets (first source: scrub oak, `add-scrub-oak-production`); growth itself comes from music played nearby, specified with the flora

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
