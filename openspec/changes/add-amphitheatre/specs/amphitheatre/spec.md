## ADDED Requirements

### Requirement: A natural amphitheatre in the chaparral
The world SHALL contain one amphitheatre: a sandstone bowl worn into tiers, framed by a natural arch, around a round dancing floor with a spring at its centre, with chaparral on its rim. It SHALL be a Jackdaw prefab placed in the scene, and SHALL use landforms only: no rock art and no copy of a real site, theatre, ceremony or sacred structure.

#### Scenario: Seen in the editor
- **WHEN** the scene is opened in Jackdaw
- **THEN** the amphitheatre is listed and placed in a chaparral setting

#### Scenario: The first view
- **WHEN** the player approaches from the bowl's open side
- **THEN** the arch frames the dancing floor and its spring

### Requirement: Found early, understood later
Before any being is seated, the amphitheatre SHALL read as a landform: no prompt, label, ring or hint, the spring dry and every seat empty. It SHALL stand outside every hearing radius from spawn.

#### Scenario: Wandering in early
- **WHEN** the player reaches the amphitheatre before seating any being
- **THEN** it shows no prompt or marker and nothing in it sounds

### Requirement: Seats follow the being atlas
Each seat SHALL belong to one being and SHALL be placed by the export script from the being atlas layout, so that beings alike in the atlas sit near each other. Seats SHALL NOT be placed by hand.

#### Scenario: The atlas changes
- **WHEN** the layout is regenerated and the amphitheatre exported again
- **THEN** the seats move to match the new layout, the same layout giving the same seats

### Requirement: Seating a being fills its seat and gives a module
When a woken being is seated, its seat SHALL show the being, give the player that being's module once, as defined by a rule row, and show hints in its neighbouring seats. The spring SHALL rise with the share of seats filled.

#### Scenario: The first being is seated
- **WHEN** the first woken being is seated
- **THEN** its seat is filled, its module is added to the inventory, its neighbours show hints, and the spring begins to rise

#### Scenario: Seating again
- **WHEN** a being whose seat is filled is seated again
- **THEN** no second module is given
