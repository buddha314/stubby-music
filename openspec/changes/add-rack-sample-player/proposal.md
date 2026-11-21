## Why

The audio engine already plays one-shot clips for interface cues. The field
recorder can discover waveform resources. Give players a way to collect clips
and choose which one a rack plays, so found sounds become part of a patch.

## What changes

- Give each sampler module a clip selector and access to the player's shared
  library of collected sounds.
- Let players audition clips in the selector and assign a clip to the current
  module. Each sampler keeps its own selection.
- Give the sampler a trigger input and an audio output so clocks and sequencers
  can play the selected clip through the rack's effects and routing.
- Start with authored clips unlocked by collection. Clip sources and the
  sampler's crafting or unlock rule need authored rows before gameplay rollout.

## Impact

The change touches collection data, rack module definitions and saved rack
configurations, the in-game rack UI, and the engine's sample loading and playback.
