## ADDED Requirements

### Requirement: The economy core has no Bevy dependency
The `stubby_economy` crate SHALL NOT depend on Bevy, directly or through another crate, and
SHALL take its rules and state as plain data.

#### Scenario: Checked in CI
- **WHEN** the local CI script checks the cores' dependency trees
- **THEN** `stubby_economy` reaches no Bevy crate

### Requirement: Checkpoints resolve in order, and a jump equals its steps
The economy SHALL process every calendar checkpoint crossed exactly once and in order, and
advancing several checkpoints at once SHALL give the same state as advancing them one by
one. It SHALL resume from the last processed checkpoint after loading.

#### Scenario: A day at high speed
- **WHEN** a settlement is advanced across one whole day in a single step
- **THEN** its stock, roster and shortfalls equal those from advancing each checkpoint of that day in turn

### Requirement: Rules produce before they consume
Within one checkpoint, producer and converter rules SHALL apply before consumer and upkeep
rules; a converter SHALL run only whole conversions it can afford; and a demand that stock
cannot meet SHALL leave the stock at zero and be reported as a deficit.

#### Scenario: Short of water
- **WHEN** twelve Dancers each need one unit of water at dawn and the settlement holds eight
- **THEN** water stock is zero and the dawn deficit records four units of water

### Requirement: Dancers are individuals
Each Dancer SHALL have a stable ID, a body shape and skin tone drawn from a seed, a home, a
profession, a current task and a local clock ID, all kept across saving and loading.
Population and profession counts SHALL be derived from the roster.

#### Scenario: Reloaded
- **WHEN** a game is saved and loaded
- **THEN** every Dancer has the same ID, appearance, home, profession and task as before

### Requirement: A cottage houses at most four Dancers, chosen at random
A dwelling SHALL house no more Dancers than its capacity; a cottage's capacity SHALL be four.
A new Dancer SHALL be housed in a dwelling with a free place chosen by a seeded random draw,
so the same seed and history house the same Dancers in the same dwellings.

#### Scenario: Two cottages, five Dancers
- **WHEN** five Dancers arrive at a settlement with two empty cottages
- **THEN** no cottage holds more than four, and running again with the same seed gives the same homes

### Requirement: Dancers arrive only to a settlement that can keep them
A Dancer SHALL arrive only while the settlement has a free home and its water and food
cover its next day's rations for everyone already there, and SHALL NOT arrive while it is
short.

#### Scenario: No room
- **WHEN** every dwelling is full
- **THEN** no Dancer arrives, however much water and food is stored

### Requirement: Shortfalls never kill
A missed ration SHALL be recorded per need and per Dancer; no Dancer SHALL die of a
shortfall. Prolonged shortfall MAY make Dancers leave.

#### Scenario: A dry day
- **WHEN** a settlement has no water for a whole day
- **THEN** every Dancer is still on the roster or has left, and the day's water deficit is recorded

### Requirement: Professions are set by count
Every Dancer SHALL start as a laborer. The player SHALL set the number of Dancers in each
profession, which SHALL NOT exceed the work places its buildings offer; builders SHALL be
drawn from laborers when there is building to do, up to the player's cap. Each Dancer SHALL
have exactly one task at a time.

#### Scenario: More asked than places
- **WHEN** the player asks for five tenders and the buildings offer three places
- **THEN** three Dancers become tenders and the rest stay laborers

### Requirement: Crewed recipes reserve Dancers
A recipe with a crew SHALL start only when that many Dancers of its profession are free,
SHALL reserve them for its duration with the recipe as their task, and SHALL release them
when it completes. A recipe that cannot start SHALL report how many Dancers are missing.

#### Scenario: Short-handed
- **WHEN** a recipe needs a crew of four weavers and two are free
- **THEN** it does not start, nothing is taken, and it reports two weavers missing

### Requirement: Settlement tiers are rows
A settlement's tier SHALL be decided by rule rows naming a population, a number of
dwellings of a kind and required buildings, starting from Farthest Frontier's Town Center
thresholds.

#### Scenario: Established
- **WHEN** a village has 30 Dancers, 8 cottages and the buildings its tier row requires
- **THEN** it reaches the established tier, and crewed recipes become available there
