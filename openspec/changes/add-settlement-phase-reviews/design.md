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

## Draft Dancer consumption schedule

Provision Dancers from the stock of the settlement they live in. Use game units
for the first balance pass. One world day includes daylight and night, currently
3,600 simulation seconds. Local BPM can change how a Dancer moves while the
settlement calendar keeps the same provisioning schedule.

| When | Per resident Dancer | What the settlement checks |
| --- | --- | --- |
| Dawn | 1 unit of clean water | Allocate the first water ration. |
| Midday | 1 food ration | Draw from edible stock, with crops as an early supply route. |
| Dusk | 1 unit of clean water | Allocate the second water ration and check how many residents have shelter. |
| Night | Shelter | One place per resident. Housing capacity carries forward. |

Twelve resident Dancers would need 24 clean-water units, 12 food rations and
12 shelter places to cover a full day. Use the actual resident count at each
scheduled withdrawal. At higher simulation speeds, process every crossed
checkpoint once and in order. Resume from the last processed checkpoint on load.

For the first prototype, record water and food shortfalls and show uncovered
shelter demand. A settlement with shortfalls can pause new arrivals while it
recovers. When reviewing stock levels, count inventory already reserved for
upcoming needs before treating goods as trade surplus. The schedule, clean
water cutoff, reserve horizon and consequences of prolonged shortages are
open balance decisions.

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
