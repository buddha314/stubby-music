//! Dev builds start the rules designer with the game (add-machine-rules D4): the game runs
//! `cargo run -p stubby_rules_server` from this checkout in the background and opens its
//! page, so tuning rules and machines is one loop with play. It does nothing when a
//! designer already answers on its port (it outlives the game, so its unsaved edits
//! survive a restart), when `STUBBY_NO_DESIGNER` is set (bounded test runs), or in release
//! builds. The designer stays its own program: nothing here links it. Its output goes to
//! `target/rules_designer.log`.
use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Where the designer serves (its default, `stubby_rules_server --port`).
const ADDRESS: ([u8; 4], u16) = ([127, 0, 0, 1], 7878);
/// How long to wait for a first build of the designer before giving up on opening it.
const BUILD_WAIT: Duration = Duration::from_secs(300);

/// Start the designer in the background, if this is a dev build and none is running.
pub fn launch() {
    if cfg!(not(debug_assertions)) || std::env::var_os("STUBBY_NO_DESIGNER").is_some() {
        return;
    }
    std::thread::spawn(|| {
        if start() {
            open();
        }
    });
}

fn answers() -> bool {
    TcpStream::connect_timeout(&SocketAddr::from(ADDRESS), Duration::from_millis(200)).is_ok()
}

/// Start the designer and wait until it answers. False when one was already running or it
/// could not start.
fn start() -> bool {
    if answers() {
        return false;
    }
    let workspace = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    // Its output goes to a log, never the game's: a designer that outlives the game must
    // not hold the game's terminal or pipe open.
    let log = std::fs::create_dir_all(format!("{workspace}/target"))
        .and_then(|()| std::fs::File::create(format!("{workspace}/target/rules_designer.log")));
    let (out, err) = match log.and_then(|log| Ok((log.try_clone()?, log))) {
        Ok((out, err)) => (Stdio::from(out), Stdio::from(err)),
        Err(_) => (Stdio::null(), Stdio::null()),
    };
    let started = Command::new(cargo)
        .args(["run", "-q", "-p", "stubby_rules_server"])
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .spawn();
    if let Err(error) = started {
        eprintln!("rules designer: could not start it: {error}");
        return false;
    }
    let since = Instant::now();
    while !answers() {
        if since.elapsed() > BUILD_WAIT {
            eprintln!("rules designer: did not start within {BUILD_WAIT:?}");
            return false;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    true
}

/// Open the designer's page in the desktop's browser.
fn open() {
    let url = format!("http://{}/", SocketAddr::from(ADDRESS));
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    if let Err(error) = Command::new(opener)
        .arg(&url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        eprintln!("rules designer: open {url} ({error})");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "spawns cargo; run by hand"]
    fn the_designer_starts_and_answers_once() {
        assert!(super::start(), "starts the designer");
        assert!(!super::start(), "a second start finds it running");
    }
}
