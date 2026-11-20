## Context

High-pass filters were assigned to the world on 2026-11-17 and decided on 2026-11-19:
one per rock opening, one per hollow trunk, not every trunk hollow. The echo grotto's
boulders (`add-echo-grotto`) are the first rock formation, and the boulder kit
(`assets/props/rocks/`, five variants) exists.

## Decisions

### D1. One per opening, one per hollow, back half a day later
Each opening and each hollow is its own resource instance with a stable ID and yields one
`high_pass_filter`. It **recurs semi-daily** (user, 2026-11-19): the existing `after`
replenishment at **1800 s**, which counts calendar time, so half a day (a day and night
are 60 minutes at 1x) at any speed. No new rule type. A collected opening keeps its look;
only its yield (and its sound, D3) is gone until it returns.

### D2. A formation is a cluster; an opening is a socket on it
A rock formation is one placeable model made of several boulders, so seeded placement
places formations, not single rocks. A formation with an opening carries one or more
`Opening` sockets (empties outside its `_LOD` nodes, recreated by the binding as for
`Interact`), each a pickup. A hollow oak variant carries a `Hollow` socket at the trunk.
This keeps the seeded rule simple (pick a formation variant) and lets art decide where
the gap is.

### D3. Heard first
A source within hearing of something sounding passes it high-passed: the bright, thin part
only. This is how the player learns what a high-pass filter does before ever patching
one. Silence stays silence: an opening near nothing sounding is quiet, so the opening
does not break the silent start. **Collecting stops the sound** (user, 2026-11-19) until
the filter returns, so a sounding opening always means one to take. Implementation belongs
to the audio stream; if it is late, the pickups ship without it.

### D4. Hollow trunks are seeded per oak
Whether an oak is hollow is part of its per-instance variation from the world seed and
its instance ID, at a rate tuned in its `asset.json` (draft: one in four). **A hollow can be
collected while its oak is dormant** (user, 2026-11-19), so it is a source from the start.

## Risks / Trade-offs

- A formation with sockets needs the socket binding that already exists for `Interact`;
  more than one opening per formation needs distinct instance IDs (`<formation>-opening-<k>`).
- Making an opening audible depends on spatial audio work that is not scheduled yet.

## Open Questions

None open. Answered 2026-11-19: hollows give filters while dormant; collecting silences the
source until it returns; sources recur semi-daily.
