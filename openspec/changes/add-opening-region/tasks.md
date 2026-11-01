## 0. Decisions
- [x] 0.1 Where the first clean water comes from: a filter mini-rack, HPF → pure liquid, LPF → solids/nutrients, controlled via audio (detailed design later, `add-fluid-quality-and-viscosity`)
- [x] 0.2 Layout: prefabs placed in Jackdaw (`jackdaw/assets/prefabs/`, made with Jackdaw's Save as Prefab)
- [ ] 0.3 Reconcile `add-desert-opening` discovery-progression (square from a bug; clock is the burbler)

## 1. Layout (Claude, art and scene)
- [ ] 1.1 Region layout sketch: spawn, grove, burbler, reed beds, prop scatter, with distances; top-down plan in `docs/art/`
- [x] 1.0 Prefabs for every placeable world asset, model-owned `_LOD0`/`_LOD1` nodes, models posed by node scale; `jackdaw/tests/prefabs.rs` (claimed: claude)
- [ ] 1.2 Lay out the region in the Jackdaw opening scene from the prefabs
- [ ] 1.3 Lighting pass on the region at dusk and night with the bleak keys

## 2. Game (Codex)
- [x] 2.5 (claimed: codex) Bind behaviour to placed prefabs: a placed `ResourceNode` lightning bug gets its flash, light, shadow and click voice on its `PulseFollower` clock (replacing fixed `BUG_POSITIONS`); placed oaks and modules join the ledger and the engine; reject duplicate `instance` IDs
  - **Note for Codex (from Claude, 2026-11-01), for the `bind-placed-prefab-runtime` worktree:**
    - **The user wants this now:** pressing Play in Jackdaw still adds the procedural bugs (`BUG_POSITIONS`) beside the placed one. The design's "an authored level disables the legacy fixed bug spawn" is exactly what they asked for.
    - **Rebase onto current `main` before merging.** The worktree is based on main from before PRs #36–#43, so its diff currently undoes merged work. Keep:
      - `tools/jackdaw_lods.py` (its diff deletes it)
      - task 0.1 resolved, with no option list: the first clean water comes from the filter mini-rack, and the design's "Where the first clean water comes from" open question was answered in #40 (its diff restores the old options)
      - models in category folders (`models/<category>/...`), the oak and reed levels of detail as `<id>_LOD0`/`_LOD1` nodes inside each glb, and the `.glb.meta` import settings (#38, #42, #43)
      - prefabs without `LodGroup` (the prefab test fails on one)
    - **Identify by model stem, not by node path:** levels now sit under `<id>_LOD0`, so `Limb_*`, `Foliage_*`, `Standing`, `Stubble` and so on are one level deeper than before.
    - **Reed yield conflict:** this design says "1 reed per clump", but `add-reeds` suggests 6 per clump (one stream segment's worth), because the user asked for reeds to be "very easy to get, like leaves in Satisfactory". Use 6, or ask the user, before writing the catalog entry.
    - **The lightning bug prefab now has a ground pivot (2026-11-01):** the prefab root (what the designer drags, and what Jackdaw snaps to the floor) sits at ground level, and the bug is its child `Lightning bug hover`, 1.8 m up. `GltfSource`, `ResourceNode` and `PulseFollower` are on that **child**. Bind the flash, light, click voice, hearing range and E reach to the entity carrying `ResourceNode` (the child), using its world `GlobalTransform`. Put the ground shadow at the terrain under it. Designers set a bug's hover height on the child. The model's origin stays at the body centre, because the flash scales around it.
    - **The standalone game and LOD:** see `add-desert-opening` 1.5. Jackdaw `.glb.meta` files now sit beside 17 models, and they name Jackdaw's `ModelLoader`.

- [ ] 2.1 Region generation from the seed: anchors, bug scatter near the grove, reed beds on wet ground, prop scatter; determinism test
- [ ] 2.2 Placement constraint tests across 1,000 seeds (silent spawn, oak distance, affordable first stream)
- [ ] 2.3 Starting conditions: winter, dormant oaks, light quality 0
- [ ] 2.4 Wire the loop end to end once its parts land: bug → burbler voice → stream → oak growth → bug production
- [ ] 2.7 Placed player start (Codex; design "Player start"): a `PlayerStart` marker component (reflected, registered, inspector category "Actor"); an `On<Insert<PlayerStart>>` observer reads its world `GlobalTransform` (Jackdaw's `insert_observer_global_transform.rs` shows it is final at scene load) and moves the `controls::Player` camera there, on the ground, facing its yaw; fall back to `controls::spawn_transform()` when none is placed; warn on more than one; a test that the opening scene's start is more than 35 m from every bug. No new crate, no editor change.
- [ ] 2.7a Player start marker (Claude): the model is done (`tools/blender/create_player_start.py`, `models/actors/player_start_marker.glb`: footprint ring, facing arrow along −Z, eye post at 1.65 m); still to do, once 2.7 lands, a prefab made in Jackdaw with Save as Prefab: the marker glb, `HiddenInGame`, `PlayerStart`, under `prefabs/actors/`

### Runtime binding checks (task 2.5)

Completed against main through PR #50: category paths and import settings retained,
bug child bound at its world position, six reeds per clump. Native Jackdaw LOD
render parts carry game animation metadata; no replacement LOD tools or scene
reconstruction. The 20-second review passed near/far/return; see `jackdaw/README.md`.

- [x] 2.5a Translate authored glTF sources to registered game asset identities; preserve scene transforms and report duplicate instance/clock IDs
- [x] 2.5b Bind bug presentation/voices, module profiles and mine registry with reload/despawn cleanup
- [x] 2.5c Play authored animation clips on sample-clock phase and apply flora/prop visual states
- [x] 2.5d Emit hover/use events and show contextual interaction feedback
- [x] 2.5e Test resolved prefabs through the runtime and validate both Bevy builds
