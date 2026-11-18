include!("steam_audio_lib.rs");

fn main() {
    if std::env::var_os("CARGO_FEATURE_AUDIONIMBUS").is_none() {
        return;
    }
    let Some(dir) = steam_audio_lib_dir() else {
        return;
    };
    // Link the cached library into OUT_DIR and search there: Cargo puts a build script's
    // OUT_DIR search paths on the library path of `cargo test` and `cargo run` for every
    // package that links this crate (feature unification can make that any of them), so
    // their test binaries find it too. The game's binaries also record the cache in their
    // run path, for running them directly.
    let linked = std::env::var_os("OUT_DIR")
        .map(std::path::PathBuf::from)
        .filter(|out| {
            let linked = out.join("libphonon.so");
            let _ = std::fs::remove_file(&linked);
            std::os::unix::fs::symlink(dir.join("libphonon.so"), &linked).is_ok()
        });
    let search = linked.as_deref().unwrap_or(&dir);
    println!("cargo::rustc-link-search=native={}", search.display());
    println!("cargo::rustc-link-arg=-Wl,-rpath,{}", dir.display());
}
