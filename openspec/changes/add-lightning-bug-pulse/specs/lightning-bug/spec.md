## ADDED Requirements
### Requirement: Invisible resting lightning bug
The lightning bug SHALL be stored as a reusable GLB with a reproducible Blender source and a named white_flash preview animation. The entire model SHALL be invisible and its local light intensity SHALL be zero between pulses.
#### Scenario: Resting bug
- **WHEN** no pulse envelope is active
- **THEN** no body, wings or light are visible
### Requirement: White flash follower
On a pulse, the bug SHALL briefly appear with neutral white emission, a white local light and a short nearby audio-rate click, attack rapidly and decay smoothly to complete invisibility. All timing SHALL derive from the sample-timestamped pulse, not a free-running animation or frame timer.
#### Scenario: Pulse and return to rest
- **WHEN** a follower reaches its pulse instant
- **THEN** a white flash rises and decays over 0.18 seconds
- **THEN** the model and its light return to complete invisibility
