## Why

`stubby_game` and `stubby_jackdaw` depend on five Jackdaw crates (`jackdaw_terrain`,
`jackdaw_prefab`, `jackdaw_runtime`, `jackdaw_scene_types`, and `jackdaw_bsn` for
tests) by relative path: `../../../jackdaw/crates/...`. The game builds only when a
Jackdaw checkout happens to sit beside this repository, and it builds against whatever
that checkout holds, not the revision `compatibility.json` records. On 2026-11-15 the
shared checkout (82018644) was one commit ahead of the pin (3c891ed). A fresh clone
cannot build at all, and CI recreates the sibling layout by hand to work around it.
Cargo's normal way to depend on a crate from another repository at an exact
revision is a git dependency, and the lockfile then records it. That makes the build
reproducible from this repository alone.

## What Changes

- Declare the Jackdaw crates once, in the root `[workspace.dependencies]`, as git
  dependencies on `https://github.com/buddha314/jackdaw.git` at `rev = <pinned
  revision>`. Members inherit them (`jackdaw_terrain.workspace = true`, adding
  features where needed). `Cargo.lock` records the same revision.
- **BREAKING (tooling):** `tools/check_jackdaw_versions.py` stops requiring path
  dependencies. It requires the git source at exactly the revision in
  `compatibility.json`, in both `Cargo.toml` and `Cargo.lock`.
- `tools/jackdaw_sync.py` re-pins by rewriting that one `rev` and running
  `cargo update` for the Jackdaw crates, alongside the pins it already writes.
- **Local editor development** keeps working without editing manifests:
  `tools/jackdaw.sh link` writes an untracked `.cargo/config.toml` that `[patch]`es the
  git source with the shared checkout's paths, and `tools/jackdaw.sh unlink` removes it.
  The checker refuses a `Cargo.lock` that records linked paths, so a linked lock is
  never committed.
- CI builds from this repository alone; the second checkout of the fork remains only
  for the fork-drift warning.
- Unchanged: the shared editor still runs from `~/src/jackdaw`, and a managed launch
  still requires the editor's revision to equal the pin, because Play talks to the
  game over Jackdaw's PIE protocol.

## Capabilities

### New Capabilities
- `jackdaw-game-dependencies`: how the game names, pins and locally overrides its
  Jackdaw crate dependencies.

### Modified Capabilities
None in `openspec/specs/` (it is empty). The requirements of
`normalize-jackdaw-versions` (`jackdaw-toolchain`: one workspace and lockfile, an
explicit recorded revision, read-only validation) are kept; only the path-dependency
rule in its checker changes.

## Impact

- Root `Cargo.toml`, `crates/stubby_game/Cargo.toml`, `crates/stubby_jackdaw/Cargo.toml`,
  `Cargo.lock` (shared files; keep the edit to the Jackdaw lines).
- `tools/check_jackdaw_versions.py`, `tools/test_jackdaw_versions.py`,
  `tools/jackdaw_sync.py`, `tools/jackdaw.sh` (`link`, `unlink`).
- `.github/workflows/stubby-audio.yml`, `.gitignore` (`.cargo/config.toml`).
- `docs/research/jackdaw-version-policy.md`, AGENTS.md (*Jackdaw is the version
  authority*).
- Land before `split-game-into-systems` (#120), which edits the same manifests.
- Editor extensions such as the crafting designer (#121) are unaffected: Jackdaw builds
  them against its prebuilt SDK, not through our Cargo dependencies.
