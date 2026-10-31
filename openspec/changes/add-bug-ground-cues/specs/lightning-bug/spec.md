## ADDED Requirements
### Requirement: Hover and ground cue
Each lightning bug SHALL hover between 1 and 3 metres above terrain with a soft ground shadow directly below it that pulses with its sample-clock flash.
#### Scenario: Pulse
- **WHEN** a lightning bug flashes
- **THEN** its ground shadow appears with the same envelope and fades out between flashes
#### Scenario: Position
- **WHEN** bugs spawn
- **THEN** their heights are measured relative to terrain and remain within 1–3 metres
