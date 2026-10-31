## Why

The first minutes set the tone: a silent, hostile, beautiful place where sound has to
be found. This change delivers the playable opening — a procedural mudflat in a
windstorm, the square wave as the first discovery, the clock as the second — and the
asset pipeline every later visual depends on.

## What Changes

- Seeded procedural generation of a cracked-mud desert flat, streamed in chunks.
- A silent windstorm: GPU blowing sand, volumetric fog, and gusts that vary visibility.
- A global mood value that moves the visuals from oppressive toward cheerful as the
  player progresses.
- Discovery flow: hear and find the Square VCO (the first sound in the game, audible only within range), then the Clock, which
  appears as a pulsating light synced to the audio clock.
- A Blender → glTF export pipeline for props and modules.

## Capabilities

### New Capabilities
- `world-generation`: seeded, chunked, deterministic terrain and placement of discoverables.
- `atmosphere`: sandstorm, fog, lighting, and the mood progression that drives them.
- `discovery-progression`: the ordered discovery of the first modules and its effects.
- `asset-pipeline`: Blender source files, headless glTF export, and runtime loading.

### Modified Capabilities
<!-- none -->

## Impact

- `crates/stubby_game`: new world, atmosphere, and discovery plugins.
- Depends on `add-audio-engine` for Square VCO, Clock, and audio-time sync.
- Dependencies: `noise`, `rand_chacha`, `bevy_hanabi` (check compatibility with Bevy 0.20).
- New dirs: `assets/`, `assets-src/blender/`, `tools/blender/`.
