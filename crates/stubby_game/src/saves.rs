//! Saved games, first slice (add-save-slots): where the player stands, which scene and seed,
//! and the time of day. Enough to open a game where it was left, or into a place someone
//! prepared (the City neighbourhood ships as `assets/saves/city_neighbourhood.json`).
//! The roadmap's full save (laid streams, racks, inventory, Dancers) grows from this file's
//! `version`.
//!
//! - `--load=<slot>` starts the game in a save: a path, a slot in the player's saves folder
//!   or a slot shipped in `assets/saves/`. The binary reads it before the world is built
//!   ([`SaveGame::from_args`]), so the scene and seed are the save's; the loading screen
//!   stays up until the player stands where the save says ([`PendingRestore`]).
//! - The main menu's Continue relaunches into the newest save ([`newest`]).
//! - The pause menu's Save Game writes the slot `quicksave` ([`write_quicksave`]).
//!
//! Saves are JSON in the OS's data folder (`~/.local/share/<APP_ID>/saves/` on Linux).
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The save format's version. Loading refuses a newer one.
pub const VERSION: u32 = 1;
/// The slot Save Game writes.
pub const QUICKSAVE: &str = "quicksave";

/// A saved game.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SaveGame {
    pub version: u32,
    /// What the player called it, shown on Continue.
    pub name: String,
    /// The scene, as `--scene` takes it (`world/asset_reviews/city_neighbourhood.bsn`).
    pub scene: String,
    /// The world seed, as `--seed` takes it.
    pub seed: String,
    /// Whether the opening's seeded layout spawns around the scene (off for authored places).
    #[serde(default = "yes")]
    pub seeded_layout: bool,
    pub player: SavedPlayer,
    /// The calendar's simulation time, in seconds.
    pub calendar_seconds: f32,
    /// When it was saved, as seconds since the Unix epoch (Continue opens the newest).
    #[serde(default)]
    pub saved_at: u64,
}

fn yes() -> bool {
    true
}

/// Where the player stood: feet position in the world (Bevy axes, metres) and facing.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct SavedPlayer {
    pub feet: [f32; 3],
    pub yaw_degrees: f32,
    #[serde(default)]
    pub pitch_degrees: f32,
}

/// The save the game was started in, waiting to put the player in place once the world is
/// up; the loading screen holds until it is gone.
#[derive(Resource, Clone, Debug)]
pub struct PendingRestore(pub SaveGame);

/// The opening's seeded layout is off for this run (a save of an authored place).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SkipSeededLayout;

impl SaveGame {
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let save: Self =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if save.version > VERSION {
            return Err(format!(
                "{}: save version {} is newer than this game's {VERSION}",
                path.display(),
                save.version
            ));
        }
        Ok(save)
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, text + "\n").map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The save named by `--load=<slot>`, if any.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Result<Self, String>> {
        let slot = args
            .into_iter()
            .find_map(|arg| arg.strip_prefix("--load=").map(str::to_owned))?;
        Some(resolve(&slot).and_then(|path| Self::read(&path)))
    }

    /// The launch arguments that open the game in this save's scene and seed.
    pub fn world_args(&self) -> Vec<String> {
        let mut args = vec![
            format!("--scene={}", self.scene),
            format!("--seed={}", self.seed),
        ];
        if !self.seeded_layout {
            args.push("--no-seeded-layout".into());
        }
        args
    }
}

/// The player's saves folder.
pub fn saves_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    };
    base.unwrap_or_else(|| PathBuf::from("."))
        .join(crate::settings::APP_ID)
        .join("saves")
}

/// Saves shipped with the game.
pub fn shipped_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/saves"))
}

/// A slot's file: a path as given, else `<slot>.json` in the player's saves, else shipped.
pub fn resolve(slot: &str) -> Result<PathBuf, String> {
    let given = PathBuf::from(slot);
    if given.extension().is_some() && given.is_file() {
        return Ok(given);
    }
    for dir in [saves_dir(), shipped_dir()] {
        let path = dir.join(format!("{slot}.json"));
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!(
        "no save named {slot:?} (looked in {} and {})",
        saves_dir().display(),
        shipped_dir().display()
    ))
}

/// The newest save, the player's own or shipped: what Continue opens.
pub fn newest() -> Option<(PathBuf, SaveGame)> {
    let mut found: Vec<(PathBuf, SaveGame)> = [saves_dir(), shipped_dir()]
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .filter_map(|path| SaveGame::read(&path).ok().map(|save| (path, save)))
        .collect();
    found.sort_by_key(|(_, save)| save.saved_at);
    found.pop()
}

/// Relaunch the game into a save: the sure way to start a different world (as Exit to Main
/// Menu does).
pub fn relaunch_into(path: &Path, exit: &mut MessageWriter<AppExit>, commands: &mut Commands) {
    crate::settings::save_now(commands);
    if let Err(error) = std::env::current_exe().and_then(|exe| {
        std::process::Command::new(exe)
            .arg(format!("--load={}", path.display()))
            .spawn()
    }) {
        error!("Could not open the save {}: {error}", path.display());
        return;
    }
    exit.write(AppExit::Success);
}

