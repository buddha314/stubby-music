## 0. Proposal and existing-tool fit
- [x] 0.1 (claimed: codex) Inspect native Jackdaw scatter, masks, provenance, promotion and existing game prefab identity; document reuse and integration gaps.
- [x] 0.2 (claimed: codex) Reconcile opening-region generation tasks and replace the 1,000-seed sweep with seed 17 only.
- [x] 0.3 Seed first (user, 2026-11-19): the opening's resources are scattered too; hand placement is an overlay only if the seeded layout fails (design "Scope and first deliverable").

## 1. Shared configuration and native scatter adapter
- [x] 1.1 (claimed: codex) Share validated world configuration across game, adapter and export; record revisions and explicit seed 17 fixture; report malformed input.
- [x] 1.2 (claimed: codex) Adapt existing ground height sampling to native heightmap/channel inputs in one finite region; call native scatter, preserving the mudflat renderer. Stop and document an API incompatibility before substituting a package.
- [x] 1.3 (claimed: codex; finished by claude 2026-11-15) Resolve versioned masks, authored clearance and candidate keys before filtering; preserve authored IDs and transforms. Masks and keys were Codex's (`resolve_layout`, `PlacementKey`, `blocked_cells` under the rule revision); authored clearance now comes from `WorldConfig::authored_clearance` over authored objects' positions and the per-asset `authored_footprints`, reading positions only.

## 2. Designer control and runtime binding
- [x] 2.1 (claimed: codex; verified by claude 2026-11-15) Reuse native masks and supported promotion/edit persistence; add only missing suppression/transform overlay data through existing Serde and game extension points. Native `ScatterMask`/`ChannelData`; `PlacementEdit::{Suppress, Move}` in `WorldConfig` round-trip through `world-config.json` (Codex's seed-17 test). Showing them in preview is 2.2.
- [ ] 2.2 Wire one decorative rule to matching preview and runtime; verify rerun does not recreate suppressed originals or duplicate promoted objects.
- [x] 2.3 (claimed: codex; verified by claude 2026-11-15) Report conflicting/stale overrides and unsupported versions; retain edits without guessing a migration. Already in Codex's code and tests: unsupported schema/generator revisions are refused, conflicting moves and stale edits are reported, edits are kept.
- [ ] 2.4 Before enabling any interactive scatter, instantiate existing prefabs with stable keys and existing bridge/ledger cleanup; verify batched decoration is never mistaken for a live resource.

## 2b. The opening's rules, in order (design "Scope and first deliverable")
- [ ] 2b.1 Ordered rule resolution: a rule may read earlier rules' placements (distance, clearance)
- [ ] 2b.2 Burblers: spacing, seeded tempo on the 24 BPM grid, unique instance and module IDs
- [ ] 2b.3 Wetness channel (burbler pools, low ground); reed beds clustered on it; ripple pools in it (`add-ripple-pool`)
- [ ] 2b.4 Oak grove 20-40 m from a burbler; some trunks hollow (`add-high-pass-sources`)
- [ ] 2b.5 Rock formations: boulder clusters, some with an opening (art: `add-high-pass-sources`)
- [ ] 2b.6 Desert props as decoration (`add-desert-opening` 2.6)
- [ ] 2b.7 Remove the hand-placed opening objects from `scene.bsn` once their kinds are seeded; keep the grotto authored only if the seeded layout cannot place it well

## 3. Focused verification and handoff
- [ ] 3.1 Use one seed-17 fixture for repeatability, protected/authored priority, move/remove persistence and unaffected-key stability. No additional seed sweep.
- [ ] 3.2 Extend existing bounded diagnostic selector to carry the same config; one 20-second 60 Hz preview/Play review if runtime integration changed.
- [ ] 3.3 Update the existing research runbook with actual supported commands and editor authoring steps; record the placement identity/save boundary for future persistence work.

## Foundation evidence (2026-11-01)
- Shared configuration is consumed by both game entry points; export writes the
  resolved `world-config.json`. Seed-17 example and supported commands are in the
  research runbook. Scatter settings/masks/overlays still belong to tasks 1.3–2.1.
- Native `jackdaw_terrain` with default features disabled samples a heightmap made
  from the existing analytical ground. No editor changes, new RNG or runtime
  scatter spawning. At most one candidate per native cell in this first adapter.
- Two focused seed-17 unit tests passed; both changed game binaries passed
  `cargo check --bins`. Both affected OpenSpec changes passed strict validation.
- Corrected two pre-existing inventory-test module paths required to compile the
  focused library tests. No inventory behavior changed.
- Native mask filtering, stable placement keys, persistent move/remove overlays,
  runtime placement and bounded visual review remain pending. No graphical launch.
