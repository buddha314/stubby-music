## ADDED Requirements

### Requirement: Authored layout takes priority
The game SHALL preserve saved authored prefab transforms and native placement IDs
when optional procedural scatter is resolved. The opening's essential resources
SHALL remain authored in the first implementation.

#### Scenario: Scatter near an authored machine
- **WHEN** scatter proposes a decoration inside an authored burbler's clearance
- **THEN** the candidate is rejected and the saved burbler is unchanged

### Requirement: Explicit reproducible configuration
Preview, export and Play SHALL resolve the same explicit seed, settings, authored
content, masks and generator revision. Unsupported or malformed inputs SHALL be
reported without silently selecting another seed or revision.

#### Scenario: Repeated prototype generation
- **WHEN** seed 17 and identical recorded inputs are resolved twice
- **THEN** the resulting placement keys, assets and transforms are identical

#### Scenario: Invalid configuration
- **WHEN** a requested seed cannot be parsed or its generator revision is unsupported
- **THEN** generation reports the invalid input rather than falling back to seed 42

### Requirement: Reuse native placement tools
Implementation SHALL use existing Jackdaw scatter, masks, scene/prefab tools and
Bevy facilities where they meet the requirement. Any missing behavior SHALL be
limited to game-specific adapter or metadata glue with its incompatibility recorded.

#### Scenario: Native scatter can sample the surface
- **WHEN** the mesh's height sampler can supply the native heightmap/channel inputs
- **THEN** candidate generation calls native scatter without a replacement RNG or sampler

### Requirement: Protected placement areas
Procedural candidates and transform overrides SHALL respect configured exclusions,
authored clearance and opening constraints. Invalid edits SHALL produce diagnostics
without relocating authored placements or silently choosing replacement locations.

#### Scenario: Excluded cell
- **WHEN** a seed-17 candidate falls in a cell masked out for its rule
- **THEN** no generated instance is placed there

#### Scenario: Invalid manual move
- **WHEN** a transform override moves a generated object into protected spawn space
- **THEN** the edit is reported invalid and no conflicting placement is committed

### Requirement: Persistent designer overrides
Generated objects SHALL have stable keys independent of ECS IDs, transforms and
filtered output order. A designer SHALL be able to suppress or reposition a generated
object using existing editor operations and persisted game metadata where needed.
Regeneration SHALL retain those edits and SHALL report stale keys without retargeting.

#### Scenario: Delete and regenerate
- **WHEN** a generated key is suppressed and the seed-17 layout is regenerated
- **THEN** that object stays absent and unrelated candidate keys stay unchanged

#### Scenario: Move and reopen
- **WHEN** a valid generated-object transform override is saved and the layout reopens
- **THEN** the object has the saved transform with no duplicate at its original position

#### Scenario: Generator changed
- **WHEN** an overlay references keys unavailable in the selected generator revision
- **THEN** the unresolved edits are retained and reported rather than applied to other objects

### Requirement: Existing live prefab behavior
Interactive generated placements SHALL use existing gameplay prefab binding and
resource identity semantics. Decorative batching SHALL NOT be treated as registration
of harvestable resources. Designer overlays SHALL NOT be treated as gameplay saves.

#### Scenario: Interactive scatter enabled later
- **WHEN** a generated bug prefab is instantiated after interactive scatter is enabled
- **THEN** the existing bridge supplies independent timing and collection behavior under
  its stable world identity rather than merely drawing a batched bug model

#### Scenario: Regenerate with an existing ledger
- **WHEN** a layout resolves while collected-resource state is already registered
- **THEN** generation does not clear that ledger or claim that the seed restores played state

### Requirement: Minimal prototype validation
The prototype SHALL use seed 17 as its sole seed fixture. Validation SHALL focus on
repeatability, authored priority and move/remove behavior without a multi-seed sweep.
Any graphical diagnostic SHALL retain the independent 20-second watchdog.

#### Scenario: Prototype acceptance
- **WHEN** the hybrid placement prototype is checked
- **THEN** one seed-17 fixture covers the declared placement guarantees and no
  1,000-seed test is required
