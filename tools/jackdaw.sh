#!/usr/bin/env bash
# The game's one entry point to the shared Jackdaw editor and the engine pins.
#
#   tools/jackdaw.sh [open]   open this project in the shared editor (../jackdaw)
#   tools/jackdaw.sh prepare  build the shared editor at the pin and record its hashes
#   tools/jackdaw.sh sync     Bevy main -> avian forks -> our Jackdaw fork -> this game,
#                             then prepare (tools/engine_sync.py; docs/ENGINE-UPDATES.md)
#   tools/jackdaw.sh ceiling  newest Bevy main commit this game's avian forks build on
#   tools/jackdaw.sh smoke    one bounded 20-second Play in the editor (tools/timed_play.py)
#   tools/jackdaw.sh build|run|doctor   the editor's `jd` for this project
#   tools/jackdaw.sh link|unlink        build against the local editor checkout, or the pin
#   tools/jackdaw.sh check    the pins and the shared checkout agree (changes nothing)
#
# JACKDAW_SOURCE overrides the shared checkout (default: ../jackdaw beside this repo).
#
# This file is shared, byte for byte, by every game made from the template
# (VerdantAI/us-outpost-31; tools/update_engine_tools.sh copies it). A game's own
# steps go in tools/game_hooks.sh, which may define:
#   game_env      before anything builds or runs the game (prepare, build, run,
#                 doctor, open, smoke): export environment, build native SDKs
#   game_prepare  in `prepare`, before the shared editor is built: generated assets
#   game_ready    in `open`; return non-zero when `prepare` must run first
set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${JACKDAW_SOURCE:-${repo_dir}/../jackdaw}"
export PATH="${HOME}/.cargo/bin:${PATH}"
cli_path="${source_dir}/target/debug/jd"
cd "$repo_dir"
command_name="${1:-open}"
check() { python3 "${repo_dir}/tools/check_engine_pins.py" --source "$source_dir" "$@"; }
game_env() { :; }
game_prepare() { :; }
game_ready() { return 0; }
if [[ -f "${repo_dir}/tools/game_hooks.sh" ]]; then
    # shellcheck source=/dev/null
    source "${repo_dir}/tools/game_hooks.sh"
fi

# The game takes Jackdaw from the fork at the pinned revision (Cargo.toml). `link`
# builds it against the shared checkout instead, through an untracked
# .cargo/config.toml [patch]; `unlink` returns to the pin. Neither edits a tracked
# file except Cargo.lock, and the pin check fails while linked, so it cannot be
# committed by accident.
link_config="${repo_dir}/.cargo/config.toml"
case "$command_name" in
    link)
        python3 - "$repo_dir" "$source_dir" "$link_config" <<'PYLINK'
import json, pathlib, sys, tomllib
repo, source, config = map(pathlib.Path, sys.argv[1:])
marker = '# Written by tools/jackdaw.sh link'
if config.exists() and not config.read_text().startswith(marker):
    sys.exit(f'{config} exists and was not written by link; merge the [patch] by hand.')
origin = json.loads((repo / 'compatibility.json').read_text())['jackdaw']['origin']
names = sorted(n for n in tomllib.loads((repo / 'Cargo.toml').read_text())['workspace']['dependencies']
               if n.startswith('jackdaw_'))
lines = [marker + '; tools/jackdaw.sh unlink removes it. Never commit it.', f'[patch."{origin}"]']
for name in names:
    crate = (source / 'crates' / name).resolve()
    if not (crate / 'Cargo.toml').is_file():
        sys.exit(f'{crate} is not a Jackdaw crate; set JACKDAW_SOURCE to the shared checkout.')
    lines.append(f'{name} = {{ path = "{crate}" }}')
