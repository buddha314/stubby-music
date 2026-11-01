# Desert opening in Jackdaw

Use the shared source checkout `/home/buddha/src/jackdaw`. This folder is a game
project, not an editor fork. Both editor and CLI come from that shared checkout.
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

The shared editor executable is
`/home/buddha/src/jackdaw/target/debug/jackdaw`. **Jackdaw selects our versions**:
`compatibility.json` records its source commit, upstream base, Bevy version and
compiler. The standalone game and this adapter share the root Cargo workspace,
Cargo.lock and Rust toolchain. The adapter package remains for saved-prefab type
paths, with `JackdawPlugin` and `maybe_windowless` for Play. Runtime dependencies
point to the sibling shared Jackdaw checkout, not a private game editor.

Run `tools/jackdaw.sh check` for read-only source/version validation and
`tools/jackdaw.sh prepare` to rebuild matching shared binaries and the floor preview.
Prepare refuses while that shared editor is running. Managed launch/build verifies
a local source-linked binary receipt; stale or missing receipts require Prepare.
See [the upgrade and fork policy](../docs/research/jackdaw-version-policy.md).

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

The lightning bug models are shared through `assets/models`, a symlink into the
main runtime asset folder. Play compiles the shared pulse code and uses the
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

The prefabs live under `assets/prefabs/`: `flora/` (scrub oaks, reed clumps),
`props/` (desert brush, stumps, mud debris), `creatures/` (the lightning bug),
`modules/` (VCOs, clock lamp), `machines/` (the mudpit burbler) and `streams/` (a
stream segment). Drag one from the Project window into the viewport to place an
instance; move, rotate and scale it like any entity. Prefabs are ordinary editor
assets: open one to edit it, and every instance follows.

**Making a prefab for a new asset:** drag its `.glb` from `models/<category>/` into the viewport,
add its game component in the inspector (such as `MineNode`), then right-click it in the hierarchy and choose
**Save as Prefab**, saving under `prefabs/<group>/`.

- Each prefab names its model with a `GltfSource` (under `models/<category>/`, the shared
  runtime models). Prefabs never carry their own levels of detail.

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
  `Lightning bug hover`, 1.8 m up, carrying the model and `ResourceNode`. Select the
  child to change one bug's height.
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
  clock mine with a seeded BPM, never an E pickup. E emits a use event; its full rack
  interface, water transport and oak production are still separate tasks.
- `AssetState` exposes growth, water/light quality and stream-port highlighting for
  visual bindings. Burble clips follow audio sample phase. Flora sway uses game time.
  Each instance has its own material state. Reload/removal retires helpers and audio.
  Saw and triangle modules retain their existing visual-only status until their DSP
  modules land.
- Game animation metadata is attached to Jackdaw's existing render parts. Bevy and
  Jackdaw own the meshes, LOD generation and runtime switching; the game does not
  rebuild scenes or supply another LOD implementation.

`cargo test --test prefabs` (from this folder) loads every prefab the way the editor
does and fails on an unknown type, a missing model or a missing LOD level. Run it
after adding or editing prefabs.

## Live listen

Choose **Live listen: clock + square** in the editor's run configurations, then
Play. It places a clock and square VCO near spawn and patches that exact clock
into the VCO's envelope/VCA, so the lamp and audible low square bursts share
one source of timing. The normal Desert opening configuration remains silent
until discovery. Leave F8's independent demo off during this check.

Standalone equivalent: `cargo run -p stubby_game -- --seed=42 --module-demo`.
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

## Bounded prefab review

Build the adapter, then run the seven-instance saved-prefab fixture:

```sh
cargo +1.99.0 build --bin stubby_jackdaw --manifest-path jackdaw/Cargo.toml
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

The shared development editor currently refuses its Physics module on Bevy main.
Do not enable an unbuildable `[modules] physics = true` entry or add a second Bevy
through Avian's released dependencies. Task `add-desert-opening` 4.2d remains open;
[the compatibility check and resume steps](../docs/research/player-controller-and-physics.md#jackdaw-development-compatibility-check-2026-11-01)
record the exact CLI rejection. The collider-shape JSONs remain ready for authoring
once the shared module supports this SDK.

## Machine configuration interaction

Within 3 m along the ground of a mudpit's authored Interact node, the game shows
E | Configure burbler. E opens the rack HUD from the machine's configure profile;
a closer finite pickup wins. The HUD uses rack_frame.glb, the existing clock
model and labelled empty slot assets. It views the mine's existing clock rather
than creating another audio module. Escape closes it and restores mouse look.
The initial rack view is implemented; mounting, knob editing and cable patching
remain the subsequent rack tasks.

Checked with one focused interaction regression and one 20-second bounded review.
The review confirmed the actual rack assets loaded and the screen opened/closed.
