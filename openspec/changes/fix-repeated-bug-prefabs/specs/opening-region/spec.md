## ADDED Requirements

### Requirement: Reusable bug prefab state is local to its placement
The game SHALL give each placement inheriting the lightning bug template identity its own resource identity using Jackdaw placement and prefab-local identifiers, and SHALL allocate an independent clock for its inherited default clock. Explicit non-template identities and non-default clock overrides SHALL retain their existing semantics.

#### Scenario: Repeated placements
- **WHEN** several copies of the default lightning bug prefab are placed and Play begins
- **THEN** each binds with a distinct resource identity and independent clock without editing the scene or prefab template

#### Scenario: Moving a saved placement
- **WHEN** a persisted placement is moved without changing its SceneNodeId
- **THEN** its resource identity stays the same

#### Scenario: Explicit authoring
- **WHEN** a placement overrides the template resource identity or default clock
- **THEN** those explicit values retain their existing duplicate validation
