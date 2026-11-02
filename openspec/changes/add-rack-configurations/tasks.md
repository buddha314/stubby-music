## 1. Art (Claude)
- [x] 1.1 Rack frame, slot outlines (4–16 HP) and slot labels (CLOCK, VCO, VCF, VCA) (`tools/blender/create_hud_rack.py`, `assets/hud/`) (claimed: claude)
- [x] 1.4 Patch guidance mocks: ghost patches, colour-matched pods, step hints (`hud_mock_ready.png`, `hud_mock_step2.png`, `hud_mock.png`); `configure.cables` names each cable's jacks, step and hint (claimed: claude)
- [x] 1.3 Cable comb on the rack frame (`CableComb`, `Comb_<n>`, plug rest nodes) and a pulsing halo on every jack (`Jack_<port>_Halo`); burbler lists two spare cables; mocks show construction, ready (cables on the comb, pods lit) and patched (`docs/art/hud_mock_ready.png`) (claimed: claude)
- [x] 1.2 Mocks rendered from the assets and the burbler's configuration (`docs/art/hud_mock_empty.png`, `hud_mock.png`) (claimed: claude)

## 2. Game (Codex)
- [x] 2.1 (claimed: claude) `Rack` component, `RackSlot`, `Cable`, `ComponentKind`, `RackConfiguration` loaded from a machine profile's `configure` block
- [x] 2.2 (claimed: claude; done as data, see design "The rack pattern as built", no trait needed yet) `RackMachine` trait with defaults (`header`, `normalled_output`, `on_patch_changed`); the burbler implements it
- [ ] 2.3 Rack screen: its own camera and render layer; stretch `rack_frame` to the slots' width; mount modules or show labelled outlines; dim and blur the world (`assets/hud/rack_frame/asset.json`, `screen`)
- [x] 2.4 (claimed: claude) Mount by kind from the inventory; fixed slots cannot be emptied
- [ ] 2.5 Common open/close: E within range of `Interact`, prompt, nearer target wins, Escape closes
- [x] 2.6 (claimed: claude) Patching and the engine patch from modules and cables (shared with `add-mudpit-burbler` 2.4b, 2.5)
- [x] 2.7 (claimed: claude) Cable comb: hang `configure.cables` on `Comb_<n>` (stored-cable curve in `rack_frame` profile `cable_comb`); dragging a plug off the comb starts a cable; unplugging both ends hangs it back
- [x] 2.8 (claimed: claude) Pulse the `Jack_<port>_Halo` of each jack a waiting `configure.cables` entry names (its `from`/`to`), tinted the cable's colour (`rest_strength` 0 → `peak_strength` 2, `period_s` 1.2); rest otherwise and once patched
- [x] 2.9 (claimed: claude) Patch guidance: ghost cable (build-ghost colour) for each waiting `configure.cables` entry; hint line shows `<step> of <n>  ·  <hint>`, then `patched_hint`

Prototype view landed with mudpit interaction 2.3: the profile lays out the existing
rack_frame, the fixed burbler_clock, slot outlines and selected slot_labels in a
native render-to-image HUD camera. ScreenMode owns cursor/movement and Escape.
Full mounting, patching, knob interaction and backdrop blur remain unfinished.
Do not treat this initial view as completion of those tasks.
- [ ] 2.10 Drag patching: drag a plug from the comb or a jack and drop it on a jack (now: click one jack, then the other)
- [x] 2.11 (claimed: claude) Relevant-inventory strip on the rack screen (number keys 1-0, click) and the rest of the inventory; starter inventory of 10 patch cables (`start.json`); comb cables free
- [x] 2.12 (claimed: claude) Player patch cables: patch with an inventory cable between any compatible jacks, an output then an input (uses one; unplugging returns it). A comb cable is taken by clicking it (or dragging it onto a jack); its plugs and tube are children of their comb slot so they follow the fitted frame. Bird-nest cables (crafting) remain future work
- [x] 2.13 (claimed: claude) Item icons on the Relevant strip (`assets/ui/icons/<item_id>.png`, `tools/icons/make_item_icons.py`); drag an item onto its empty slot to mount it (the slot outline is the drop target). VCV Rack cable gestures: drag jack to jack (or a comb cable onto a jack), the cable in hand drawn live to the pointer with its free plug, shown once (not also on the comb)
