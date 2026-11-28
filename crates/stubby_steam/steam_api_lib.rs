// Steam's own library for build scripts (`include!`d by stubby_game and stubby_jackdaw,
// as `steam_audio_lib.rs` is): steamworks links `libsteam_api.so` dynamically, so a binary
// with Steam in it must find the library at run time. `$ORIGIN` finds it beside the
// executable (a release package, a Steam depot); development builds also look in
// steamworks-sys's build output, so `cargo run`, the editor's Play and tests need no
// environment. Release builds carry no machine path. From the template us-outpost-31's
// `crates/game/build.rs`.
//
// It also stamps `STUBBY_BUILD` (version and commit), which a Steam host publishes on its
// lobby and a joining player must match (`stubby_game::multiplayer::steam`).

#[allow(dead_code)]
fn steam_api_run_path() {
    if !std::env::var("TARGET").unwrap_or_default().contains("linux") {
        return; // Windows finds steam_api64.dll beside the .exe
    }
    println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN");
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        return;
    }
    if let Some(steam) = steamworks_build_output() {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{}", steam.display());
    }
}

/// The newest `target/<profile>/build/steamworks-sys-*/out` holding `libsteam_api.so`.
/// Build scripts of dependencies run first, so it exists by now.
#[allow(dead_code)]
fn steamworks_build_output() -> Option<std::path::PathBuf> {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").ok()?);
    let builds = out.parent()?.parent()?; // target/<profile>/build
    std::fs::read_dir(builds)
        .ok()?
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("steamworks-sys-")
        })
        .map(|entry| entry.path().join("out"))
        .filter(|dir| dir.join("libsteam_api.so").is_file())
        .max_by_key(|dir| dir.metadata().and_then(|m| m.modified()).ok())
}

/// `STUBBY_BUILD`: `<version>-<commit>`, or `<version>` outside a git checkout.
#[allow(dead_code)]
fn stamp_build_id() {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let git = manifest.join("../../.git");
    // HEAD's log changes on every commit and checkout (a worktree's .git is a file).
    for watched in ["HEAD", "logs/HEAD"] {
        if git.join(watched).exists() {
            println!("cargo::rerun-if-changed={}", git.join(watched).display());
        }
    }
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .current_dir(&manifest)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string());
    let id = match commit {
        Some(commit) => format!("{version}-{commit}"),
        None => version,
    };
    println!("cargo::rustc-env=STUBBY_BUILD={id}");
}
