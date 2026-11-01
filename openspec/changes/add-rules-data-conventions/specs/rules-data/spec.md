## ADDED Requirements

### Requirement: Rules are data rows in code-owned schemas
Game rules SHALL be JSON files under `assets/recipes/`, one per domain, each with a
`format` version, interpreted by `stubby_recipes`. A change that only adds or edits
rows of existing types SHALL need no code or specification change. Rows SHALL NOT
contain conditions, formulas or scripts.

#### Scenario: A new recipe
- **WHEN** the designer adds a recipe row using existing items to `recipes.json`
- **THEN** the game loads it and offers the recipe with no code change

#### Scenario: A row that needs logic
- **WHEN** a goal needs a condition the schema cannot express
- **THEN** it is proposed as a new rule type in an OpenSpec change, not written into the row

### Requirement: Loading is strict
Loading SHALL reject unknown fields, IDs that are not `snake_case` `[a-z0-9_]`
starting with a letter, an ID used twice in any registry, a reference to an ID that
does not exist, zero quantities, empty item lists, prerequisite cycles and recipes
whose inputs nothing can produce.

#### Scenario: Misspelled field
- **WHEN** a resource row says `"yeilds"` instead of `"yields"`
- **THEN** loading fails naming the file, the resource and the field

#### Scenario: Shared ID
- **WHEN** an item and a resource both use the ID `clock`
- **THEN** loading fails naming both

#### Scenario: Uppercase ID
- **WHEN** an item's ID is `Square-Wave`
- **THEN** loading fails naming the ID

### Requirement: Errors say where, all at once
Validation SHALL report every problem found in one run, each with the file, the ID,
the field and a message.

#### Scenario: Two mistakes
- **WHEN** one file has an unknown item reference and another an invalid ID
- **THEN** both are reported in one run

### Requirement: Files stay canonical
Each rules file SHALL hold one object per entry, sorted by `id`, in the canonical
pretty-printed form, and a test SHALL fail on a file that is not.

#### Scenario: Out of order
- **WHEN** a row is appended at the end out of ID order
- **THEN** the format test fails and names the command that fixes it

### Requirement: IDs are save API
Renaming or removing a rule ID SHALL require a row in `assets/recipes/migrations.json`.
Validation SHALL fail when a renamed or removed ID still exists, a migration's target
does not, or an ID disappears without a migration. Loading a save SHALL apply its
pending migrations before restoring progress, and SHALL drop progress for IDs that no
longer exist, logging each.

#### Scenario: Rename
- **WHEN** resource `clock` is renamed `clock_source` with a migration and an old save holds progress for `clock`
- **THEN** the progress is restored under `clock_source`

#### Scenario: Rename without migration
- **WHEN** an ID present in the previous rules is gone and no migration names it
- **THEN** validation fails naming the ID

### Requirement: The shipped rules are tested
The test suite SHALL load the real files in `assets/recipes/` and fail on any
validation error, and SHALL run a seeded economy simulation that proves every authored
goal can be completed from the world's resources within a bound.

#### Scenario: Unreachable goal
- **WHEN** a goal requires an item no resource, mine or recipe can produce
- **THEN** the simulation test fails naming the goal and the item

### Requirement: A failed reload keeps the old rules
When rules are reloaded while the game runs, a set that fails validation SHALL leave
the previous rules in effect and show the errors.

#### Scenario: Bad edit during play
- **WHEN** the designer saves a rules file with an error while the game is running
- **THEN** play continues on the previous rules and the errors are shown
