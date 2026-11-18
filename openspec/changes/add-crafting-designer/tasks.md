## 1. Editing model (`stubby_recipes`, no Bevy)

- [x] 1.1 `rules::edit::RulesDocument`: load a rules directory; `Add`, `Set`, `Remove` edits on any registry, each recording its inverse
- [x] 1.2 Validate the whole rule set after each edit, reporting every error with registry, ID and field
- [x] 1.3 Undo and redo
- [x] 1.4 `Rename` updates every reference and adds a `migrations.json` row; removing a locked ID requires a target or retirement
- [x] 1.5 Save through `canonical_files` and `updated_ids_lock`, writing only changed files; refuse while invalid
- [x] 1.6 Where-used query over all registries
- [x] 1.7 Tests: add a recipe; dangling reference reported; undo restores; rename writes a migration; saved output passes `rules-fmt --check`

## 2. Schema annotations (Rule PR)

- [x] 2.1 Mark every ID-reference field with `x-stubby-ref: <registry>` through schemars
- [x] 2.2 Run `rules-fmt` to regenerate `schema/`; `cargo test -p stubby_recipes` passes

## 3. The web designer (amended 2026-11-16; claimed: claude)

- [x] 3.1 Confirm how a project-local Jackdaw extension is built and installed (findings in design.md: separate project, signed bundle beside the game, no text input in the extension UI kit). Led to the web designer.
- [x] 3.2 `crates/stubby_rules_server`: `tiny_http` on 127.0.0.1 over one `RulesDocument`; JSON API for registries and row schemas, rows, edits (`Edit` deserialised), undo, redo, save, problems, where-used; page and vendored `json-editor` embedded
- [x] 3.3 Page: registries and rows, search by ID and name, add and remove
- [x] 3.4 Row form from the schema with `json-editor`; `x-stubby-ref` fields as choices of existing IDs
- [x] 3.5 Problems list; selecting a problem selects its row
- [x] 3.6 Where-used for the selected row
- [x] 3.7 Undo, redo and save (Ctrl+Z, Ctrl+Shift+Z, Ctrl+S); refuse to save while invalid; reload from disk when files change and nothing is unsaved
- [x] 3.8 Live panel: dev builds of the game add `bevy_remote` (127.0.0.1, CORS); the page polls `world.get_resources` for `BasinAir`, `CameraLightQuality` and `Calendar`
- [x] 3.9 `deny.toml`: the server is a tool (no Bevy, no game); licenses allowed

## 4. Playtest

- [x] 4.1 `bevy_stubby_recipes`: dev builds load rules from `assets/recipes/` and hot-reload on change, keeping the old rules and reporting errors on a failed reload; release builds stay embedded (claimed: claude; checked live: editing `air.json` during Play reloads within a second)
- [x] 4.2 Smoke test: a rules edit reaching a running game (done 2026-11-18 through `add-machine-rules` 4.2: a `machines.json` edit mid-run reloaded and changed the campfire's targets live; the bug-yield variant is the same reload path)

## 5. Docs

- [x] 5.1 `assets/recipes/README.md`: designing with the web designer; hand edits still work
- [x] 5.2 AGENTS.md: the designer and the editing model in the rules section and repo map

## 6. Later

- [ ] 6.1 A Jackdaw extension front end over the same model, if the extension UI kit gains a text input upstream
- [ ] 6.2 Recipe graph in the page once recipes exist
