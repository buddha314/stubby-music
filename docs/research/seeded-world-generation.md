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
