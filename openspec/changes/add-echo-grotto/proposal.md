## Why

Spatial audio now runs on AudioNimbus / Steam Audio (`add-rack-and-spatial-audio` 4.x):
distance, air absorption and geometry occlusion work, and reflections/reverb (4.8) are
being integrated in a separate work stream (Codex). That stream needs an **authored**
enclosed space to listen in: `docs/audio/pre-level-validation.md` keeps synthetic boxes
for engine checks but says listening reviews, cave boundaries and reflection tuning
need real level geometry, and task 4.6 still lacks "authored walls/caves ... and an
in-game occlusion listening check".

The user asked (2026-11-17) for that space in the opening scene: a **rock grotto** in
the distance, three to five large boulders making a shelter the player can stand in,
with a **mudpit burbler** inside so its voice echoes off the rock.

## What Changes

- **Boulders, made with Blender's Rock Generator.** A new prop family
  `assets/props/rocks/boulder_<k>/` (four or five variants), generated headlessly by
  `tools/blender/create_boulders.py` with the Rock Generator add-on (Extra Mesh Objects
  extension, `mesh.add_mesh_rock`) at fixed seeds, its displacement baked into the
  mesh, a plain PBR rock material, model-owned LODs, colliders and a Jackdaw prefab
  each. Generated geometry, so the shipped assets are CC0.
- **Acoustic rock by data.** Each boulder mesh node carries glTF extras
  `"acoustic": "rock"`; one asset-agnostic system marks such meshes with
  `AcousticOccluder::ROCK`, so the grotto's walls and roof enter the AudioNimbus scene
  without per-asset code.
- **The grotto in the opening scene.** After a Mojave boulder cluster (the user's
  reference): rounded granite boulders leaning on each other, a small grotto at the base.
  The boulders are placed in `assets/scene.bsn` (the opening level, so they show and can
  be moved in the Jackdaw editor), mouth toward spawn. A second `mudpit_burbler` sits at
  the mouth. 95 m from spawn and 61 m from the opening burbler, so their 30 m hearing radii
  do not overlap and the opening stays silent.
- **Docs.** The audio validation and authoring guides name the grotto as the first
  authored acoustic space; AGENTS.md records where it is and who owns what.

## Capabilities

### New Capabilities
- `echo-grotto`: the rock shelter and its burbler in the opening scene, and acoustic rock
  marking by glTF extras.

### Modified Capabilities
None in accepted specs. It serves the in-flight `add-rack-and-spatial-audio` 4.6
(authored occlusion listening) and 4.8 (reflections/reverb).

## Impact

- Art (Claude): `tools/blender/create_boulders.py`, `assets-src/blender/props/rocks/`,
  `assets/props/rocks/`, previews in `previews/props/rocks/`, `tools/assets/lod_policy.json`,
  the family's colliders.
- Code: one small system in `stubby_world` (glTF extras `acoustic` → `AcousticOccluder`).
- Scene: `assets/scene.bsn` gains the boulder placements and a second burbler.
- Audio stream (Codex): reflections/reverb, routing and the listening review use the
  grotto; no audio-engine change here.
