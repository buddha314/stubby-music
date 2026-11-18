## Why

Machines are tuned by playing them: the campfire's backbeat, the purifier's notes, how far
a camp reaches for a clock. Those settings live in machine profiles
(`assets/machines/…/asset.json`, `assets/structures/campfire/asset.json`), which are
compiled into the game, so every change is a rebuild. The user expects to iterate on them
a lot (2026-11-18), and more settings will follow.

The rules designer (`add-crafting-designer`) already gives rules a tight loop: schema
forms, live problems, undo, canonical save, and dev builds reload `assets/recipes/` within
a second. Machine settings belong in that loop. The designer also has to be started by
hand; the user wants it launched with the game.

## What Changes

- **A `machines` registry** (`assets/recipes/machines.json`, a new rule type): one row per
  machine kind, keyed by its placed asset ID (`mudpit_burbler`, `campfire`), holding the
  settings the game tunes:
  - `targets`: the knobs its work needs, each `{place, knob, note | value}` (moved from the
    profiles' `configure.targets`, `add-reed-water-purifier` D3);
  - `clock_reach_m`: how far a machine with a borrowed clock reaches for one (moved from
    the campfire's profile).
  More settings join the registry as they need tuning. Profiles keep what art owns: model,
  places, cables, nodes.
- **The rack reads them from the rules.** The game's glue copies each row into the rack's
  machine profiles when the rules load or reload, so a saved edit reaches a running game
  without a rebuild.
- **The designer's live panel shows machines**: for each placed machine with a rack, its
  instance, whether it sounds, and whether its targets are met, naming each missed target
  and its current value. Read over Bevy's remote protocol, as the panel already reads air.
- **Dev builds launch the designer.** Starting the game in a dev build runs the designer
  (`cargo run -p stubby_rules_server`, from the checkout) unless one is already listening
  on its port, and opens it in the browser. Release builds never do. `STUBBY_NO_DESIGNER=1`
  turns it off (bounded test runs set it).

## Impact

- `stubby_recipes`: the `machines` registry, schema, validation, editing model.
- `bevy_stubby_rack`: targets come from outside the profile; `RackTargets` gains the missed
  list and is reflected.
- `stubby_game`: glue from rules to rack profiles; the designer launcher.
- `stubby_rules_server` page: the machines panel.
- Profiles: `configure.targets` and the campfire's `clock_reach_m` move to `machines.json`.
