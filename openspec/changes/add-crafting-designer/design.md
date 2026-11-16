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

### A local web designer first (amended 2026-11-16)
The user asked for a rules editor to iterate on gameplay, preferred a tool beside the
game over an in-game panel, and chose a small local HTML server after this research:

- **Jackdaw extension** (the original plan): extensions get only buttons, cards, field
  rows and scrub chips from the editor's UI kit, no text input, and an extension loads
  beside the open game project only as a signed `.jdext` bundle (publisher key, trust
  prompt, reinstall on every change). Exposing a text field to extensions would be an
  editor feature, which the editor policy rules out. Deferred.
- **In-game panel**: every form hand-built in Bevy UI, usable only while playing.
- **Spreadsheet tools (Depot, CastleDB)**: their own file formats; we would lose the
  canonical rules files, `rules-fmt` and the validator.
- **VS Code on the JSON**: already works (every file names its `$schema`), but it is
  not a designer: no ID pickers across files, no where-used, no undo across files.
- **Local web designer** (chosen): the schemas already exist and `json-editor` renders
  forms from JSON Schema with no framework; the editing model already validates, undoes
  and saves; the game already hot-reloads. Least new code, usable while the game runs.

### The server: `tiny_http` over the editing model
`stubby_rules_server` holds one `RulesDocument` behind a mutex and exposes it as a small
JSON API on `127.0.0.1` (never another interface): registries and their row schemas,
rows, edits (the model's own `Edit`, deserialised as sent), undo, redo, save,
problems and where-used. `tiny_http` (MIT/Apache) is a small synchronous server, right
for one user on localhost; `axum` would bring tokio for nothing this tool needs. The page
and `json-editor` are embedded with `include_str!`, so the binary is self-contained.

### Live game values through Bevy's remote protocol
Dev builds of the game add `bevy_remote` (`RemotePlugin`, `RemoteHttpPlugin` on
127.0.0.1:15702 with CORS for the page). The page calls `world.get_resources` once a
second for a list of resource type paths it shows; the resources derive `Reflect`.
Nothing new is written for it: it is Bevy's protocol, and later tools (an MCP bridge, a
graph of the basin's air) can use it the same way.

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
`stubby_rules_server` depends on `stubby_recipes` and its HTTP and serde crates. It
never depends on the game or Bevy, so it builds in seconds and runs whether or not the
game compiles, and the game never depends on it.

## Risks / Trade-offs

- [`json-editor` is in maintenance mode] → it is vendored at a fixed version and does
  one job (forms from a schema); its successor or a hand-written form can replace it
  behind the same API.
- [A local server is reachable by other local processes] → it binds 127.0.0.1 only and
  writes only the rules directory, through the editing model's checks.
- [Two writers: the designer and a hand edit or an agent at the same time] → the model
  reloads when files change on disk while it has no unsaved edits, and refuses to save
  over a newer file, showing both versions.
- [`serde_json::Value` rows lose type safety inside the model] → every edit is
  validated against the typed definitions immediately; the typed loader stays the
  only way the game reads rules.

## Migration Plan

Additive. Hand editing and `rules-fmt` keep working; files written by either are
identical. Rollback is not running the server.

## Open Questions

- Should goals get their own tree view (long-term goals over short-term chores),
  or is the generic form enough at first? (Proposed: generic first.)
- A recipe graph: draw it in the page later (a small graph library), once recipes exist.
