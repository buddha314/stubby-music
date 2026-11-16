## ADDED Requirements

### Requirement: Major goals own ordered minor steps
Rules SHALL support major goals with ordered minor steps, active-goal progression and prerequisites, retaining stable existing goal IDs through explicit migrations where needed. Goals SHALL never end play.

#### Scenario: Major goals own ordered minor steps
- **WHEN** the last active minor step completes
- **THEN** its major completes and the next eligible major activates

### Requirement: Conditions latch once
Minor goals SHALL support item_held and tree_grown conditions alongside machine_configured and tree_watered. Completion SHALL latch once, including if a condition already holds on activation.

#### Scenario: Conditions latch once
- **WHEN** a step requiring twelve reeds activates with twelve already held
- **THEN** it completes once and remains complete after the reeds are spent

### Requirement: Opening milestone rows and validation
The shipped major build_a_camp SHALL contain catch_a_spark, build_a_heartbeat, cut_reeds, water_the_tree and stop_the_wind in source order. Validation SHALL reject unknown references, invalid bloom levels, cycles and incompatible major/minor fields with file, ID and field diagnostics.

#### Scenario: Opening milestone rows and validation
- **WHEN** a minor references an unknown major
- **THEN** the strict loader reports that field instead of silently omitting the row
