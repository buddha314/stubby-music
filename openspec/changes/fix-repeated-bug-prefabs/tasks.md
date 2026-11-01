- [x] 1.1 (claimed: codex) Scope legacy bug template identities by native placement/local IDs and allocate independent default clocks.
- [x] 1.2 (claimed: codex) One focused regression for repeated prefabs, stable identity after moving, and explicit overrides; build the adapter for Play.

Evidence: one focused adapter regression passed (7 repeated native-ID placements, independent default-clock allocation, move stability, explicit override preservation). Adapter build/schema passed. A copied saved scene reached connected Play within the independent 20-second 60 Hz watchdog, without prefab refusal errors in the editor log. The user scene was not edited. Visual confirmation of every individual flash remains a play observation, not asserted by this check.
