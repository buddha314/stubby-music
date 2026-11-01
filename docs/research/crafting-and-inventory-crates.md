# Crafting, inventory and factory crates for Bevy

Research, 2026-11-01, for the builder-game side of stubby-music: items, recipes,
inventories, the rack configuration screen, save/load and (later) timed machine
production. Question: what can we take from crates.io instead of writing it?

Versions and Bevy requirements below were read from the crates.io API on
2026-11-01. We are on **Bevy 0.20.1** (released 2026-10-28), so a crate that does
not depend on `bevy ^0.20` cannot be used as-is.

## Summary

- **No crafting or inventory crate is worth adopting.** The ones that exist are tiny
  (tens to hundreds of downloads), target older Bevy, and model generic RPG
  inventories, not a factory builder's catalog. Keep `stubby_recipes`, which already
  has the catalog, items, resources, mines, replenishment, inventory, harvest ledger,
  recipe book and crafting, as a Bevy-free crate with tests.
- **Adopt the established infrastructure crates around it,** all current for Bevy 0.20
  and widely used:
  - `bevy_common_assets`: the catalog as a hot-reloading Bevy asset
  - `moonshine-save`: save and load
  - `bevy-inspector-egui`: inspecting inventories and catalogs while developing
- **Use Bevy's built-in picking for drag and drop** (inventory to rack slot, jack to
  jack, knobs). No crate needed, and mesh picking works on the 3D rack.
- **Factory logistics and timed production have no crate.** Write them ourselves.
  A few open-source Bevy factory games are useful as references only.

## Candidates

### Crafting and inventory

