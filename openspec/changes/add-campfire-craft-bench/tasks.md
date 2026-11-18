Owners follow AGENTS.md: Claude builds the camp's art; Codex builds rules and crates
unless the user assigns otherwise. Decisions first (design, Open Questions).

## 0. Decide (user)
- [x] 0.0 The craft bench is a camp: a bundle of sticks (replacing the stone ring and lean-to), lit by a voice and a sequencer (user, 2026-11-17)
- [x] 0.6 Recipes have a BPM range; the bench's clock must be in it (rhythm management); tempos on a 24 BPM grid for now (user, 2026-11-17)
- [x] 0.1 Clock source: a running burbler or flowing stream within `clock_reach_m` (5 m to start, profile data, tuned later) (user, 2026-11-17)
- [x] 0.2 Reed cost: 25 (user, 2026-11-17). Open: whether mounted items stay in the rack (draft: yes)
- [x] 0.3 The fire burns only while its clock runs (user, 2026-11-17)
- [x] 0.4 A lit fire does not change air or light quality (user, 2026-11-17)
- [ ] 0.5 First recipes at the bench

## 1. Rules (rule PR)
- [ ] 1.1 `fire_lit { buildable }` condition kind in `stubby_recipes::goals`, validated against `buildables.json`; rules-fmt; event-table test
- [ ] 1.2 Camp row in `buildables.json`; "Build a campfire" step under "Build a camp" (with `sync-major-minor-milestones`)

## 2. Art (Claude)
- [x] 2.1 `tools/blender/create_campfire.py` → `assets-src/blender/structures/campfire.blend` → `assets/structures/campfire/campfire.glb`: a bundle of fourteen sticks over ash with kindling; `Fire` and `Embers` hidden by start_scale 0; `Light_fire`, `Sparks`, `Interact`; LOD0/LOD1 nodes; previews unlit, burning, embers
- [x] 2.2 Profile with the `configure` block (D2), `clock_reach_m`, `light_pattern`; Jackdaw prefab; collider; LOD policy and metadata
- [ ] 2.3 Item/buildable icon via `tools/icons/make_item_icons.py`
- [ ] 2.4 Build hologram for the camp (pale amber, as streams)

## 2b. Recipes at a tempo (D9)
- [x] 2b.1 `RecipeDefinition.bpm` (`BpmRange`, multiples of `BPM_STEP` 24, validated), `Catalog::craft_at` and `RecipeError::OffTempo`; `CraftRequest.bpm`; tests
- [x] 2b.2 The 24 BPM grid: the burbler clock's BPM knob `step: 24` (`stubby_rack` knob step), new burblers seed on it (48 to 192)
- [ ] 2b.3 The crafting screen shows each recipe's range and the bench clock's BPM; the bench sends its clock's BPM with each `CraftRequest`
- [ ] 2b.4 First recipe rows with ranges (with 0.5)

## 3. Game
- [ ] 3.1 Camp binder (`AssetBinder`), build placement with Q, reed cost
- [ ] 3.2 Lighting rule in glue from `StepTick` and rack completeness (D3); sparks from the bundle on the heard instant; report `fire_lit`
- [ ] 3.2a The camp's clock from the nearest running burbler or flowing stream within `clock_reach_m` (D7); the fire dies when that beat stops (D1)
- [ ] 3.3 Crafting screen in the Craft Bench layout (D4) over `CraftRequest`, with pinning; shared screen mode
- [ ] 3.4 Fire crackle and light (with `add-rack-and-spatial-audio`)
- [ ] 3.5 Bounded watchdog review: build, patch, light, craft
