## 1. Art (Claude)
- [x] 1.1 Rack frame, slot outlines (4–16 HP) and slot labels (CLOCK, VCO, VCF, VCA) (`tools/blender/create_hud_rack.py`, `assets/hud/`) (claimed: claude)
- [x] 1.2 Mocks rendered from the assets and the burbler's configuration (`docs/art/hud_mock_empty.png`, `hud_mock.png`) (claimed: claude)

## 2. Game (Codex)
- [ ] 2.1 `Rack` component, `RackSlot`, `Cable`, `ComponentKind`, `RackConfiguration` loaded from a machine profile's `configure` block
- [ ] 2.2 `RackMachine` trait with defaults (`header`, `normalled_output`, `on_patch_changed`); the burbler implements it
- [ ] 2.3 Rack screen: its own camera and render layer; stretch `rack_frame` to the slots' width; mount modules or show labelled outlines; dim and blur the world (`assets/hud/rack_frame/asset.json`, `screen`)
- [ ] 2.4 Mount by kind from the inventory; fixed slots cannot be emptied
- [ ] 2.5 Common open/close: E within range of `Interact`, prompt, nearer target wins, Escape closes
- [ ] 2.6 Patching and the engine patch from modules and cables (shared with `add-mudpit-burbler` 2.4b, 2.5)
