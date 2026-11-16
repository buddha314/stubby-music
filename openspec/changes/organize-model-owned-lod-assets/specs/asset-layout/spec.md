## ADDED Requirements

### Requirement: Runtime assets are grouped by family
The project SHALL keep each placeable asset family's profiles, collider data, models and adjacent import metadata, with one master Jackdaw prefab per authored variant beside its model under `assets/<category>/<id>/`. Variant models SHALL live beneath their family. Blender sources and previews SHALL remain outside runtime assets, and shared resources MAY remain in shared directories.

#### Scenario: A designer locates a family
- **WHEN** a designer opens a scrub oak family folder
- **THEN** the family profiles are present and its variant prefabs, models and adjacent `.glb.meta` files are found beneath it

### Requirement: LOD import belongs to the model
Authored levels SHALL remain embedded named `_LOD<n>` groups in their model. Models requiring LOD SHALL commit adjacent Jackdaw import settings produced through its import or generation and apply workflow. Ordinary placements SHALL use one master prefab and Jackdaw runtime switching without placement-owned `LodGroup` or a game-specific LOD scheduler.

#### Scenario: A model is placed twice
- **WHEN** two instances reference one LOD-configured model
- **THEN** both use that model's import settings and select detail automatically for their placement

### Requirement: Migration preserves asset references and behavior
A folder migration SHALL update profiles, prefabs, scenes, runtime code, exporters, import tooling and packaging references together. It SHALL preserve scene selectors, gameplay IDs, seeded variation, authored state parts, colliders and animation bindings.

#### Scenario: A migrated model loads
- **WHEN** an existing oak placement loads from the new paths
- **THEN** it retains its identity and variant and its growth state is represented correctly at every authored level

### Requirement: Transition policy is explicit and validated
Model import settings SHALL own runtime thresholds. Distance-derived authoring inputs SHALL document reference FOV and model size conversion. Adjacent levels SHALL transition without a visibility gap and final culling SHALL be independently reviewed.

#### Scenario: Camera or instance scale changes
- **WHEN** an LOD-configured model is viewed at a changed perspective FOV or placement scale
- **THEN** Jackdaw derives its visibility ranges from the configured screen thresholds and model size without introducing a second game distance selector

### Requirement: Packaged runtime retains model import configuration
The standalone game SHALL load the same committed model metadata and Jackdaw LOD runtime support used by the editor, without requiring a pre-existing editor generation cache. Validation SHALL detect broken model references and missing authored levels.

#### Scenario: Fresh standalone load
- **WHEN** the standalone game loads a migrated family with no editor cache
- **THEN** its configured authored or generated levels resolve through the registered Jackdaw pipeline

#### Scenario: An authored level is missing
- **WHEN** import metadata references a node absent from its model
- **THEN** asset validation fails with the model path and missing level
