## Implementation (claimed: claude, 2026-11-16)

- [x] 1.1 Compare `stubby_world`'s calendar.rs and current environment-map fix against Sky3D behavior; record existing parity and remaining visual differences. (design.md, "Parity with Sky3D")
- [ ] 2.1 Add authored opening time and moon/star/cloud motion with shared compass coordinates; preserve default sunrise until the explicit parity preset is selected. (Moon and stars done; clouds and authored opening time open.)
- [ ] 3.1 Adapt echoed seeded mountain generation; reuse Bevy sky and environment tools and art-authored lighting keys instead of importing Godot shaders. (Bevy atmosphere done; echoed ridges and lighting keys open.)
- [ ] 4.1 Test solar boundaries, frame cap, screen gating, moon angles and audio independence; capture bounded midnight/dawn/noon/dusk reviews and environment update cost. (Moon angles, the `--sky` review and cost done; frame cap and screen gating open.)
