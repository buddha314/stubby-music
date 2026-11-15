## Context

Rules are seven domain files under `assets/recipes/` (`items`, `resources`, `mines`,
`recipes`, `buildables`, `goals`, `start`) plus `migrations.json` and `ids.lock`.
`stubby_recipes::rules` already loads them strictly (`Rules::load`), validates and
reports every error (`RulesErrors`), writes canonical files (`canonical_files`),
updates `ids.lock` (`updated_ids_lock`) and generates JSON Schemas (`schemas()`, via
schemars). `rules-fmt` is a thin CLI over that. What is missing is a way to *design*
with it that is not a text editor. `recipes.json` is still empty.

Jackdaw (shared fork at `~/src/jackdaw`, 82018644) has a public extension API
(`jackdaw_api` / `jackdaw_extension`): `JackdawExtension::register` adds dock windows
(`WindowDescriptor`), menus, operators and key bindings; extensions build as
`cdylib` and are installed as signed `.jdext` libraries sharing the editor's SDK
dylib. It also has a domain-agnostic node graph canvas (`jackdaw_node_graph`), used by
the animation editor. Editor policy forbids changing the editor for the game, but
extension points are exactly what it allows.

## Goals / Non-Goals

**Goals:**
- Designers never need to read or write JSON, IDs by hand, or `ids.lock`.
- One editing model that any front end uses, so the rules stay validated the same way
  everywhere.
- A rule-type change in `stubby_recipes` reaches the designer without designer code.
- Change a rule, see it in Play in about a second.

**Non-Goals:**
- An in-game designer (later, over the same model).
- Designing racks, module definitions or world placement (other tools, other changes).
- Replacing JSON as storage, or Jackdaw's own asset-kind editing for these files.
- Balancing tools (rate calculators, production-chain solvers). The graph is a view.

## Decisions

### Jackdaw extension first, in-game later
Jackdaw is where levels and prefabs are already designed, and it gives us docking,
menus, operators, an inspector-style UI kit and the node graph for free. An in-game
tool would have to build all of that in Bevy UI first, and is only usable while
playing. The in-game view is still worth having for tuning while playing, and the
editing model makes it cheap later.
*Alternative:* in-game first. Rejected for now: more UI to build, and less of the
editor's tooling.

### A Bevy-free editing model in `stubby_recipes`
`rules::edit::RulesDocument` wraps the loaded files and applies `Edit` values
(`Add`, `Set { registry, id, field_path, value }`, `Remove`, `Rename`). Each applied
edit records its inverse for undo. After each edit it re-runs the existing
validation; on save it uses `canonical_files` and `updated_ids_lock`. Rows are held as
`serde_json::Value` checked against their typed definitions, so the model handles
every registry generically and is ready for new ones.
*Alternative:* let the extension edit typed Rust structs directly. Rejected: every
rule type would need designer code, and a second front end would repeat it.

### Forms from the JSON Schema, IDs annotated
The designer walks the schema `rules-fmt` already generates to build forms. To turn
strings into pickers, ID fields gain a schemars extension, `"x-stubby-ref": "<registry>"`
(for example `Stack.item` → `item`). That is a Rule PR in `stubby_recipes` (schema
code, regenerated `schema/`), not designer code, and it also gives editors that read
the schema autocomplete for free.

### Undo inside the designer, not the scene history
Rules edits do not change the scene, so they do not go through scene operators or
`op::with_history_span`. The designer keeps the model's own undo stack and binds
Ctrl+Z/Ctrl+Shift+Z while its windows have focus. If Jackdaw's extension API cannot
scope a binding to a window, that is an extension-API bug to fix in the fork, not a
reason to change the design.

### Playtest by hot reload in dev builds only
`bevy_stubby_recipes` reads the rules from `assets/recipes/` in dev builds and watches
them; on change it reloads through `Rules::load`, swaps the catalog, and on failure
keeps the old rules and logs every error. Release builds keep the embedded rules, so
shipped behaviour is unchanged. This is also the `bevy_common_assets` direction in
`docs/research/crafting-and-inventory-crates.md`.

### The designer is a tool, outside the game's layers
`stubby_crafting_designer` depends on `stubby_recipes`, Bevy and Jackdaw's extension
crates. It never depends on the game, so it builds and loads in the editor whether
or not the game compiles, and the game never depends on it.

## Risks / Trade-offs

- [Extension ABI must match the shared editor exactly] → build it with
  `tools/jackdaw.sh prepare` against the same SDK; `tools/jackdaw.sh sync` rebuilds it
  with the editor.
- [Jackdaw's UI kit lacks a widget the forms need (e.g. a searchable picker)] → build
  it in the extension from the kit's parts; fix only genuine editor bugs in the fork.
- [Two writers: the designer and a hand edit or an agent at the same time] → the model
  reloads when files change on disk while it has no unsaved edits, and refuses to save
  over a newer file, showing both versions.
- [`serde_json::Value` rows lose type safety inside the model] → every edit is
  validated against the typed definitions immediately; the typed loader stays the
  only way the game reads rules.

## Migration Plan

Additive. Hand editing and `rules-fmt` keep working; files written by either are
identical. Rollback is disabling the extension.

## Open Questions

- Installing a project's own extension: does `jd` install a local `cdylib` as a
  `.jdext` for development without publishing, and does signing allow a local key?
  (Task 3.1 confirms; if not, the gap goes to the fork as an extension-API fix.)
- Should goals get their own tree view (long-term goals over short-term chores),
  or is the generic form enough at first? (Proposed: generic first.)
