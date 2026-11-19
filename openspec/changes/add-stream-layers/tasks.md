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
- [x] 2.1 Placement snaps beside a stream within reach (`build::beside_stream`, 2.2 m from its centre line on the aim's side); a campfire joins the nearest stream point within reach (`campfire::StreamJoin`)
- [x] 2.2 Rack inheritance: `RackPlace::inherited` and `RackCable::inherited` (made, free, never unplugged; `RackState::inherited`); the campfire's `voice` is the stream's VCO, fixed, its pitch the burbler's (`campfire::inherit_voice`); only the sequencer is open. Its voice is heard from the stream downstream of the join (`stubby_streams::HeardDownstream`)
- [x] 2.3 The ghostly staff from the join to the fire and rising, a pulse travelling up it (`campfire::ghost_staff`, procedural; while the layer plays)

## 3. Review
- [x] 3.1 Bounded review (2026-11-18): the scene campfire by a sounding stream, sequencer patched, no square wave: it sounds and burns; join 5.2 m down; heartbeat and ember notes downstream; the ghost staff runs from the stream to the fire and rises
- [x] 3.3 A join updates the chain (D8): `stubby_streams::ChainUpdated` silences the burbler and diverts older notes at the join (`DivertedNote`); the campfire draws them along its ghost staff into the fire and up (`campfire::draw_in_notes`). Bounded review: no heartbeat note past the join; notes drawn in; ember notes downstream
- [x] 3.4 The stream blank divider (D9) and the campfire's order: inherited clock and VCO, the blank, then the camp's sequencer and VCA; bounded run shows it
- [ ] 3.2 The user plays it: placing by a stream, the inherited rack, hearing only 2 and 4 at and below the fire
