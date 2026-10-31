## ADDED Requirements

### Requirement: Emitters
A module MAY declare one output as its emission. That output SHALL be heard from the
module's world position. A module with no emission SHALL still run, because its
signals may feed other modules.

#### Scenario: Module sounds from where it stands
- **WHEN** an emitting module stands 10 m to the listener's left
- **THEN** its sound is heard from the left

### Requirement: Listener follows the camera
The listener's position and orientation SHALL follow the camera every frame, so
turning resolves front/back and left/right ambiguity.

#### Scenario: Turn to locate
- **WHEN** the player turns the camera 180° with an emitter directly ahead
- **THEN** the emitter is heard from directly behind

### Requirement: Spatialization tiers with a voice budget
Each frame, emitters SHALL be ranked by their audibility at the listener. The top
`hrtf_voices` emitters (default 16) SHALL be spatialized with HRTF, the next
`pan_voices` (default 32) with basic panning, and the rest SHALL be virtual: silent,
but still computed by the rack. A changing ranking SHALL use hysteresis, and tier
changes SHALL crossfade over at least 20 ms.

#### Scenario: Over budget
- **WHEN** 100 emitters are audible
- **THEN** the 16 most audible use HRTF, the next 32 use panning, and 52 are virtual

#### Scenario: No flapping
- **WHEN** two emitters swap rank back and forth within a small audibility margin
- **THEN** neither changes tier more than once per second

#### Scenario: Smooth promotion
- **WHEN** an emitter moves from the panning tier to HRTF
- **THEN** its sound crossfades without a click

### Requirement: Distance cues and hearing radius
Emitters SHALL attenuate with distance and lose high frequencies with distance (air
absorption). Each emitter SHALL have a hearing radius beyond which it is silent.
Within the radius, gain SHALL fall smoothly to zero at its edge.

#### Scenario: Out of range
- **WHEN** the listener is 36 m from an emitter with a 35 m hearing radius
- **THEN** that emitter is inaudible

#### Scenario: Coming into range
- **WHEN** the listener crosses into the hearing radius
- **THEN** the emitter fades in from silence, muffled, and grows louder and brighter as the listener approaches

### Requirement: Occlusion
The game SHALL supply an occlusion factor in [0, 1] per audible emitter, from a ray
cast between listener and emitter. Occlusion SHALL lower the emitter's low-pass
cutoff and gain, smoothed over about 100 ms.

#### Scenario: Behind cover
- **WHEN** a structure blocks the line between listener and emitter
- **THEN** the emitter becomes duller and quieter within about 100 ms, and recovers when the line clears

### Requirement: Speaker fallback
The player SHALL be able to switch HRTF off, for speakers. Every spatialized emitter
then uses panning, with all other cues unchanged.

#### Scenario: Speakers
- **WHEN** HRTF is turned off
- **THEN** no emitter uses HRTF, and distance, hearing radius and occlusion still apply

### Requirement: Spatial budget
The spatial layer SHALL render 16 moving HRTF emitters plus 32 moving panned
emitters in under 15% of a 256-frame block on average, on the reference machine, with
no audio-thread allocation.

#### Scenario: Default budget
- **WHEN** 16 HRTF and 32 panned emitters move every frame for 1,500 blocks offline
- **THEN** the mean block time is under 0.8 ms and nothing allocates
