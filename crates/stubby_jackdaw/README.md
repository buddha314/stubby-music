# Desert opening in Jackdaw

Use the shared source checkout `/home/buddha/src/jackdaw`. The repo root is the game
project (`jackdaw.toml`), and this crate is what Jackdaw builds; neither is an editor fork. Both editor and CLI come from that shared checkout.
The system-wide `~/.cargo/bin/jd` and `~/.cargo/bin/jackdaw` are soft links to that shared build.

The shared checkout uses `https://github.com/buddha314/jackdaw.git` as development
`origin`, with `https://github.com/jbuehler23/jackdaw.git` retained as `upstream`.
Publish and use fixes from our fork without waiting for upstream PRs to merge.

From the repository root:

```sh
tools/jackdaw.sh prepare   # regenerate floor preview; build the matching shared CLI
tools/jackdaw.sh doctor
tools/jackdaw.sh build     # build the game and extract component schema
tools/jackdaw.sh open      # open the shared editor on assets/scene.bsn
```

The game enables AudioNimbus by default. `prepare`, `build`, `run`, and `open`
check/build the pinned Steam Audio SDK and set its runtime library path; unchanged
SDK inputs use the cache, so this does not rebuild the native SDK on every game build.

The shared editor executable is
`/home/buddha/src/jackdaw/target/debug/jackdaw`. **Jackdaw selects our versions**:
`compatibility.json` records its source commit, upstream base, Bevy version and
compiler. **This package is the game**: `cargo run` (the workspace default) and
`tools/run_game.sh` run it with `assets/scene.bsn`, the seeded layout
(`src/seeded_layout.rs`, rules in `assets/world/opening.json`) and the prefab bridge;
`tools/run_game.sh --bare` runs `stubby_game` alone, with no scene. It shares the root
Cargo workspace, Cargo.lock and Rust toolchain, with `JackdawPlugin` and
`maybe_windowless` for Play. Jackdaw's crates are git dependencies on the buddha314 fork,
pinned by `rev`; `tools/jackdaw.sh link` builds against `~/src/jackdaw` instead.

Run `tools/jackdaw.sh check` for read-only source/version validation and
`tools/jackdaw.sh prepare` to rebuild matching shared binaries and the floor preview.
Prepare refuses while that shared editor is running. Managed launch/build verifies
a local source-linked binary receipt; stale or missing receipts require Prepare.
See [the upgrade and fork policy](../../docs/research/jackdaw-version-policy.md).

The edit viewport loads `generated/mudflat.glb` through `GltfSource` (a raw
`SceneRoot` is not registered in this editor). Re-run `prepare` after changing the
floor generator. These files are reproducible and ignored by Git. The preview
uses editor lighting; Play generates the full desert sky, haze and mountains.
The preview floor and light carry `HiddenInGame`, so they do not duplicate the
procedural scene in Play. Other entities authored in `scene.bsn` load in Play.
Moving the preview proxy does not change the procedural generator's parameters.

The shared Jackdaw Play fix excludes process-local light-probe components from
PIE snapshots and projection. Without it, a streamed generated environment map
is rebuilt with a null image handle in the editor, causing storage-format and
texture-usage GPU validation errors. Rebuild the shared editor after changing
its source; no game-specific editor binary is used.

Verified on 2026-10-31: `jd doctor` passes; `jd build` extracts 486 registered
components; the floor loads in the edit viewport; embedded Play connects and
streams the desert view; Stop restores authoring. The shared Jackdaw regression
and all 23 PIE projection tests pass. The game workspace tests and both game
and adapter Clippy checks pass.

The game and Play share one asset folder, the repo root's `assets/`. Play compiles the
shared pulse code and uses the
standalone `stubby_audio` crate through its own public API. The edit viewport's
floor proxy does not animate the creatures; Play does.
Audio: Play runs the game's audio engine too. The run configuration
"Desert opening + audio demo" starts a test patch (the opening's square wave plus an
audible clock); in either configuration F8 toggles it. Click into the Play panel
first, so keys reach the game rather than the editor.

