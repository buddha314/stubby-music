# Runtime assets

The initial desert scene generates its meshes, mud albedo/normal textures and
half-float HDR cubemap in Rust at startup. No stock reference image is loaded.
Future exported props belong in `models/` as `.glb` files.
