Owners follow AGENTS.md: Claude builds the camp's art; Codex builds rules and crates
unless the user assigns otherwise. Decisions first (design, Open Questions).

## 0. Decide (user)
- [x] 0.0 The craft bench is a camp: a bundle of sticks (replacing the stone ring and lean-to), lit by a voice and a sequencer (user, 2026-11-17)
- [x] 0.6 Recipes have a BPM range; the bench's clock must be in it (rhythm management); tempos on a 24 BPM grid for now (user, 2026-11-17)
- [x] 0.1 Clock source: a running burbler or flowing stream within `clock_reach_m` (5 m to start, profile data, tuned later) (user, 2026-11-17)
- [x] 0.2 Reed cost: 25 (user, 2026-11-17). Open: whether mounted items stay in the rack (draft: yes)
- [x] 0.3 The fire burns only while its clock runs (user, 2026-11-17)
- [x] 0.4 A lit fire does not change air or light quality (user, 2026-11-17)
- [x] 0.5 First recipes at the bench: the reed water purifier (`add-reed-water-purifier` 0.4)

## 1. Rules (rule PR)
- [x] 1.1 `fire_lit { buildable }` condition kind in `stubby_recipes::goals`, validated against `buildables.json`; rules-fmt; event-table test
- [x] 1.2 Camp row in `buildables.json` (`campfire`, 25 reeds until campfire sticks are an item); "Light the fire" step (`fire_lit`) under "Build a camp" (with `sync-major-minor-milestones`)

## 2. Art (Claude)
- [x] 2.1 `tools/blender/create_campfire.py` → `assets-src/blender/structures/campfire.blend` → `assets/structures/campfire/campfire.glb`: a bundle of fourteen sticks over ash with kindling; `Fire` and `Embers` hidden by start_scale 0; `Light_fire`, `Sparks`, `Interact`; LOD0/LOD1 nodes; previews unlit, burning, embers
- [x] 2.2 Profile with the `configure` block (D2), `clock_reach_m`, `light_pattern`; Jackdaw prefab; collider; LOD policy and metadata
- [x] 2.3 Item/buildable icon via `tools/icons/make_item_icons.py`
- [x] 2.4 Build hologram for the camp (pale amber, as streams; red when the reeds are short; `stubby_game::build`)

## 2b. Recipes at a tempo (D9)
- [x] 2b.1 `RecipeDefinition.bpm` (`BpmRange`, multiples of `BPM_STEP` 24, validated), `Catalog::craft_at` and `RecipeError::OffTempo`; `CraftRequest.bpm`; tests
- [x] 2b.2 The 24 BPM grid: the burbler clock's BPM knob `step: 24` (`stubby_rack` knob step), new burblers seed on it (48 to 192)
- [x] 2b.3 (claimed: claude; done 2026-11-20) The crafting screen shows each recipe's range and the bench clock's BPM; the bench sends its clock's BPM with each `CraftRequest`
- [x] 2b.4 First recipe rows with ranges (with 0.5): `craft_reed_water_purifier`, 96-144 BPM (#260)

## 3. Game
- [x] 3.1 Camp binder (`AssetBinder`), build placement with Q, reed cost (Q opens the build menu, Satisfactory-style: Stream or Campfire; the binder is scenery until the rack, 3.2a). The opening scene no longer places a camp (2026-11-18): the player starts with 25 reeds and builds one as the first step, "Build a campfire" (`built` condition)
- [x] 3.2 Lighting rule (`stubby_game::campfire`) in glue from `StepTick` and rack completeness (D3); sparks from the bundle on the heard instant; report `fire_lit`
- [x] 3.2a The camp's clock from the nearest running burbler or flowing stream within `clock_reach_m` (D7); the fire dies when that beat stops (D1). The rack's clock place is `borrowed`: it plays the lent `AssetClock`, its knob does not turn, and it is empty with nothing in reach
- [x] 3.3 (claimed: claude; done 2026-11-20) Crafting screen in the Craft Bench layout (D4) over `CraftRequest`, with pinning; shared screen mode. `stubby_game::crafting_screen`; `Screen::Crafting(bench)`; a burning fire carries `stubby_core::CraftBench`, so E there opens crafting instead of the rack (generic: any machine with it is a bench), and the RACK tab opens the rack. No recipe categories yet (recipes have none); a toggle shows only what can be made now
- [ ] 3.4 Fire crackle and light (with `add-rack-and-spatial-audio`)
- [ ] 3.5 Bounded watchdog review: build, patch, light, craft
