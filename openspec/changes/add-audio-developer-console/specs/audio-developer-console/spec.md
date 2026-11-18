## ADDED Requirements

### Requirement: Developers can tune live spatial audio
Dev builds SHALL expose a local developer control for the per-emitter reflection wet
mix through the existing Rules Designer HTML page. The control SHALL read and update the
running game through Bevy Remote Protocol and SHALL apply without restarting the game.
The control SHALL be session-only and SHALL identify that listener-centric reverb is not
currently routed.

#### Scenario: Compare dry and wet reflection output
- **WHEN** a developer sets reflection wet mix to 0.0 and applies it in a running dev game
- **THEN** per-emitter reflection output is absent while direct spatial audio remains

#### Scenario: Increase reflection audibility
- **WHEN** a developer applies a reflection wet mix between 0.0 and 1.0
- **THEN** existing and newly-created spatial emitters use that wet gain without restarting

#### Scenario: Game is unavailable
- **WHEN** the dev game is stopped, is not a dev build, or does not expose the reflected setting
- **THEN** the page reports that tuning is unavailable and does not report the edit as applied

#### Scenario: Release build
- **WHEN** the game is built without its dev feature
- **THEN** the developer settings resource and control integration are absent
