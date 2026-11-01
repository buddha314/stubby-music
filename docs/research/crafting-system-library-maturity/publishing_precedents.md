# Publishing and sustaining an engine-agnostic game-systems library (Rust)

Context: these notes support a plan to extract `stubby_recipes` (a crafting, inventory and goals crate with no Bevy dependency) from the stubby-music game repo into its own repository and crates.io package. Research date: 2026-11-01. Bevy 0.20.0 was published 2026-10-08.

Most version and date facts below come from the crates.io JSON API (`https://crates.io/api/v1/crates/<name>` and `/<name>/<ver>/dependencies`), queried 2026-11-01. They are cited as "crates.io API" with the crate's crates.io page as the link.

## Precedents: core/adapter splits, and how they version against Bevy

### Takeaway
The libraries that last keep a pure core with a stable **data format** (ink JSON, compiled Yarn, LDtk JSON) or a stable algorithmic API (pathfinding, parry/rapier, hecs), and put engine glue in thin `bevy_foo` crates. Those adapter crates take a semver-breaking 0.x bump for every Bevy release and publish a compatibility table. Projects whose single crate depends on Bevy (big-brain) stall when the maintainer stops upgrading. A core that lets an optional `bevy` feature leak into it (yarnspinner_core) ends up pinned to Bevy versions anyway.

### Cited Findings

