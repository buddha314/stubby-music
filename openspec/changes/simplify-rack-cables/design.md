## Decisions

### D1. Cables are inventory items only (user, 2026-11-18)
The comb's free cables and the player's own were two ways to do one thing. Now there is
one: a cable is a `patch_cable` item. A rack's `configure.cables` are suggested patches
(ghosts, pods, colour), not objects. Starting a cable with none held says so and does
nothing.

### D2. Racks are specific to the machine (user, 2026-11-18)
"If a burbler does not need a sequencer, it should not be available." The burbler's rack
is clock, VCO and volume; the melody it could take (sequencer CV to V/OCT) is gone with
its place. The sequencer belongs to machines whose work needs one (the campfire).

### D3. Hover lights the whole cable
Each patched cable is pickable with its own material; under the pointer its emissive
rises to six times its colour, so the cable reads from end to end.

### D4. The second end is guided
While a cable's first end is in one end of an unmade suggested patch, the pods show only
that patch's other jack, blinking quickly and brightly, and the cable in hand is the
patch's colour. Otherwise every waiting pod pulses as before.

### D5. Cables start on a jack
Only the far end snaps from within 40 px. Starting by snapping would let any click near a
jack (after turning a knob) take a cable.

## Not done here
- Why a clock did not run with a sequencer patched on the burbler (user's report): moot
  for the burbler now; the campfire's sequencer runs from its borrowed clock.