config.parent.mkdir(exist_ok=True)
config.write_text('\n'.join(lines) + '\n')
print(f'Linked {len(names)} Jackdaw crates to {source.resolve()}; run tools/jackdaw.sh unlink before committing.')
PYLINK
        cargo metadata --format-version 1 >/dev/null
        exit 0
        ;;
    unlink)
        if [[ -f "$link_config" ]] && head -n1 "$link_config" | grep -q '^# Written by tools/jackdaw.sh link'; then
            rm "$link_config"
            rmdir --ignore-fail-on-non-empty "${repo_dir}/.cargo"
        fi
        cargo metadata --format-version 1 >/dev/null
        check
        exit 0
        ;;
    ceiling)
        shift
        exec python3 "${source_dir}/scripts/bevy_ceiling.py" ceiling \
            --forks "${repo_dir}/bevy_main_forks.json" "$@"
        ;;
esac

# The shared editor must match the pin before anything uses it. `sync` is exempt:
# moving the editor to the newest fork revision and re-pinning is its job.
if [[ "$command_name" != sync ]]; then
    check --editor
fi
case "$command_name" in
    check)
        exit 0
        ;;
    prepare)
        game_env
        # A shared binary update must not replace an editor someone is running.
        python3 - "$source_dir" <<'PYIDLE'
import os, pathlib, sys
expected = str((pathlib.Path(sys.argv[1]) / 'target/debug/jackdaw').resolve())
for process in pathlib.Path('/proc').iterdir():
    if not process.name.isdecimal():
        continue
    try:
        executable = os.readlink(process / 'exe').removesuffix(' (deleted)')
    except (OSError, PermissionError):
        continue
    if executable == expected:
        sys.exit('Shared Jackdaw is running; close it before rebuilding shared binaries.')
PYIDLE
        game_prepare
        rust_version="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["rust"])' "${repo_dir}/compatibility.json")"
        cargo "+${rust_version}" build --manifest-path "${source_dir}/Cargo.toml" -p jackdaw -p jd --locked
        check --editor
        python3 - "$repo_dir" "$source_dir" <<'PYRECEIPT'
import hashlib, json, pathlib, sys
project, source = map(pathlib.Path, sys.argv[1:])
record = json.loads((project / 'compatibility.json').read_text())
receipt = {'revision': record['jackdaw']['revision'], 'sha256': {}}
for name in ('jackdaw', 'jd'):
    with (source / 'target/debug' / name).open('rb') as stream:
        receipt['sha256'][name] = hashlib.file_digest(stream, 'sha256').hexdigest()
path = project / '.jackdaw/editor-build.json'
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(json.dumps(receipt, indent=2) + '\n')
PYRECEIPT
        # Play builds the game into .jackdaw/target/game; share the workspace target
        # so the editor's first Play does not compile Bevy a second time.
        mkdir -p "${repo_dir}/.jackdaw/target"
        if [[ ! -e "${repo_dir}/.jackdaw/target/game" ]]; then
            ln -s ../../target "${repo_dir}/.jackdaw/target/game"
        fi
        ;;
    sync)
        JACKDAW_SOURCE="$source_dir" python3 "${repo_dir}/tools/engine_sync.py"
        "$0" prepare
        ;;
    smoke)
        shift
        game_env
        exec python3 "${repo_dir}/tools/timed_play.py" --source "$source_dir" "$@"
        ;;
    build|doctor|run)
        game_env
        check --binaries
        exec "$cli_path" "$command_name" --project "$repo_dir"
        ;;
    open)
        game_env
        if [[ ! -f "${repo_dir}/.jackdaw/editor-build.json" ]] || ! game_ready; then
            "$0" prepare
        fi
        check --binaries
        export JACKDAW_OPEN_PROJECT="$repo_dir"
        cd "$source_dir"
        exec "${source_dir}/target/debug/jackdaw"
        ;;
    *)
        echo "Usage: $0 [open|prepare|sync|ceiling|smoke|build|run|doctor|link|unlink|check]" >&2
        exit 2
        ;;
esac
