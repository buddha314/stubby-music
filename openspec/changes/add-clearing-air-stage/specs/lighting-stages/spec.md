## ADDED Requirements

### Requirement: Three lighting stages

The lighting data SHALL define three moods at every time of day: `bleak`, `clearing` and `bloomed`, in that order of rising light quality.

#### Scenario: Reading the keys

- **WHEN** a tool or the game loads `assets/lighting/lighting_keys.json`
- **THEN** each of the three moods has dawn, day, dusk and night keys

### Requirement: The clearing stage reveals the sky

The `clearing` stage SHALL have much less fog than `bleak`, a saturated sky and violet-leaning dark silhouettes, with flora accents absent or low. Its dusk SHALL be keyed against the reference in the style guide.

#### Scenario: Dusk at the clearing stage

- **WHEN** the clearing dusk key is rendered
- **THEN** it shows a saturated orange sky behind hard, near-black silhouettes with no visible haze between them and the viewer

### Requirement: Stages blend without a hard cut

Light quality SHALL blend continuously through the stops listed in the data, and the stops SHALL be data, not code.

#### Scenario: Light quality between stops

- **WHEN** light quality lies between two stops
- **THEN** the look is a blend of those two moods only

#### Scenario: Sweeping across the stages

- **WHEN** light quality rises steadily from 0 to 1
- **THEN** no value produces a visible jump

### Requirement: The signal stays amber

Module signal light SHALL remain amber and readable in every `clearing` key. If it does not read, the key SHALL change, not the module.

#### Scenario: A module against an orange sky

- **WHEN** a module is seen against the clearing dusk sky
- **THEN** its amber lights still read
