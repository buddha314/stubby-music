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
While the layer plays, a ghostly staff (the stream's staff lines, translucent) is drawn
from the join along the ground into the fire's base, as air is pulled into a fire (user,
2026-11-18: "enters at the bottom and drifts upwards"), then drifts up from it like heat,
wavering and spreading as it rises, and fades. It is the
campfire's own art; the stream draws nothing new at the join itself.

### D5. Notes flow from the join
On each beat the layer's sequencer gates, a note appears on the staff at the join and
rides downstream at the water's speed, as bar lines do, timed by the audio clock. Notes
never appear upstream of their join.

### D6. Decided (user, 2026-11-18)
- **Sound and sight.** Downstream of a join, the stream sounds the layered composition
  (the heartbeat and the campfire's 2 and 4), heard from the stream, and the staff shows it.
- **Heartbeat notes.** The heartbeat puts a note on the staff on every beat, from the
  burbler's mouth, so a campfire's notes read as a second layer.
- **Inheritance.** A machine's rack shows every layer upstream of its join, fixed: the
  burbler's voice, and any machine's layer between.

### D8. A join updates the chain; it does not add to it (user, 2026-11-18)
Additive layers made the burbler's heartbeat and the campfire's backbeat sound together,
so the player heard every beat where the patch said 2 and 4 ("we want to update with the
campfire"). Decided: a chain plays its latest version. While a joined machine's layer
plays, the burbler is `ChainUpdated { at_m }`: its own voice falls silent, and from the
join down only the joined machine's version sounds and shows. The heartbeat's notes ride
to the join and leave the staff there (`DivertedNote`), drawn along the ghost staff into
the fire and up with the heat; the camp's notes ride on downstream. When the camp stops,
the heartbeat plays the whole chain again. A machine further down inherits the latest
version and updates it again, so "layers" accumulate as transformations, not as voices.
This replaces D6's "the stream sounds the layered composition".

### D9. The rack shows what was inherited (user, 2026-11-18)
A joined machine's rack keeps the inherited modules, then a **stream blank**: a 2HP
divider (`assets/modules/stream_blank/`, `create_stream_blank.py`; modelled on Eurorack 2HP
blanks, 10.16 mm, two diagonal screws), dark anodised with a small burbler bubble glyph and
"@bitlung_" running up it in blue neon (Mozilla Headline, OFL, baked to geometry; user,
2026-11-18), so the player sees where the burbler's part ends. The
machine's own modules follow it. The campfire: burbler clock and VCO | stream blank |
snail sequencer and VCA (the VCA is the camp's, not inherited); the VCO's OUT to the VCA's
IN stays made for the player.

### D7. How layers reach the staff
`stubby_streams` owns the staff. A machine adds a note with a `LayerNote` message: the
chain's burbler, the join's distance down the chain, the instant the note is heard (the
audio clock's), and its staff step and colour. The heartbeat's notes come from the
burbler's own clock ticks. Who sends layer notes is glue (the campfire's sequencer steps).
