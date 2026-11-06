## ADDED Requirements

### Requirement: Solid assets block the player

A placed asset whose `colliders.json` lists collider shapes SHALL stop the player from walking through it, in embedded Play and in the standalone game.

#### Scenario: Walking into a scrub oak

- **WHEN** the player walks straight at a placed scrub oak
- **THEN** the player stops at the edge of its trunk collider

#### Scenario: Walking past at an angle

- **WHEN** the player walks into a scrub oak at a glancing angle
- **THEN** the player slides around the collider and keeps moving

### Requirement: Colliders are authored on prefabs

Colliders SHALL be Jackdaw `AvianCollider` components on child entities of the prefab, with the shape and translation given in the family's `colliders.json` and no `RigidBody`. A level designer SHALL get them by placing the prefab, without adding anything to the placement.

#### Scenario: Placing an oak prefab

- **WHEN** a scrub oak prefab is placed in a level and the level is played
- **THEN** that placement has a static collider at the position `colliders.json` gives for its variant

#### Scenario: Prefab validation

- **WHEN** `cargo test -p stubby_jackdaw --test prefabs` runs
- **THEN** every prefab carrying an `AvianCollider` loads with known types

### Requirement: Small assets stay walk-through

Assets that `colliders.json` marks `walk_through`, and assets with no entry, SHALL NOT block the player.

#### Scenario: Walking through reeds

- **WHEN** the player walks into a reed clump
- **THEN** the player passes through it and can still harvest it with E

### Requirement: Colliders leave interaction unchanged

Colliders SHALL NOT change what E acts on: `InteractTarget` SHALL still come only from the interaction markers.

#### Scenario: Collecting beside an oak

- **WHEN** the player stands beside a scrub oak within reach of a lightning bug
- **THEN** the bug is the interaction target and E collects it

### Requirement: One physics dependency graph

Adding the collider integration SHALL leave one Bevy and one avian in the game's dependency graph, with avian from the generated Bevy-main fork.

#### Scenario: Checking the graph

- **WHEN** `tools/check_jackdaw_versions.py` runs after the integration is added
- **THEN** it passes, and `Cargo.lock` holds a single `bevy` and a single `avian3d`
