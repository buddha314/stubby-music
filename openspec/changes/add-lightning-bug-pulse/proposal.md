## Why
The first reusable creature asset is a lightning bug that is invisible between white flashes. Its pulses need an audio-clock source that other assets can follow.

## What Changes
- Add a reproducible Blender source, white-flash animation and runtime GLB under the existing assets pipeline.
- Add a seeded random-interval clock to stubby_audio, emitting timestamped ClockTick events and a trigger output with a nearby short click composed from existing square VCO, envelope and VCA modules.
- Add a shared pulse bus and lightning-bug follower in the game, with independent random clocks for the initial bugs.
- Make the shared assets available to the isolated Jackdaw game adapter.

## Capabilities
### New Capabilities
- `pulse-clock`: seeded random sample-clock ticks and shared visual delivery.
- `lightning-bug`: invisible resting asset and white flash envelope.

## Impact
Touches stubby_audio, stubby_game, the isolated Jackdaw adapter, assets and Blender tools. No commit or merge is part of this feature request.

The clock abstraction is the public `PulseClock` trait, implemented by both `PeriodicClock` and `RandomClock`. Both audio processors step their clock through that common sample interface.
