## Context

The oscilloscope's source was set as "rippling water" on 2026-11-17; on 2026-11-19 the
user asked for a new pool rather than reusing the burbler's pool or a stream.

## Decisions

### D1. A pool that draws sound
The ripple pool is clear, still water in a shallow hollow, unlike the burbler's hot mud.
Its rings travel out from the centre and their shape follows the sound reaching the pool
(level and rough waveform), so a pool by a square-voiced burbler ripples in square-ish
steps on the beat. Silent world, still pool. The pool shows the player what a scope does
before they own one.

### D2. Ripples from the audio clock, not the frame
Ripple timing follows the audio clock events, as the burbler's bubbles do; the shape comes
from a short ring of recent samples near the pool (the scope's mechanism, audio-thread safe).
If that is not ready, rings pulse on the nearest clock's beat as a fallback.

### D3. Collecting does not drain the pool
E takes the oscilloscope; the pool keeps rippling, as a landmark and a free scope to look at.

## Open Questions

1. One oscilloscope per pool, never back (draft), or does it replenish?
2. Does a ripple pool count as water for trees or reeds (wetness, watering), or is it only
   a source?
