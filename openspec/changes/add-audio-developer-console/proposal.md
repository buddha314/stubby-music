# Add an audio developer console

## Why

Spatial audio currently has a fixed reflection wet mix, so developers cannot quickly
compare dry and wet playback while auditing authored spaces. The repository already
ships a local HTML rules designer and dev builds expose Bevy Remote Protocol (BRP),
which can provide a small, low-maintenance tuning panel without adding a UI framework.

## What changes

- Add a developer-only spatial-audio panel to the existing local rules designer.
- Read and update the running dev game's reflection wet mix over Bevy Remote Protocol.
- Apply changes live through the existing audio command boundary; keep the audio
  callback lock-free and allocation-free.
- Keep the control session-only and make its scope clear: it tunes the per-emitter
  reflection path, not the currently unrouted listener-centric reverb output.

## Out of scope

- Player-facing audio settings or an in-game settings screen.
- Persistent audio preferences or shipping developer controls in release builds.
- Rebuilding AudioNimbus simulation settings such as ray count or maximum duration at
  runtime. These remain fixed and can be considered for a later restart-based workflow.
- Claiming the listener-centric environmental reverb bus is implemented.

## Impact

- `crates/stubby_rules_server/web/index.html`
- `crates/stubby_game/src/` developer audio resource and synchronization
- `crates/stubby_audio/src/` runtime wet-mix command and spatial processor
- `openspec/specs/` developer-tooling requirement
