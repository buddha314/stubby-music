## Context and reuse audit
Inspected shared Jackdaw revision `ee0ea338e535dac9cd51b785868c95d534424eeb`.
Use the game's recorded compatibility pin, not an independently upgraded Bevy.

| Need | Existing implementation | Decision |
| --- | --- | --- |
| Seeded scatter | `jackdaw_terrain/src/scatter.rs`: `scatter`, `ScatterParams`, `ScatterMask`, `Placement.cell` | First choice for candidate sampling; no replacement scatter algorithm or RNG |
| Exclusions and density | Native paint-channel masks and weight channels | Prefer an eligibility mask over adding another volume/painting editor |
| Stored decoration | Native terrain sidecar scatter groups and batched runtime rendering | Reuse where no interactive ECS resource is required |
| Hand editing | `terrain.scatter.promote`, native transforms, scene save; legacy entity-backed `ScatterInstance` provenance | Reuse supported operations; verify rerun semantics instead of assuming all storage modes preserve edits |
| Interactive placement | Existing prefabs, `IsA`, `SceneNodeId`, `PrefabEntityId`, game prefab bridge | Preserve native IDs and existing resource/clock registration; never treat a render batch as a live resource |
| Serialization | Existing Serde/JSON configuration; Jackdaw BSN scene serialization | No custom parser, scene format or generic save framework |
| Bounds/queries | Bevy math/bounding primitives; Avian queries if collision checks need them | No new geometry or physics engine |

Native scatter requires a heightmap and channels; today's mudflat is a generated
mesh and preview GLB, not a native `Terrain` asset. First inspect the pinned APIs
and make the smallest adapter experiment using the existing terrain height sampler
and native heightmap/channel types. Preserve the cracked-mud renderer. Do not
replace the terrain or fork Jackdaw merely to satisfy scatter inputs. If this
adapter is impractical, record the precise incompatibility before proposing an
alternative package; do not silently implement a second sampler.

Stored scatter can render prefab assets, but does not establish our gameplay
components per instance. The native promote operator explicitly refuses prefab
placements and directs the caller to place the prefab by hand. A promoted model
is not automatically a lightning bug or mine. The entity-backed scatter mode
compares current versus originally generated transforms to preserve edits; this
is not proof that stored groups preserve deletions on regeneration.

## Scope and first deliverable
Keep spawn, burbler, grove, loose opening bugs and required reeds authored. Add
one existing decorative prop rule, disabled by default until explicitly enabled.
Select its approved asset from the existing catalog with the art owner; no new
models or resource sources. The first deliverable is one seed-17 layout whose
config, masks and edits survive preview, Play and regeneration. Resource scatter
can then use the same records, with live-prefab instantiation proven first.
Streaming, infinite terrain, biome synthesis, a seed UI, multiplayer replication,
full ecological saves and cross-platform replay guarantees are deferred.

## Configuration and deterministic generation
Use a shared typed game-side configuration loaded by both entry points and export.
Store schema/generator versions, seed (decimal `u64`, serialized as a string), scene
and content revision references, resolved scatter settings, and override asset
reference. Initial prototype fixture is 17; do not silently change the existing
CLI default 42 for unrelated diagnostics. Explicit malformed seeds/configs fail
with an actionable error instead of falling back. Unknown revisions are rejected;
there is no automatic migration in this prototype.

Native scatter owns sampling and deterministic candidate order. Keep its input
heightmap, channels, dimensions and palette ordering versioned with the rules.
Its current fixed-order minimum-spacing rejection means arbitrary tiled generation
is not automatically equivalent to a whole-region pass: first generate one finite
opening region. No chunk streaming or order-independence claim is needed now.
Keep the existing mudflat coordinate hash unchanged. If a later independent RNG
is actually required, select a named portable ChaCha implementation from the
installed rand family and pin its value-producing versions; do not introduce it
just for this proposal. Native scatter's internal RNG is reused, not copied.

## Composition and priority
1. Load the scene and resolve authored world transforms and native IDs.
2. Load the recorded configuration, native sampling inputs and eligibility mask.
3. Produce deterministic candidate records with the existing scatter package.
4. Reject candidates in protected cells or authored-object clearance footprints.
5. Apply keyed designer suppressions or absolute transform replacements.
6. Validate final bounds, clearances and opening constraints. Report conflicts;
   never move an authored object or silently relocate a designer override.
