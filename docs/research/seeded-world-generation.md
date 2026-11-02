# Seeded world generation dossier: Factorio, Satisfactory and Stubby

Researched **2026-11-01**. Prepared for later design work; this document does not
approve a new generator or change current OpenSpec requirements. External facts,
repository observations and proposed choices are distinguished below. Research
and synthesis by OpenAI Codex at the user's direction.

## Main findings

A seed makes a deterministic procedure repeatable. It does not, on its own, specify
that procedure, its settings, its content, or the player's subsequent actions.
For our game, the useful promise is:

> The same world configuration and supported generator/content revision reproduce
> the same initial world, regardless of exploration order.

Factorio and Satisfactory offer two useful models rather than two identical ones:

| Question | Factorio | Satisfactory 1.2 family |
| --- | --- | --- |
| Terrain | Procedurally generated | Existing hand-authored world |
| Seeded variation | Terrain and resource generation | World/resource randomization settings layered onto the world |
| Reproduction input | Seed plus generation settings; shareable map exchange configuration | Seed plus matching world settings |
| Useful lesson for us | Generate varied geography with explicit configuration | Keep authored geography/readability while varying resources |

The Factorio comparison follows its [map-generator reference][factorio-map] and
[developer explanation of noise expressions][factorio-noise]. Satisfactory's
[developer announcement][satisfactory-march] explicitly introduced shareable world
seeds and separately selected randomization settings. Its [official wiki's world
reference][satisfactory-world] describes the hand-authored terrain. That wiki is
community-maintained: its older wording about no procedural world generation
should not be interpreted as a claim that 1.2 has no seed feature.

**Recommendation, not an accepted design:** retain our authored opening and use
seeded generation around it, then decide how much of the wider landscape should
be generated. Do not replace the saved scene merely to add a seed field.

## Factorio: what is worth borrowing

### Configuration, preview and sharing

The generator exposes presets and controls, including resource frequency, patch
size and richness. It offers a map preview before play. A map exchange string
carries the generation settings, making it more useful than sharing a seed alone.
Generation proceeds as chunks are needed rather than materializing the whole
world at creation. These are documented behaviors, not claims about proprietary
implementation details. [Factorio map-generator reference][factorio-map]

The official API represents the seed as a `uint32` and exposes generation controls
such as starting areas, starting points and property-expression overrides. The API
page inspected during this research identifies itself as version **2.1.24**; that
is a documentation snapshot, not a claim about the user's installed game or a
promise that older versions generate the same map. [MapGenSettings][factorio-api]

Factorio also supports command-line preview generation with a seed and a settings
file. A comparable small preview/export command could help us inspect a world
without launching the entire game. Its map exchange configuration is not a copy
of a developed factory or the result of subsequent manual map edits.
[Command-line documentation][factorio-cli]

### Generation structure

