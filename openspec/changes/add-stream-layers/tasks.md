## 0. Decide (user)
- [x] 0.1 The campfire's rack inherits the stream's voice, prefilled (user, 2026-11-18)
- [x] 0.2 Join: snap within 5 m; a ghostly staff from the stream to the fire and up into the air; new notes appear downstream of the join (user, 2026-11-18)
- [x] 0.3 Sound: downstream of a join the stream sounds the layered composition, and the staff shows it (user, 2026-11-18: "both")
- [x] 0.4 The heartbeat gets a note on every beat (user, 2026-11-18)
- [x] 0.5 A machine downstream shows every upstream layer in its rack (user, 2026-11-18)

## 1. Streams
- [x] 1.1 Layers on a chain: a machine sends `stubby_streams::LayerNote` (the chain's burbler, the join's distance down it, the heard instant, staff step, colour); the heartbeat joins at 0 from the burbler's clock ticks (claimed: claude)
- [x] 1.2 Notes: the `Pulse_0` note rides the staff from its join on the audio clock (`emit_notes`, `ride_notes`); the heartbeat a note on every beat; the campfire's ember notes (a step up the staff) from its join (`campfire::StreamJoin`, nearest stream point within `clock_reach_m`) on each gated step. Bounded review: join 5.2 m down the chain, heartbeat notes from the mouth, ember notes only from 5.6 m on

## 2. Campfire
- [ ] 2.1 Placement snaps beside a stream within 5 m; the placed campfire records its join
- [ ] 2.2 Rack inheritance: the stream's clock, VCO and volume fixed and patched; only the sequencer open
- [ ] 2.3 The ghostly staff from the join to the fire and rising (art: campfire profile and model)

## 3. Review
- [ ] 3.1 Bounded review: place a campfire by a sounding stream, patch the sequencer, see the join and 2 and 4 notes downstream only
