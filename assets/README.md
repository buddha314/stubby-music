# Runtime assets

The initial desert scene generates its meshes, mud albedo/normal textures and
half-float HDR cubemap in Rust at startup. No stock reference image is loaded.
Exported models live in `models/<category>/` (`creatures`, `flora/<family>`,
`machines`, `modules`, `props/<family>`, `streams`), with each far LOD
(`<id>_LOD1.glb`) beside its model. Profiles and previews live under the same
category folder at the top level (`flora/scrub_oak/`, `modules/square_vco/`, ...), and
Blender sources under the same folder in `assets-src/blender/`.

Creature profiles and previews live under `creatures/`. The first is [the lightning bug](creatures/lightning_bug/README.md),
which stays invisible between sample-clock-driven white flashes. Desert props (dead brush, stumps, mud debris) live under
`props/` ([README](props/README.md)). Flora lives under
`flora/`: [the scrub oak](flora/README.md), dormant until clean water brings it to
bloom. Editable Blender
sources live in `assets-src/blender/` and builders in `tools/blender/`.
