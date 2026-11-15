## 1. Editing model (`stubby_recipes`, no Bevy)

- [ ] 1.1 `rules::edit::RulesDocument`: load a rules directory; `Add`, `Set`, `Remove` edits on any registry, each recording its inverse
- [ ] 1.2 Validate the whole rule set after each edit, reporting every error with registry, ID and field
- [ ] 1.3 Undo and redo
- [ ] 1.4 `Rename` updates every reference and adds a `migrations.json` row; removing a locked ID requires a target or retirement
- [ ] 1.5 Save through `canonical_files` and `updated_ids_lock`, writing only changed files; refuse while invalid
- [ ] 1.6 Where-used query over all registries
- [ ] 1.7 Tests: add a recipe; dangling reference reported; undo restores; rename writes a migration; saved output passes `rules-fmt --check`

## 2. Schema annotations (Rule PR)

- [ ] 2.1 Mark every ID-reference field with `x-stubby-ref: <registry>` through schemars
- [ ] 2.2 Run `rules-fmt` to regenerate `schema/`; `cargo test -p stubby_recipes` passes

## 3. The Jackdaw extension

- [ ] 3.1 Confirm how a project-local extension is built and installed as a `.jdext` against the shared editor; record the steps in the crate README (an editor gap is a fork fix)
- [ ] 3.2 Create `crates/stubby_crafting_designer` (`cdylib` + `rlib`; `stubby_recipes`, Bevy, Jackdaw extension crates only) with a **Crafting** menu
- [ ] 3.3 Rules window: registries, rows, search by name and ID, add and remove
- [ ] 3.4 Row form built from the schema: numbers with bounds, enums, lists, ID pickers from `x-stubby-ref`
- [ ] 3.5 Problems window; selecting a problem selects its row and field
- [ ] 3.6 Where-used panel for the selected item
- [ ] 3.7 Recipe graph window on `jackdaw_node_graph`: items and recipes as nodes, inputs to outputs; selecting a node selects its row
- [ ] 3.8 Undo and redo while the designer has focus; save with Ctrl+S; reload from disk when files change and nothing is unsaved
- [ ] 3.9 Check: `cargo tree -p stubby_crafting_designer` shows no game crate

## 4. Playtest

- [ ] 4.1 `bevy_stubby_recipes`: dev builds load rules from `assets/recipes/` and hot-reload on change, keeping the old rules and reporting errors on a failed reload; release builds stay embedded
- [ ] 4.2 Smoke test: change the lightning bug's yield in the designer during Play and collect a bug

## 5. Docs

- [ ] 5.1 `assets/recipes/README.md`: designing with the Crafting menu; hand edits still work
- [ ] 5.2 AGENTS.md: the designer and the editing model in the rules section and repo map
