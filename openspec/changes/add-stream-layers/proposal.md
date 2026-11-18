## Why

A stream should carry the music built along it (user, 2026-11-18). Today the campfire
borrows the nearest clock and lights on its own; nothing it does reaches the stream. The
user's direction: the campfire inherits what was built at the burbler, its sequencer adds
to the stream's staff, and anything built downstream inherits it in turn. The stream
accumulates a composition; later stages split it into several.

## What Changes

- **A stream carries layers.** The burbler's heartbeat is the first; a machine joined to
  the stream adds one at its join point, and it flows downstream from there.
- **The player's flow** (user, 2026-11-18): build a campfire from reeds (Q, 25 reeds), place
  it by a stream, then configure it (E): mount a snail shell and patch its two cables.
- **The campfire joins a stream.** Placed within 5 m of a stream, its hologram snaps to
  the stream's bank (user, 2026-11-18); that point on the stream is its join.
- **The campfire inherits the stream's voice.** Its rack shows the voice as it arrives
  from the burbler (clock, VCO, volume, patched), and the player adds only the snail
  sequencer (user, 2026-11-18: "stream's voice, prefilled"). No second square wave.
- **A visible join.** While the campfire's layer plays, a ghostly staff runs from the
  join point to the fire and rises from the fire into the air.
- **New notes downstream.** On each beat the sequencer gates (2 and 4, for the fire), a
  note appears on the staff at the join point and rides downstream; upstream of the join
  there is none.
- Later, not here: splitting a stream into several compositions.

## Capabilities

### New Capabilities
- `stream-layers`: a stream's accumulated composition, joins, and how they show.

### Modified Capabilities
None accepted. In flight: `add-campfire-craft-bench` (the campfire's rack and its clock),
`add-stream-building` (notes on the staff, 2.7), `add-localized-machine-voices`.

## Impact

`stubby_streams` (layers, notes on the staff, the join's ghost staff), `stubby_core`
contract for a join, `stubby_game` (campfire placement snap, rack inheritance, glue),
`bevy_stubby_rack` (inherited, fixed places and cables), the campfire profile and art
(the rising ghost staff).
