## ADDED Requirements

### Requirement: Vardo and cyber-cob continuity
The paired asset set SHALL depict a compact four-wheeled Irish Traveller barrel-top living wagon and a working piebald Irish cob as described in `docs/art/vardo-and-cyber-cob.md`. The wagon SHALL have its towing shafts at the front and entrance at the rear, two wheels per axle, and smaller front wheels. The horse SHALL have exactly one near-side front prosthetic leg and three natural legs.

#### Scenario: Wagon views remain consistent
- **WHEN** the wagon is reviewed in front, rear, side, and three-quarter views
- **THEN** shafts, entrance, axles, and wheel proportions describe the same vehicle, with the towing end at the front and entrance at the rear

#### Scenario: Cyber-cob limb continuity
- **WHEN** the horse is inspected from either side or a three-quarter view
- **THEN** exactly one near-side front limb is prosthetic and the remaining three limbs are natural

### Requirement: Practical grown-in-place wagon design
The wagon SHALL retain a readable continuous barrel roof and practical working construction. Its living wood SHALL form structural ribs integrated with the roof and openings; foliage and luminous accents SHALL remain secondary to the silhouette and restrained palette.

#### Scenario: Roof silhouette stays legible
- **WHEN** the wagon is viewed at gameplay distance
- **THEN** the barrel-top profile remains visible beneath the structural wood and limited vegetation

#### Scenario: Cybernetic accents stay restrained
- **WHEN** the wagon and horse are shown under the illuminated review
- **THEN** amber practical lights balance small cool cyan/teal and narrow magenta accents, and none overwhelm the wood, coat, or silhouettes

### Requirement: Reproducible game-ready asset families
The vardo and cyber-cob SHALL use the repository's reproducible Blender-to-glTF asset pipeline and family layout. Their PNG concept references SHALL remain documentation and SHALL NOT be loaded as runtime assets. Authored model LODs SHALL belong to the model.

#### Scenario: Assets are regenerated and loaded
- **WHEN** the asset generators and export pipeline run
- **THEN** Blender sources and game-loadable glTF models are reproducibly produced in the documented asset families

#### Scenario: Concept references stay out of runtime
- **WHEN** runtime asset references are checked
- **THEN** only exported game assets are loaded and the documentation PNGs are not referenced by runtime systems
