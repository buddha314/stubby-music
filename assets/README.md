# Runtime assets

Each asset lives under `assets/<category>/<id>/`: its profile, collider data,
exported `.glb`, adjacent `.glb.meta` import settings and master `.bsn` prefab.
Families such as `flora/scrub_oak`, `flora/reeds` and `props/desert` keep shared
profiles at the family root and variant models/prefabs in variant subfolders.
Each authored variant has one prefab; no prefab or folder is created per LOD.

LODs are `_LOD<n>` nodes inside the model or Jackdaw-generated levels configured
in its `.glb.meta`. Place the prefab and let Jackdaw select detail automatically.
See [the Bevy asset guide](../docs/bevy-asset-guide.md) for import and validation.

Blender sources stay in `assets-src/blender/`, generators in `tools/blender/`, and
previews in `previews/`. Shared `audio/`, `hud/`, `lighting/`, `recipes/` (the rules),
`shaders/`, `ui/` and `world/` (the seeded layout's rules, `world/README.md`) stay shared;
`rack-recipies/` holds the user's VCV Rack patches that machines copy. The desert
generates its meshes and mud textures at startup, and the sky is Bevy's atmosphere.
