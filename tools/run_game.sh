#!/usr/bin/env bash
# Run the game outside the editor.
#
#   tools/run_game.sh --seed=42                 the game: the opening scene and its seeded layout
#   tools/run_game.sh --seed=42 --bare          terrain, sky and player only (art studies)
#   STUBBY_DEV_LINKING=1 tools/run_game.sh ...  faster development linking (never for release)
#
# Other flags go to the game: --world-config=<file>, --scene=<file.bsn>, --screenshot=<png>,
# --no-seeded-layout, --layout-review=<dir>, --audio-demo and the review flags (see README).
# The first run builds the pinned Steam Audio SDK into target/native/; later runs only check it.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# This is a cache/fingerprint check on ordinary runs. The SDK rebuilds only when
# a pinned source or build input changes.
steam_audio_lib="$(tools/build_steam_audio_sdk.sh)"
export STEAMAUDIO_LIB_DIR="$steam_audio_lib"
export LD_LIBRARY_PATH="$steam_audio_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

# The game is `stubby_jackdaw`: the opening scene (`assets/scene.bsn`), its seeded layout
# and the prefab bindings. `--bare` runs `stubby_game` alone instead: the terrain, sky and
# player without the scene, for art studies and previews.
package=stubby_jackdaw
args=()
for arg in "$@"; do
    if [[ "$arg" == "--bare" ]]; then package=stubby_game; else args+=("$arg"); fi
done

features=()
if [[ "${STUBBY_DEV_LINKING:-0}" == "1" ]]; then
    features=(--features dev)
fi
exec cargo run -p "$package" "${features[@]}" -- "${args[@]}"
