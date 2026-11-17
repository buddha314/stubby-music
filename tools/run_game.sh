#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# This is a cache/fingerprint check on ordinary runs. The SDK rebuilds only when
# a pinned source or build input changes.
steam_audio_lib="$(tools/build_steam_audio_sdk.sh)"
export STEAMAUDIO_LIB_DIR="$steam_audio_lib"
export LD_LIBRARY_PATH="$steam_audio_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

if [[ "${STUBBY_DEV_LINKING:-0}" == "1" ]]; then
    exec cargo run -p stubby_game --features dev -- "$@"
fi
exec cargo run -p stubby_game -- "$@"
