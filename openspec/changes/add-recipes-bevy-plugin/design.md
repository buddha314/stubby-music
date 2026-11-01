## Context
`stubby_recipes` (no Bevy) has the rules files and validation (`rules.rs`), `Catalog`,
`Inventory`, `RecipeBook`, `HarvestLedger`, source objects, replenishment and atomic
crafting. `stubby_game::resources` defines `ResourceRecipesPlugin` with
`PlayerInventory` and `RecipeCatalog` resources, `ResourceCollected`, `CraftRequest`
and `CraftCompleted` messages, `ResourceNode`/`MineNode` components, and systems that
mix rules work (`craft_requests`, `replenish_nodes`, the harvest in `collect_nearby`)
with game presentation (`update_hud`, the collection popup, E-key reach checks).
`ItemDefinition` carries the game's `waveform` and `uses` (VCO, LFO, gate).

Research (`docs/research/crafting-system-library-maturity.md`, recommending *not yet* for publishing): no engine ships
crafting; engine plugins (Opsive for Unity, Arc and Narrative for Unreal, expressobits
for Godot) are bound to their engine's data types; mature Rust game libraries keep a
pure core and a thin Bevy adapter in one repo; Bevy-bound cores churn or go stale.

## Goals / Non-Goals
- Goals: install with one `add_plugins`; Bevy-native API (components, messages,
  assets, reflection); a core that never sees Bevy; Bevy upgrades touch one thin
  crate; game-specific concepts outside the plugin and core; ready to extract.
- Non-goals: publishing now; multiplayer replication; a UI (inventory and goals
  screens stay in the game, as plugins in this space leave presentation to the game);
  timed production and machines (they will build on this, not in this change).

## Decisions

### D1. Two crates, one direction
`stubby_recipes` (core) ← `bevy_stubby_recipes` (plugin) ← `stubby_game`. The core
has no Bevy dependency at all, not even optional; Bevy versions appear only in the
plugin's `Cargo.toml`, pinned to one minor (`bevy = "~0.20"`, `default-features =
false`, only the features it uses). The plugin README carries a Bevy compatibility
table.

### D2. What moves into the plugin
| Now (in `stubby_game`) | In the plugin |
| --- | --- |
| `RecipeCatalog` resource + `include_str!` loading | `RulesAsset` loaded by an `AssetLoader` (via `bevy_common_assets` or a small loader for the rules folder), hot reload keeping the last valid rules, errors as a `RulesInvalid` message |
| `PlayerInventory` resource | `Inventory`, `RecipeBook`, `HarvestLedger` **components** on an owner entity (D3) |
| `ResourceNode`, `MineNode` components | Same components, generic (no lightning-bug constructors; those stay in the game) |
| `ResourceCollected`, `CraftRequest`, `CraftCompleted` | Same messages, each naming the owner entity; plus `HarvestRequest` |
| `craft_requests`, `replenish_nodes`, the harvest step of `collect_nearby` | Plugin systems in a public `RecipesSystems` set the game can order against |
| Reach checks, E key, HUD, popup, recipe-book text | Stay in the game: the game decides *when* to harvest and sends `HarvestRequest` |

The calendar the replenishment reads (night boundaries, elapsed time) is supplied by
the game through a small `ResourceClock` resource the plugin reads, so the plugin does
not own the day/night cycle.

### D3. Inventories are components
An inventory belongs to an entity (the player now; machines and storage later, as
Satisfactory's stations own an output inventory and Unreal's Games-by-Hyper stations
hold output until collected). Messages name the owner. A single-player game simply
puts one on the player. This is the main API change from today's global resource.

### D4. Reflection and saving without Bevy in the core
Core types derive `serde` and `schemars` only. The plugin wraps them in reflected
components (`#[derive(Component, Reflect)]` newtypes with `#[reflect(opaque,
Serialize, Deserialize)]`) so the inspector and `moonshine-save` see them, and the
save stays the core's serde shape (`add-inventory-and-goals-hud` D10). No
`bevy_reflect` in the core.

### D5. Game metadata through an extension, not core fields
`ItemDefinition` keeps `id`, `name`, `description`, plus `extensions:
BTreeMap<String, serde_json::Value>` that the core validates only as JSON and passes
through. The game reads its own typed view (`waveform`, `uses`) from
`extensions["stubby"]` with its own schema fragment, so the rules files keep one
schema and the core stays game-agnostic. (Alternative: a type parameter
`ItemDefinition<M>`; rejected because it spreads a generic through every API and the
asset loader.) This is a rule-type change and needs `rules-fmt` to rewrite
`items.json` and its migration of the field layout (`format` 2).

### D6. Names and publishing (later)
In-repo names: `stubby_recipes` and `bevy_stubby_recipes`. Before publishing, choose a
distinctive name (Bevy's guidance: do not claim generic `bevy_` names such as
`bevy_crafting` without asking), dual-license MIT OR Apache-2.0, start at 0.1.0, mark
public structs and enums `#[non_exhaustive]`, add a non-Bevy example and a Bevy
example, and a README "Non-goals". Extract with `git filter-repo
--subdirectory-filter` into one repository holding both crates (Rapier's layout); the
game then depends on the published versions and uses `[patch.crates-io]` with local
paths only while developing both.

## Risks / Trade-offs
- Components instead of a resource ripple through game code → done in one move with
  the game's tests; the player gets the components at spawn.
- Extension metadata is less typed than fields → the game's typed view and its schema
  fragment are validated by `rules-fmt` like everything else.
- Two crates to maintain → the plugin is thin by design; Bevy upgrades are its only
  routine churn.
- Demand for a published crate is unproven → shaping costs little now; publishing waits
  for its own decision (Open questions).

### D7. When to publish: not yet
The research recommends shaping now and publishing later, when all of these hold:
the core has no stubby-specific types (D5, and the productivity-factor and
night-boundary names generalised); the goals reducer and at least one real production
recipe have shipped in the game; a non-Bevy example runs; and the public API has gone
about two months without a breaking change. Plus a pull: someone outside asked, or the
designer accepts a release step on every cross-cutting change. The likelier draw for
outside users is a production-rate model and planner, which no library offers
(FactorioLab is an app), so "publish once machines and rates exist" is the trigger.
The first release would be the core and `rules-fmt` only; the Bevy plugin follows, so
the published core is never on Bevy's release cycle. The game repo is MIT; dual
licensing MIT OR Apache-2.0 needs the copyright holder's agreement. A scheduled CI job
can build the game against the library's `main` to catch drift.

## Open questions
- The published name: the research suggests `rulebook` (the strength is validated rules
  data), or `craftbook` / `recipe_book`; all unclaimed on crates.io on 2026-11-01.
- `bevy_common_assets` versus a dedicated loader for a folder of rules files.
