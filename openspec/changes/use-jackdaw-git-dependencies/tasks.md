## 1. Manifests

- [ ] 1.1 Root `[workspace.dependencies]`: `jackdaw_terrain`, `jackdaw_prefab`, `jackdaw_runtime`, `jackdaw_scene_types`, `jackdaw_bsn` as `git = "https://github.com/buddha314/jackdaw.git"`, `rev` = the recorded revision
- [ ] 1.2 `stubby_game` and `stubby_jackdaw` inherit them (`default-features = false` on terrain, `pie` on runtime); no Jackdaw `path` left
- [ ] 1.3 `cargo update` for the Jackdaw crates only; confirm `Cargo.lock` still resolves one Bevy
- [ ] 1.4 Check: `cargo build -p stubby_game -p stubby_jackdaw` and `cargo test -p stubby_jackdaw --test prefabs`, with `~/src/jackdaw` temporarily out of reach (`JACKDAW_SOURCE` unset, built in a fresh worktree)

## 2. Validation

- [ ] 2.1 `check_jackdaw_versions.py`: require the fork's git source at the recorded `rev` in the workspace and in `Cargo.lock`; reject Jackdaw `path` dependencies; a source-less `jackdaw_*` lock entry reports an active link and `unlink`
- [ ] 2.2 Keep the checkout-drift check on the managed-launch and `prepare` paths only
- [ ] 2.3 Update `test_jackdaw_versions.py` for the new rules (stray path, lock/record mismatch, linked lock)

## 3. Tooling

- [ ] 3.1 `jackdaw_sync.py` `repin`: rewrite the workspace `rev` and update the Jackdaw crates before the Bevy forks and Bevy
- [ ] 3.2 `tools/jackdaw.sh link` / `unlink`: write or remove the untracked `.cargo/config.toml` `[patch]` for the crates the workspace uses, honouring `JACKDAW_SOURCE`; confirm Cargo applies config `[patch]` on the pinned toolchain
- [ ] 3.3 Add `.cargo/config.toml` to `.gitignore`

## 4. CI and docs

- [ ] 4.1 `stubby-audio.yml`: build from this repository alone; keep the fork checkout only for the drift warning
- [ ] 4.2 `docs/research/jackdaw-version-policy.md` and AGENTS.md: git pin, `link`/`unlink`, and why the editor revision must still match
