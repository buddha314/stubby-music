## 1. Rack
- [x] 1.1 Remove the comb: `HeldCable`, `CombCable`, `CombNode`, comb drawing and gestures; `RackState::own` is every cable; `press_jack` takes and returns inventory cables (claimed: claude)
- [x] 1.2 Hover lights a patched cable (`PatchedCable`, `HoveredCable`)
- [x] 1.3 Pods blink only the other end of a known patch while its first end is in; the cable in hand takes the patch's colour
- [x] 1.4 Burbler profile and `create_burbler.py`: no `seq` place, no optional melody patch; rack frame without its comb (`create_hud_rack.py`, re-exported)
- [x] 1.5 Tests: cables from and back to the inventory; racks offer only needed places; the scripted rack review patches with inventory cables (8 of 10 left) and passes
- [ ] 1.6 The user patches in play: hover and blink read right
