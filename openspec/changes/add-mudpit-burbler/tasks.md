## 1. Art (Claude)

- [x] 1.1 Document the burbler, configuration screen, material language and luminescence (AGENTS.md, style guide) (claimed: claude)
- [x] 1.2 `tools/blender/create_burbler.py -- pit`: organically round mud pool with a 1 m banked stream; bubbles with baked one-beat `Bubble_<n>_Burble` clips, wisps, steam, motes, port node with upright halo arch, interact node; profile and previews (claimed: claude)
- [x] 1.2a Keep the rack version as the concept `mudpit_burbler_rig` (`-- rig`) (claimed: claude)
- [x] 1.3 Burbler clock panel module for the clock place: read-only BPM display, jellyfish-bell beat lamp, GATE/PHASE with soft lights (`tools/blender/create_hud_modules.py`, `burbler_clock`) (claimed: claude)
- [x] 1.4 Volume module: one large `gain` knob in a soft level ring, IN/OUT, engine `Vca` (`volume`) (claimed: claude)
- [x] 1.5 Configuration HUD look: a basic Eurorack framed as a Satisfactory-style HUD (`docs/art/hud_mock.png`, `tools/blender/render_hud_mock.py`) (claimed: claude)
- [ ] 1.7 Stream segment asset: nodes for the BPM pulse to travel along, and a look per clarity stage from muddy to clear
- [ ] 1.8 Reeds as conveyor-belt segments (needs its own change once designed)
- [ ] 1.9 Stream hologram: the planned-stream preview shown while extending (separate asset)
- [ ] 1.6 Asset test for the burbler glb's nodes and extras, like `module_assets.rs`

## 2. Game (Codex)

- [ ] 2.1 `mudpit_burbler` mine definition with output `clock`; seeded per-instance tempo
- [ ] 2.2 Burbler world placement (relative to the opening's discoveries)
- [ ] 2.3 Interact (E) opens the configuration screen; Escape closes; cursor and movement handling
- [ ] 2.4 Configuration HUD: three-place rack, VCO slot from inventory, volume knob; follow `docs/art/hud_mock.png`
- [ ] 2.4a Register `burbler_clock` and `volume` in `ModuleCatalog` (`synth_modules.rs`); the clock's bpm comes from the burbler, not a knob; drive `Light_level` from the VCA's output level
- [ ] 2.5 Engine patch per burbler: Clock → fixed Envelope → VCA (level = volume) ← VCO
- [ ] 2.6 Drive the asset from clock ticks: play each `Bubble_<n>_Burble` at BPM/60/division, phase-locked; wisps, steam, motes
- [ ] 2.7 Mud streams from `Outlet_0`: pulse at the BPM along the stream; port highlighting on `Outlet_0_Halo` while extending, softer once connected; hologram preview while planning
