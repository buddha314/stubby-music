## Context

`stubby_game` (12,572 lines, 25 files) holds every Bevy-side system. Measured on
`main` at 382e226, modules import each other in both directions:

| Cycle | Through |
| --- | --- |
| `rack` ↔ `machine_config` | rack screen state, `RackHudCamera`, jack hits |
| `pulse` ↔ `prefabs` | `AuthoredLevel`, `ResourceNode`, pulse ordering |
| `resources` ↔ `prefabs` | `PrefabBinding`, `PrefabError` |
| `pulse` ↔ `machine_config` | `RackHudCamera` used as a query filter |

`streams` imports 8 sibling modules and `prefab_review` 7. Most edges carry a handful
of shared items that live in the wrong place: `ScreenMode`/`Screen` and key constants
(`controls`), `RackHudCamera` and `MachineInteractionSet` (`machine_config`),
`PrefabBinding` (`prefabs`), `InteractTarget` (`interaction`) and `terrain::height`.
`calendar` depends on `stubby_recipes::ResourceTime`, so time knows about crafting.

The editor adapter compiles all of `stubby_game` as a private module
(`stubby_jackdaw/src/lib.rs`: `#[path = "../../stubby_game/src/lib.rs"] mod desert`), and
`stubby_game` does the same with `bevy_stubby_recipes` behind `embedded_recipes_plugin`.
That was needed when the editor ran another Bevy revision; the workspace now has one
Bevy and one `Cargo.lock`, so it only costs us: the crate split means nothing inside
the editor, and saved prefabs name `stubby_jackdaw::desert::…` paths. Three such types
are saved today (`resources::MineNode` ×7, `resources::ResourceNode` ×2,
`pulse::PulseFollower` ×2).

`stubby_audio` and `stubby_recipes` + `bevy_stubby_recipes` already follow the pattern
this change generalises.

## Goals / Non-Goals

**Goals:**
- Independent systems, each a crate with one plugin, buildable and testable alone.
- One-way layering the compiler and a small check enforce.
- Cross-system rules gathered in one visible place, the game's glue.
- No change in gameplay, look or sound.

**Non-Goals:**
- Making every system Bevy-free. Cores are Bevy-free; system crates are Bevy plugins.
  Extracting more Bevy-free cores (e.g. world generation over `glam`) can follow.
- Redesigning any system's behaviour, or switching input to `bevy_enhanced_input`
  (a later change can do that inside `stubby_core`).
- Publishing crates to crates.io.

## Decisions

### Four layers

```
game      stubby_game (glue, plugin assembly, review drivers)   stubby_jackdaw (editor adapter)
            │ may use everything below
systems   bevy_stubby_rack  bevy_stubby_recipes  stubby_world  stubby_player  stubby_life  stubby_streams
            │ may use services and their own core; never each other
services  stubby_core  bevy_stubby_audio
            │ may use cores
cores     stubby_audio  stubby_rack  stubby_recipes        (no Bevy, ever)
```

