## ADDED Requirements

### Requirement: Design rules in a local web designer
The crafting designer SHALL be a local web tool built in this repository:
`stubby_rules_server` serves one page on 127.0.0.1 over the rules in `assets/recipes/`,
listing every registry (items, resources, mines, recipes, buildables, goals, air) and its
rows. It SHALL listen on the loopback interface only.

#### Scenario: Open the designer
- **WHEN** `cargo run -p stubby_rules_server` is running and its address is opened in a browser
- **THEN** the page lists the registries of `assets/recipes/` and the rows of the selected one

### Requirement: Rows are edited in forms, not text
Selecting a row SHALL show a form built from the generated JSON Schema: numbers as
number fields with their bounds, enums as choices, lists as addable rows, and every
ID reference (`x-stubby-ref`) as a choice of the IDs its registry holds. A new field
SHALL need no designer code.

#### Scenario: Pick an ingredient
- **WHEN** the designer adds an input to a recipe
- **THEN** the item field offers only existing item IDs

#### Scenario: A new rule field appears
- **WHEN** a Rule PR adds a field to a registry and regenerates the schema
- **THEN** that registry's form shows the field without a change to the designer

### Requirement: Problems are shown where they are
The designer SHALL show the editing model's validation as a problems list after every
edit, each naming its file, row and field, and selecting a problem SHALL select its row.

#### Scenario: Jump to a problem
- **WHEN** a recipe names a removed item and the designer clicks that problem
- **THEN** that recipe's form is shown

### Requirement: Where used
For any row the designer SHALL list where its ID is used across every registry.

#### Scenario: What uses square waves
- **WHEN** the designer opens where-used for `square_wave`
- **THEN** it lists the lightning bug resource that yields it and every recipe and goal that consumes it

### Requirement: Undo, save and playtest
Edits SHALL be undoable and redoable. Saving SHALL write through the editing model and
SHALL be refused while the rules are invalid. In a dev build, the running game SHALL
reload the rules from disk after a save, and a reload that fails validation SHALL keep
the previous rules and report the errors.

#### Scenario: Retune a yield during Play
- **WHEN** the lightning bug's yield is changed from 2 to 3 and saved while the game is running
- **THEN** the next bug collected yields 3 square waves

### Requirement: Live game values beside the rules
Dev builds of the game SHALL serve Bevy's remote protocol on 127.0.0.1, and the designer
SHALL show chosen game resources from the running game, refreshed about once a second,
and show that the game is not running when it cannot reach it.

#### Scenario: Watch the basin clear
- **WHEN** the game is running and the designer shows `BasinAir`
- **THEN** its `clean` value updates as foliage cleans the basin

### Requirement: The designer stands apart from the game
The designer SHALL depend only on `stubby_recipes` and its HTTP and serialisation
crates, and SHALL NOT depend on Bevy, `stubby_game` or any game system crate.

#### Scenario: Built alone
- **WHEN** `cargo tree -p stubby_rules_server` is listed
- **THEN** it shows no Bevy crate and no game crate
