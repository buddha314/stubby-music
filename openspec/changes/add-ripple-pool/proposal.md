## Why

The reed water purifier needs an **oscilloscope** (`add-reed-water-purifier`), collected
from rippling water. Nothing in the world ripples yet: the burbler's pool bubbles mud and
streams flow. The user asked for **a new pool** for it (2026-11-19).

## What Changes

- **A new world asset, the ripple pool**: a small still pool of water on wet ground whose
  surface ripples in rings, a waveform drawn on water. Its ripples are the oscilloscope:
  they trace the sound arriving at the pool, so a pool near a sounding burbler shows that
  burbler's wave.
- **It yields an oscilloscope** when collected with E (draft: one per pool, no
  replenishment, as the high-pass sources; Open Question 1).
- **Placement**: ripple pools sit in the wetness channel of the seeded opening
  (`add-hybrid-world-placement` 2b.3), near reed beds; the opening holds at least one.
- **Rules data**: the `oscilloscope` item, the `ripple_pool` resource, its recipe-book entry.

## Capabilities

### New Capabilities

- `ripple-pool`: the ripple pool asset and how it gives an oscilloscope.

## Impact

- Art (Claude): `assets/water/ripple_pool/` (or `assets/flora/`-style family), model,
  ripple shader or animated rings, LODs, prefab, icon.
- Rules (data): `items.json`, `resources.json`.
- Game (Codex): binder, pickup, ripples driven by the sound reaching the pool (the
  scope's sample ring from `add-reed-water-purifier` 2.4 can feed both).
