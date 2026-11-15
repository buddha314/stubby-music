## ADDED Requirements

### Requirement: Design rules in Jackdaw
The crafting designer SHALL be a Jackdaw extension built in this repository, adding a
**Crafting** menu and dockable windows to the editor without changing the editor.
It SHALL open the rules of the open project's `assets/recipes/`.

#### Scenario: Open the designer
- **WHEN** the designer extension is enabled and the stubby-music project is open
- **THEN** Crafting > Rules opens a window listing the items, resources, mines, recipes, buildables, goals and start of the project

### Requirement: Rows are edited in forms, not text
Selecting a row SHALL show a form with one field per schema property: numbers as
number fields with their bounds, enums as choices, lists as addable rows, and every
ID reference as a picker of the IDs its registry holds. The designer SHALL build the
form from the generated JSON Schema, so a new field needs no designer code.

#### Scenario: Pick an ingredient
- **WHEN** the designer adds an input to a recipe
- **THEN** the item field offers only existing item IDs, searchable by name and ID

#### Scenario: A new rule field appears
- **WHEN** a Rule PR adds a field to recipes and regenerates the schema
- **THEN** the recipe form shows the field without a change to the designer crate

### Requirement: Problems are shown where they are
The designer SHALL show the editing model's validation as a problems list, and
selecting a problem SHALL select its row and field.

#### Scenario: Jump to a problem
- **WHEN** a recipe names a removed item and the designer clicks that problem
- **THEN** the recipe is selected and its input field is marked

### Requirement: Where used and the recipe graph
For any item the designer SHALL list what produces and consumes it (resources,
mines, recipes, buildables, goals). A recipe graph window SHALL draw items and recipes
as nodes on Jackdaw's node graph canvas, wired from inputs to outputs, and selecting a
node SHALL select its row.

#### Scenario: What uses square waves
- **WHEN** the designer opens where-used for `square_wave`
- **THEN** it lists the lightning bug resource that yields it and every recipe and goal that consumes it

### Requirement: Save and playtest
Saving SHALL write through the editing model. In a dev build, a running Play SHALL
reload the rules from disk after a save, and a reload that fails validation SHALL keep
the previous rules and report the errors.

#### Scenario: Retune a yield during Play
- **WHEN** the lightning bug's yield is changed from 2 to 3 and saved while Play is running
- **THEN** the next bug collected in Play yields 3 square waves

### Requirement: The designer stands apart from the game
The designer crate SHALL depend only on `stubby_recipes`, Bevy and Jackdaw's extension
crates, and SHALL NOT depend on `stubby_game` or any game system crate.

#### Scenario: Built alone
- **WHEN** `cargo tree -p stubby_crafting_designer` is listed
- **THEN** it shows no `stubby_game`, `bevy_stubby_recipes` or other game crate
