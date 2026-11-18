## Context

A laid stream knows its source burbler (`LaidStream::source`) and where along the chain
it starts (`start`). Bar lines ride the staff from the audio clock (`emit_bars`,
`ride_bars`); there are no notes yet. The campfire borrows the nearest running clock
within `clock_reach_m` (5 m) and lights when its sequencer's gates are on 2 and 4.

## Decisions

### D1. A stream's composition is layers by distance (user, 2026-11-18)
A chain carries an ordered list of layers. Each layer has a join: a distance along the
chain from the burbler. At any point of the chain, the composition is every layer joined
upstream of it. The burbler's heartbeat joins at 0. This first stage only accumulates;
splitting a stream into several compositions is a later stage.

### D2. The campfire snaps to a stream within 5 m
While a campfire is being placed, a stream bank within 5 m of the aim snaps the hologram
beside it (as streams snap beside trees). The join is the stream point nearest the fire.
A campfire with no stream in reach can still be placed, and joins nothing.

### D3. The campfire's rack inherits the stream's voice
The rack shows what arrives at the join: the burbler's clock (at its tempo), its VCO (at
its pitch) and a volume, patched as at the burbler (OUT to IN), all fixed: shown, not
removable, and costing nothing. The player mounts a snail shell in the sequencer place
and patches it in (clock GATE to sequencer CLOCK, sequencer GATE to volume TRIG) with
inventory cables. The fire lights as now, while the targets (gates on 2 and 4) are met.

### D4. The join is seen
While the layer plays, a ghostly staff (the stream's staff lines, translucent) runs from
the join point to the fire, then rises from the fire into the air and fades. It is the
campfire's own art; the stream draws nothing new at the join itself.

### D5. Notes flow from the join
On each beat the layer's sequencer gates, a note appears on the staff at the join and
rides downstream at the water's speed, as bar lines do, timed by the audio clock. Notes
never appear upstream of their join.

## Open Questions

1. **Sound (user to decide).** Downstream of the join, does the stream's heard voice
   become the layered composition (heartbeat plus 2 and 4), or does only the staff show
   it for now?
2. The heartbeat's own notes (every beat) on the staff: draw them too (stream task 2.7),
   so the campfire's are the second layer, or leave the heartbeat as bar lines only?
3. A second machine downstream: which layers does its rack show (all of them as fixed
   modules, or only the nearest machine's)?
