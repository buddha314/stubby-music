#!/usr/bin/env bash
# Build and launch the compatibility set selected by the shared Jackdaw revision.
set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${JACKDAW_SOURCE:-${repo_dir}/../jackdaw}"
export PATH="${HOME}/.cargo/bin:${PATH}"
cli_path="${source_dir}/target/debug/jd"
project_dir="${repo_dir}/jackdaw"
cd "$repo_dir"
command_name="${1:-open}"
python3 "${repo_dir}/tools/check_jackdaw_versions.py" --source "$source_dir"
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
        python3 "${repo_dir}/tools/check_jackdaw_versions.py" --source "$source_dir"
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
            ln -s ../../../target "${project_dir}/.jackdaw/target/game"
        fi
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
        echo "Usage: $0 [check|prepare|doctor|build|run|open]" >&2
        exit 2
        ;;
esac
