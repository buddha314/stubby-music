## Why

Recipes are instant today: a click on the craft button makes one. The user decided on
2026-11-20:

- **Recipes take time.** Early ones should feel almost instant.
- **Progress shows as a bar**, with Satisfactory's Craft Bench as the example.
- **There are two ways to craft**: **by hand** at a crafting station, and **delegated to
  Dancers**, who make it while the player does something else.

Satisfactory's bench: the player holds Craft (or taps Space to latch it), a bar fills in
steps, and each recipe's time is counted in hammers of about 0.25 s (an iron ingot is
3 hammers, about 0.75 s). Holding on repeats the craft while the ingredients last.
[Crafting Bench](https://satisfactory.wiki.gg/wiki/Crafting_Bench) ·
[Equipment Workshop](https://satisfactory.wiki.gg/wiki/Workbench)

## What Changes

- A recipe row says **how** it can be made: `by_hand` (seconds at a station) and/or `crew`
  (Dancers of a profession, and a positive whole number of worksite quarter notes). At least one; early recipes are
  by hand only and short (about 0.25 to 1 s).
- **By hand**: hold Craft or latch it with Space; a bar fills in 0.25 s steps; the inputs
  are taken and the outputs given when it fills; holding on starts the next; letting go
  pauses it.
- **Delegated**: at a station in an established village, the player queues a crewed recipe
  for Dancers; it advances on worksite quarter notes with its own bar, on the station's screen and above
  the station in the world, while the player is free.
- **One progress bar widget** in `stubby_ui` for both, and for machines later.

## Impact

- `stubby_recipes`: the `by_hand` field and its validation; the rows gain durations.
  The `crew` field and the Dancers who fill it come from `add-settlement-economy`, which
  this change and that one share.
- `stubby_ui`: a progress bar.
- `stubby_game::crafting_screen`: timed hand crafting, then a Dancers mode.
- `add-campfire-craft-bench`: its instant click becomes a timed craft.
