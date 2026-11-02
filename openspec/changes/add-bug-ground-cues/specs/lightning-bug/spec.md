## ADDED Requirements
### Requirement: Hover and ground cue
Each lightning bug SHALL hover between 1 and 2.2 metres above terrain (no higher, so it stays in view as the player walks up to it); a placed bug takes a fresh seeded height in its asset's `hover_metres` range (1.3-1.7 m) at each flash and turns broadside to the player's camera. It SHALL have a soft ground shadow directly below it that pulses with its sample-clock flash.
#### Scenario: Pulse
- **WHEN** a lightning bug flashes
- **THEN** its ground shadow appears with the same envelope and fades out between flashes
#### Scenario: Position
- **WHEN** bugs spawn
- **THEN** their heights are measured relative to terrain and remain within 1–2.2 metres
