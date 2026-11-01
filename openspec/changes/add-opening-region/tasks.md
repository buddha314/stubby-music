## 0. Decisions
- [ ] 0.1 Where the first clean water comes from (design: open questions)
- [x] 0.2 Layout: prefabs placed in Jackdaw (`jackdaw/assets/prefabs/`, made with Jackdaw's Save as Prefab)
- [ ] 0.3 Reconcile `add-desert-opening` discovery-progression (square from a bug; clock is the burbler)

## 1. Layout (Claude, art and scene)
- [ ] 1.1 Region layout sketch: spawn, grove, burbler, reed beds, prop scatter, with distances; top-down plan in `docs/art/`
- [x] 1.0 Prefabs for every placeable world asset, far LODs as `_LOD1.glb`, models posed by node scale; `jackdaw/tests/prefabs.rs` (claimed: claude)
- [ ] 1.2 Lay out the region in the Jackdaw opening scene from the prefabs
- [ ] 1.3 Lighting pass on the region at dusk and night with the bleak keys

## 2. Game (Codex)
- [ ] 2.5 Bind behaviour to placed prefabs: a placed `ResourceNode` lightning bug gets its flash, light, shadow and click voice on its `PulseFollower` clock (replacing fixed `BUG_POSITIONS`); placed oaks and modules join the ledger and the engine; reject duplicate `instance` IDs
- [ ] 2.1 Region generation from the seed: anchors, bug scatter near the grove, reed beds on wet ground, prop scatter; determinism test
- [ ] 2.2 Placement constraint tests across 1,000 seeds (silent spawn, oak distance, affordable first stream)
- [ ] 2.3 Starting conditions: winter, dormant oaks, light quality 0
- [ ] 2.4 Wire the loop end to end once its parts land: bug → burbler voice → stream → oak growth → bug production
