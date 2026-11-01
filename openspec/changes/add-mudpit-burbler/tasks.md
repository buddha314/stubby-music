## 1. Art (Claude)

- [x] 1.1 Document the burbler, configuration screen, material language and luminescence (AGENTS.md, style guide) (claimed: claude)
- [x] 1.2 `tools/blender/create_burbler.py -- pit`: organically round mud pool with a 1 m banked stream; bubbles with baked one-beat `Bubble_<n>_Burble` clips, wisps, steam, motes, port node with upright halo arch, interact node; profile and previews (claimed: claude)
- [x] 1.2a Keep the rack version as the concept `mudpit_burbler_rig` (`-- rig`) (claimed: claude)
- [x] 1.3 Burbler clock panel module for the clock place: read-only BPM display, jellyfish-bell beat lamp, GATE/PHASE with soft lights (`tools/blender/create_hud_modules.py`, `burbler_clock`) (claimed: claude)
- [x] 1.4 Volume module: one large `gain` knob in a soft level ring, IN/OUT, engine `Vca` (`volume`) (claimed: claude)
- [x] 1.5 Configuration HUD look: a basic Eurorack framed as a Satisfactory-style HUD (`docs/art/hud_mock.png`, `tools/blender/render_hud_mock.py`) (claimed: claude)
- [x] 1.7 Stream segment asset: tileable, bendable 4 m channel; ports with halo arches; wavy five-line staff floating on the water at every clarity, brighter as it clears; `Pulse_0` is a note riding the staff; clarity stages muddy/silty/clear (`tools/blender/create_stream.py`, `assets/streams/`) (claimed: claude)
- [x] 1.8 Reeds and streams: a stream segment is built from 6 reeds, and reeds outline its banks (`add-reeds`; `Reeds_L`/`Reeds_R` in `stream_segment`) (claimed: claude)
- [x] 1.9 Stream hologram: the planned-stream preview shown while extending (separate asset): ghost of `stream_segment` on the same subdivisions; `Flow` chevrons; valid/invalid tints (`tools/blender/create_stream_hologram.py`, `assets/streams/stream_hologram/`) (claimed: claude)
- [ ] 1.6 Asset test for the burbler glb's nodes and extras, like `module_assets.rs`

## 2. Game (Codex)

- [ ] 2.1 `mudpit_burbler` mine definition with output `clock`; seeded per-instance tempo
- [ ] 2.2 Burbler world placement (relative to the opening's discoveries)
- [ ] 2.3 Interact (E) opens the configuration screen; Escape closes; cursor and movement handling
- [ ] 2.4 Configuration HUD: three-place rack, VCO slot from inventory, volume knob; follow `docs/art/hud_mock.png`
- [ ] 2.4a Register `burbler_clock` and `volume` in `ModuleCatalog` (`synth_modules.rs`); the clock's bpm comes from the burbler, not a knob; drive `Light_level` from the VCA's output level
- [ ] 2.5 Engine patch per burbler: Clock → fixed Envelope → VCA (level = volume) ← VCO
- [ ] 2.6 Drive the asset from clock ticks: play each `Bubble_<n>_Burble` at BPM/60/division, phase-locked; wisps, steam, motes
- [ ] 2.7 Mud streams from `Outlet_0`: lay `stream_segment` along the planned path (bend it; chain `Port_out` to `Port_in`); emit a `Pulse_0` note per clock tick that rides the staff downstream at `flow_speed_mps` (y follows the staff wave; spacing = speed × 60 / BPM); blend `Stream_Water` and `Stream_Staff` strength by clarity; port highlighting on the halo arches while extending, softer once connected; hologram preview while planning: bend `stream_hologram` along the plan like the segment, scroll `Flow` along +X wrapping at `spacing_m`, tint its materials by `states` (valid/invalid)
