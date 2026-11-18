include!("steam_audio_lib.rs");

fn main() {
    if std::env::var_os("CARGO_FEATURE_AUDIONIMBUS").is_none() {
        return;
    }
    if let Some(dir) = steam_audio_lib_dir() {
        // Propagates to every binary that links this crate.
        println!("cargo::rustc-link-search=native={}", dir.display());
        println!("cargo::rustc-link-arg=-Wl,-rpath,{}", dir.display());
    }
}
