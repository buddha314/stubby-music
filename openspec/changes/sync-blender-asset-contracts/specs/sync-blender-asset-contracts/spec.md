## ADDED Requirements

### Requirement: State parts retain their child transforms
Exporters SHALL preserve child transforms under hidden state parts using full-scale export plus start_scale metadata. Bevy SHALL restore the authored hidden state before the first visible frame.

#### Scenario: State parts retain their child transforms
- **WHEN** a dormant oak is loaded and subsequently grown
- **THEN** its foliage and catkins appear at their authored transforms without an initial full-bloom flash

### Requirement: Portable assets keep native paths
Shared Blender geometry, materials, profiles and sources SHALL retain provenance and CC0 outside-asset licensing; the Bevy adapter SHALL emit assets/models paths, glTF scene selectors, previews outside assets, and Jackdaw model-owned LOD metadata. Godot scenes and import files SHALL NOT replace prefabs.

#### Scenario: Portable assets keep native paths
- **WHEN** an upstream generator changes
- **THEN** a mapped Bevy export preserves node names, collider alignment, LODs and runtime paths
