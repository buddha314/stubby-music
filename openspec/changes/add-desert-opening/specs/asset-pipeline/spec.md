## ADDED Requirements

### Requirement: Blender sources, glTF at runtime
Source models SHALL live as `.blend` files under `assets-src/blender/`, and the game
SHALL load only exported `.glb` files from `assets/models/`, grouped in category
folders that mirror the sources.

#### Scenario: No blend at runtime
- **WHEN** the game's asset loads are inspected
- **THEN** no `.blend` file is referenced

### Requirement: Headless reproducible export
A script under `tools/blender/` SHALL export any source file to `.glb` headlessly,
with transforms applied and Bevy-compatible axes and units.

#### Scenario: One-command export
- **WHEN** `blender -b assets-src/blender/modules/square_vco.blend -P tools/blender/export_gltf.py` runs
- **THEN** `assets/models/modules/square_vco.glb` is written and loads in Bevy at the correct scale and orientation

### Requirement: Engine-compatible materials
Exported materials SHALL use only glTF PBR properties (base colour, metallic,
roughness, normal, emissive) so they render the same in Bevy as intended in Blender.

#### Scenario: Emissive survives export
- **WHEN** the clock lamp model with an emissive material is exported and loaded
- **THEN** its emissive channel is present on the Bevy `StandardMaterial`

### Requirement: Jackdaw opening project
The repository SHALL provide an isolated Jackdaw project matching the local
editor's Bevy revision, without migrating the released game workspace. Its edit
viewport SHALL load a reproducible generated desert preview, and Play SHALL run
the shared procedural scene with Jackdaw's runtime integration.

#### Scenario: Open the initial level
- **WHEN** the matching Jackdaw editor opens the supplied editor project
- **THEN** the desert preview loads without version or scene errors
- **THEN** building the game extracts its schema and Play runs the procedural opening

### Requirement: Levels of detail belong to the model
A model's levels of detail SHALL live with the model: hand-made levels as nodes named
`<id>_LOD0`, `<id>_LOD1`, ... inside its glb, and the level settings in the model's
import settings (`<id>.glb.meta`), as Jackdaw's model importer writes them. Placements
and prefabs SHALL NOT carry their own levels. Both Jackdaw's Play and the standalone
game SHALL switch levels by screen size from the model alone.

#### Scenario: One choice to place
- **WHEN** a level designer places a scrub oak
- **THEN** they place one prefab naming one model, and it draws its far level when far away

#### Scenario: Standalone game
- **WHEN** the standalone game loads a model that has `_LOD` nodes and a Jackdaw `.meta`
- **THEN** it loads the model and draws one level at a time, never both
