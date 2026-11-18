# stubby_audio

The realtime audio engine for stubby-music: a modular synth that the game patches
and plays. It has no dependency on the Bevy engine, so it builds, tests and runs on
its own.

## Using it

```rust
use stubby_audio::{Command, Engine, ModuleId, ModuleKind, PortRef};

let mut engine = Engine::start()?; // default output device
let square = ModuleId(1);
engine.send(Command::AddModule { id: square, kind: ModuleKind::SquareVco { freq_hz: 220.0 } })?;
engine.send(Command::Connect { from: PortRef::new(square, "out"), to: PortRef::output() })?;

// Once per game frame:
engine.flush()?;
let mut events = Vec::new();
engine.drain_events(&mut events); // e.g. ClockTick { sample_time, .. }
```

- `send` checks a command and stages it. `flush` publishes the frame's edits to the
  audio thread without blocking.
- Events carry exact audio-clock sample times. `sample_to_instant` turns them into
  wall-clock times for visuals.
- `Engine::offline` renders into a buffer with no device. That's what the tests use.
- `Engine` is not `Send`; it lives on one thread.

## Working on it

```sh
cargo test -p stubby_audio                              # offline tests, under an allocation guard
cargo clippy -p stubby_audio --all-targets -- -D warnings
cargo run --release -p stubby_audio --example play      # listen on the default device
```

On Linux, building needs the ALSA headers (`alsa-lib-devel` on Fedora,
`libasound2-dev` on Debian/Ubuntu).

### AudioNimbus spatial effects (in progress)

The `audionimbus` Cargo feature enables the AudioNimbus/Steam Audio adapter. The
engine crate keeps it optional; `stubby_game` and `stubby_jackdaw` enable spatial audio
by default:

```sh
cargo build -p stubby_audio --features audionimbus   # after tools/build_steam_audio_sdk.sh
```

The native Steam Audio 4.8.1 library must be built or provisioned separately and
available to the runtime loader. The standard game launch handles this setup:

```sh
tools/run_game.sh --seed=42
```

The helper stores the SDK beneath `target/native/steam-audio/`, skips the build when
the pinned source, dependency manifest, build script, toolchain, configuration and
artifact hash are unchanged, and rebuilds when any of those inputs change. Use
`tools/build_steam_audio_sdk.sh --force` to rebuild explicitly. It is not called from
Cargo, so ordinary builds do not pay this build cost. Build scripts (`steam_audio_lib.rs`)
point the linker at that cache and put it in the binaries' run path, so direct Cargo
builds and runs need no `STEAMAUDIO_LIB_DIR` or `LD_LIBRARY_PATH`; setting
`STEAMAUDIO_LIB_DIR` overrides the cache. The source
build uses Steam Audio's minimal configuration
with IPP disabled; do not enable AudioNimbus `auto-install` in reproducible or offline
builds.
`cargo check` validates Rust code but does not link the native library. Windows and
macOS packaging, device-restart handling, spatial voice budgeting, and environmental
scene simulation remain in progress. The node adapter buffers fixed AudioNimbus blocks
and adds one block of latency. Pass `--no-default-features` to a game package for a
temporary build without AudioNimbus.

### OMG Audio review build (experimental)

The pinned `omg_audio` feature and Jackdaw's `omg-audio` feature expose an
experimental static-snapshot route for the existing grotto review driver. This is
not the default spatial backend and does not support smooth moving source/listener
updates. The simulation traces synchronously on the audio control thread when the
review moves to a new camera position; the Firewheel callback only renders the
preinstalled OMG frame.

From the repository root, build the game and then run the existing 20-second
watchdog with this audio route:

```sh
cargo build -p stubby_jackdaw --no-default-features --features omg-audio,dev
STUBBY_OMG_AUDIO_REVIEW=1 python3 tools/jackdaw_timed_test.py --seconds 20 --name omg-grotto --command cargo run -p stubby_jackdaw --no-default-features --features omg-audio,dev -- --grotto-audio-review=/tmp/omg-grotto-review
```

This uses the existing inside / blocked-outside / clear-outside review views. It is
for a bounded listening experiment only. The fixed-frame implementation and isolated
two-source shared-bus spike are documented in `docs/research/spatial-audio-alternatives.md`.

To launch the Jackdaw development game with the spatial audio demo:

```sh
tools/build_steam_audio_sdk.sh
cargo run -p stubby_jackdaw --features dev -- --audio-demo
```

The demo places its square tone left and clock blip right of the listener. Use
headphones to hear the HRTF cues. The Jackdaw package enables AudioNimbus by default;
`--features dev` adds development linking without changing the audio backend.

Rules for code that runs on the audio thread (no allocation, no locks, no panics) are
in the repo's `AGENTS.md`. The crate denies `unwrap`, `expect` and `panic!`.

## Design

- Requirements and decisions: `openspec/changes/add-audio-engine` (engine core) and
  `openspec/changes/add-rack-and-spatial-audio` (the planned VCV-style rack and
  spatial emission).
- Research: `docs/research/audio-engine.md`, `docs/research/rack-and-spatial-audio.md`.

## Why it lives in this repo

It is a crate in the game's workspace, not a separate repository, while the game is
its only user and its API is still changing. Engine and game changes then land
together. It is kept ready to split: it depends on nothing else in the workspace, and
CI builds and tests it on its own. If it gains a second user or gets published, extract
it with history using `git subtree split --prefix crates/stubby_audio`.

`ModuleKind::RandomClock { seed, min_interval_s, max_interval_s }` emits seeded,
sample-accurate `ClockTick` events after an initial resting interval and exposes a
5 ms `gate` trigger. Bounds must be finite and ordered within 0.01–60 seconds.
They are fixed for the module lifetime. Leave its gate unpatched for a silent
visual pulse source; use its event timestamps to synchronize followers.

Both clock sources implement the public `PulseClock` trait. `PeriodicClock`
preserves phase when tempo changes; `RandomClock` owns a seeded interval stream.
The audio processors call `advance()` once per sample and publish events on true.
Each lightning bug has a distinct random-clock instance; followers can select a
clock ID explicitly when they should synchronize.
