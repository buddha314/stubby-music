## Context
Streams are laid as `stream_segment`s from a burbler's outlet along a planned path, each
built from 6 reeds (`add-mudpit-burbler` 2.7, `add-reeds`). The mudflat is generated
in-engine from the world seed. Scrub oaks start dormant and bloom as clean water reaches
them (`add-scrub-oak-production`). Nothing yet lets the player change the ground.

## Open questions
- What digging is: a tool in the hand (a new `HandItem` with its own key), a build-menu
  action (Q), or part of laying a stream (the plan cuts its own channel)?
- What it costs: time, effort, an item, nothing?
- How dug ground is represented: a terrain edit stored per save (deterministic from seed
  plus edits), or placed channel pieces?
- When a tree counts as watered: a stream segment's `Port_out` within reach of its roots,
  or water quality above a threshold for some time?
- Whether muddy water waters a tree at all, or only cleaned water (the fluid change says
  quality matters; `add-fluid-quality-and-viscosity`).
