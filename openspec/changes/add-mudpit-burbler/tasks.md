## 1. Art (Claude)

- [x] 1.1 Document the burbler, configuration screen, material language and luminescence (AGENTS.md, style guide) (claimed: claude)
- [x] 1.2 `tools/blender/create_burbler.py -- pit`: hot-spring mud pot with bubbles, ripples, wisps, steam, motes, mud-stream outlet with connection halo, and interact node; profile and previews (claimed: claude)
- [x] 1.2a Keep the rack version as the concept `mudpit_burbler_rig` (`-- rig`) (claimed: claude)
- [ ] 1.3 Burbler clock panel module for the clock place (tempo readout, luminous beat)
- [ ] 1.4 Volume module (one large knob) for the volume place
- [ ] 1.5 Configuration HUD look: a basic Eurorack framed as a Satisfactory-style HUD
- [ ] 1.7 Mud stream segment asset: raw, muddy, with nodes for the BPM pulse to travel along
- [ ] 1.8 Reeds as conveyor-belt segments (needs its own change once designed)
- [ ] 1.6 Asset test for the burbler glb's nodes and extras, like `module_assets.rs`

## 2. Game (Codex)

- [ ] 2.1 `mudpit_burbler` mine definition with output `clock`; seeded per-instance tempo
- [ ] 2.2 Burbler world placement (relative to the opening's discoveries)
- [ ] 2.3 Interact (E) opens the configuration screen; Escape closes; cursor and movement handling
- [ ] 2.4 Configuration HUD: three-place rack, VCO slot from inventory, volume knob
- [ ] 2.5 Engine patch per burbler: Clock → fixed Envelope → VCA (level = volume) ← VCO
- [ ] 2.6 Drive the asset from clock ticks: bubbles, ripples, wisps, steam, motes
- [ ] 2.7 Mud streams from `Outlet_0`: pulse at the BPM along the stream; connection lights `Outlet_0_Halo`
