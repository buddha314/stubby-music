## Why
Our crafting system should be a **Bevy community plugin**: something a Bevy game adds
with `app.add_plugins(...)`, the way games add `bevy_rapier3d` or `bevy_ecs_ldtk`.
Today it is not. The rules model in `stubby_recipes` is Bevy-free, but every piece of
Bevy integration (the inventory and catalog resources, the collect and craft
messages, the systems, rules loading) is hand-wired inside the game, in
`stubby_game::resources`, mixed with game-only HUD and lightning-bug code.

Bevy plugins that last share one structure: a core crate with no engine dependency,
plus a thin `bevy_` plugin crate in the same repository, versioned separately (Rapier
and `bevy_rapier3d`; `pathfinding`; `hecs`). Plugins whose core depends on Bevy are
rewritten at every Bevy release (every 3 to 5 months) and many go stale (big-brain,
archived 2025; Yarn Spinner's Rust core, still without a Bevy 0.20 release weeks
after it). No engine ships a crafting system and no engine-independent crafting
library is mature, so a well-shaped one fills a gap.

Research: `docs/research/crafting-system-library-maturity.md` (landing with this
change), `docs/research/crafting-and-inventory-crates.md`.

## What Changes
- **New crate `bevy_stubby_recipes`**, the plugin: `RecipesPlugin` loads the rules as
  a Bevy asset with hot reload (a failed reload keeps the old rules), and provides the
  inventory, recipe book and harvest ledger as components, the collect, craft and
  (later) deliver messages and the systems that apply them through the core.
- **`stubby_recipes` stays Bevy-free** and holds every rule: files, validation,
  inventory, recipe book, crafting, goals, migrations. Its tests need no Bevy.
- **`stubby_game` keeps only game-specific parts**: HUD, prompts, the reticle,
  lightning-bug and burbler behaviour, Tab and goals screens. It adds the plugin.
- **Game-specific data leaves the core's types** before any publication: waveforms
  and signal uses become the game's item metadata, carried by a generic extension the
  core does not interpret.
- **Publishing is a later, separate decision** (its own repository, crates.io, a
  distinctive name, MIT OR Apache-2.0); this change shapes the crates so that move is a
  `git filter-repo`, not a rewrite.

## Capabilities
### New Capabilities
- `recipes-plugin`: the crafting system as a Bevy plugin over a Bevy-free core.

## Impact
New `crates/bevy_stubby_recipes`; code moved out of `stubby_game::resources` (and its
users in `prefabs.rs`, `review.rs`, `prefab_review.rs`, `waveform_audio.rs`);
`stubby_recipes` item metadata; workspace `Cargo.toml`; AGENTS.md repo map. Codex
owns the implementation (`crates/`). Builds on `add-rules-data-conventions` and comes
before the goal systems of `add-inventory-and-goals-hud` (2.x), which then live in the
plugin.