pub struct SavesPlugin;
impl Plugin for SavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, restore);
    }
}

/// How long after the world is up the player is placed: the scene's colliders must exist
/// (they are built the frame after the models appear) before the body stands on them.
const SETTLE_SECONDS: f32 = 0.75;

/// Once the world is up, stand the player where the save says and set the clock.
fn restore(
    pending: Option<Res<PendingRestore>>,
    ready: Option<Res<crate::menus::WorldReady>>,
    time: Res<Time<Real>>,
    mut since: Local<Option<f32>>,
    mut calendar: ResMut<crate::calendar::Calendar>,
    mut bodies: Query<(
        &stubby_player::movement::MoveBody,
        &mut Transform,
        Option<&mut avian3d::prelude::Position>,
        Option<&mut avian3d::prelude::LinearVelocity>,
    )>,
    mut cameras: Query<
        &mut Transform,
        (
            With<stubby_player::controls::Player>,
            Without<stubby_player::movement::MoveBody>,
        ),
    >,
    mut commands: Commands,
) {
    let (Some(pending), Some(_)) = (pending, ready) else {
        return;
    };
    let now = time.elapsed_secs();
    let start = *since.get_or_insert(now);
    if now - start < SETTLE_SECONDS {
        return;
    }
    let Some((body, mut transform, position, velocity)) = bodies.iter_mut().next() else {
        return;
    };
    let save = &pending.0;
    let [x, y, z] = save.player.feet;
    let centre = stubby_player::movement::body_centre(Vec3::new(x, y, z));
    transform.translation = centre;
    if let Some(mut position) = position {
        position.0 = centre;
    }
    if let Some(mut velocity) = velocity {
        velocity.0 = Vec3::ZERO;
    }
    let facing = Quat::from_euler(
        EulerRot::YXZ,
        save.player.yaw_degrees.to_radians(),
        save.player.pitch_degrees.to_radians(),
        0.,
    );
    if let Ok(mut camera) = cameras.get_mut(body.camera) {
        camera.translation = Vec3::new(x, y + stubby_player::controls::EYE_HEIGHT, z);
        camera.rotation = facing;
    }
    calendar.elapsed = std::time::Duration::from_secs_f32(save.calendar_seconds.max(0.));
    info!("Opened the save {:?} ({})", save.name, save.scene);
    commands.remove_resource::<PendingRestore>();
}

/// Write the slot `quicksave` from the game as it is now.
pub fn write_quicksave(world: &mut World) -> Result<PathBuf, String> {
    let config = world
        .get_resource::<crate::world_generation::WorldConfig>()
        .ok_or("no world config")?
        .clone();
    let calendar_seconds = world
        .get_resource::<crate::calendar::Calendar>()
        .map_or(0., |c| c.elapsed.as_secs_f32());
    let mut cameras = world.query_filtered::<&Transform, With<stubby_player::controls::Player>>();
    let camera = *cameras.iter(world).next().ok_or("no player")?;
    let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let feet = camera.translation - Vec3::Y * stubby_player::controls::EYE_HEIGHT;
    let save = SaveGame {
        version: VERSION,
        name: "Quicksave".into(),
        scene: config.scene,
        seed: config.seed,
        seeded_layout: !world.contains_resource::<SkipSeededLayout>(),
        player: SavedPlayer {
            feet: feet.to_array(),
            yaw_degrees: yaw.to_degrees(),
            pitch_degrees: pitch.to_degrees(),
        },
        calendar_seconds,
        saved_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    };
    let path = saves_dir().join(format!("{QUICKSAVE}.json"));
    save.write(&path)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_save_round_trips_and_names_its_world() {
        let dir = std::env::temp_dir().join(format!("stubby-save-test-{}", std::process::id()));
        let save = SaveGame {
            version: VERSION,
            name: "Test".into(),
            scene: "world/asset_reviews/city_neighbourhood.bsn".into(),
            seed: "42".into(),
            seeded_layout: false,
            player: SavedPlayer {
                feet: [1., 2., 3.],
                yaw_degrees: -15.,
                pitch_degrees: 0.,
            },
            calendar_seconds: 2610.,
            saved_at: 7,
        };
        let path = dir.join("slot.json");
        save.write(&path).unwrap();
        assert_eq!(SaveGame::read(&path).unwrap(), save);
        assert_eq!(
            save.world_args(),
            [
                "--scene=world/asset_reviews/city_neighbourhood.bsn",
                "--seed=42",
                "--no-seeded-layout"
            ]
        );
        let loaded = SaveGame::from_args([format!("--load={}", path.display())])
            .unwrap()
            .unwrap();
        assert_eq!(loaded.name, "Test");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn the_shipped_city_save_reads() {
        let save = SaveGame::read(&shipped_dir().join("city_neighbourhood.json")).unwrap();
        assert!(!save.seeded_layout);
        assert!(save.scene.ends_with(".bsn"));
    }
}
