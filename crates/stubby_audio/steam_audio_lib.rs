// Where this checkout's Steam Audio library is, for build scripts (`include!`d by
// stubby_audio, stubby_game and stubby_jackdaw). Cargo never builds the library: it is a
// large C++ build from pinned source (`tools/build_steam_audio_sdk.sh`, which caches it
// under `target/native/steam-audio/<commit>/`). The build scripts only point the linker
// at that cache and record it in the binaries' run path, so `cargo run` needs neither
// `STEAMAUDIO_LIB_DIR` nor `LD_LIBRARY_PATH`. Setting `STEAMAUDIO_LIB_DIR` still wins.

#[allow(dead_code)]
fn steam_audio_lib_dir() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    println!("cargo::rerun-if-env-changed=STEAMAUDIO_LIB_DIR");
    println!("cargo::rerun-if-env-changed=STEAMAUDIO_CACHE_DIR");
    if let Some(dir) = std::env::var_os("STEAMAUDIO_LIB_DIR") {
        return Some(dir.into());
    }
    let workspace = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")?).join("../..");
    let workspace = workspace.canonicalize().ok()?;
    let lock = workspace.join("tools/steam-audio-sdk.lock");
    println!("cargo::rerun-if-changed={}", lock.display());
    let lock = std::fs::read_to_string(lock).ok()?;
    let commit = lock
        .lines()
        .find_map(|line| line.trim().strip_prefix("\"commit\": \""))?
        .split('"')
        .next()?
        .to_owned();
    let cache = std::env::var_os("STEAMAUDIO_CACHE_DIR")
        .map_or_else(|| workspace.join("target/native/steam-audio"), PathBuf::from);
    let dir = cache.join(commit).join("lib/linux-x64");
    let library = dir.join("libphonon.so");
    println!("cargo::rerun-if-changed={}", library.display());
    if library.is_file() {
        Some(dir)
    } else {
        println!(
            "cargo::warning=Steam Audio is not built for this checkout; run tools/build_steam_audio_sdk.sh (linking will fail until it is)"
        );
        None
    }
}

/// Record the library's directory in this package's binaries and tests, so they find it
/// at run time.
#[allow(dead_code)]
fn steam_audio_run_path() {
    if let Some(dir) = steam_audio_lib_dir() {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{}", dir.display());
    }
}
