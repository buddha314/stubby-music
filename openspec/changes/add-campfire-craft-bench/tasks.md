Owners follow AGENTS.md: Claude builds the camp's art; Codex builds rules and crates
unless the user assigns otherwise. Decisions first (design, Open Questions).

## 0. Decide (user)
- [x] 0.0 The craft bench is a camp: campfire with a lean-to, lit by a voice and a sequencer (user, 2026-11-17)
- [x] 0.1 Clock source: a running burbler or flowing stream within `clock_reach_m` (5 m to start, profile data, tuned later) (user, 2026-11-17)
- [x] 0.2 Reed cost: 25 (user, 2026-11-17). Open: whether mounted items stay in the rack (draft: yes)
- [x] 0.3 The fire burns only while its clock runs (user, 2026-11-17)
- [x] 0.4 A lit fire does not change air or light quality (user, 2026-11-17)
- [ ] 0.5 First recipes at the bench

## 1. Rules (rule PR)
- [ ] 1.1 `fire_lit { buildable }` condition kind in `stubby_recipes::goals`, validated against `buildables.json`; rules-fmt; event-table test
- [ ] 1.2 Camp row in `buildables.json`; "Build a campfire" step under "Build a camp" (with `sync-major-minor-milestones`)

## 2. Art (Claude)
- [ ] 2.1 `tools/blender/create_campfire.py` → `assets-src/blender/structures/campfire.blend` → `assets/structures/campfire/campfire.glb`: stone ring (four step stones with glow sockets), kindling, lean-to; unlit and burning states by node scale; LOD0–2 nodes
- [ ] 2.2 Profile with the `configure` block (D2) and `light_pattern`; Jackdaw prefab; preview renders in `previews/structures/campfire/`
- [ ] 2.3 Item/buildable icon via `tools/icons/make_item_icons.py`
- [ ] 2.4 Build hologram for the camp (pale amber, as streams)

## 3. Game
- [ ] 3.1 Camp binder (`AssetBinder`), build placement with Q, reed cost
- [ ] 3.2 Lighting rule in glue from `StepTick` and rack completeness (D3); sparks at the stone on the heard instant; report `fire_lit`
- [ ] 3.2a The camp's clock from the nearest running burbler or flowing stream within `clock_reach_m` (D7); the fire dies when that beat stops (D1)
- [ ] 3.3 Crafting screen in the Craft Bench layout (D4) over `CraftRequest`, with pinning; shared screen mode
- [ ] 3.4 Fire crackle and light (with `add-rack-and-spatial-audio`)
- [ ] 3.5 Bounded watchdog review: build, patch, light, craft
