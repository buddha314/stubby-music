## ADDED Requirements

### Requirement: Aim ranks physics-query candidates
Interaction SHALL retain avian3d reach-query candidates; within reach, the marker closest to the view axis within a 20 degree cone SHALL win, falling back to nearest when none qualifies. Prompts, E and outlines SHALL share that target.

#### Scenario: Aim ranks physics-query candidates
- **WHEN** a shell is aimed at beside a slightly nearer reed
- **THEN** the shell is the single interaction target

### Requirement: One readable pickup outline
Only the targeted available pickup SHALL have a white data-driven outline with default width 2 screen pixels, following the visible body and LOD. It SHALL disappear on collection, build mode or screen capture. A targeted bug SHALL retain a dark silhouette between flashes without emitting light.

#### Scenario: One readable pickup outline
- **WHEN** an outlined bug is between flashes
- **THEN** its visible body remains identifiable while its flash light is off

### Requirement: Compass follows view
The HUD SHALL show heading and cardinal directions during free play, hide behind screens, and open at bearing 75 degrees so the eastward sunrise lies 15 degrees to the right. Compass and moon bearings SHALL share the same north/east convention.

#### Scenario: Compass follows view
- **WHEN** the player rotates from east to north
- **THEN** the compass moves from E to N without changing world orientation
