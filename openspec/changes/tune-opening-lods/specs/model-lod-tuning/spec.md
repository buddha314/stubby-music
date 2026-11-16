## ADDED Requirements

### Requirement: Reproducible model tuning
The authoring tool SHALL apply explicit handover, final-cull and crossfade settings for all currently LOD-configured model families, including shells, while preserving their level sources and simplification ratios. Reference distances SHALL document FOV and placement-scale assumptions. Flora handovers SHALL follow their existing family profiles.

#### Scenario: Reapply tuning
- **WHEN** the same tuning policy is applied twice
- **THEN** all configured metadata files are unchanged on the second application

### Requirement: Review the actual runtime levels
The opt-in review SHALL record camera distance, FOV, placement scale, requested/ready levels and screenshots for representative near, transition, far and culled views. It SHALL use Jackdaw's runtime selection without a separate game selector and SHALL include dormant/grown and standing/harvested states.

#### Scenario: Review a handover
- **WHEN** the review camera enters a configured transition band
- **THEN** evidence reports the runtime's requested and ready levels for that view

### Requirement: Bounded graphical validation
Graphical tuning SHALL build the game first and run exclusively through tools/jackdaw_timed_test.py with its independent maximum-20-second watchdog. A tuning report SHALL distinguish observed behavior from unreviewed variants and performance assumptions.

#### Scenario: The review runs
- **WHEN** the review starts drawing frames
- **THEN** the independent watchdog still stops the graphical process within its configured 20-second limit
