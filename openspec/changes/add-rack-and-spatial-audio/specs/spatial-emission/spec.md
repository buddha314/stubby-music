## ADDED Requirements

### Requirement: Default spatial audio backend
The standalone game and Jackdaw game entry point SHALL enable AudioNimbus spatial audio
by default on supported Linux x86_64 builds. Their documented launch workflows SHALL
locate or build the pinned native SDK and expose it to the runtime loader without
building the SDK as part of ordinary Cargo builds. SDK rebuilds SHALL occur only when
pinned source or build inputs change.

#### Scenario: Launch with the default backend
- **WHEN** a player launches the game through its documented launcher with no audio
  feature override
- **THEN** AudioNimbus binaural rendering and direct propagation are enabled and the
  pinned native library loads successfully

#### Scenario: Reuse the native SDK cache
- **WHEN** the documented launcher runs and the pinned SDK build inputs are unchanged
- **THEN** it reuses the cached native library without rebuilding the SDK

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

### Requirement: Geometry-based sound propagation
Spatial emitters SHALL use AudioNimbus/Steam Audio scene simulation for direct
propagation, including distance attenuation, air absorption, geometry occlusion, and
material transmission when supported by the configured source. Reflection and
listener-centric reverb outputs SHALL be available to the audio graph as asynchronous
wet signals. Scene simulation and geometry updates MUST NOT run or block on the audio
callback; the callback SHALL consume the latest available outputs.

#### Scenario: Material transmission through cover
- **WHEN** a source is behind an acoustic wall whose material permits partial transmission
- **THEN** the direct signal is attenuated and frequency-shaped according to the simulated occlusion and transmission, then recovers when the path clears

#### Scenario: Cave reflections
- **WHEN** an emitter and listener are inside an enclosed acoustic scene
- **THEN** direct sound and asynchronous reflections/reverb are mixed through the spatial audio graph, and the reflection tail remains audible after the source stops

#### Scenario: Simulation does not block playback
- **WHEN** the audio callback renders while geometry or reflection simulation is running
- **THEN** it uses the latest published simulation output without waiting, allocating sample buffers, or mutating the scene

### Requirement: Environmental audio features remain extensible
The spatial audio graph SHALL support multichannel environmental processing, including
Ambisonic encoding/rotation/decoding and baked reflection or pathing probe data, where
those AudioNimbus/Steam Audio capabilities meet the project's platform and performance
requirements. An unsupported or unsuitable feature SHALL have a recorded concrete
compatibility reason before a game-owned replacement is introduced.

#### Scenario: Baked environmental data
- **WHEN** a scene has compatible baked reflection or pathing probe data
- **THEN** the runtime loads the data and applies it to the matching acoustic scene and sources

#### Scenario: Surround or binaural listener output
- **WHEN** the output configuration selects stereo headphones or a multichannel environment bus
- **THEN** the audio graph applies the corresponding binaural or Ambisonic decode path without changing the source simulation results

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
