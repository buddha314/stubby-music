#!/usr/bin/env bash
# Build and launch the compatibility set selected by the shared Jackdaw revision.
set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${JACKDAW_SOURCE:-${repo_dir}/../jackdaw}"
export PATH="${HOME}/.cargo/bin:${PATH}"
cli_path="${source_dir}/target/debug/jd"
project_dir="${repo_dir}"
cd "$repo_dir"
command_name="${1:-open}"
# The game takes Jackdaw from the fork at the pinned revision (Cargo.toml). `link` builds it
# against the shared checkout instead, through an untracked .cargo/config.toml [patch];
# `unlink` returns to the pin. Neither edits a tracked file except Cargo.lock.
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
        python3 "${repo_dir}/tools/check_jackdaw_versions.py"
        exit 0
        ;;
esac
# The shared editor must match the pin before anything uses it. `sync` is exempt: moving
# the editor to the newest fork revision and re-pinning is its job, so drift is expected.
if [[ "$command_name" != sync ]]; then
    python3 "${repo_dir}/tools/check_jackdaw_versions.py" --source "$source_dir" --editor
fi
case "$command_name" in
    check)
        exit 0
        ;;
    prepare)
        # A shared binary update must not replace an editor the user is running.
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
        cargo run --manifest-path "${repo_dir}/Cargo.toml" -p stubby_game --locked -- --export="${project_dir}/assets/generated"
        rust_version="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["rust"])' "${project_dir}/compatibility.json")"
        cargo "+${rust_version}" build --manifest-path "${source_dir}/Cargo.toml" -p jackdaw -p jd --locked
        python3 "${repo_dir}/tools/check_jackdaw_versions.py" --source "$source_dir" --editor
        python3 - "$project_dir" "$source_dir" <<'PYRECEIPT'
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
        mkdir -p "${project_dir}/.jackdaw/target"
        if [[ ! -e "${project_dir}/.jackdaw/target/game" ]]; then
            ln -s ../../target "${project_dir}/.jackdaw/target/game"
        fi
        ;;
    sync)
        # Bevy main -> jbuehler23 main -> our fork -> this game; see tools/jackdaw_sync.py.
        JACKDAW_SOURCE="$source_dir" python3 "${repo_dir}/tools/jackdaw_sync.py"
        "$0" prepare
        ;;
    build|doctor|run)
        python3 "${repo_dir}/tools/check_jackdaw_versions.py" --source "$source_dir" --binaries
        exec "$cli_path" "$command_name" --project "$project_dir"
        ;;
    open)
        python3 "${repo_dir}/tools/check_jackdaw_versions.py" --source "$source_dir" --binaries
        if [[ ! -f "${project_dir}/assets/generated/mudflat.glb" ]]; then
            "$0" prepare
        fi
        export JACKDAW_OPEN_PROJECT="$project_dir"
        cd "$source_dir"
        exec "${source_dir}/target/debug/jackdaw"
        ;;
    *)
        echo "Usage: $0 [check|prepare|sync|link|unlink|doctor|build|run|open]" >&2
        exit 2
        ;;
esac
