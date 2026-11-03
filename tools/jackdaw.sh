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
    sync)
        # Follow our fork's tested line: fast-forward the shared checkout to
        # origin/dev/0.20, re-pin compatibility.json, refresh Cargo.lock, verify,
        # and rebuild the shared editor. Commit the result on a branch.
        branch="${JACKDAW_BRANCH:-dev/0.20}"
        record="${project_dir}/compatibility.json"
        git -C "$source_dir" fetch --quiet origin
        git -C "$source_dir" fetch --quiet upstream
        pinned="$(git -C "$source_dir" rev-parse HEAD)"
        target="$(git -C "$source_dir" rev-parse "origin/${branch}")"
        if [[ "$pinned" == "$target" ]]; then
            echo "Already at origin/${branch} (${target:0:8})."
            exit 0
        fi
        [[ "$(git -C "$source_dir" branch --show-current)" == "$branch" ]] \
            || { echo "Shared Jackdaw is not on ${branch}; sync it by hand." >&2; exit 1; }
        git -C "$source_dir" merge-base --is-ancestor "$pinned" "$target" \
            || { echo "origin/${branch} does not contain ${pinned:0:8}; integrate deliberately." >&2; exit 1; }
        upstream_branch="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["jackdaw"]["upstream_branch"])' "$record")"
        git -C "$source_dir" merge --quiet --ff-only "$target"
        base="$(git -C "$source_dir" merge-base "$target" "upstream/${upstream_branch}")"
        python3 - "$record" "$target" "$base" <<'PYPIN'
import json, pathlib, sys
path, revision, base = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
record = json.loads(path.read_text())
record['jackdaw']['revision'] = revision
record['jackdaw']['upstream_revision'] = base
path.write_text(json.dumps(record, indent=2) + '\n')
PYPIN
        # Refresh the lock for the moved path crates (no --locked here on purpose).
        cargo metadata --manifest-path "${repo_dir}/Cargo.toml" --format-version 1 > /dev/null
        if ! python3 "${repo_dir}/tools/check_jackdaw_versions.py" --source "$source_dir"; then
            echo "The new fork revision changes the Bevy/Rust set; integrate it deliberately." >&2
            echo "Roll back the shared checkout and the pin with:" >&2
            echo "  git -C ${source_dir} checkout ${branch} && git -C ${source_dir} reset --keep ${pinned}" >&2
            echo "  git -C ${repo_dir} checkout -- jackdaw/compatibility.json Cargo.lock" >&2
            exit 1
        fi
        "$0" prepare
        echo "Synced Jackdaw ${pinned:0:8} -> ${target:0:8}. Commit jackdaw/compatibility.json and Cargo.lock on a branch."
        git -C "$repo_dir" status --short -- jackdaw/compatibility.json Cargo.lock
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
        echo "Usage: $0 [check|prepare|sync|doctor|build|run|open]" >&2
        exit 2
        ;;
esac
