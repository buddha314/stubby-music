## Context

The oscilloscope's source was set as "rippling water" on 2026-11-17. On 2026-11-19 the
user placed it in streams: pools at Poisson-random places along a stream, rippling to
the beat, refreshing semi-daily, because the game needs many oscilloscopes.

## Decisions

### D1. Poisson along the stream
Pool positions along a stream are a Poisson process in arc length: gaps are drawn from an
exponential distribution with a mean spacing set in data (draft: one pool per 12 m, three
segments). The RNG is seeded from the world seed and the segment's stable ID, per segment,
so pools do not move when the stream is extended or another stream is laid, and
dismantling a segment takes its pools with it (re-laying it brings the same ones back).
A pool keeps clear of a segment's ends and of anything placed in the stream (a check dam).
Deterministic, no `thread_rng`.

### D2. Ripples on the beat
A pool is a small widening of the bed where the water stills. On a sounding stream, a
baked one-beat ring clip plays on each beat of the stream's clock, phase-locked to the
audio clock as the burbler's bubbles are. A silent stream's pools are still, so the
opening stays silent and still. Later the ring's shape may follow the sound's waveform
(the scope's sample ring); not in this change.

### D3. One oscilloscope, back half a day later
Each pool is a resource instance (ID from its segment and index), yields one
`oscilloscope` with E, and returns after 1800 s of calendar time (`after`), half a day
at any speed. Collecting does not stop the ripples. The scanner (B) shows uncollected
pools as it shows other pickups.

## Open Questions

1. Mean spacing (draft 12 m).
2. Do pools also form on wild water (burbler pools, wet ground), or only on laid streams?
