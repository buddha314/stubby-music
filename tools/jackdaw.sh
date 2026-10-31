#!/usr/bin/env bash
# Use the local editor and matching CLI, never the stale installed jd.
set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${JACKDAW_SOURCE:-/home/buddha/src/jackdaw}"
export PATH="${HOME}/.cargo/bin:${PATH}"
cli_path="${source_dir}/target/debug/jd"
project_dir="${repo_dir}/jackdaw"
cd "$repo_dir"
command_name="${1:-open}"
case "$command_name" in
    prepare)
        cargo run --manifest-path "${repo_dir}/Cargo.toml" -p stubby_game -- --export="${project_dir}/assets/generated"
        cargo +1.99.0 build --manifest-path "${source_dir}/Cargo.toml" -p jd --locked
        ;;
    build|doctor|run)
        if [[ ! -x "$cli_path" ]] || [[ "$("$cli_path" --version)" != "jd 0.21.0-dev (targets bevy main at c9c397ba1a73e6cfffe957318e0359330b701865)" ]]; then
            "$0" prepare
        fi
        exec "$cli_path" "$command_name" --project "$project_dir"
        ;;
    open)
        if [[ ! -f "${project_dir}/assets/generated/mudflat.glb" ]]; then
            "$0" prepare
        fi
        export JACKDAW_OPEN_PROJECT="$project_dir"
        cd "$source_dir"
        exec "${source_dir}/target/debug/jackdaw"
        ;;
    *)
        echo "Usage: $0 [prepare|doctor|build|run|open]" >&2
        exit 2
        ;;
esac
