## ADDED Requirements

### Requirement: A rock grotto in the opening scene
The opening scene SHALL contain a rock shelter of three to five large boulders, placed as
prefabs in `assets/scene.bsn` so they appear and can be edited in the Jackdaw editor, with
an open mouth and a roof slab the player can stand under. It SHALL stand far enough from
spawn and from the opening burbler that the opening stays silent.

#### Scenario: Seen in the editor
- **WHEN** the opening scene is opened in Jackdaw
- **THEN** the boulders and the grotto's burbler are listed and placed in the scene

#### Scenario: Standing inside
- **WHEN** the player walks into the grotto's mouth
- **THEN** rock stands on three sides and the top boulder is overhead, and the colliders keep the player out of the rock

#### Scenario: The opening stays silent
- **WHEN** the game starts at spawn
- **THEN** neither burbler is within its hearing radius of the player

### Requirement: A burbler at the grotto
A mudpit burbler SHALL be placed at the grotto's mouth, close enough that its voice
reaches the alcove, an ordinary placed burbler with its
own instance, seeded tempo and rack, silent until the player configures it.

#### Scenario: Configuring the grotto burbler
- **WHEN** the player configures the grotto's burbler
- **THEN** it sounds like any burbler, and the grotto's rock shapes what the player hears once reflections are enabled

### Requirement: Rock is acoustic by data
A model's mesh nodes with glTF extras `"acoustic": "rock"` SHALL be marked as acoustic rock
occluders when spawned, with no code naming the asset; the boulders' walls and roof SHALL
use it.

#### Scenario: A placed boulder
- **WHEN** a boulder prefab is placed and spawns
- **THEN** its near-level mesh entities carry `AcousticOccluder::ROCK` and enter the acoustic scene

#### Scenario: Unmarked models
- **WHEN** a model has no `acoustic` extras
- **THEN** none of its meshes become occluders
