include!("../stubby_audio/steam_audio_lib.rs");
include!("../stubby_steam/steam_api_lib.rs");

fn main() {
    if std::env::var_os("CARGO_FEATURE_SPATIAL_AUDIO").is_some() {
        steam_audio_run_path();
    }
    steam_api_run_path();
    stamp_build_id();
}
