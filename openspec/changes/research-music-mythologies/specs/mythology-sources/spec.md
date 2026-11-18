## ADDED Requirements

### Requirement: Mythic figures come only from openly shared stories

The game SHALL use a mythic figure, story or image only when the research note records it as openly shared by its own community, with its source. Restricted knowledge, sacred objects and ceremonies, and figures whose sharing is unclear SHALL NOT be used.

#### Scenario: A restricted story

- **WHEN** the research finds that a figure's story is restricted to initiates, to one gender or to its community
- **THEN** the note marks it restricted and no region, shrine or text uses it

#### Scenario: Sharing is unclear

- **WHEN** no community source says how a story is shared
- **THEN** the note marks it unclear, and it is not used until that is settled

### Requirement: Each figure is recorded with its source and sharing check

The research note SHALL record, for each figure, its people, what it is tied to, a summary in the project's own words, its source and the kind of source, and the sharing check with its reason. It SHALL keep game ideas apart from what the sources say.

#### Scenario: Reading an entry

- **WHEN** a designer reads an entry
- **THEN** they can see who tells the story, where it was found, whether it may be used, and which notes are the project's own ideas

### Requirement: Region and shrine changes use only cleared figures

A change that places a mythic figure in the game SHALL name its entry in the research note, and SHALL NOT use a figure the note has not marked open.

#### Scenario: Designing a shrine

- **WHEN** `add-deity-shrines` designs a carving for a figure
- **THEN** it cites the figure's entry, and its art is the project's own design, not a copy of a real rock-art panel
