## 1. Art (Claude)
- [x] 1.1 Rack frame, slot outlines (4–16 HP) and slot labels (CLOCK, VCO, VCF, VCA) (`tools/blender/create_hud_rack.py`, `assets/hud/`) (claimed: claude)
- [x] 1.4 Patch guidance mocks: ghost patches, colour-matched pods, step hints (`hud_mock_ready.png`, `hud_mock_step2.png`, `hud_mock.png`); `configure.cables` names each cable's jacks, step and hint (claimed: claude)
- [x] 1.3 Cable comb on the rack frame (`CableComb`, `Comb_<n>`, plug rest nodes) and a pulsing halo on every jack (`Jack_<port>_Halo`); burbler lists two spare cables; mocks show construction, ready (cables on the comb, pods lit) and patched (`docs/art/hud_mock_ready.png`) (claimed: claude)
- [x] 1.2 Mocks rendered from the assets and the burbler's configuration (`docs/art/hud_mock_empty.png`, `hud_mock.png`) (claimed: claude)

## 2. Game (Codex)
- [ ] 2.1 `Rack` component, `RackSlot`, `Cable`, `ComponentKind`, `RackConfiguration` loaded from a machine profile's `configure` block
- [ ] 2.2 `RackMachine` trait with defaults (`header`, `normalled_output`, `on_patch_changed`); the burbler implements it
- [ ] 2.3 Rack screen: its own camera and render layer; stretch `rack_frame` to the slots' width; mount modules or show labelled outlines; dim and blur the world (`assets/hud/rack_frame/asset.json`, `screen`)
- [ ] 2.4 Mount by kind from the inventory; fixed slots cannot be emptied
- [ ] 2.5 Common open/close: E within range of `Interact`, prompt, nearer target wins, Escape closes
- [ ] 2.6 Patching and the engine patch from modules and cables (shared with `add-mudpit-burbler` 2.4b, 2.5)
- [ ] 2.7 Cable comb: hang `configure.cables` on `Comb_<n>` (stored-cable curve in `rack_frame` profile `cable_comb`); dragging a plug off the comb starts a cable; unplugging both ends hangs it back
- [ ] 2.8 Pulse the `Jack_<port>_Halo` of each jack a waiting `configure.cables` entry names (its `from`/`to`), tinted the cable's colour (`rest_strength` 0 → `peak_strength` 2, `period_s` 1.2); rest otherwise and once patched
- [ ] 2.9 Patch guidance: ghost cable (build-ghost colour) for each waiting `configure.cables` entry; hint line shows `<step> of <n>  ·  <hint>`, then `patched_hint`
