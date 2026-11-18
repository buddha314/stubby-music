## Context

Rules (`assets/recipes/`) are data the designer owns, validated by `stubby_recipes`, and
hot-reloaded by dev builds. Machine profiles are art's: models, panels, places, cables.
Target settings and reaches are game rules that get tuned in play, so they move to rules.

## Decisions

### D1. A `machines` registry keyed by placed asset ID
Rows are `{id, targets: [], clock_reach_m?}`. `id` is the machine's placed asset kind
(`campfire`, `mudpit_burbler`), not an item: machines are placed things, some of them
mines (the burbler) and some buildables (the campfire). Validation in `stubby_recipes`
checks shape only (a target has a `note` or a `value`, not both; a note parses), because
the core does not read profiles or modules. Whether each target names a real place and
knob is checked where both are known: the rack's test over the shipped rules and
profiles (`target_problems`).

### D2. The game's glue joins rules to racks
`stubby_recipes` and `bevy_stubby_rack` never depend on each other. The glue
(`stubby_game`) writes each row's targets into `bevy_stubby_rack::Profiles` whenever the
catalog loads or reloads, and the campfire rule reads its reach from the row. A machine
without a row has no targets.

### D3. The live panel reads racks over the remote protocol
`RackTargets` becomes `{met, missed: Vec<String>}` and is reflected; with `PlacedAsset`
and `MachineVoice` reflected, the page queries them with `world.query` and lists each
machine. The page already polls `world.get_resources` for air.

### D4. The designer launches with dev builds
In debug builds the game starts `cargo run -q -p stubby_rules_server` with the
workspace as its directory (known at compile time), in the background, then opens the
page with the desktop's opener. If something already answers on 127.0.0.1:7878, it does
neither: the designer outlives the game, so unsaved edits survive restarts. The designer
keeps running when the game exits. `STUBBY_NO_DESIGNER=1` skips it, and
`tools/jackdaw_timed_test.py` sets it. The designer stays a separate tool: nothing in the
game links it (`deny.toml`: "never the game").

## Risks / Trade-offs

- Running cargo from the game costs a build the first time. It is in the background and
  only in dev builds.
- Jackdaw's Play starts the game too; the port check keeps it to one designer.
