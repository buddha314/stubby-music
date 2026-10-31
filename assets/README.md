# Runtime assets

The initial desert scene generates its meshes, mud albedo/normal textures and
half-float HDR cubemap in Rust at startup. No stock reference image is loaded.
Future exported props belong in `models/` as `.glb` files.

Creature profiles and previews live under `creatures/`; exported meshes live in
`models/`. The first is [the lightning bug](creatures/lightning_bug/README.md),
which stays invisible between sample-clock-driven white flashes. Editable Blender
sources live in `assets-src/blender/` and builders in `tools/blender/`.
