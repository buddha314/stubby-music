## Why

The reed water purifier needs an **oscilloscope** (`add-reed-water-purifier`), collected
from rippling water, and the game needs **a lot of them** (user, 2026-11-19). Ripple pools
**form in streams**: small still eddies along a laid stream whose surface ripples to the
stream's beat. Laying streams is how the player finds them.

## What Changes

- **Ripple pools form along laid streams**, at seeded random places: a Poisson process
  along the stream's length, so gaps between pools are random but their average is set.
  The same stream laid the same way in the same world gets the same pools.
- **A ripple animation on the beat**: on a sounding stream, rings leave each pool's centre
  on every beat of the stream's clock. A silent stream's pools are still.
- **Each pool yields one oscilloscope** with E, and **refreshes semi-daily** (half a day of
  simulation time, the existing `after` replenishment at 1800 s).
- **Rules data**: the `oscilloscope` item, the `ripple_pool` resource and its recipe-book
  entry; the pool spacing as stream data.

## Capabilities

### New Capabilities

- `ripple-pool`: ripple pools in streams and how they give oscilloscopes.

## Impact

- Art (Claude): the pool model, an eddy that widens the stream bed, and its beat ripple
  clip; icon.
- Rules (data): `items.json`, `resources.json`.
- Game (Codex): `stubby_streams` places pools along laid segments (seeded Poisson) and
  removes them with their segment; the ripple clip plays on the stream's clock; pickups
  through the existing E path.
