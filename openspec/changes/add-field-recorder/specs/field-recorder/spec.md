## ADDED Requirements

### Requirement: H brings up the field recorder
The player's hand SHALL hold at most one item. H SHALL bring the field recorder into
the hand, shown in the lower right of the view at real size, windguard forward, and
pressing H again SHALL put it away. The hand starts empty, and H does nothing while a
screen is open. Each future hand item gets its own key in the same list.

#### Scenario: Bringing it up and putting it away
- **WHEN** the player presses H during play, then H again
- **THEN** the recorder appears in the lower right of the view, then is put away

#### Scenario: Screen open
- **WHEN** the inventory screen is open and the player presses H
- **THEN** the hand does not change

### Requirement: The recorder shows collection
While E collects, the recorder's `Light_rec` SHALL pulse and its `Level_<n>` bars SHALL
show activity; otherwise the light rests dark.

#### Scenario: Collecting a bug
- **WHEN** the player holds E within reach of a lightning bug
- **THEN** the record light pulses and the level bars move until the collection completes