## Placing world assets

Levels are built from **prefabs**, the usual editor workflow: the art pipeline
delivers a model (`.glb`), a prefab wraps it with its gameplay components, and level
design places prefabs.

The prefabs live beside their models under `assets/<category>/<id>/`: `flora/<family>/<variant>/` (scrub oaks, reed clumps),
`props/desert/` (brush, stumps, mud debris), `props/rocks/` (the grotto's boulders and the
rock formations), `creatures/` (the lightning bug, the snail shell), `modules/` (VCOs,
clock lamp), `machines/` (the mudpit burbler, the reed water purifier), `structures/` (the
campfire) and `streams/` (a stream segment, the ripple pool). Drag one from the Project window into the viewport to place an
instance; move, rotate and scale it like any entity. Prefabs are ordinary editor
assets: open one to edit it, and every instance follows.

**Making a prefab for a new asset:** drag its `.glb` from its asset folder into the viewport,
add its game component in the inspector (such as `MineNode`), then right-click it in the hierarchy and choose
**Save as Prefab**, saving beside the model as `<id>.bsn`.

For large surfaces that affect sound paths, also read
[`docs/audio/acoustic-level-authoring.md`](../../docs/audio/acoustic-level-authoring.md).
Add `AcousticOccluder` to the actual `Mesh3d` child in the prefab, select acoustic
absorption/scattering/transmission values, and include the source/listener path audit
in the change review. The prefab root marker alone does not tag its child meshes.

- Each prefab names its model with a `GltfSource` (the model beside it). Prefabs never carry their own levels of detail.

**Levels of detail belong to the model**, as in Unity, Unreal and Godot. Select the
model in the Project window and open its import settings:
- **Import LOD levels** reads the `<id>_LOD0`, `<id>_LOD1` nodes the art pipeline puts
  inside a model (the scrub oaks and reed clumps).
- **Generate** gives a model without hand-made levels simplified ones from its own
  meshes.
- **Apply** writes them to `<model>.glb.meta`; commit that file.
- **All models at once:** the editor does this per model, so with the project open run
  `python3 tools/jackdaw_lods.py` (from the repository root). It calls the same
  operators through Jackdaw's remote control (`jd mcp`): import for models with
  `_LOD` nodes, generate for the heavy models it lists, then apply. Every placement of the
  model then switches level by screen size on its own; `LodOverride` adjusts one
  placement if it must.
- Models ship in their starting pose, set by node scale so they look right before any
  game code runs: oaks dormant, reed clumps standing, props bleak, streams muddy.
- **Things that hover have a ground pivot.** The lightning bug prefab's root sits on
  the ground, so dropping it snaps to the floor, and the bug itself is the child
  `Lightning bug hover`, carrying the model and `ResourceNode`. Each flash appears at a
  fresh seeded height within `hover_metres` (1.3 to 1.7 m, `asset.json`), so the child's
  authored height does not set it.
- Scrub oaks carry `MineNode`. Give every placed oak its own `instance` (the prefab's
  `scrub_oak-<k>` is only a default), since the harvest ledger keys on it.
- Play binds supported models by file stem. Placed bugs get their own seeded clock,
  flash, shadow, light and existing square/envelope/VCA voice; the authored level
  disables the legacy fixed bug scatter. Bindings follow the resource child and its
  world transform, including the bug's ground-pivot parent.
- Bugs and oaks need unique authored `instance` IDs; owned clocks also need unique
  IDs. Duplicates are reported and quarantined until corrected. Other placements
  derive deterministic identities from the authored names and transforms.
- Reeds become finite `reed_clump` resources: E yields six reeds, changes the visual
  to stubble, and the clump returns after 60 game seconds. The burbler is an infinite
  clock mine with a seeded BPM, never an E pickup. E opens its rack; its streams carry
  its voice and water oaks, which grow toward the cap their water's clarity allows.
