# stubby-music's own steps for the shared tools/jackdaw.sh (sourced, not run).
# Everything else in tools/ that jackdaw.sh uses is shared with the template
# (VerdantAI/us-outpost-31); tools/update_engine_tools.sh refreshes it and never
# touches this file.

# Steam Audio: build or reuse the native SDK, and make it findable at build and run
# time, before anything builds or runs the game.
game_env() {
    local steam_audio_lib
    steam_audio_lib="$("${repo_dir}/tools/build_steam_audio_sdk.sh")"
    export STEAMAUDIO_LIB_DIR="$steam_audio_lib"
    export LD_LIBRARY_PATH="$steam_audio_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
}

# Export the generated mudflat preview the editor's scene uses.
game_prepare() {
    cargo run --manifest-path "${repo_dir}/Cargo.toml" -p stubby_game --locked -- \
        --export="${repo_dir}/assets/generated"
}

# Opening needs the exported preview; without it, prepare first.
game_ready() {
    [[ -f "${repo_dir}/assets/generated/mudflat.glb" ]]
}
