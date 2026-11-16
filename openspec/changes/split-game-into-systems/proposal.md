## Why

`stubby_game` is one 12,500-line crate in which the systems import each other freely,
and four pairs import each other both ways (`rack` ↔ `machine_config`, `pulse` ↔
`prefabs`, `resources` ↔ `prefabs`, `pulse` ↔ `machine_config`). The synthesis code
knows about inventory, the stream code about racks and reeds, the calendar about
recipes. Nothing stops this from getting worse, because Rust only enforces boundaries
between crates, not between modules in one crate. The audio engine and the crafting
system already show the shape we want (a Bevy-free core and a thin plugin, used by the
game but never using it); the rest of the game should follow, so each system can be
understood, tested and replaced on its own, as in Rust and Bevy best practice.

## What Changes

- Split `stubby_game` into **independent system crates in four layers**: Bevy-free
  cores, shared services, feature systems, and the game. A crate depends only on
  layers below it; **feature systems never depend on each other**.
- Add **`stubby_core`**, a deliberately tiny shared crate: screen state, input
  bindings, the HUD camera marker, the interaction target contract, shared system-set
  labels and ground height. Moving these out of feature modules breaks most cycles.
- New system crates: `bevy_stubby_audio` (engine bridge, module IDs, the pulse bus),
  `bevy_stubby_rack` (the rack screen, cables, module assets; over `stubby_rack` from
  `refactor-rack-modules-as-data`), `stubby_world` (terrain, sky, seeded generation,
  placed-prefab lifecycle, calendar), `stubby_player` (controls, hand, interaction),
  `stubby_life` (lightning bugs, scrub oak, reeds), `stubby_streams` (stream building).
- `stubby_game` keeps only **glue**: it adds the plugins and holds every rule that
  joins two systems (an item mounts as a module, reeds pay for a stream, a burbler is
  a clock with a rack and an outlet), plus review drivers.
- Harvestable-source components (`ResourceNode`, `MineNode`) move to
  `bevy_stubby_recipes`, which owns resources and mines.
- **BREAKING (saved data):** the editor adapter stops compiling the game's source by
  `#[path]` and depends on the crates normally; reflected type paths in saved prefabs
  change (`stubby_jackdaw::desert::…` → the owning crate). 11 lines across the `.bsn`
  files are rewritten in the same PRs. `embedded_recipes_plugin` is removed.
- A **layer check** (`cargo deny check bans` over `deny.toml`) fails when a crate
  breaks the layering.
- No gameplay, look or sound changes.

## Capabilities

### New Capabilities
- `system-architecture`: the game as independent system crates in layers, the rules
  for what may depend on what, how systems talk, and how the layering is enforced.

### Modified Capabilities
None in `openspec/specs/` (it is empty). Behavioural requirements of in-flight changes
are unchanged; their code moves.

## Impact

- `crates/stubby_game` shrinks to plugin assembly and glue; most files move to seven
  new crates. Root `Cargo.toml` members, `Cargo.lock`, `stubby_jackdaw/Cargo.toml`.
- `crates/stubby_jackdaw/src/lib.rs` loses its `#[path]` include; `prefab_bridge.rs`
  imports from the new crates.
- `assets/prefabs/**/*.bsn`, `assets/scene.bsn`, `assets/review_prefab_runtime.bsn`:
  type paths rewritten.
- `crates/stubby_game/src/calendar.rs`: the `stubby_recipes::ResourceTime` conversion
  moves to glue, so the world does not depend on crafting.
- Depends on `refactor-rack-modules-as-data` (PR #119) for `stubby_rack`.
- Touches files in flight elsewhere, notably `world_generation` (Codex,
  `add-hybrid-world-placement`) and `streams.rs` (`add-mudpit-burbler` 2.7). Moves land
  one crate per PR, coordinated with those owners.
- AGENTS.md: repo map, architecture rules and the adapter's type-path rule.