**Bevy release cadence (sets the adapter treadmill)**
- Bevy major release dates: 0.12 2023-11-04, 0.13 2024-02-17, 0.14 2024-07-04, 0.15 2024-11-29, 0.16 2025-04-24, 0.17 2025-09-30, 0.18 2026-01-13, 0.19 2026-06-19, 0.20 2026-10-08. That is roughly one breaking release every 3–5 months. — [crates.io API: bevy](https://crates.io/crates/bevy)

**ink (inkle)**
- ink is MIT-licensed, with a C# core containing the compiler and runtime. The `inklecate` CLI compiles to a **JSON story format**, which the runtime loads (`Continue()`, `currentChoices`, `ChooseChoiceIndex()`). The engine keeps internal story-format and save-format versions. — [inkle/ink README](https://github.com/inkle/ink)
- Ports and integrations: **inkjs** (a community JavaScript port, "usually one major version behind"), **ink-unity-integration** (bundles the engine source and auto-compiles `.ink`), and the **Inky** editor. Matching release versions of ink, Inky and the Unity integration are meant to work together. — [inkle/ink README](https://github.com/inkle/ink)
- The runtime is meant to slot into the game's own code and UI rather than act as a game engine. The repo shows about 5.0k stars and 549 forks, plus an Epic MegaGrant. — [inkle/ink README](https://github.com/inkle/ink)
- Rust port: `bladeink` (blade-ink-rs, Apache-2.0) runs compiled `.ink.json` and claims full compatibility with the reference version, backed by a `conformance-tests/` suite. It is split into `bladeink` (runtime; `std` / `no_std + alloc` features, pluggable JSON backends, binary `.inkb` format), `bladeink-compiler`, the `rinklecate` CLI, and C bindings in a separate `blade-ink-ffi` repo. It has no Bevy integration. — [blade-ink-rs README](https://github.com/bladecoder/blade-ink-rs)
- `bladeink` 2.1.0 was updated 2026-10-17 (created 2023-10-10, ~21k downloads). — [crates.io API: bladeink](https://crates.io/crates/bladeink)

**Yarn Spinner and yarnspinner-rust**
- The Rust workspace publishes `yarnspinner_core`, `yarnspinner_compiler`, `yarnspinner_runtime`, the facade `yarnspinner`, and the adapter `bevy_yarnspinner`, all at 0.9.0 (2026-08-07), MIT OR Apache-2.0. The split crates were created 2024-01-30; the `yarnspinner` name dates from 2020. — [crates.io API: yarnspinner](https://crates.io/crates/yarnspinner), [bevy_yarnspinner](https://crates.io/crates/bevy_yarnspinner)
- **Leakage:** `yarnspinner_core` 0.9.0 has an optional `bevy ^0.19` dependency (feature `bevy`) next to optional `serde`. The facade `yarnspinner` also has optional `bevy ^0.19`. `bevy_yarnspinner` enables `yarnspinner` with features `bevy,serde`. The "core" therefore still names a specific Bevy version. — [crates.io API: yarnspinner_core 0.9.0 deps](https://crates.io/crates/yarnspinner_core)
- README compatibility table: Bevy 0.12→0.1, 0.13→0.2, 0.14→0.3, 0.15→0.4, 0.16→0.5, 0.17→0.6, 0.18→0.7 and 0.8, 0.19→0.9. A third column gives the upstream Yarn Spinner language version (2.3.0, then 2.5.0 from 0.8). Every Bevy bump is one 0.x minor of the whole workspace. — [YarnSpinner-Rust README](https://github.com/YarnSpinnerTool/YarnSpinner-Rust)
- As of 2026-11-01 there is no Bevy 0.20 release, more than three weeks after Bevy 0.20 shipped (latest is 0.9.0 against Bevy 0.19). — [crates.io API: bevy_yarnspinner](https://crates.io/crates/bevy_yarnspinner)
- The README says the project "offers first class support for Bevy and assumes you are using it". It ships `examples/yarnspinner_without_bevy` and a book section on working without Bevy, and calls itself a work in progress with no official support. — [YarnSpinner-Rust README](https://github.com/YarnSpinnerTool/YarnSpinner-Rust)
- History: it was announced on r/rust_gamedev in January 2024, previously named "Yarn Slinger", and Secret Lab allowed it to use the Yarn Spinner trademark. The official docs list it as a "Yarn Labs" project, "not fully or officially supported". — [Reddit announcement mirror](https://red.127.pp.ua/r/rust_gamedev/duplicates/1aed99e); [Yarn Spinner docs PDF](https://docs.yarnspinner.dev/next/~gitbook/pdf)

**Dimforge: parry / rapier / bevy_rapier**
- `rapier3d` 0.36.1 (2026-10-10, MSRV 1.86, Apache-2.0, ~2.0M downloads) depends on `parry3d ^0.31.1`. Its features include `serde-serialize`, `parallel`, `enhanced-determinism`, `simd8`, `std` and `alloc`. — [crates.io API: rapier3d](https://crates.io/crates/rapier3d)
- `parry3d` 0.31.1 (2026-09-18, ~3.1M downloads, more than rapier itself) is the geometry and collision layer, reused outside rapier. — [crates.io API: parry3d](https://crates.io/crates/parry3d)
- `bevy_rapier3d` 0.37.0 depends on `bevy ^0.20` (default-features off, `std` only) and `rapier3d ^0.36.1`, and was released 2026-10-10, **two days after Bevy 0.20**. Adapter and core are versioned independently: the adapter is 0.37 while the core is 0.36. — [crates.io API: bevy_rapier3d](https://crates.io/crates/bevy_rapier3d)
- The Bevy plugin now sits in the rapier repo under `bindings/bevy_rapier/` (`bevy_rapier2d`, `bevy_rapier3d`), next to C/C++, JS/TS and Python bindings. The core is engine-free and every engine is a "binding". — [dimforge/rapier repo](https://github.com/dimforge/rapier)
- Compare Avian (`avian3d` 0.8.0, 2026-10-20, MIT OR Apache-2.0), which is Bevy-native rather than a core plus adapter. — [crates.io API: avian3d](https://crates.io/crates/avian3d)

**Behavior trees and utility AI**
- `bonsai-bt` 0.14.0 (2026-09-22, MIT, MSRV 1.80, ~115k downloads) is engine-agnostic with zero required deps. `serde`, `petgraph` and `tungstenite` are optional behind a `visualize` feature, and there is an `f32` feature. — [crates.io API: bonsai-bt](https://crates.io/crates/bonsai-bt)
- bonsai calls itself a "Rust implementation of behavior trees for deterministic AI (now with Python bindings)". Topic tags list bevy, piston, unreal-engine, ros and ros2, but the README has no engine-specific integration. It has about 1.1k stars. — [Sollimann/bonsai](https://github.com/Sollimann/bonsai)
- `big-brain` (utility AI, a single crate depending directly on `bevy`). The last crates.io release is 0.22.0 on 2024-11-30, depending on `bevy ^0.15`. The GitHub repo was **archived 2025-10-07**, development moved to Codeberg, and the README states compatibility with Bevy 0.16.0. Recent downloads are ~1.7k against ~65k total. — [zkat/big-brain](https://github.com/zkat/big-brain); [crates.io API: big-brain](https://crates.io/crates/big-brain)
- `bevy_behave` 0.5.0 (2026-02-06, created 2025-02-04) is Bevy-native, with no release in the 8+ months since. — [crates.io API: bevy_behave](https://crates.io/crates/bevy_behave)
- BehaviorTree.CPP and Articy runtime: not researched in this pass (see Gaps).

**pathfinding, hecs, legion**
- `pathfinding` 4.16.0 (2026-09-07, Apache-2.0/MIT, MSRV 1.88) has 134 versions since 2016-12-27 and ~3.2M downloads. It is generic over node and cost types (closures for successors and heuristics), has no engine deps, and carries a `deprecate-until` dependency to manage API deprecations. — [crates.io API: pathfinding](https://crates.io/crates/pathfinding)
- `hecs` 0.11.2 (2026-10-04, MIT OR Apache-2.0, MSRV 1.81, ~656k downloads) describes itself as "a library, not a framework": no System abstraction, "organize your application however you like". It explicitly says it is not suited to every game and should not be the only data structure. — [Ralith/hecs](https://github.com/Ralith/hecs); [crates.io API: hecs](https://crates.io/crates/hecs)
- `legion` 0.4.0 was last updated 2021-02-25 and is effectively abandoned (Amethyst wound down). — [crates.io API: legion](https://crates.io/crates/legion)

**LDtk: data format with an adapter versus standalone parsers**
- `bevy_ecs_ldtk` 0.16.0 (2026-10-25, MIT OR Apache-2.0) depends on `bevy ^0.20` and `bevy_ecs_tilemap ^0.20`, plus `serde`/`serde_json`. It ships its own LDtk JSON types rather than depending on a separate parser crate. — [crates.io API: bevy_ecs_ldtk](https://crates.io/crates/bevy_ecs_ldtk)
- The standalone parsers went stale: `ldtk_rust` 0.6.0 last updated 2022-05-09, `ldtk2` 0.8.0 on 2024-03-11 (182 recent downloads). The adapter kept the users and the pure parsers did not. — [crates.io API: ldtk_rust](https://crates.io/crates/ldtk_rust), [ldtk2](https://crates.io/crates/ldtk2)
- LDtk publishes a JSON format reference "directly generated from latest source code", a JSON schema file, and a `jsonVersion` field in each project file, with a documented deprecation cycle. — [LDtk JSON overview](https://ldtk.io/docs/game-dev/json-overview/)

**Bevy adapter release latency (other examples, all after Bevy 0.20 on 2026-10-08)**
- `bevy_rapier3d` 0.37.0 2026-10-10; `bevy_egui` 0.43.1 2026-10-10; `bevy_asset_loader` 0.28.1 2026-10-13; `leafwing-input-manager` 0.22.0 2026-10-15; `avian3d` 0.8.0 2026-10-20; `bevy_ecs_tilemap` 0.20.0 2026-10-21; `bevy_ecs_ldtk` 0.16.0 2026-10-25; `bevy-tnua` 0.33.1 2026-10-30. — [crates.io API](https://crates.io/) (individual crate pages)
- `bevy-tnua` uses a backend-adapter split: `bevy-tnua` plus `bevy-tnua-avian3d` and similar per-physics-engine crates (this repo pins 0.33.1 / 0.13.1). — [crates.io API: bevy-tnua](https://crates.io/crates/bevy-tnua); stubby-music AGENTS.md stack table

### Inferences
- The durable pattern is **core crate (no engine) + `bevy_<name>` adapter in the same repo/workspace**, with the adapter versioned independently (rapier 0.36 / bevy_rapier 0.37). The core then does not take a breaking bump every time Bevy does.
- Avoid Yarn's approach of an optional `bevy` feature *in the core* (used for `Reflect` derives). It ties the core's release to Bevy's cadence. Put `Reflect`/`Component` impls in the adapter, using newtype wrappers or `#[reflect(remote = ...)]`-style remote reflection, or skip Reflect for core data.
- The data format is what users adopt, more than the API: ink JSON, compiled Yarn and LDtk JSON outlive individual runtimes and spawn ports. For a crafting crate, the equivalent is a documented, versioned recipe/item catalog format (with a JSON Schema and a `format_version` field).
- Pure crates whose only consumers are Bevy users can lose to an adapter that inlines its own parser (LDtk). A core needs at least one non-Bevy consumer or example, or it effectively becomes part of the adapter.
- Expect adapter releases 2 days to 4 weeks after each Bevy release, and budget for 3–4 such releases per year.
- No existing Rust crate dominates the crafting/inventory niche. crates.io searches (2026-11-01) found only tiny or new crates: `game_inventory` 2.1.0 (~10k downloads, 2024-10-29), `inv-sys` (2021), `crafting-helper` (2019), `bevy_inventory` 0.1.0 (201 downloads, "pure Rust core with optional Bevy ECS"), `game-utils-inventory` 0.4.3 (81 downloads). The niche is open, but there is no precedent proving demand. — [crates.io search API](https://crates.io/search?q=game%20inventory)

### Gaps
- BehaviorTree.CPP (Groot editor, XML format) and Articy:draft's runtime/exporter: not fetched in this pass. From prior knowledge only, both follow "editor/data format + per-engine runtime", but this is unverified here.
- I could not find an exact date for when bevy_rapier moved from its own `dimforge/bevy_rapier` repo into `rapier/bindings/`. The current location was verified; the history was not.
- I could not fetch the yarnspinner-rust book's "working without Bevy" chapter (docs URLs returned 404).

## Rust publishing practice

### Takeaway
The norms are: a Cargo workspace with core plus adapters, `MIT OR Apache-2.0`, additive `dep:`-style optional features (`serde`, `schemars`), a declared `rust-version`, 0.x semver where the minor is breaking, full Cargo metadata so docs.rs and crates.io render well, runnable examples, and announcement through Bevy Assets, Discord `#crates` and Reddit.

### Cited Findings
- **License:** Bevy is MIT OR Apache-2.0, and its plugin guide recommends plugins use the same dual license "for compatibility and simpler license compliance". — [Bevy plugin development guide](https://bevy.org/learn/quick-start/plugin-development/)
- Observed licenses: yarnspinner, hecs, avian, bevy-tnua, leafwing, bevy_asset_loader and bevy_ecs_ldtk are MIT OR Apache-2.0. Dimforge (rapier, parry, bevy_rapier) and big-brain are Apache-2.0 only. bonsai-bt, bevy_egui and moonshine-save are MIT only. — [crates.io API](https://crates.io/)
- **Naming:** Bevy says to use `bevy_xxx` "reasonably" and to ask before claiming generic names (`bevy_animation`, `bevy_editor`). — [Bevy plugin guide](https://bevy.org/learn/quick-start/plugin-development/)
- **Compatibility table:** document plugin↔Bevy versions, for example with a README table. Main-branch tracking pins a git commit, and badges can show released versus main. — [Bevy plugin guide](https://bevy.org/learn/quick-start/plugin-development/)
- **Bevy dependency hygiene:** features are additive, so set `default-features = false` on `bevy` and enable only what is needed. Avoid large dependencies, check duplicates with `cargo tree`/`cargo-deny`, and put optional parts behind features. Document SystemSets and their ordering plus components, and include screenshots/GIFs. The guide also covers `no_std`, tests/CI, and filling in Cargo.toml metadata. — [Bevy plugin guide](https://bevy.org/learn/quick-start/plugin-development/)
- Examples in practice: `bevy_rapier3d` 0.37 uses `bevy` with `default-features=false, features=["std"]`, and `bevy_yarnspinner` uses `bevy_asset,multi_threaded,bevy_log` only. — [crates.io API deps](https://crates.io/crates/bevy_rapier3d)
- **Semver 0.x:** in `0.y.z`, a change to `y` is major and a change to `z` is minor. `0.0.z` makes every change major. — [Cargo SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- Breaking (major) changes include renaming, moving or removing public items, adding an enum variant without `#[non_exhaustive]`, adding a public field to an all-public struct, adding a non-defaulted trait item, and **removing a Cargo feature**. Adding a feature is minor. Use `dep:` syntax so an optional dependency does not leak an implicit feature that would be breaking to remove. — [Cargo SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- **MSRV:** raising it is "possibly-breaking", and Cargo recommends treating it as minor. Set `package.rust-version` so the resolver can pick older compatible versions. Mitigations include supporting a window of older Rust releases. — [Cargo SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- Observed MSRVs: rapier3d 1.86, pathfinding 1.88, hecs 1.81, bonsai 1.80, egui/bevy_egui 1.95, bevy_asset_loader 1.96. Many Bevy crates declare none. big-brain's README says its MSRV is Bevy's ("latest stable"). — [crates.io API](https://crates.io/); [big-brain README](https://github.com/zkat/big-brain)
- **Rust API Guidelines checklist items:** C-SERDE (data structures implement Serialize/Deserialize), C-FEATURE (no placeholder words in feature names, e.g. `serde`, not `use-serde`/`with-serde`), C-CRATE-DOC (thorough crate docs with examples), C-EXAMPLE (all items have a rustdoc example), C-METADATA (Cargo.toml common metadata), C-SEALED (sealed traits), C-STRUCT-PRIVATE (private fields for future-proofing), C-STABLE (public deps of a stable crate are stable), C-PERMISSIVE. — [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html)
- **Bevy Assets submission:** open a PR to `bevyengine/bevy-assets` with a TOML file containing `name`, `description` (<100 chars, no formatting), `link`, and optional `image` (PNG/JPG/GIF/WebP, <2 MB, 16:9, ≤600 px wide) and `crate`. It goes in the Assets, Apps or Learning folder. There is no required Bevy-version field. — [bevyengine/bevy-assets](https://github.com/bevyengine/bevy-assets)
- **Promotion:** submit to Bevy Assets and announce on Discord `#crates` and Reddit. — [Bevy plugin guide](https://bevy.org/learn/quick-start/plugin-development/)

### Inferences
- A suggested layout for the extracted repo: `crates/<core>` (no engine deps; optional `serde`, `schemars`), `crates/bevy_<core>` (depends on core plus `bevy = { version = "0.20", default-features = false }`), `examples/` including at least one non-Bevy CLI example, and `schema/` holding a generated JSON Schema checked in CI.
- Mark public enums and structs (item kinds, recipe outcomes, errors) `#[non_exhaustive]` and keep fields private behind constructors from day one, so that 0.x minors are not consumed by trivial additions.
- Follow C-FEATURE: name features `serde` and `schemars`, not `with-serde`, and declare them with `dep:`.
- Announcing in This Week in Rust (via a PR to its repo's crate/blog sections) and on r/rust_gamedev is customary, but this pass found no primary source confirming the procedure (see Gaps).

### Gaps
- I did not fetch This Week in Rust submission rules or the r/rust_gamedev norms.
- I did not research CI templates (e.g. `cargo-semver-checks`, `cargo-msrv`, `cargo-deny` actions) from a primary source. `cargo-semver-checks` is a widely used semver lint, but it is unverified here.
- I found no source specifying docs.rs `[package.metadata.docs.rs] all-features = true` practice (common, but uncited).

## Keeping two repos in sync (extracting from a game)

### Takeaway
Extract with `git filter-repo --subdirectory-filter` to keep history. Have the game depend on the crates.io version, and use `[patch.crates-io]` pointing at a local checkout (ideally in an uncommitted `.cargo/config.toml`) while co-developing. Cut releases when the game needs them, and keep the game-specific layer in the game repo.

### Cited Findings
- `git filter-repo --subdirectory-filter <dir>` keeps only history touching `<dir>` and makes it the root. It is equivalent to `--path <dir>/ --path-rename <dir>/:`. It needs Git ≥ 2.36 and Python ≥ 3.6 and rewrites in place, so run it on a fresh clone. — [git-filter-repo man page](https://www.mankier.com/1/git-filter-repo); [Mintlify filter-repo docs](https://www.mintlify.com/newren/git-filter-repo); [Jeff Kreeftmeijer, git extract](https://jeffkreeftmeijer.com/git-extract/)
- `[patch.crates-io] foo = { path = "../foo" }` substitutes a local checkout without changing the version requirement. It is only used if its version satisfies the requirement, otherwise run `cargo update foo --precise <ver>`. Switch it to `git = ...` once the change is merged, and delete it when that version is published. — [Cargo book: overriding dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)
- `[patch]` is honoured only in the root workspace manifest, and patches inside dependencies are ignored. It can be put in `.cargo/config.toml` or passed with `--config` so it is not committed. — [Cargo book: overriding dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)
- Precedent of a monorepo of core plus adapters and bindings: Dimforge keeps the Bevy plugin and the JS/Python/C bindings inside the rapier repo, which avoids cross-repo drift between core and adapter. — [dimforge/rapier](https://github.com/dimforge/rapier)
- Precedent of a separate-repo port: inkjs is a community port "usually one major version behind" upstream ink. Separate repos with separate maintainers drift. — [inkle/ink README](https://github.com/inkle/ink)

### Inferences
- Recommended flow for stubby-music:
  1. Clone fresh, run `git filter-repo --subdirectory-filter crates/stubby_recipes`, rename the crate, and push to a new repo.
  2. Publish `0.1.0`.
  3. In the game, replace the path dependency with `<crate> = "0.1"` and keep the game-specific layer (waveforms, `square_wave`, signal-use roles, lightning-bug yields) in a game-side crate, e.g. keep `stubby_recipes` as a thin wrapper over the new crate.
  4. During co-development, put `[patch.crates-io]` in an uncommitted `.cargo/config.toml` or in the workspace root.
- Keep the core crate and the `bevy_` adapter in **one** repo (the rapier pattern). Keeping the game's Bevy glue in the game avoids a third repo.
- To avoid drift, CI in the game repo can build against the library's `main` via a git dependency on a scheduled job, while releases stay pinned to crates.io.
- The multi-agent worktree rules in this repo (owners, `Cargo.lock` conflicts) carry over: a separate repo removes `stubby_recipes` from the `crates/` ownership split, but adds a release step to every cross-cutting change.

### Gaps
- I found no published case study of an indie studio extracting a Rust crate from a game repo, with cadence numbers. Advice here is assembled from tooling docs and precedents' structure.
- I did not compare git subtree or git submodule workflows from a primary source.

## Maintenance burden and adoption signals

### Takeaway
Bevy-dependent crates need a breaking release per Bevy release, 3–4 per year. Crates that miss even one release lose users quickly: big-brain stopped at Bevy 0.15/0.16 and was archived, legion stopped in 2021, and the LDtk standalone parsers went stale. Engine-free cores with broad audiences (pathfinding, parry, hecs, bonsai) keep releasing with far less churn. No quantitative ecosystem-wide staleness statistic was found.

### Cited Findings
- No measured data on the fraction of stale Bevy crates was found. Community reports are anecdotal: the "whole ecosystem becomes broken with a new release and can take weeks to catch up", and some users pin old versions or fork. — [Hacker News thread](https://news.ycombinator.com/item?id=41722787)
- lib.rs crate-health pages flag crates pinned to old Bevy versions because the mismatch causes duplicate Bevy dependencies downstream. — [lib.rs crate health example](https://lib.rs/~Sheepyhead/dash)
- Bevy's upstreaming guide calls a thriving third-party ecosystem essential. It weighs maintenance burden when deciding whether to upstream a crate, and says it should try to help when essential crates' authors need maintenance help. — [Bevy upstreaming guide](https://bevy.org/learn/contribute/project-information/upstreaming/)
- Stale or abandoned examples: big-brain (last release 2024-11-30, archived 2025-10-07), legion (2021-02-25), ldtk_rust (2022-05-09), ldtk2 (2024-03-11). — [crates.io API](https://crates.io/); [big-brain repo](https://github.com/zkat/big-brain)
- Lagging example: yarnspinner-rust has no Bevy 0.20 release 24 days after Bevy 0.20. — [crates.io API: bevy_yarnspinner](https://crates.io/crates/bevy_yarnspinner)
- Healthy engine-free examples, by recent (90-day) downloads: pathfinding ~613k, parry3d ~958k, rapier3d ~569k, hecs ~179k, bonsai-bt ~22k. — [crates.io API](https://crates.io/)
- Adoption features seen in successful precedents:
  - an editor or authoring tool: Inky for ink, Yarn Spinner's editor tooling, LDtk itself
  - a documented, versioned data format: ink JSON with story/save format versions, LDtk with a JSON schema plus `jsonVersion` and deprecation cycles
  - conformance tests that let ports prove compatibility (blade-ink `conformance-tests/`)
  - bindings to other languages (rapier JS/Python/C, bonsai Python)
  - a sample game (ink's The Intercept, Yarn's `demo/`)
  - [inkle/ink](https://github.com/inkle/ink); [LDtk JSON](https://ldtk.io/docs/game-dev/json-overview/); [blade-ink-rs](https://github.com/bladecoder/blade-ink-rs); [dimforge/rapier](https://github.com/dimforge/rapier); [bonsai](https://github.com/Sollimann/bonsai); [YarnSpinner-Rust](https://github.com/YarnSpinnerTool/YarnSpinner-Rust)

### Inferences
- Splitting core from adapter moves the Bevy treadmill onto a small adapter crate. Even if the adapter lags, the core keeps working for any Bevy version (or none), which is what keeps users through a lag.
- For a crafting/inventory library, the highest-leverage adoption assets are:
  1. a JSON/RON recipe-and-item format with a published JSON Schema (via `schemars`), so editors such as VS Code and tools like Jackdaw validate data
  2. a non-Bevy example and a Bevy example
  3. a small demo
  4. a compatibility table
- The maintainer's own game is the main guarantee of upkeep. Plan for releases to follow the game's needs and state that openly in the README to set expectations.

### Gaps
- No ecosystem-wide percentage of stale Bevy crates (I looked; none was found).
- No download data showing whether JSON-schema availability measurably drives adoption.

## Risks: scope creep, API stability, game-specific leakage

### Takeaway
Keep the core domain-neutral (items, stacks, inventories, recipes, recipe-book discovery, goals) with extension points: generic metadata type parameters or opaque extension maps. Waveforms, signal roles and the lightning bug stay in the game. Use `#[non_exhaustive]`, private fields and a narrow scope statement (hecs-style "library, not framework") to resist feature requests and keep 0.x minors meaningful.

### Cited Findings
- hecs sets explicit scope limits (not for every game, not your only data structure, use separate indexes) and is "lightweight and unobtrusive". — [hecs README](https://github.com/Ralith/hecs)
- pathfinding stays generic by taking closures for successors, costs and heuristics, and has no domain types. It has run 134 releases since 2016 with a deprecation helper dependency. — [crates.io API: pathfinding](https://crates.io/crates/pathfinding)
- ink scopes its runtime to slot into the game's own code and UI, not to be a game engine. — [inkle/ink](https://github.com/inkle/ink)
- Semver hazards that make additions breaking: new enum variants (without `#[non_exhaustive]`), new public fields, and new non-defaulted trait items. — [Cargo SemVer](https://doc.rust-lang.org/cargo/reference/semver.html)
- C-STRUCT-PRIVATE, C-SEALED and C-STABLE (public dependencies of a stable crate must be stable) are future-proofing guidelines. A public `bevy` type in the core's API would violate C-STABLE in spirit, because Bevy is 0.x. — [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)
- Leakage precedent: yarnspinner_core's optional `bevy` feature couples the core to Bevy 0.19. — [crates.io API: yarnspinner_core](https://crates.io/crates/yarnspinner_core)
- From this repo's AGENTS.md, which defines what is game-specific: `square_wave` item ID, waveform metadata, VCO/LFO/gate "uses of the same primitive", lightning-bug yields of 2, and replenishment at the night boundary. Data-driven recipe-book entries persist when inventory hits zero. — `/home/buddha/src/stubby-music/AGENTS.md`

### Inferences
- **Generic core design options:**
  - (a) `Item<M = ()>` / `Catalog<M>` generic over game metadata `M: Serialize + DeserializeOwned`, with zero cost and type-safe.
  - (b) An opaque `extensions: BTreeMap<String, serde_json::Value>` (or `toml::Value`/RON) on item and recipe definitions, preserved round-trip, which suits a data format and a JSON Schema with `additionalProperties`.
  - (c) Trait hooks (`RecipeCondition`, `ReplenishPolicy`) for behaviour.
  - Combining (a) or (b) for data with (c) for policy keeps waveforms and signal roles in the game.
- Recipe-book "discovered stays discovered" and replenishment policies (night boundary, timed, none) are generic enough for the core. "Night" should be an abstract calendar event supplied by the game.
- Write a README "Non-goals" section (no UI, no ECS, no networking, no save format beyond serde) to deflect scope creep. Route engine-specific requests to the adapter crate.
- Start at `0.1.0`, not `1.0`. Bevy-ecosystem norms and the Yarn/rapier precedents show long 0.x lifetimes. Use `cargo-semver-checks` in CI (unverified tooling claim; see Gaps above).

### Gaps
- I found no maintainer post-mortem specifically about scope creep in a Rust gamedev library.
- I did not survey how existing inventory crates (e.g. `game_inventory`, "independent of item data") model extension data. Their READMEs were not fetched.
