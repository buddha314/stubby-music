## MODIFIED Requirements

### Requirement: Environmental spatial audio uses a compatible FOSS implementation
The spatial audio implementation SHALL evaluate maintained FOSS libraries for HRTF,
occlusion, reflections/reverb, and environmental attenuation. It SHALL adopt a library
only when its license, target support, realtime integration, and required behavior are
verified. The implementation SHALL continue to expose bounded spatial parameters to
the audio engine through a stable boundary.

#### Scenario: AudioNimbus integration is viable
- **WHEN** AudioNimbus/Steam Audio passes license, platform, build, geometry-update,
  realtime-handoff, and Firewheel integration checks
- **THEN** its simulation supplies environmental audio parameters through the audio
  boundary without doing unbounded simulation work on the callback

#### Scenario: AudioNimbus integration is not viable
- **WHEN** SDK distribution, compatibility, or realtime constraints prevent adoption
- **THEN** the project records the reason and uses the existing bounded HRTF and
  game-supplied occlusion design

#### Scenario: Environmental effects are unavailable
- **WHEN** reflections or reverb are disabled or unsupported on a target
- **THEN** distance, hearing radius, occlusion, and spatialization continue to work
  through the established fallback path