| Crate | Latest | Updated | Downloads | Bevy | Verdict |
| --- | --- | --- | --- | --- | --- |
| [`bevy_inventory`](https://docs.rs/bevy_inventory/latest/bevy_inventory/) | 0.1.0 | 2026-05 | 201 | `^0.18`, **optional** | Pure-Rust slot inventory (stacking, swap, capacity, serde snapshots) with optional Bevy glue. Its core would compile with any Bevy. Too young to depend on; a reference for slot semantics. |
| [`bevy_items`](https://docs.rs/bevy_items/latest/bevy_items/all.html) | 0.1.1 | 2026-08 | 60 | `^0.19` | `CraftingRecipe`/`CraftingIngredient` plus RPG items and professions on top of `bevy_inventory`. Wrong Bevy, wrong genre, unproven. |
| [`bevy_adventure`](https://docs.rs/bevy_adventure) | 0.6.0 | 2023-11 | 24k | `^0.12` | Point-and-click item combining. Stale. |
| [`bevy_mod_static_inventory`](https://crates.io/crates/bevy_mod_static_inventory) | 0.1.0 | 2023-09 | 1.6k | `^0.11` | Stale. |
| [`bevy-slotted`](https://github.com/JedimEmO/bevy-slotted) | git only | 2026-10 | n/a | 0.19.1 | Minecraft-style inventory, container and item-browser UI with a headless test harness. Apache-2.0, not on crates.io, no stars yet. Worth reading for slot-UI behaviour (click modes, drag-painting); code may be borrowed with attribution. |
| [`bevy_dragndrop`](https://github.com/cadyn/bevy_dragndrop) | 0.2.0 | 2023-11 | 4.7k | `^0.12` | Superseded by Bevy's own picking. |

Why ours stays: our model is a factory builder's, not an RPG's. It has waveform items
with signal uses, exhaustible resources, infinite mines with productivity factors,
replenishment policies on the shared calendar, harvest instance IDs, and soon rack
configuration recipes. None of the crates above has those ideas, and adapting one
would cost more than the ~750 lines we already own.

### Data: the catalog as an asset

| Crate | Latest | Bevy | Use |
| --- | --- | --- | --- |
| [`bevy_common_assets`](https://github.com/NiklasEi/bevy_common_assets) | 0.18.0 (2026-10) | `^0.20` | Load `assets/recipes/catalog.json` (and machine profiles) as typed Bevy assets, with hot reload while tuning recipes. JSON, RON, TOML and more. 354k downloads. |
| [`bevy_asset_loader`](https://docs.rs/bevy_asset_loader) | 0.28.1 (2026-10) | `^0.20` | Loading states: wait for the catalog, models and profiles before entering play. 675k downloads. Optional; adopt when startup loading grows. |

`stubby_recipes::Catalog::from_json` stays the validator. The asset loader hands it
the bytes, so the crate stays Bevy-free.

### Save and load

| Crate | Latest | Bevy | Verdict |
| --- | --- | --- | --- |
| [`moonshine-save`](https://docs.rs/moonshine-save) | 0.8.0 (2026-10) | `^0.20` | **Adopt** when saving lands. Reflect-based save/load of marked entities and resources, triggered by events or observers, to files or streams. Fits our design: inventory, harvest ledger, recipe book, rack configurations, placed-prefab state. |
| [`bevy_save`](https://crates.io/crates/bevy_save) | 2.0.1 (2025-08) | `^0.16.1` | Behind Bevy; skip. |

### UI and interaction

- **Bevy built-in picking (`bevy_picking`):** pointer events (hover, press, drag,
  drag-and-drop) on UI nodes and, with mesh picking, on 3D meshes. This covers dragging
  a VCO from the inventory into a rack slot, dragging cable ends between jacks and
  turning knobs on the 3D rack screen, with no extra crate.
- **[`bevy_egui`](https://crates.io/crates/bevy_egui)** 0.43.1 and
  **[`bevy-inspector-egui`](https://crates.io/crates/bevy-inspector-egui)** 0.38.0
  (both Bevy `^0.20`): developer tools only, such as a live inventory/catalog
  inspector and debug panels. The player-facing screens stay in Bevy UI and the 3D
  rack, matching `docs/art/hud_mock.png`.
- **[`bevy_tweening`](https://crates.io/crates/bevy_tweening)** 0.16.0 needs Bevy
  `^0.19`. Not yet usable for UI motion; wait for a 0.20 release or animate directly.

### Factory logistics and timed production

No crate provides machines with input/output buffers, production cycles or belt
(stream) transport. That is the core of the game and ours to write. For reading:

- [GRIDLOCK](https://github.com/Insufficient-Charge/bevy-classic-factory-game): a Bevy
  factory game with handcrafting (`crafting.rs`) and multi-layer conveyors
  (`logistics.rs`). No license is declared, so read it, don't copy it.
- [bevy-factory-game](https://github.com/someguynamedjosh/bevy-factory-game) and
  [bevysfactory](https://github.com/afonsolage/bevysfactory) (stale since 2023): older
  examples of grid factories in Bevy.
- [bevy_survival_crafting_game](https://github.com/mwbryant/bevy_survival_crafting_game):
  loads crafting recipes from RON at runtime, the same data-driven pattern we already
  follow.

## Recommendation

| Need | Use |
| --- | --- |
| Catalog, items, recipes, inventory, crafting | Keep `stubby_recipes` (ours) |
| Catalog and profiles as hot-reloading assets | `bevy_common_assets` 0.18 |
| Startup loading | `bevy_asset_loader` 0.28, when needed |
| Save/load | `moonshine-save` 0.8 |
| Drag and drop, rack knobs, jacks, cables | Bevy built-in picking (+ mesh picking) |
| Developer inspection | `bevy-inspector-egui` 0.38 (dev builds) |
| Machine production, stream logistics | Ours; see the references above |

**Found in passing (outside crafting):** [`bevy_hanabi`](https://crates.io/crates/bevy_hanabi)
0.19.0, the stack's choice for sand particles in AGENTS.md, requires Bevy `^0.19`.
There is no 0.20 release yet (checked 2026-11-01), so `add-desert-opening` 3.3 needs its
planned CPU fallback, or a wait for a hanabi release, before the sandstorm lands.

Per AGENTS.md, check each crate's Bevy compatibility again before adding it, and pin
exact minor versions. Adoption is Codex's call per task. The likely first one is
`bevy_common_assets`, for hot-reloading the catalog while tuning yields (the reed and
bug yields under discussion).
