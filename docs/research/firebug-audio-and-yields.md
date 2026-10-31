# Firebug sound duration and mining yields

Researched 2026-10-31.

## Flash duration

Fireflies have species-specific flash durations; there is no universal firefly flash length.
[Vukusic and Chittka (2013), Visual Signals in Animal Communication](https://chittkalab.sbcs.qmul.ac.uk/2013/Vukusic_Chittka_2013.pdf)
describes adult flashes as lasting a few hundred milliseconds. The
[Firefly Atlas southwest guide (Walker, 2024)](https://www.fireflyatlas.org/wp-content/uploads/2024/06/Guide-to-Fireflies-of-the-Southwest_V1_Jun2024.pdf)
also describes some flashing species with flashes of half a second or less.

Our design choice is 300ms: 15ms attack and 285ms decay, with a 300ms visual flash.
This models the duration of light, not a natural sound made by fireflies: the
square-wave sound is the game's synthesis metaphor. Preserve the seeded independent
random clock cadence while tuning duration. Pitch is still a provisional 2200Hz.

## Satisfactory mining yields

The [official Resource Node purity table](https://satisfactory.wiki.gg/wiki/Resource_Node)
reports manual amounts per cycle: impure 1, normal 2, pure 3, resource deposit 6.
Infinite nodes do not deplete; finite deposits yield a variable total, generally
under 100. Per-cycle yield is different from total yield or items per minute.
The [Miner reference](https://satisfactory.wiki.gg/wiki/Miner) gives the normal-node
Mk.1 baseline as 60 items/minute; purity and miner tier change throughput.

Our initial balance choice is **2 square_wave items per finite bug collection**,
configured in the resource catalog. This borrows the normal manual-mining amount,
not infinite-node behavior. Collect each instance once and remove it. Avoid random
quantity code until the design calls for per-instance richness.

## Shared waveform collection feedback

Successful harvest messages carry actual item stacks. A generic audio listener
looks up each item's waveform metadata and creates one confirmation voice per
distinct waveform type, independent of stack quantity. Failed or duplicate
harvests emit nothing. The listener uses the same existing VCO/envelope/VCA patch
as world pulses, with its own constant trigger and temporary IDs. The confirmation
plays at listener level, so collection reach and bug height cannot mute it.

**Square wave is in development.** Stable item ID, waveform metadata, uses and yield
transactions are integration points; do not invent a finished square-wave model,
recipe, machine, or asset format. Keep sound presets and yield quantities tunable.
