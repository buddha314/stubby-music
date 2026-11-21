## Phase intent

| Phase | Player capability to explore | Evidence in the world |
| --- | --- | --- |
| Camp | Find a waveform and patch a local heartbeat | Burbler, campfire and the head of a mud channel |
| Village | Tune a chain of machines and bring clearer water to living things | A check dam, reeds and a mature group of oaks downstream |
| Town | Sustain several useful destinations from a shared network | A path alongside the water, a cluster of low buildings and a commons |
| City | Shape a mix across a district and connect several destinations | A civic canal and plaza framed by taller blocks and roof gardens |

## Draft progression rules

| Transition | Guideline | Open details |
| --- | --- | --- |
| Camp → Village | There is enough clean water to plant a crop. | First crop, minimum clarity and supply, and how to measure available water. |
| Village → Town | Three villages trade goods with one another. | What establishes a village, which goods count, and how much exchange over what period qualifies. |
| Town → City | The player can build an art gallery, a concert hall and at least one light rail system. | What demonstrates construction readiness and what qualifies as a light rail system. |

Use these guidelines to evaluate the phases. The relationship to the goal hub,
recipe unlocks and separate regional schedules is open.

## Population

The settlement population is called **Dancers**. It grows as settlements become
able to support more people. In Farthest Frontier, housing and food affect immigration, and a
larger population supports more jobs while consuming more supplies. For Stubby
Music, explore how clean water, food and shelter make it possible to welcome
residents, and how residents help sustain trade and civic life. Arrival and growth
rates, household or individual simulation, and the scope of work and needs remain
open. [Farthest Frontier population guide](https://www.farthestfrontier.com/guide/villagers/population/)

**Decided (user, 2026-11-20):** Dancers are individuals. A cottage holds at most four,
filled at random; larger dwellings follow. Settlement size starts from Farthest Frontier's
Town Center thresholds (30, 150 and 300 villagers) and is adjusted as we go. Established
villages have recipes that need a crew of Dancers, and every Dancer's task is tracked as in
Farthest Frontier. The model and its rules are `add-settlement-economy`.

## Draft Dancer consumption schedule

Provision Dancers from their settlement's stock. Clean water, food and fresh air are
budgeted per quarter note of each Dancer's local clock. Faster BPM makes Dancers
build faster and increases recurring consumption over the same elapsed time.
Shelter still needs one place per resident and is checked at night.

At the standard **120 BPM**, one day's food, clean water and fresh air sustain a Dancer
working continuously from dawn to dawn at normal simulation speed. Spread each
daily budget over the quarter notes in that reference day. On the current
3,600-second day at 1×, that is 7,200 quarter notes. At 60 BPM, the same span
uses half the daily budget. Carry each clock's cursor across
save/load, and count each crossed note once when advancing the simulation.

Fresh-air use lowers air quality where Dancers are working or resting. Plants
and algae replenish air within their cleaning radius. The visible air quality
reflects the balance between replenishment and local demand.

Record shortfalls per resident and need. An enabled crafted spring invokes
Dancers who descend from the sky and settle in its region. Candidate arrivals
follow a Poisson rate set by local air quality. Each arrival needs a vacant
home and seven in-game days of uncommitted clean water at the local tempo.
The player can turn invocation on or off. The spring recipe, rate curve and
per-note ration quantities remain open for tuning.
[Spring invocation design](../add-settlement-economy/design.md#spring-invocation)

The [Burb Sweeper economy review](../../../docs/research/burb-sweeper-economy.md)
records precedents for unmet demand, housing capacity, shared labour and
long-run balance tests. Consider these when prototyping the schedule.

## Dancer appearance and clocks

Use mostly silhouetted figures with simplified faces and clear poses. Take
SUPERHOT's readable outlines as a visual reference and give the Dancers their
own shapes, materials and movement. Try low-saturation surfaces at a distance
and let skin tones read where light reaches them, with a subtle rim in dark
scenes. Compare the same figures against bright sand, grove canopy and city
lighting. [SUPERHOT Team game page](https://store.steampowered.com/app/322500/)

Start with four body shapes and four skin tones within the requested range of
three to five each. The shapes below indicate silhouette direction. The palette
swatches are draft sRGB base colours.

| Body shape | Silhouette to test |
| --- | --- |
| Compact | Shorter stance and rounded torso |
| Lanky | Tall frame with long limbs |
| Broad | Wide shoulders and a sturdy stance |
| Full | Softer, rounded profile through the torso and hips |

| Skin tone | Draft swatch |
| --- | --- |
| Solar amber | `#D89B54` |
| Clay coral | `#C96D74` |
| Leaf teal | `#4F9D88` |
| Dusk violet | `#8878AB` |

Keep body and skin as independent choices on a shared rig. Preview all 16
combinations and keep each resident's choice stable across reloads. Every
combination can use the same dances and resident roles.

Associate each Dancer with an explicit local clock ID, such as the clock at a
village commons or concert hall. Use audio-clock beat events for step accents
and interpolate between them. Dancers following the same clock can share a
downbeat and vary their poses. A stopped or missing clock leaves the Dancer in
an idle pose until a clock is available. Clock assignment and reassignment
rules are open.

### The Dancer models (2026-11-20)

Built on a rigged CC0 base (user, 2026-11-20: reuse existing rigs): Quaternius's Universal
Animation Library mannequin (`assets-src/third_party/quaternius/`), its 53-joint humanoid rig
and the clips a resident needs (Idle, Dance, Walk, Talk, Sit, Tend, Interact).
`tools/blender/create_dancers.py` reduces it to about 1.4k triangles and makes the four body
shapes on the one rig (`assets/characters/dancers/dancer_<shape>/`): compact, lanky, broad and
full, each thickened or thinned about its bones and scaled (1.61 to 1.98 m). Faces are blank;
the figure reads as a silhouette. Skin tone is not baked: a placement's `MaterialTint` recolours
`Dancer_Skin` to one of the four draft tones, so any shape takes any tone (the sheet
`previews/characters/dancers/sheet.png` shows all 16). A placement's `Idle` loops in time with
the nearest clock, or the clip its `LoopClip` names (`Dance`). Review in the game:
`python3 tools/jackdaw_timed_test.py --asset-review dancers`. Still open: the subtle rim in
dark scenes, and choosing shape and tone from each Dancer's seed (the settlement system).

## Shared site

The burbler is the head of an eastbound water route. The campfire sits on
its north bank and a window rock marks the south bank. Each scene retains
the same transforms for those features and the same opening view at noon. The
Village extends water and vegetation eastward. The Town puts its commons
beside that route. The City extends the same line past a civic plaza and
keeps direct pedestrian access to the old camp.

Keep runtime families under their current categories. The phase index names
their first review use and later reuse. Town and City use Jackdaw brushes
for volume, scale and sightlines. Future production buildings will have
their own assets, acoustic surfaces and acoustic review.

## Acoustic review paths

The representative path is from the burbler outlet east along the canal,
then to the commons and civic plaza. A second path crosses the south bank
past the existing window rock, which keeps its authored rock acoustics.
The brush masses are back from the canal and represent layout volumes.
Acoustic treatment for the street-facing walls and roofs belongs to the
future building geometry and the corresponding listening review.
