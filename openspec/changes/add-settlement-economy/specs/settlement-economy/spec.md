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

### Requirement: Quarter notes drive work and resource consumption
Each Dancer's food, clean-water and fresh-air demand SHALL accrue per quarter note of their local clock,
including while idle. Construction, crewed crafting and other clocked settlement work
SHALL advance on the clock bound to the job. General recurring resource consumption SHALL
be specified per quarter note of the consuming process's clock. One BPM unit means one
quarter note per minute. A faster clock SHALL increase work and resource use per elapsed
minute while preserving the configured rates per quarter note. Fixed inputs for individual
recipes or buildings remain fixed costs.

The standard provisioning tempo SHALL be 120 BPM. One standard daily budget of food, clean water and
fresh air SHALL sustain a Dancer working continuously through one full
in-game day at 120 BPM at normal simulation speed. Each per-note rate SHALL be the
daily budget for that resource divided by the number of quarter notes in that
reference day.

Quarter notes SHALL be counted from sample-timestamped audio-clock events. The economy SHALL
process each note once across catch-up and save/load. If a clock stops, the work and
recurring demand it drives SHALL pause.

#### Scenario: Standard ration budget
- **WHEN** a Dancer works continuously at 120 BPM from one dawn to the next at normal simulation speed
- **THEN** the Dancer can work for the full day on one standard daily budget of food, clean water and fresh air

#### Scenario: Double the tempo
- **WHEN** otherwise identical, fully supplied sites run at 48 and 96 BPM for one minute with Dancers following their respective site clocks
- **THEN** the faster site accrues twice the work progress and twice the recurring resource demand, with equal work and demand per quarter note

### Requirement: Dancers draw fresh air from their surroundings
On each local quarter note, a Dancer SHALL draw fresh air at their current
location. That draw SHALL reduce local air quality and SHALL count toward the
same per-Dancer demand, amount met and shortfall metrics as food and clean water.
Fresh air SHALL be resolved against the shared air-quality field rather than
settlement inventory. Plants and algae SHALL replenish that field through the
authored air-cleaner rules. Simultaneous draws SHALL share the available local
air without using the same supply twice.

#### Scenario: More demand in the same grove
- **WHEN** Dancers in a grove switch from 120 to 240 BPM while its plants and algae keep the same cleaning strength
- **THEN** the Dancers request twice as much fresh air over the same elapsed time, and air quality drops locally when demand exceeds replenishment

#### Scenario: The grove recovers
- **WHEN** the Dancers leave a depleted area and its plants and algae remain
- **THEN** those cleaners raise the area's air quality toward its undepleted level

### Requirement: Rules produce before they consume
Within one economy resolution, producer and converter rules SHALL apply before consumer and upkeep
rules; a converter SHALL run only whole conversions it can afford; and a demand that stock
cannot meet SHALL leave the stock at zero and be reported as a deficit.

#### Scenario: Short of water
- **WHEN** twelve Dancers each need one unit of water on a quarter note and the settlement holds eight
- **THEN** water stock is zero and the deficit records four units of water

### Requirement: Dancers are individuals
Each Dancer SHALL have a stable ID, a body shape and skin tone drawn from a seed, a home, a
profession, a current task and a local clock ID, all kept across saving and loading.
Population and profession counts SHALL be derived from the roster.

#### Scenario: Reloaded
- **WHEN** a game is saved and loaded
- **THEN** every Dancer has the same ID, appearance, home, profession and task as before

### Requirement: A cottage houses at most four Dancers, chosen at random
A dwelling SHALL house no more Dancers than its configured capacity. The initial cottage capacity SHALL be four.
A new Dancer SHALL be housed in a dwelling with a free place chosen by a seeded random draw,
so the same seed and history house the same Dancers in the same dwellings.

#### Scenario: Two cottages, five Dancers
- **WHEN** five Dancers arrive at a settlement with two empty cottages
- **THEN** no cottage holds more than four, and running again with the same seed gives the same homes

### Requirement: Springs invoke Dancers
A crafted spring SHALL act as an invocation site. The player SHALL be able to turn
invocation on or off per spring. While enabled, the spring SHALL generate candidate
arrivals as a seeded Poisson process in world time, with a tunable rate determined
by the local air quality at the spring. Cleaner air SHALL increase the rate.
A candidate SHALL descend from the sky and join the spring's local settlement only
if it has a free home and enough uncommitted clean water for the configured reserve horizon
of the new Dancer's demand at the local tempo. The water SHALL be reserved when
the Dancer is admitted so another arrival cannot claim it. Disabled or rejected
arrivals SHALL NOT accumulate for later release.

#### Scenario: Six days of water
- **WHEN** an enabled spring generates an arrival candidate under the initial profile and the settlement has a vacancy but only six days of uncommitted clean water for that Dancer
- **THEN** the Dancer does not descend

#### Scenario: A successful invocation
- **WHEN** an enabled spring generates an arrival candidate and the settlement has a vacancy and at least seven days of uncommitted clean water for the new Dancer under the initial profile
- **THEN** the Dancer descends into the spring's region and the settlement reserves the seven-day supply

#### Scenario: Invocation paused
- **WHEN** the player turns invocation off at a spring while water, housing and local air would otherwise support arrivals
- **THEN** no Dancers descend at that spring until it is enabled again

### Requirement: Spring and provisioning balance is editable in the Rules Designer
Spring invocation profiles SHALL expose the water reserve horizon in game days
and the air-quality-to-arrival-rate curve through schema-backed rule rows.
The initial water reserve horizon SHALL be seven game days. Dancer ration
budgets and cleaner replenishment settings SHALL be tunable through the same
Rules Designer. The rules loader SHALL reject invalid values and the designer
SHALL show the affected fields. Game and headless simulation SHALL consume the
same validated rule data. Per-spring invocation state and existing reservations
SHALL be carried forward when tuning values change.

#### Scenario: Tune the spring
- **WHEN** a designer edits a valid air-quality curve in the web Rules Designer and saves it
- **THEN** the spring uses the saved curve for future arrivals and the designer shows the rates in expected arrivals per game day

#### Scenario: Invalid curve
- **WHEN** a designer enters a negative arrival rate or repeats an air-quality point
- **THEN** validation identifies the affected field and retains the last valid rules

### Requirement: Settlement balance rows are editable in the Rules Designer
The Rules Designer SHALL expose schema-backed dwelling capacities, profession
workplaces, shortage-driven leaving settings, crewed recipe requirements and
settlement tier thresholds. Item, dwelling, building and profession references
SHALL be validated by the rules loader and offered by the editor where available.
Authored Dancer appearance profiles SHALL be editable as references to their
body and palette variants when those assets are available.

#### Scenario: Change a tier threshold
- **WHEN** a designer saves a valid population or dwelling threshold in the Rules Designer
- **THEN** the game and headless simulation use that threshold for subsequent tier evaluations

### Requirement: Shortfalls never kill
A missed ration SHALL be recorded per need and per Dancer; no Dancer SHALL die of a
shortfall. Prolonged shortfall MAY make Dancers leave according to a configured policy that defaults to disabled.

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
SHALL reserve them for its required quarter notes with the recipe as their task, and SHALL release them
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