`bevy_stubby_audio` is a service, not a system, because many systems legitimately
need sound and clock ticks (bugs, streams' bar lines, the rack). Making it a peer would
force every one of those through glue for no gain.
*Alternative:* two layers (cores and everything else) with only a "no cycles" rule.
Rejected: cargo already forbids crate cycles; the problem is the web, not loops.

### Where each file goes

| Today (`stubby_game/src`) | Goes to |
| --- | --- |
| `controls.rs` screen state and key constants, `RackHudCamera`, `InteractTarget`, `PrefabBinding`/`MachineInteractionSet` labels, ground-height contract | `stubby_core` |
| `audio.rs`, `module_ids.rs`, pulse bus and `PulseFollower` from `pulse.rs` | `bevy_stubby_audio` |
| `rack.rs`, `synth_modules.rs`, the generic rack HUD of `machine_config.rs` | `bevy_stubby_rack` |
| `ResourceNode`, `MineNode`, harvest systems of `resources.rs` | `bevy_stubby_recipes` |
| `terrain.rs`, `sky.rs`, ground and mountains from `lib.rs`, `world_generation/`, `export.rs`, `prefabs.rs` lifecycle, `calendar.rs` | `stubby_world` |
| player half of `controls.rs`, `hand.rs`, `interaction.rs` | `stubby_player` |
| lightning bug half of `pulse.rs`, the bug's click preset | `stubby_life` (the shared voice patch goes to `bevy_stubby_audio::voice`) |
| `prefab_presentation.rs` (generic placed-asset presenter; flora is data in its profiles) | `stubby_world`, with the prefab lifecycle |
| `streams.rs` | `stubby_streams` |
| `inventory_screen.rs`, burbler-specific parts of `machine_config.rs` and `prefab_presentation.rs`, collection cues, `WorldSources`, the calendar → `ResourceTime` conversion, `review.rs`, `prefab_review.rs` | `stubby_game` (glue) |

`stubby_life` holds plants and animals together: the theme makes them one system
(music grows life), and splitting flora from fauna now would be premature.

### Systems talk through contracts they own; glue translates
Each system publishes what it needs and what it did as its own messages and
resources, without naming another system. Examples:

- **The player's stock:** `bevy_stubby_rack` takes and gives items through the
  `stubby_core::Stock` trait, which the crafting plugin implements for its inventory;
  `RackPlugin<S: Stock>` is generic over it. (A message pair, `RackOffers` /
  `MountRequested`, was the first idea; it was dropped because a mount or a cable must
  succeed or fail in the frame the player acts, which a request answered next frame
  cannot give.)
- **Stream cost:** `stubby_streams` reads a `BuildBudget` resource and emits
  `SegmentLaid`. Glue keeps the budget equal to reeds held and removes six per segment.
- **Ground height:** `stubby_core` defines a `GroundHeight` resource (a function
  handle); `stubby_world` installs it.

*Alternative:* systems depend on each other through traits in a shared crate.
Rejected: it moves the coupling into `stubby_core` and grows it without limit.

### `stubby_core` has an entry test
An item goes into `stubby_core` only if at least two systems need it and it carries no
behaviour. Anything else stays in its system or in glue. This is what keeps the core
from becoming the new `stubby_game`.

### Unwind the `#[path]` includes first, and rewrite type paths
The adapter depends on `stubby_game` normally, `embedded_recipes_plugin` goes, and the
three saved paths are rewritten mechanically (11 lines, with the editor closed) to the
owning crate as each type moves. The `prefabs` test proves every saved prefab still
deserialises.
*Alternative:* `#[type_path = "stubby_jackdaw::desert::…"]` on the moved types, so
saved files never change. Rejected: it fossilises a module that no longer exists and
hides which crate owns a component. The rewrite is small now and only gets bigger.

### Enforce with cargo plus one small script
Crate boundaries make module cycles impossible. `cargo-deny`'s bans (`deny.toml`, `wrappers`; it replaced a
hand-written `tools/check_layers.py` on 2026-11-15) add the rules
cargo cannot express (layer order, no system → system edge, no Bevy in cores), from
`cargo metadata --no-deps`, in well under a second. CI runs it with the other checks.

## Risks / Trade-offs

- [Moves conflict with in-flight work (`world_generation` claimed by Codex in
  `add-hybrid-world-placement`; `streams.rs` in `add-mudpit-burbler` 2.7)] → one
  crate per PR, move-only commits separate from edits, and `stubby_world` and
  `stubby_streams` scheduled with their owners rather than moved under them.
- [Saved prefabs edited in Jackdaw while a path rewrite lands] → rewrite only with the
  shared editor idle; the `prefabs` test fails loudly on a missed path.
- [More crates, slower clean builds] → incremental builds get faster, since a change
  to one system no longer rebuilds all of `stubby_game`; `dynamic_linking` dev
  builds are unaffected.
- [Glue becomes the new dumping ground] → glue holds only translations between
  systems; behaviour of one system belongs in that system. Review against the
  where-it-goes table.
- [Visibility churn: `pub` items that are only used inside a system] → each new crate
  starts with `pub(crate)` by default and exports only its plugin and contracts.

## Migration Plan

Land in order; each step builds, passes `cargo test -p stubby_jackdaw --test prefabs`,
and is revertible on its own:

1. Unwind the `#[path]` includes; rewrite `stubby_jackdaw::desert::` → `stubby_game::`.
2. Extract `stubby_core`; repoint imports. Most cycles disappear here.
3. `bevy_stubby_audio`.
4. `bevy_stubby_rack` (after `refactor-rack-modules-as-data`).
5. `ResourceNode`/`MineNode` and harvesting into `bevy_stubby_recipes`.
6. `stubby_player`.
7. `stubby_life`.
8. `stubby_world` (with Codex).
9. `stubby_streams` (with the stream owner).
10. Layer check on, AGENTS.md updated.

## Open Questions

- Should the inventory screen and goals HUD become their own crafting UI crate, or
  stay in glue? (Proposed: glue for now; AGENTS.md already places crafting
  presentation in the game.)
- Is the burbler a system (`stubby_machines`) or glue? It is mostly a composition of
  rack, streams and world today. (Proposed: glue until a second machine exists.)
