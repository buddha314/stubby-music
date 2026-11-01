## 1. Binding (done, PR #17)

- [x] 1.1 Module profiles generated with each model (`asset.json`) and embedded in the game (`ModuleCatalog`)
- [x] 1.2 `SynthModule` + `ModuleParams`: placing a module adds its engine module; despawning removes it; kinds the engine lacks are shown with a warning
- [x] 1.3 Parameter changes sent to the engine (`SetParam`), unchanged values not resent
- [x] 1.4 Shared `PulseBusPlugin`; clock lamp and GATE light flash on engine ticks with the profile envelope; PHASE light ramps per beat
- [x] 1.5 BPM display segments and knob rotation (linear and log taper) follow parameters
- [x] 1.6 Per-instance glowing materials and a point light on the clock lamp
- [x] 1.7 `--module-demo` places a clock (120 BPM) and a square VCO near spawn, in the game and Jackdaw
- [x] 1.8 Tests: profiles parse, display, knob tapers, and clock flashes scheduled one beat apart at 120 and 240 BPM with one flash per tick

## 2. Open

- [ ] 2.1 Shared engine `ModuleId` allocator, used by module assets, lightning bugs, the audio demo and collection cues; remove the hand-picked ranges from `AGENTS.md`
- [ ] 2.2 VCO `Light_out` follows the module's output level, once the rack publishes meters (`add-rack-and-spatial-audio` 3.2)
- [ ] 2.3 Saw and triangle VCOs sound, once the engine has those oscillators
- [ ] 2.4 On-screen timing: measure that the clock light peaks within 10 ms of each tick in the running game (also closes `add-desert-opening` 4.5)
- [x] 2.5 (Codex, add-jackdaw-live-listen) See it live: run with `--module-demo` (game and Jackdaw) and confirm the clock flashes on the beat you hear

  Live listening evidence (2026-10-31): `docs/research/jackdaw-live-listen.md`. The visible clock and audible square share one engine clock; standalone user confirmation and game/Play output captures agree with 120 BPM. Stop/replay verified. Task 2.4 remains open for a measured on-screen 10ms tolerance.

  Post-reboot 20-second 165Hz Play: user confirmed no flicker after the grubby workaround. Bounded live validation is complete.