Wube's developer account describes composing coordinate-based expressions and
coherent noise at different scales, then using their results to decide placement.
It also discusses deliberate starting-area shapes. The transferable pattern is
structured fields plus authored constraints, rather than placing every object
with unrelated random coordinates. [Friday Facts #390][factorio-noise]

An earlier developer account describes sharing intermediate calculations across
a chunk instead of repeating them independently for every tile. This is useful
performance guidance, not a reason for us to copy Factorio's custom noise code.
[Friday Facts #112][factorio-chunks]

## Satisfactory: seeds without replacing the terrain

### The version distinction matters

Coffee Stain's **17 March 2026 Experimental 1.2** announcement introduced game modes
with resource randomization, purity controls and a shareable world seed. It says
other settings must also match to reproduce a setup. The announcement described
zero as the vanilla default seed at that time; do not turn that historical value
into a universal rule about valid seeds or a randomization sentinel.
[Developer announcement][satisfactory-march]

The developer's [announcement index][satisfactory-news] dates the public 1.2 release
to **2 June 2026**. Consequently, older advice that Satisfactory has no selectable
world seed is incomplete for the 1.2 family.

The important distinction is **world terrain versus resource layout**. A seed
option does not imply a new mountain range, coastline or biome topology. The
hand-authored map remains the relevant comparison for readable first-person
exploration and designed factory sites. [World reference][satisfactory-world]

### What we can and cannot infer

The official patch notes establish the player-facing settings. They do not fully
specify the PRNG, every resource-placement constraint, numeric seed range, or
cross-version guarantees. Do not claim those internals from the menu alone.

There is a useful independently implemented research lead: the
[Satisfactory Tools generator wrapper][satisfactory-tool] accepts a seed, mode,
purity settings and baseline world data, then outputs node types, purities and
coordinates. Its author says it wraps a replica of the game's 1.2 algorithm.
This is primary documentation of that **tool**, not Coffee Stain's specification
or proof of exact compatibility. Inspect its upstream code and verify against the
game before relying on its behavior. It is not a dependency recommendation here.

For our design, borrow the separation between a readable authored world and a
configurable resource layer. Do not assume changing a seed has to regenerate all
geography, and do not promise that a bare seed reproduces every menu choice.

## What “the same world twice” should mean for Stubby

The following is engineering guidance derived from the comparisons, not a claim
that either game implements this exact schema.

| Input/state | Why retain it |
| --- | --- |
| Numeric seed | Selects deterministic variation |
| Generator revision | Placement rules and noise changes can change results |
| Generation settings | Density, spacing, biome and resource controls affect output |
| Authored layout/content revision | Edited prefab placements, geometry or collision data affect the world |
| Relevant rules/catalog revision | Resource availability and progression can change even with identical positions |
| Save-format revision | Allows later migration of stored world state |

A prototype should start with a small readable configuration, rather than a new
binary exchange format. For example, **illustrative only**:

```json
{
  "seed": "42",
  "generator_version": 1,
  "layout_revision": "opening-layout-revision",
  "rules_revision": "resource-catalog-revision",
  "settings": {
    "preset": "desert-opening-v1"
  }
}
```

Resolve presets to concrete settings when creating a save. Otherwise changing a
preset definition silently changes the meaning of an old world configuration.
References above need resolvable versions or content digests, not labels that can
silently point to new files. Their exact schema remains a future OpenSpec choice.

Our current CLI uses a Rust `u64` seed. Keeping it as a decimal string in a future
JSON sharing format would avoid precision loss in clients that parse numbers as
JavaScript floating-point values. Initially keep numeric seeds; arbitrary text
seeds would need a specified, versioned text-to-number mapping.

Separate three operations in the eventual UI:

1. **Recreate initial world:** seed/configuration plus matching generation content.
2. **Resume world:** load the save, including changes and simulation state.
3. **Share a developed world:** share its save/snapshot, not just its seed.

For us, harvested bugs, next-night replenishment, planted/grown trees, water
quality, player-built streams, inventory and rack patches are mutable state. They
cannot be reconstructed from the initial seed alone. Generated-but-unmodified
regions may be reproducible; modified regions need saved deltas or snapshots.
Do not imply that this alone provides bit-identical physics, audio or replays.

## Determinism practices for later implementation

### Reuse established generators and record the full pipeline

Use a named portable RNG when sequential randomness is needed, rather than making
a new generator. The Rust `rand_chacha` documentation offers deterministic,
portable ChaCha variants. The Rand book warns that `StdRng`/`SmallRng` are not
portable output contracts and that stochastic algorithms can change output even
when their APIs remain compatible. Record versions and sampling procedures as
part of the generator revision; a named RNG alone is insufficient.
[ChaCha documentation][chacha], [Rand reproducibility guidance][rand-repro]

No library is added by this dossier. Keep the existing coordinate hash and clock
interfaces until a concrete generation proposal identifies a need to change them.
In particular, do not replace `PulseClock`, Firewheel or existing DSP as part of
world generation.

### Keep generation independent of execution order

For our future implementation, generation of a region should depend on its stable
world coordinates, root configuration and purpose, not on which region happened
to finish first. Use deterministic per-region/per-purpose streams or coordinate
sampling, following the selected library's documented seeding/stream facilities.
Specify any derivation and canonical input encoding; avoid ad hoc string hashing
or runtime-default hashers whose output is not a saved-world contract.

Keep terrain, resource placement, cosmetic scatter and individual resource timing
separate. Adding a decorative rock should not consume a shared RNG draw and move
the grove or change every bug's clock. Canonical candidate ordering and fixed
sampling rules matter as much as the root seed. Avoid making ECS entity IDs,
thread scheduling, unordered collection iteration or frame timing generation inputs.

For chunk boundaries, sample continuous fields in global coordinates, use a
neighborhood/halo for spacing decisions, and assign each accepted placement one
canonical owner. Use consistent coordinate division for negative positions.
These are proposed implementation constraints; chunk dimensions and algorithms
are deliberately undecided.

### Version intentionally

Define an initial modest guarantee: identical configuration under the same
supported generator/content revision recreates the same placement data. Broader
cross-platform or cross-version promises need evidence. Floating-point thresholds,
noise implementations and sampling distributions can change results; the Rand
book specifically discusses platform-dependent floating-point behavior.
[Rand reproducibility guidance][rand-repro]

When generation changes, select one explicit policy before release: retain the old
generator, migrate stored regions, or reject unsupported regeneration with a clear
message. Mixing old saved regions and newly generated regions without a policy can
produce seams and different resource distribution. Do not silently regenerate
player modifications on load.

## Fit with the repository today

Observed on the documentation branch based on main during this research:

| Existing piece | Present behavior | Design implication |
| --- | --- | --- |
| `crates/stubby_game/src/main.rs` | Reads `--seed=`, defaulting to 42 | There is already a seed entry point; no world-selection UI is established |
| `jackdaw/src/main.rs` | Same seed input, loads an authored scene and normally inserts `AuthoredLevel` | Seed selection alone does not rearrange saved prefab placements |
| `crates/stubby_game/src/terrain.rs` | Coordinate/seed hash builds a finite cracked-mud mesh | Deterministic surface detail exists; it is not yet a general streamed world generator |
| `crates/stubby_game/src/lib.rs` | Uses `WorldSeed` for ground texture/mesh and environmental variation | Keep one explicit world configuration flowing through those consumers |
| `jackdaw/assets/scene.bsn` | Saved authored opening, including placed bugs and burbler | Reproduction needs this layout's revision as well as the seed |
| `openspec/changes/add-opening-region/design.md` | Prefab-authored layout; later seeded scatter allowed around it | Hybrid generation is compatible with current direction; replacing the opening is a separate design decision |
| Editor floor preview | Exported generated model, separate from runtime generation | Future preview and Play must consume the same selected configuration; a stale preview export is not proof of a seed bug |

Both CLI parsers currently discard parse failures and fall back to the default
seed. A future user-facing selector should report invalid input rather than
silently producing seed 42. Treat zero as a real seed if supported; represent
“choose a new seed” separately, not by overloading a valid numeric value.

Current opening constraints already include spawn outside bug hearing range,
a reachable grove, a nearby burbler and enough reeds for the first stream. Preserve
these constraints as generation acceptance criteria; deterministic does not
necessarily mean playable. See [the opening-region design](../../openspec/changes/add-opening-region/design.md).

## Suggested prototype sequence, for later proposal

1. **Choose the scope.** Prefer an authored opening plus seeded resource/scatter
   variation first. Decide separately whether later terrain is finite, authored,
   assembled from authored regions, or generated in streamed regions.
2. **Define the reproduction configuration.** Record seed, concrete settings and
   generator/layout/rules revisions. Persist it in saves and expose it for copying.
3. **Add a small selection flow.** Seed field, choose-new-seed action, preview,
   and copy/import configuration. A new seed is chosen once and recorded; loading
   or previewing does not silently choose another.
4. **Generate placement data independently of rendering.** Produce resource kinds,
   stable generated identities and transforms. Instantiate existing Jackdaw prefab
   assets through the game's existing runtime support. Keep model LODs native.
5. **Protect the opening loop.** Verify hearing distance, reachability, grove/burbler
   separation and available reed yield. If a candidate is invalid, retry in a
   deterministic bounded order and report failure rather than hanging.
6. **Save changes separately from the initial description.** Preserve collection,
   replenishment, growth, fluid state and construction through reloads.
7. **Grow only when needed.** Add streaming, additional biomes and more generator
   controls after the first varied opening is fun and reproducible.

Minimal validation when implementation begins: one focused placement fixture
checks repeated generation and differing region request orders; one reload check
covers a collected resource and a built stream when those saves exist; a bounded
visual comparison checks preview/Play alignment. No broad test matrix or test
infrastructure is justified for this research-only commit.

## Operator runbook: reproduce and compare a world

This separates **commands supported today** from the **planned world workflow**.
It is an operational reference, not a record that these commands were executed
for this documentation change. Do not run every step for every small edit.

### 1. Freeze the inputs in an isolated worktree

Use your own warm worktree off the intended game revision and its own build
cache. Save the scene in Jackdaw before taking a copy. Never switch, pull into,
edit or launch diagnostics against another agent's live checkout. Record any
relevant uncommitted scene/model/catalog changes as actual archived files; a
commit ID or checksum cannot reconstruct content that was never retained.

From that worktree:

```bash
python3 tools/check_jackdaw_versions.py --binaries
git status --short
world_review_dir="$(mktemp -d /tmp/stubby-worldgen-review.XXXXXX)"
git rev-parse HEAD > "$world_review_dir/game-revision.txt"
git -C /home/buddha/src/jackdaw rev-parse HEAD > "$world_review_dir/editor-revision.txt"
cp jackdaw/compatibility.json "$world_review_dir/compatibility.json"
cp jackdaw/assets/scene.bsn "$world_review_dir/scene.bsn"
sha256sum Cargo.lock jackdaw/assets/scene.bsn > "$world_review_dir/input-checksums.txt"
```

The copied scene protects the layout used for the experiment; it is not a full
archive of models, rules or save state. Use clean committed content or archive
those relevant inputs too. If the compatibility guard fails, resolve the selected
source/compiler/binary set through [the version policy](jackdaw-version-policy.md);
do not rewrite the pin to suppress the mismatch. Do not rebuild the shared editor
while someone is using it.

### 2. Exercise the existing headless terrain exporter

Current support: numeric `--seed=<u64>` and `--export=<directory>` in the standalone
entry point. This exports the finite mudflat mesh, albedo, normal map and seed
record without creating a window or starting the audio engine. Build once using
the worktree's cache, then reuse that executable:

```bash
cargo build -p stubby_game --bin stubby_game --locked
target/debug/stubby_game --seed=42 --export="$world_review_dir/a"
target/debug/stubby_game --seed=42 --export="$world_review_dir/b"
target/debug/stubby_game --seed=43 --export="$world_review_dir/c"
```

Each output directory contains `mudflat.glb`, `mud.png`, `mud-normal.png` and
`seed.txt`. Confirm the seed records before comparing:

```bash
cat "$world_review_dir/a/seed.txt" "$world_review_dir/b/seed.txt" "$world_review_dir/c/seed.txt"
(cd "$world_review_dir/a" && sha256sum mudflat.glb mud.png mud-normal.png) > "$world_review_dir/a.sha256"
(cd "$world_review_dir/b" && sha256sum mudflat.glb mud.png mud-normal.png) > "$world_review_dir/b.sha256"
(cd "$world_review_dir/c" && sha256sum mudflat.glb mud.png mud-normal.png) > "$world_review_dir/c.sha256"
diff -u "$world_review_dir/a.sha256" "$world_review_dir/b.sha256"
diff -u "$world_review_dir/a.sha256" "$world_review_dir/c.sha256"
```

Expected: seed records are 42, 42, 43; A/B have identical content hashes; A/C
differ in generated geometry and/or texture content. The second `diff` normally
returns exit status 1 because a difference is expected. Do not interpret that as
a failed build. A changed `seed.txt` alone is not evidence of a changed landscape.
This experiment establishes repeatability of the current CPU export for the
recorded environment, not a cross-platform contract or complete world generation.

Keep this export outside the runtime asset folder initially. To inspect one in the
editor, copy the **whole selected export**, including both PNG dependencies, into
the isolated project's generated preview directory. Do not replace the live
project's preview or scene. Record which export was installed for that inspection.

### 3. Inspect preview and Play with the bounded diagnostic

Use only `tools/jackdaw_timed_test.py` for graphical diagnostics. Check that its
configured endpoint is unused and that the target project is your worktree.
Before sending remote commands, verify both the reported project path and the
process executable; a reachable port alone is insufficient.

```bash
python3 tools/jackdaw_timed_test.py --seconds 20 --refresh 60 --name worldgen-preview
```

That command inspects the editor; it does not select a new game seed. The existing
`--play`, `--prefabs` and `--review` switches select specific diagnostic presets.
In particular, `--play` currently moves the seed-42 live-listen preset to the
front. Do not claim an arbitrary seed or copied scene was tested merely because
Play connected. The current helper has no general world-config selector; add one
through the implementation proposal before using it for arbitrary-world reviews.
Its custom-command option is documented for watchdog self-tests, not as a way
to bypass compatibility or endpoint checks.

If the adapter needs rebuilding for the selected content, run the existing
`tools/jackdaw.sh build` once before the diagnostic; do not rebuild the editor or
run a workspace suite just because a seed changed. Preserve the independent
20-second kill timer and 60 Hz recovery for every later graphical review.

Compare the actual chosen seed/configuration and authored layout, not camera
position or screenshots alone. Current prefab placements remain authored even
when the terrain seed changes. Pixel hashes of rendered screenshots are not the
world identity and need not be stable across GPUs or lighting times.

### 4. Planned creation/resume/share workflow

These steps require the future implementation; no new CLI flags are implied:

1. Load a validated world configuration; resolve preset values and matching
   generator, layout and rules revisions. Reject malformed or unsupported input.
2. Generate the initial placement data and a preview from the same configuration.
   Validate opening reachability/resource constraints before accepting it.
3. Confirm the configuration, create the world, and store it with the new save.
   Previewing, streaming or loading does not choose another seed.
4. On resume, load the saved configuration and mutable state together. Preserve
   collection/replenishment, constructed streams, growth and rack patches.
5. For a fresh reproduction, share the configuration and its content/version
   references. For an already-developed world, share the save as well.
6. If a matching generator is unavailable, use the chosen migration policy or
   report the incompatibility. Do not silently substitute new rules or seed 42.

### 5. Record the result and diagnose mismatches

Retain the configuration/seed, game and editor revisions, layout/rules content,
commands and exit status, output hashes, selected run preset, and any diagnostic
report/screenshots. For a future placement fixture, compare canonical resource
records and transforms, excluding transient ECS IDs, wall-clock timestamps,
GPU resources and audio playback instants.

| Symptom | First check | Response |
| --- | --- | --- |
| Seed appears to have no effect | `seed.txt`, exact `--seed=...` argument, selected run preset and binary | Current invalid CLI values can fall back to 42; distinguish terrain changes from fixed prefab placements |
| Editor and Play differ | Export/config selected for the editor, runtime seed and saved scene revision | Refresh only the isolated preview from matching inputs; do not reposition authored assets to hide drift |
| Same seed differs between machines/builds | Generator, settings, content and dependency revisions | Compare placement/export data before attributing a difference to graphics |
| Region request order changes placement | RNG ownership, candidate ordering and shared mutable generation state | Reproduce with the same small fixture and fix the order dependency |
| Collected bugs or constructions return after loading | Saved deltas, stable resource keys and calendar state | Treat this as a persistence/identity issue, not a new-seed feature |
| Boundary discontinuity appears after an update | Region ownership and old/new generator versions | Apply the chosen migration/old-generator policy; retain player modifications |

## Implementation outline: a small hybrid generator

This is a proposed work breakdown for a later OpenSpec change. It does not change
the accepted authored-opening approach or authorize implementation in this docs
commit. Start with the smallest useful seeded variation, not a replacement world
engine, new framework or custom RNG.

| Phase | Deliverable and likely touchpoints | Completion evidence |
| --- | --- | --- |
| 0. Agree the contract | OpenSpec proposal linked to `add-opening-region`; choose which resources/scatter vary and which placements stay authored | Example configuration and explicit initial-world versus save-state promise |
| 1. Configuration | Small shared game-side configuration module using existing serde/JSON; common parsing for `crates/stubby_game/src/main.rs`, `jackdaw/src/main.rs` and export | Invalid seeds fail clearly; both entry points resolve the same configuration; copy/import round-trip retains concrete settings |
| 2. Placement data | A pure generation module under `crates/stubby_game/src/`, reusing current terrain functions and a documented RNG/noise library where needed | One focused fixture covers repeated generation, changed seed and reordered region requests |
| 3. Runtime/preview integration | Use existing prefab binding, authored `SceneNodeId`/`PrefabEntityId` support and native model/LOD pipeline; extend `export.rs` and the bounded diagnostic selector | Preview and Play identify the same configuration and placement set |
| 4. Persistence | Initial configuration plus stable-keyed resource/construction deltas, through the game's chosen save boundary | Resume preserves one representative collection and construction case once that save support exists |
| 5. Selection and iteration | In-game seed/configuration selection, preview and sharing; retain existing controls and data-driven resources | One bounded review demonstrates a repeatable new-world choice without altering authored opening assets |

### Data flow and ownership

The proposed flow is:

```text
validated world configuration + versioned authored layout/catalog
    -> deterministic terrain/placement data
    -> opening constraint checks
    -> existing prefab/runtime binding and preview export
    -> mutable game simulation
    -> configuration + saved deltas/snapshots
```

A possible generation boundary is `generate_region(config, region_coord)` returning
placement records or an explicit failure. This is an illustrative boundary, not an
existing API. Prefer a small module in the existing game crate; a separate crate
is not justified until there is an actual sharing requirement. Do not pass ECS
queries, render handles, audio devices or frame `Time` into the generation logic.

Generated records need a resource/prefab identifier, deterministic placement key,
transform and initial gameplay metadata. Keep authored native IDs intact. Define
generated keys and their migration rules in the proposal before connecting them
to inventory/collection saves; never use transient entity indices as save keys.
Keep the bug's ground pivot and hover child arrangement and the existing resource
catalog/yield semantics. Do not invent new waveform sources or machine recipes.

Reuse the current `WorldSeed`, coordinate hash and existing `PulseClock` interfaces
where applicable. If a new random stream is needed, select and pin an established
implementation with documented reproducibility behavior. Resource timing remains
independent per bug, with audio-rate timing handled by the existing audio module.
World generation, fluid transport and saves stay off the realtime audio callback.

World selection is game functionality. Jackdaw remains the shared general editor:
authored prefabs, reflected metadata and its existing extension points are enough.
Do not add a game-specific generator or asset-placement panel to the editor.

### First implementation slice and stop condition

Start by carrying one explicit configuration through both entry points and export,
with a clearly recorded seed and layout revision. Preserve every authored opening
placement. Add one approved seeded scatter/resource-placement pass only after its
kind and opening constraints are chosen. Use the same result for preview and Play.

Stop that slice when a copied configuration recreates its initial placement data
and the opening remains playable. Streamed terrain, extra biomes, compressed world
codes, multiplayer lockstep and exhaustive cross-platform replay are later choices.
Keep validation focused: extend the same placement fixture for new guarantees and
run save or bounded graphical checks only when the changed behavior needs them.

## Decisions still open

- Which parts vary first: terrain, resources, plant variants, or only scatter?
- Is the opening permanently fixed, selected from authored layouts, or generated
  subject to constraints? Current direction is fixed prefab authoring.
- What changes count as a generator revision versus an asset-only visual update?
- How much old-generation support is worth retaining during prototyping?
- How are generated identities kept stable when placement rules change?
- Are spawn location, initial calendar state and difficulty part of a shared setup?
- How much of a preview may reveal discoveries without spoiling exploration by ear?

## Sources and evidence limits

All links accessed 2026-11-01. Developer publications and library documentation
anchor technical guidance. The Satisfactory wiki supplies the map-description
reference; the third-party tool is explicitly labelled as its author's replica.
No game installation was run or reverse-engineered for this dossier. Do not treat
version-sensitive source claims as permanent engine contracts.

- [Factorio map generator][factorio-map] — settings, preview, exchange configuration and demand-driven generation.
- [Factorio MapGenSettings API][factorio-api] — typed seed and generation settings; inspected page version 2.1.24.
- [Factorio command-line reference][factorio-cli] — explicit seed/settings inputs and map preview export.
- [Wube, Friday Facts #390][factorio-noise] — coordinate expressions, coherent noise and shaped starting areas.
- [Wube, Friday Facts #112][factorio-chunks] — chunk-wise reuse of generation calculations.
- [Coffee Stain's March 2026 announcement][satisfactory-march] — 1.2 seeds, randomization and settings matching.
- [Developer announcement index][satisfactory-news] — June 2026 public release chronology.
- [Satisfactory world reference][satisfactory-world] — community-maintained official-wiki description of the authored map.
- [Satisfactory Tools generator wrapper][satisfactory-tool] — independently documented replica, not an official game API.
- [Rust Rand reproducibility chapter][rand-repro] — output stability, versions and portability limits.
- [rand_chacha 0.10 documentation][chacha] — named portable PRNG family; not a dependency added here.

[factorio-map]: https://wiki.factorio.com/Map_generator
[factorio-api]: https://lua-api.factorio.com/latest/concepts/MapGenSettings.html
[factorio-cli]: https://wiki.factorio.com/Command_line_parameters
[factorio-noise]: https://factorio.com/blog/post/fff-390
[factorio-chunks]: https://factorio.com/blog/post/fff-112
[satisfactory-march]: https://store.steampowered.com/news/posts/?appgroupname=Satisfactory&appids=526870&enddate=1774365242&feed=steam_community_announcements
[satisfactory-news]: https://store.steampowered.com/oldnews/?appgroupname=Satisfactory&appids=526870&feed=steam_community_announcements&headlines=1
[satisfactory-world]: https://satisfactory.wiki.gg/wiki/World
[satisfactory-tool]: https://github.com/SatisfactoryTools/world-data-generator-wrapper
[rand-repro]: https://rust-random.github.io/book/crate-reprod.html
[chacha]: https://docs.rs/rand_chacha/0.10.0/rand_chacha/
