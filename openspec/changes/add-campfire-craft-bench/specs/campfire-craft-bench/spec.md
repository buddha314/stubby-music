## ADDED Requirements

### Requirement: The camp is a buildable placed unlit
The camp (a campfire in a ring of four stones with a lean-to open toward it) SHALL be a
buildable laid with Q from reeds, at the cost its `buildables.json` row sets. It SHALL
be placed unlit.

#### Scenario: Building a camp
- **WHEN** the player places a camp holding at least its reed cost
- **THEN** the reeds are spent and an unlit camp stands where it was placed

#### Scenario: Not enough reeds
- **WHEN** the player holds fewer reeds than the camp costs
- **THEN** the hologram shows invalid and nothing is placed

### Requirement: The unlit camp is a rack
E at an unlit camp SHALL open its rack, defined by its profile's `configure` block, with
a place for a voice (accepting `square_wave`) and a place for a sequencer (accepting
`snail_shell`), and the rack SHALL play only the modules and cables the player sees.

#### Scenario: Opening the camp's rack
- **WHEN** the player presses E at an unlit camp
- **THEN** the camp's rack opens with its voice and sequencer places, and Escape closes it

#### Scenario: Nothing hidden
- **WHEN** the camp's rack has no cables patched
- **THEN** no voice sounds and no strike reaches the fire

### Requirement: A backbeat lights the fire
The fire SHALL catch after one full bar, timed by the audio clock, in which the
sequencer's gate struck the spark voice on the steps the camp's `light_pattern` sets
(beats 2 and 4) and on no others. Each strike SHALL show sparks at its step's stone at
the strike's heard instant.

#### Scenario: Lighting with the backbeat
- **WHEN** the player patches clock, sequencer and voice to the fire, lights steps 2 and 4, and a full bar plays
- **THEN** the fire catches and the `fire_lit` condition is reported once

#### Scenario: The wrong rhythm
- **WHEN** a full bar plays with steps 1 and 3 lit
- **THEN** sparks fly at stones 1 and 3 and the fire does not catch

#### Scenario: Sparks are on the beat
- **WHEN** a strike is heard
- **THEN** its sparks appear at that step's stone at the strike's heard instant, not at the frame it was scheduled

### Requirement: The lit camp is the craft bench
E at a lit camp SHALL open the crafting screen, laid out as Satisfactory's Craft Bench:
a recipe list on the left, the selected recipe's inputs (`have / need`) and output on the
right, and a craft button. It SHALL craft only through `CraftRequest`.

#### Scenario: Crafting a recipe
- **WHEN** the player selects a recipe whose inputs they hold and presses craft
- **THEN** one craft is made, its inputs are consumed and its outputs added

#### Scenario: Missing inputs
- **WHEN** the player selects a recipe they lack inputs for
- **THEN** the missing inputs show `have / need` short of the need and crafting takes nothing

#### Scenario: Pinning from the bench
- **WHEN** the player pins the selected recipe with a count
- **THEN** it appears in the hub's To-Do List with its ingredient totals

### Requirement: Lighting the camp completes "Build a camp"
The `fire_lit` condition SHALL be a goal condition kind validated by `Rules::load`, and
"Build a campfire" SHALL be the last step of the "Build a camp" major goal.

#### Scenario: Last step
- **WHEN** the camp's fire catches while "Build a campfire" is open
- **THEN** the step completes, and with it the major goal "Build a camp"
