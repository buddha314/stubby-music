## Why

The game's core loop is building a modular synth patch, so sound is the product, not
decoration. The engine has to accept runtime patch edits from gameplay without ever
glitching, and stay independent of Bevy's frame loop. Every later feature (modules,
factory timing, the clock light) depends on this, so it comes first.

## What Changes

- New workspace crate `crates/stubby_audio` with no `bevy` dependency. It owns the
  output device via `cpal` and runs a runtime-editable DSP graph on the realtime
  audio thread.
- A lock-free command/event protocol between game and audio thread (`rtrb` rings,
  `triple_buffer` snapshots).
- A sample-accurate audio clock that publishes timestamped events back to the game.
- A Bevy plugin in `crates/stubby_game` that exposes the engine as a `Resource` and
  turns audio events into Bevy messages.
- The first two modules: **Sine VCO** and **Clock**.
- A time-boxed spike comparing `fundsp::Net` and `firewheel` behind the same boundary.

## Capabilities

### New Capabilities
- `audio-engine`: realtime audio thread, the game↔audio message boundary, graph
  editing, the audio clock as timing source of truth, and realtime-safety guarantees.
- `synth-modules`: the module/port model, Eurorack signal conventions, and the first
  modules (Sine VCO, Clock).

### Modified Capabilities
<!-- none: no existing specs -->

## Impact

- New crates: `stubby_audio`, `stubby_game` (workspace root `Cargo.toml`).
- Dependencies: `cpal`, `fundsp` (and `firewheel` during the spike), `rtrb`,
  `triple_buffer`, `assert_no_alloc` (dev/debug).
- Linux: needs ALSA dev headers (`alsa-lib-devel` on Fedora) to build `cpal`.
- Research basis: `docs/research/audio-engine.md`.
