## 1. Asset pipeline

- [ ] 1.1 Create `assets/`, `assets-src/blender/`, `tools/blender/`
- [ ] 1.2 Write `tools/blender/export_gltf.py` (apply transforms, +Y up, metres, .glb)
- [ ] 1.3 Blockout `sine_vco.blend` and `clock_lamp.blend` (emissive lamp); export and load in Bevy
- [ ] 1.4 Decide on Git LFS for `.blend` files

## 2. World generation

- [ ] 2.1 Seeded per-chunk RNG keyed by (seed, cx, cz); determinism test
- [ ] 2.2 Voronoi crack heightfield + low-amplitude fBm; flatness test for spawn area
- [ ] 2.3 Seamless chunk borders (shared edge vertices and normals); seam test
- [ ] 2.4 Async chunk streaming and despawn around the player
- [ ] 2.5 Placement of Sine VCO (40–120 m) and Clock (farther, ≤300 m); placement tests across 1,000 seeds

## 3. Atmosphere

- [ ] 3.1 Seeded `StormIntensity` gust curve with guaranteed troughs every 20–40 s
- [ ] 3.2 Volumetric fog bound to storm intensity
- [ ] 3.3 Camera-following sand particles (`bevy_hanabi`, CPU fallback if incompatible)
- [ ] 3.4 `Mood` resource, authored curves asset, eased mapping to grading/fog/light/storm

## 4. Player and discovery

- [ ] 4.1 Resolve camera question (first- vs third-person) in design.md
- [ ] 4.2 Player controller and camera
- [ ] 4.3 Sine visual cue (sand ripple + emissive bands); discovery adds Sine VCO to patch with 1 s fade-in
- [ ] 4.4 Clock gating (not collectable before sine) and cue toward the sine
- [ ] 4.5 Clock light driven by audio tick display-time prediction; verify ≤10 ms offset
- [ ] 4.6 Mood increases on each discovery
