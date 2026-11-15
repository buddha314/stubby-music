## ADDED Requirements

### Requirement: Jackdaw crates are pinned git dependencies
The workspace SHALL declare every Jackdaw crate it uses once, in the root
`[workspace.dependencies]`, as a git dependency on the buddha314 fork at the exact
revision recorded in `compatibility.json`. Members SHALL inherit those declarations
and SHALL NOT name Jackdaw crates by path.

#### Scenario: Building from a fresh clone
- **WHEN** a developer clones only stubby-music and runs `cargo build -p stubby_game`
- **THEN** Cargo fetches Jackdaw at the pinned revision and the game builds without a sibling checkout

#### Scenario: One place to pin
- **WHEN** the pinned revision changes
- **THEN** exactly one `rev` in the root `Cargo.toml` and the matching `Cargo.lock` entries change

### Requirement: The pin is validated
`tools/check_jackdaw_versions.py` SHALL fail, naming the crate and both revisions,
when a Jackdaw crate in `Cargo.toml` or `Cargo.lock` is not the fork's git source at the
revision in `compatibility.json`, or when a member names a Jackdaw crate by path.

#### Scenario: A stray path dependency
- **WHEN** a member declares `jackdaw_prefab = { path = "../../../jackdaw/crates/jackdaw_prefab" }`
- **THEN** the check fails naming the member and `jackdaw_prefab`

#### Scenario: Lock and record disagree
- **WHEN** `Cargo.lock` resolves `jackdaw_runtime` at a revision other than the recorded one
- **THEN** the check fails naming `jackdaw_runtime`, the locked revision and the recorded revision

### Requirement: Local Jackdaw development without manifest edits
A developer SHALL be able to build the game against the shared Jackdaw checkout
(`tools/jackdaw.sh link`) and return to the pinned git source (`tools/jackdaw.sh
unlink`) without editing any tracked file. The link SHALL live in an untracked
`.cargo/config.toml`, and the version check SHALL fail while `Cargo.lock` records
linked path sources.

#### Scenario: Testing an unpushed editor fix
- **WHEN** a developer runs `tools/jackdaw.sh link` with a fix in `~/src/jackdaw` not yet pushed
- **THEN** the game builds against the checkout's sources and `git status` shows no tracked change except possibly `Cargo.lock`

#### Scenario: A linked lock is not committed
- **WHEN** the version check runs while linked
- **THEN** it fails saying the lock records linked Jackdaw paths and to run `tools/jackdaw.sh unlink`

### Requirement: The editor still matches the pin
A managed launch SHALL still require the shared editor's checkout revision to equal
the recorded revision, because the game's PIE runtime and the editor must speak the
same protocol.

#### Scenario: Editor ahead of the pin
- **WHEN** the shared checkout is one commit ahead of the recorded revision and a managed launch starts
- **THEN** validation fails with both revisions and the instruction to sync or check out the pin
