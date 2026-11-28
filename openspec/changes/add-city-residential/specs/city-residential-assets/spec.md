## ADDED Requirements

### Requirement: Uses read without labels
Mixed-use buildings SHALL stack shops on the ground storey, offices above them and homes above those, under a shared roof garden. Each use SHALL be distinguishable by its openings and contents: shopfronts with display glass and awnings, offices with continuous glazing and workstations, homes with smaller openings, balconies, curtains and domestic furniture.

#### Scenario: A home storey is not an office storey
- **WHEN** a home storey and an office storey of the same building are compared
- **THEN** the home storey has separate windows, balcony doors or loggias with domestic contents, and the office storey has continuous glazing with workstations and no balconies

### Requirement: Homes are individual and private
Home storeys SHALL vary bay types, curtains and balcony contents from a seed while reusing the same pieces. Each home storey SHALL include solid wall and privacy elements, so no storey is glazed end to end.

#### Scenario: Two storeys of one building differ
- **WHEN** two home storeys of the same building are compared
- **THEN** their bay types or balcony contents differ, and both contain solid wall

### Requirement: Separate residential circulation
A building with homes SHALL have a residents' street door distinct from its shops, a stair tower with a lift, an apartment door to the tower on each home storey, and access to its roof garden.

#### Scenario: Residents reach their roof
- **WHEN** the circulation of a mixed-use building is traced
- **THEN** a route runs from the residents' street door, through the stair tower, to each home storey and to the roof, without passing through a shop or an office

### Requirement: Shared pieces across building types
The street wall, rounded corner block, stepped garden tower and taller tower SHALL be assembled from the same structural, facade and interior pieces on the City kit's 4 m grid. Furniture, fabric and plants SHALL be separate pieces, never baked into structure.

#### Scenario: The four buildings share their pieces
- **WHEN** the pieces used by the four building types are listed
- **THEN** their facades, slabs, balconies and interior sets come from one shared set of pieces

### Requirement: Validated before multiplied
One complete mixed-use bay SHALL be exported, validated (every material has an acoustic class) and loaded in the game before the neighbourhood is assembled.

#### Scenario: The bay loads
- **WHEN** `tools/jackdaw_timed_test.py --asset-review city_mixed_bay` runs
- **THEN** the bay's prefabs load and render without errors within the watchdog
