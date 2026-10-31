## 1. Asset pipeline
- [x] 1.1 Create creature asset folders, descriptor and reproducible Blender builder/export tools
- [x] 1.2 Build and inspect the lightning bug through Blender MCP; save blend and GLB with white_flash clip
## 2. Clock
- [x] 2.1 Implement seeded sample-accurate RandomClock with bounded intervals and timestamped ticks
- [x] 2.2 Test determinism, bounds, silence, block-size independence, removal and realtime allocation safety
## 3. Game
- [x] 3.1 Add shared audio pulse delivery and timestamp-driven white flash followers
- [x] 3.2 Give each bug its own random clock in the opening; expose runtime assets to Jackdaw
- [x] 3.3 Verify invisible rest, white flash, independent bugs and opt-in synchronized followers; run checks and document use

- [x] 2.3 Introduce the PulseClock trait and both PeriodicClock and RandomClock implementations

- [x] 3.4 Compose nearby click from existing square VCO/envelope/VCA; validate audio-rate transient and LFO/gate reuse