7. Feed the resulting placement set to preview and runtime. Use existing native
   batching for decoration or the native prefab path for interactive instances.

Designer-authored placements have highest priority. Generated content cannot
move or replace them. An override cannot bypass protected spawn/path space;
invalid edited positions remain uncommitted and show a useful diagnostic.
Authored edits must be saved before resolving a layout. Changes in configuration
or authored content yield a new layout revision, not an implicit save migration.

## Exclusions
Prefer native categorical channels: excluded cells are ineligible for a rule.
For the mesh adapter, keep the mask as versioned data using native channel types.
Protect the opening's audible-silence radius and designed routes; resource rules
must also obey existing wet-ground and affordability constraints. Do not silently
invent wet-ground simulation to enable reeds.

Where an authored object needs clearance, use its configured footprint and existing
bounding primitives to produce ineligible cells. Evaluate moved candidates against
the final footprint too. If native channels cannot express a designer's needed
shape, add a small reflected **game component** using existing Bevy shapes and
Jackdaw transform/inspector support; no custom editor panel. Only implement that
fallback when a concrete authoring case requires it.

## Stable identity and edits
Authored resources retain `(SceneNodeId, PrefabEntityId)` identity through the
existing bridge. Generated keys use generator revision, stable rule ID, region ID
and the native source-cell ID, plus candidate ordinal only if the sampler actually
permits multiple candidates per cell. Do not number accepted outputs: rejecting
one candidate must not renumber another. Keys are structured serializable fields,
not a new hash algorithm. Include the world configuration identity when storing
per-world state. Never use ECS entity IDs, transforms or vector offsets as keys.

Reuse native edit persistence if it satisfies this contract. For missing behavior,
the smallest game-side overlay is a Serde map from generated key to `Suppress` or
`TransformOverride`; these names describe domain data, not a new editor framework.
Apply suppressions after candidate generation without altering subsequent samples.
Keep suppressed keys in the overlay so regeneration cannot resurrect a deletion.
Unknown keys are reported as stale edits, retained for inspection, and never
retargeted to the nearest object. Rule/input changes require a new revision.

For the initial authoring workflow: regenerate in an isolated scene, inspect the
records, then use native promotion for supported model placements or place an
existing prefab at the chosen transform. Record suppression of the original key
before saving, so rerunning cannot leave both generated and hand-placed copies.
Do not promise a new one-click override UI. A small game-side command/config edit
may bridge this workflow only where the native operation lacks the required data.

## Persistence boundary
The configuration, authored scene, masks, rule data and designer overlay reproduce
the initial layout. They do not resume a played world. Retain stable keys for later
collection/replenishment and construction deltas, but do not build a second save
system here. Regeneration must not clear an existing in-memory resource ledger;
rerunning after a played session needs the owning save/ledger integration before
being exposed. Runtime simulations and exports remain off the audio callback.

## Validation and runbook
Use **only seed 17**. One small placement fixture checks repeated resolution,
authored priority, an exclusion, one moved object, one suppressed object and stable
unaffected keys. A config round-trip and invalid-input case may share this fixture.
No 1,000-seed sweep, workspace-wide tests, benchmark matrix or cold editor rebuild.
Run OpenSpec strict validation for this proposal. During implementation, follow the
existing research runbook for input recording and matching preview/runtime config;
any visual check uses the independent 20-second watchdog at 60 Hz. Extend its
selector first so the recorded test really uses seed 17, not a seed-42 preset.

## References
- Repository: `docs/research/seeded-world-generation.md` (research and runbook).
- Shared source: `book/src/user-guide/terrain.md`, `src/terrain/scatter.rs`,
  `crates/jackdaw_terrain/src/scatter.rs`, and scene-types `ScatterGroup` / `ScatterInstance`.
- [Rust Rand reproducibility](https://rust-random.github.io/book/crate-reprod.html):
  seeded alone does not guarantee stable output; sampling versions matter.
- [rand_chacha](https://docs.rs/rand_chacha/0.10.0/rand_chacha/): documented portable
  generators if native scatter cannot serve a later, distinct random requirement.
  This is a reference, not a request to upgrade our currently locked dependencies.
