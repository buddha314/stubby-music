## ADDED Requirements

### Requirement: Blender sources, glTF at runtime
Source models SHALL live as `.blend` files under `assets-src/blender/`, and the game
SHALL load only exported `.glb` files from `assets/models/`.

#### Scenario: No blend at runtime
- **WHEN** the game's asset loads are inspected
- **THEN** no `.blend` file is referenced

### Requirement: Headless reproducible export
A script under `tools/blender/` SHALL export any source file to `.glb` headlessly,
with transforms applied and Bevy-compatible axes and units.

#### Scenario: One-command export
- **WHEN** `blender -b assets-src/blender/sine_vco.blend -P tools/blender/export_gltf.py` runs
- **THEN** `assets/models/sine_vco.glb` is written and loads in Bevy at the correct scale and orientation

### Requirement: Engine-compatible materials
Exported materials SHALL use only glTF PBR properties (base colour, metallic,
roughness, normal, emissive) so they render the same in Bevy as intended in Blender.

#### Scenario: Emissive survives export
- **WHEN** the clock lamp model with an emissive material is exported and loaded
- **THEN** its emissive channel is present on the Bevy `StandardMaterial`
