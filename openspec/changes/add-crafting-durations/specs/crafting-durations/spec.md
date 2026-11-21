## ADDED Requirements

### Requirement: Recipes say how they are made and how long it takes
A recipe row SHALL allow at least one way to make it: `by_hand` with a duration in seconds,
and/or `crew` with a profession, a number of Dancers and a duration in seconds of world
time. A recipe with neither, or a duration that is not positive, SHALL fail to load.

#### Scenario: A recipe with no way to make it
- **WHEN** a recipe row has neither `by_hand` nor `crew`
- **THEN** loading the rules reports that recipe and field as an error

### Requirement: Crafting by hand fills a bar while the button is held
At a crafting station, holding the craft button on a recipe made by hand SHALL fill a
progress bar in 0.25 s steps over its duration in real time; when it fills, the inputs SHALL
be taken and the outputs given together. Letting go SHALL pause the bar and take nothing.
Space SHALL latch the button held until pressed again or the screen closes.

#### Scenario: An early recipe
- **WHEN** the player holds Craft on a 0.75 s recipe they can afford
- **THEN** the bar fills in three steps, and after 0.75 s the inputs are gone and the output is in the inventory

#### Scenario: Let go early
- **WHEN** the player lets go halfway
- **THEN** nothing is taken, and holding again resumes from halfway

### Requirement: Holding on repeats the craft
While the button stays held, a finished hand craft SHALL start the next at once while the
inputs last, counting the crafts made.

#### Scenario: Out of inputs
- **WHEN** the inputs run out during a hold
- **THEN** the bar stops empty after the last craft that could be paid for

### Requirement: Crafts can be delegated to Dancers
At a station in an established village, a recipe with a crew SHALL be queueable for
Dancers; each queued craft SHALL run in world time once its crew and inputs are reserved,
showing its progress on the station's screen and above the station in the world, and SHALL
deliver its outputs once.

#### Scenario: Delegated while away
- **WHEN** the player queues a crewed recipe and walks away
- **THEN** the job runs at world speed and its outputs are delivered when its time is up

#### Scenario: Not enough Dancers
- **WHEN** a queued craft needs four Dancers of its profession and two are free
- **THEN** it waits, the screen shows two missing, and nothing is taken
