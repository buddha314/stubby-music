## ADDED Requirements

### Requirement: Rules are edited through a typed model
`stubby_recipes` SHALL provide an editing model that loads the rules directory and
applies typed edits (add, change and remove a row in any registry) without a Bevy
dependency. Every front end SHALL change rules only through this model.

#### Scenario: Add a recipe
- **WHEN** an edit adds a test recipe that takes one `square_wave` and makes one existing item
- **THEN** the model holds the new row and reports it valid

### Requirement: Validation is live and complete
After every edit the model SHALL validate the whole rule set and report every error
with registry, ID and field, as loading does.

#### Scenario: A dangling reference
- **WHEN** an edit removes item `square_wave` while a recipe consumes it
- **THEN** the model reports the recipe, the `inputs` field and the missing item, and keeps the edit so it can be undone or fixed

### Requirement: Every edit can be undone
Each edit SHALL be undoable and redoable, in order, back to the loaded state.

#### Scenario: Undo a removal
- **WHEN** a removed item is restored with undo
- **THEN** the rules equal those before the removal and validation reports what it reported then

### Requirement: Renames keep saves loading
Renaming an ID SHALL update every reference to it and SHALL add the matching row to
`migrations.json`. Removing an ID that is in `ids.lock` SHALL require a migration
target or an explicit retirement, as the conventions require.

#### Scenario: Rename an item
- **WHEN** item `square_wave` is renamed to `square_pattern`
- **THEN** every recipe, mine and goal naming it uses the new ID and `migrations.json` maps the old ID to the new one

### Requirement: Saving is canonical
Saving SHALL write only the rules files that changed, in the same canonical form
`rules-fmt` writes, and SHALL append new IDs to `ids.lock`. Saving SHALL refuse while
the rules are invalid, naming the errors.

#### Scenario: Tool and formatter agree
- **WHEN** rules saved by the editing model are then run through `rules-fmt --check`
- **THEN** it reports nothing to change
