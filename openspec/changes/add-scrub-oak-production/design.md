## Status
Design intent captured only. Do not implement production in the source-model change.

## Decisions
- A scrub oak is an infinite mine for lightning bugs.
- Each produced bug is a distinct finite resource instance, yielding its configured
  number of square-wave items when collected.
- Increasing water quality improves bug production with light held constant.
- Increasing light quality improves bug production with water held constant.
- Worse quality can reduce throughput; it never consumes a finite mine reserve.

## Open questions
Baseline rate, quality scales, combined quality response, maximum active bugs,
spawn locations, collection automation and tree visual assets are intentionally
undecided. Do not invent a formula, respawn timer or automatic production loop yet.
Future spawning must use explicit world seed and stable generated instance IDs.

## Water-system dependency
`add-fluid-quality-and-viscosity` records the initial muddy-water resource, fluid
viscosity and cleaning progression. Cleaning provides more useful nutrients and
faster-flowing water. Future oak production should consume delivered water quality
and nutrient provision alongside light quality; do not substitute a purely visual
water material setting or assume an infinite instantaneous water supply.
