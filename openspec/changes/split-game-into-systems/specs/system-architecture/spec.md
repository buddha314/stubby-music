## ADDED Requirements

### Requirement: The game is layered system crates
The game SHALL be built from crates in four layers: **cores** (game rules and engines
with no Bevy dependency: `stubby_audio`, `stubby_rack`, `stubby_recipes`), **services**
(`stubby_core`, `bevy_stubby_audio`), **systems** (`bevy_stubby_rack`,
`bevy_stubby_recipes`, `stubby_world`, `stubby_player`, `stubby_life`,
`stubby_streams`), and the **game** (`stubby_game`, `stubby_jackdaw`). A crate SHALL
depend only on crates in lower layers, except that a system crate MAY depend on its
own core.

#### Scenario: A system uses a service
- **WHEN** `stubby_life` needs clock ticks for the lightning bug's flash
- **THEN** it depends on `bevy_stubby_audio`, a service, and builds

#### Scenario: A core reaches up
- **WHEN** a change makes `stubby_rack` depend on `bevy`
- **THEN** the layer check fails naming `stubby_rack` and `bevy`

### Requirement: Systems never depend on each other
A system crate SHALL NOT depend on another system crate. Systems SHALL communicate
only through messages, resources and components that they own or that a service
owns; the game crate SHALL translate between them.

#### Scenario: Streams need reeds
- **WHEN** a stream segment is laid and must cost six reeds
- **THEN** `stubby_streams` asks through its own build-cost contract, the game's glue answers it from `bevy_stubby_recipes`' inventory, and `stubby_streams` does not depend on `bevy_stubby_recipes`

#### Scenario: A forbidden import
- **WHEN** a change adds `stubby_world` as a dependency of `stubby_life`
- **THEN** the layer check fails naming both system crates

### Requirement: Cross-system rules live in the game
Every rule that joins two systems SHALL live in `stubby_game` or in the game's data,
never inside a system or core crate: for example an item mounting as a rack module, a
creature's pitch setting a knob, a machine being a clock with a rack and a stream
outlet, or the calendar's time driving resource replenishment.

#### Scenario: Square wave mounts as a Square VCO
- **WHEN** the player drags a square wave onto the burbler's VCO place
- **THEN** the game's glue turns the inventory item into a mount of the `square_vco` module, and neither `bevy_stubby_rack` nor `bevy_stubby_recipes` names the other's types

### Requirement: The shared core stays small
`stubby_core` SHALL hold only contracts several systems need: screen state, input
bindings, the HUD camera marker, the interaction target, shared system-set labels and
ground height. It SHALL NOT hold a system's behaviour, and it SHALL depend on nothing
of ours.

#### Scenario: Ground height
- **WHEN** the player controller and the lightning bug both need the ground's height
- **THEN** they read `stubby_core`'s ground-height contract, which `stubby_world` provides, and neither depends on `stubby_world`

### Requirement: Each system stands alone
Each system crate SHALL expose one Bevy plugin as its entry point and SHALL build on
its own. Its focused tests SHALL run with `cargo test -p <crate>` without building
`stubby_game`.

#### Scenario: Testing the rack alone
- **WHEN** `cargo test -p bevy_stubby_rack` runs
- **THEN** it does not compile `stubby_game`, `stubby_world`, `stubby_life` or `stubby_streams`

### Requirement: The editor adapter uses the crates
`stubby_jackdaw` SHALL depend on the game and system crates as ordinary dependencies
and SHALL NOT compile their source by `#[path]`. Reflected components in saved
prefabs SHALL use the type path of the crate that owns them, and a move that changes a
reflected path SHALL rewrite the saved prefabs in the same change.

#### Scenario: A bug prefab after the move
- **WHEN** `assets/creatures/lightning_bug/lightning_bug.bsn` is loaded after `PulseFollower` moves to `bevy_stubby_audio`
- **THEN** it names `bevy_stubby_audio::…::PulseFollower` and `cargo test -p stubby_jackdaw --test prefabs` passes

### Requirement: The layering is checked
`cargo deny check bans` SHALL enforce the layering from `deny.toml`, where each crate's
`wrappers` name the only crates allowed to depend on it, and fail naming the banned crate;
a `cargo tree` check SHALL fail when a core depends on the Bevy engine.

#### Scenario: Clean workspace
- **WHEN** the check runs on a workspace that follows the rules
- **THEN** it exits 0

### Requirement: Behaviour is unchanged
Splitting the game SHALL NOT change what the player sees, hears or can do.

#### Scenario: The opening after the split
- **WHEN** the game starts after the split
- **THEN** the opening, the burbler's rack and sound, stream laying and bug collection behave as before