- Any leaf socket whose glTF extras name a `resource` from the rules is a pickup of it
  (`stubby_game/src/collectibles.rs`): rock formation openings, ripple pools.
- `AssetState` exposes growth, water/light quality and stream-port highlighting for
  visual bindings. Burble clips follow audio sample phase. Flora sway uses game time.
  Each instance has its own material state. Reload/removal retires helpers and audio.
  Saw and triangle modules retain their existing visual-only status until their DSP
  modules land.
- Game animation metadata is attached to Jackdaw's existing render parts. Bevy and
  Jackdaw own the meshes, LOD generation and runtime switching; the game does not
  rebuild scenes or supply another LOD implementation.

`cargo test -p stubby_jackdaw --test prefabs` loads every prefab the way the editor
does and fails on an unknown type, a missing model or a missing LOD level. Run it
after adding or editing prefabs.

## Live listen

Choose **Live listen: clock + square** in the editor's run configurations, then
Play. It places a clock and square VCO near spawn and patches that exact clock
into the VCO's envelope/VCA, so the lamp and audible low square bursts share
one source of timing. The normal Desert opening configuration remains silent
until discovery. Leave F8's independent demo off during this check.

Standalone equivalent: `tools/run_game.sh --seed=42 --module-demo`.
At the default 120 BPM, listen for two bursts per second and watch the clock lamp.
Stop Play to stop its audio; play again to verify it starts cleanly. Audio uses the
game child's default output device, not an editor audio bridge. This check does
not establish the separate 10ms on-screen timing requirement.

## Bounded dusk and replenishment review

Build the adapter, then run:

```sh
python3 tools/jackdaw_timed_test.py --review --seconds 20 --refresh 60 --name replenishment-review
```

The watchdog stops the editor and game within 20 seconds. The opt-in review starts
five game minutes before sunset at 64×, collects one bug, waits for the next night,
collects it again, and exercises slower/faster/reset controls. It writes screenshots
and `events.jsonl` under `/tmp/stubby-replenishment-review/`. Look for a successful
`complete` event and inventory 0 → 2 → 4, with no inventory change on replenishment.
Use the watchdog when testing the review run configuration.

Normal play uses `[`/`]` for slower/faster simulation and `\` for 1×.
Music pitch, clock tempo and flash envelopes retain their audio timing.

## Bounded stream review

Build the adapter, then script laying a stream at the burbler:

```sh
cargo build -p stubby_jackdaw
python3 tools/jackdaw_timed_test.py --streams --seconds 20 --name stream-review
```

It logs to `/tmp/stubby-stream-review/events.jsonl`, including how many bar lines are alive. It
turns bar lines on without patching anything, so it stays silent and does not exercise a patched
voice. Under embedded Play it wrote no screenshots (2026-11-06). For those, run the binary in its
own window, still under the watchdog:

```sh
python3 tools/jackdaw_timed_test.py --seconds 20 --name stream-shots \
  --command target/debug/stubby_jackdaw --seed=42 --stream-review=/tmp/stubby-stream-review
