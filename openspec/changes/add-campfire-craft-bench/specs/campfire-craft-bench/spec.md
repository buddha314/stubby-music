## ADDED Requirements

### Requirement: The camp is a buildable placed unlit
The camp (a bundle of sticks leaned into a teepee over a bed of ash) SHALL be a
buildable laid with Q from reeds, at the cost its `buildables.json` row sets (25 reeds).
It SHALL be placed unlit. A lit fire SHALL NOT change air or light quality.

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
sequencer's gate struck the spark voice on the steps the camp's targets set
(beats 2 and 4) and on no others. Each strike SHALL throw sparks from the bundle at the
strike's heard instant.

#### Scenario: Lighting with the backbeat
- **WHEN** the player patches clock, sequencer and voice to the fire, lights steps 2 and 4, and a full bar plays
- **THEN** the fire catches and the `fire_lit` condition is reported once

#### Scenario: The wrong rhythm
- **WHEN** a full bar plays with steps 1 and 3 lit
- **THEN** sparks fly on beats 1 and 3 and the fire does not catch

#### Scenario: Sparks are on the beat
- **WHEN** a strike is heard
- **THEN** its sparks burst from the bundle at the strike's heard instant, not at the frame it was scheduled

### Requirement: The camp's beat comes from nearby water
The camp SHALL take its clock from the nearest running burbler or flowing stream within
the reach its profile sets (`clock_reach_m`, 5 m to start), and the reach SHALL be data,
not code. The fire SHALL burn only while that clock runs.

#### Scenario: No water in reach
- **WHEN** no running burbler or flowing stream is within the camp's reach
- **THEN** its sequencer gets no clock, nothing strikes and the fire cannot catch

#### Scenario: The beat stops
- **WHEN** a burning camp's burbler stops sounding
- **THEN** the fire dies down to embers, and the `fire_lit` goal stays complete

#### Scenario: Tuning the reach
- **WHEN** a designer changes `clock_reach_m` in the camp's profile
- **THEN** the camp takes its beat from water within the new reach, with no code change

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

### Requirement: Recipes are made at a tempo
A recipe with a BPM range SHALL be made only at a tempo inside it. The range is
`bpm: {min, max}` (whole BPM, multiples of 24); the tempo is the bench's clock's (the
nearest running burbler or the stream it feeds, within `clock_reach_m`). Off tempo, it SHALL
take nothing and tell the player the range and the tempo heard.
Tempos SHALL be on the 24 BPM grid for now: the burbler's BPM knob clicks through
multiples of 24 and new burblers seed on it.

#### Scenario: In tempo
- **WHEN** the player crafts a 96-120 BPM recipe at a camp whose stream runs at 120 BPM
- **THEN** it is made

#### Scenario: Off tempo
- **WHEN** the same recipe is crafted at a camp whose clock runs at 144 BPM
- **THEN** nothing is taken and the player is told it is made at 96-120 BPM and the clock is at 144

#### Scenario: Setting the stream
- **WHEN** the player turns the burbler's BPM knob
- **THEN** it clicks through multiples of 24, and its streams carry the new tempo to the camp

#### Scenario: A bad range
- **WHEN** a recipe row sets `bpm` to 100-130
- **THEN** loading fails naming the recipe and its `bpm` field: ranges are multiples of 24

### Requirement: Building the camp is the first step
The player SHALL start with enough reeds for one campfire (`start.json`), and "Build a
campfire" SHALL be the first step of the "Build a camp" major goal, met by the `built`
condition (validated by `Rules::load`) when the player builds one with the build menu
(user, 2026-11-18). The opening scene SHALL have no campfire placed.

#### Scenario: First step
- **WHEN** a new game starts and the player builds a campfire with Q
- **THEN** 25 reeds are spent and "Build a campfire" completes, opening the next step

### Requirement: Lighting the camp completes "Build a camp"
The `fire_lit` condition SHALL be a goal condition kind validated by `Rules::load`, and
"Light the fire" SHALL be the last step of the "Build a camp" major goal.

#### Scenario: Last step
- **WHEN** the camp's fire catches while "Light the fire" is open
- **THEN** the step completes, and with it the major goal "Build a camp"
