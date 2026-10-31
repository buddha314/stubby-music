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