```

## Bounded prefab review

Build the adapter, then run the seven-instance saved-prefab fixture:

```sh
cargo build -p stubby_jackdaw
python3 tools/jackdaw_timed_test.py --prefabs --seconds 20 --refresh 60 --name prefab-runtime
```

The opt-in review collects a flashing bug and reeds, emits hover/use events, plays
sample-timed burble clips, grows flora, replenishes reeds, and moves out to the
model's far level and back. At the burbler it opens the rack and plays it through with real
pointer clicks (bevy_picking input, so UI and mesh picking are exercised): inserts the
Square VCO, patches GATE to TRIG and OUT to IN, and checks `MachineConfigured`. Each
rack step is a `rack_step` event; `rack-patched.png` shows the finished patch.

**This is a development review, not a build or test step.** It runs only when the game
gets `--prefab-review=…`, which only `tools/jackdaw_timed_test.py --prefabs` passes;
`cargo test`, normal builds and normal Play never run it. Run it by hand when changing
prefab binding, interaction or the rack. Evidence is written to `/tmp/stubby-prefab-review/`:
`events.jsonl` must contain `complete` with `success: true` (it now also requires `rack_configured`). Screenshots show the
starting pose and the settled LOD return. This fixture is not the production region
layout. The watchdog ends the editor and game after at most 20 seconds.

Verified 2026-11-01: the complete review passed in 9.36 seconds of game lifetime,
with seven bound assets, four registered sources, two square-wave items, six reeds,
no binding errors, a visible flash, running audio-phase animations, and a far/near
return. The independent watchdog stopped the session at 20.035 seconds. This does
not establish indefinite display stability or the separate 10 ms visual timing bar.

## Physics integration status

Avian runs in the game (our `bevy-main` fork): E's reach is a spatial query, and boulders
and rock formations get static triangle-mesh colliders from their near level, marked
`stubby_core::Solid`, which the player slides along (`src/model_part_bindings.rs`). The
editor's Physics module (`[modules] physics = true`) is still off: task
`add-desert-opening` 4.2d / `add-prefab-colliders` remains open;
[the compatibility check and resume steps](../../docs/research/player-controller-and-physics.md#jackdaw-development-compatibility-check-2026-11-01)
record the earlier CLI rejection. The collider-shape JSONs are ready for authoring.

## Machine configuration interaction

When E's 3 m reach (an avian spatial query, `stubby_player/src/interaction.rs`) finds a
machine's Interact point, the game shows its prompt; E opens the rack from the machine's
configure profile (a burning campfire opens its crafting screen instead, with the rack a
tab away). Modules mount from the inventory, knobs turn, and cables patch as in VCV
Rack. Escape or E closes it and restores mouse look.

Checked with one focused interaction regression and one 20-second bounded review.
The review confirmed the actual rack assets loaded and the screen opened/closed.

Known opening-scene limitation (2026-11-18): the grotto's burbler can currently be
configured from only one side. Approach it from the open ground at the grotto mouth;
the other-side interaction reach remains a deferred level fix in
[`add-echo-grotto`, task 3.3](../../openspec/changes/add-echo-grotto/tasks.md).

## Reviews and diagnostic flags

Graphical reviews run under the 20-second watchdog, `tools/jackdaw_timed_test.py`
(`--standalone` runs the built game without the editor; `--command <cmd> …` runs any
command under the same watchdog). Flags the game itself takes:

| Flag | What it does |
| --- | --- |
| `--seed=<n>`, `--world-config=<file>`, `--scene=<file.bsn>` | the world: its seed, its rules (default `assets/world/opening.json`), its scene |
| `--no-seeded-layout` | only the authored scene, no seeded layout |
| `--layout-review=<dir>` | overhead and reed-bed screenshots of the seeded layout |
| `--screenshot=<png>` | one screenshot after 15 s of warmup |
| `--prefab-review=<dir>` | the saved-prefab fixture (`jackdaw_timed_test.py --prefabs`) |
| `--lod-review=<dir>`, `--lod-review-set=flora\|machines` | levels of detail (`--lods`) |
| `--stream-review=<dir>` | laying, affording and dismantling a stream (`--streams`) |
| `--sky-review=<dir>` | the sky through a day, and the scanner at night (`--sky`) |
| `--purifier-review` | an unpatched purifier on a running stream (`--purifier-review`, with `--standalone`) |
| `--purifier-e2e=<dir>` | the purifier loop played by itself: tempo, craft, build, patch, tune, clear, pools (`--purifier-e2e`) |
| `--grotto-audio-review=<dir>` | listening inside, behind and outside the grotto |
| `--replenishment-review=<dir>` | dusk and reed replenishment (`--review`) |
| `--audio-demo` | a sustained test tone beside the clock blips |

