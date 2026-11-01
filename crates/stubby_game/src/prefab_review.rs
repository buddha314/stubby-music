//! Opt-in evidence driver for the bounded Jackdaw prefab review.
use super::{
    calendar::Calendar,
    prefab_presentation::{AssetHoverEvent, AssetUsed},
    prefabs::{AssetState, PlacedAsset, PrefabError, WorldSources},
    resources::{PlayerInventory, ResourceNode},
};
use bevy::prelude::*;
use std::{io::Write, path::PathBuf, time::Instant};
#[derive(Resource)]
struct Review {
    path: PathBuf,
    start: Instant,
    stage: usize,
    next: f32,
    hovers: usize,
    uses: usize,
    captured: [bool; 6],
    saw_animation: bool,
    saw_flash: bool,
    near_pass: bool,
    far_seen: bool,
    rack_opened: bool,
    rack_loaded: bool,
    /// Seconds the main timeline waited while the rack steps ran.
    paused: f32,
    rack_step: usize,
    rack_wait: f32,
    release: bool,
    press_at: Vec2,
    rack_configured: bool,
}
impl Review {
    /// Seconds on the main timeline (wall time minus the rack steps' pause).
    fn now(&self) -> f32 {
        self.start.elapsed().as_secs_f32() - self.paused
    }
    /// The rack steps run once the rack is up and pause the main timeline.
    fn rack_busy(&self) -> bool {
        self.rack_loaded && self.rack_step < RACK_STEPS
    }
}
/// Mount; take the green cable from the comb, GATE, TRIG; take the red one, OUT, IN;
/// then the patched screenshot. Only what the rack shows is patched (AGENTS.md).
const RACK_STEPS: usize = 8;
pub struct PrefabReviewPlugin;
impl Plugin for PrefabReviewPlugin {
    fn build(&self, app: &mut App) {
        let Some(path) = std::env::args()
            .find_map(|arg| arg.strip_prefix("--prefab-review=").map(PathBuf::from))
        else {
            return;
        };
        app.insert_resource(Review {
            path,
            start: Instant::now(),
            stage: 0,
            next: 0.,
            hovers: 0,
            uses: 0,
            captured: [false; 6],
            saw_animation: false,
            saw_flash: false,
            near_pass: false,
            far_seen: false,
            rack_opened: false,
            rack_loaded: false,
            paused: 0.,
            rack_step: 0,
            // Let the rack's models load before the first click and screenshot.
            rack_wait: 0.5,
            release: false,
            press_at: Vec2::ZERO,
            rack_configured: false,
        })
        .add_systems(Startup, start.after(super::setup))
        .add_systems(
            PreUpdate,
            (drive_rack, drive).chain().after(bevy::input::InputSystems),
        )
        .add_systems(PostUpdate, observe.after(super::pulse::BugEffects));
    }
}
fn write(review: &Review, event: serde_json::Value) {
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(review.path.join("events.jsonl"))?;
        writeln!(file, "{event}")
    })();
    if let Err(error) = result {
        error!("prefab review evidence: {error}");
    }
}
fn start(mut review: ResMut<Review>, mut calendar: ResMut<Calendar>) {
    if let Err(error) = std::fs::create_dir_all(&review.path) {
        error!("review directory: {error}");
    }
    if let Err(error) = std::fs::File::create(review.path.join("events.jsonl")) {
        error!("review file: {error}");
    }
    review.start = Instant::now();
    calendar.elapsed = std::time::Duration::from_secs(1350);
    write(&review, serde_json::json!({"event":"started"}));
}
fn drive(
    mut review: ResMut<Review>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut camera: Query<
        &mut Transform,
        (
            With<Camera3d>,
            Without<super::machine_config::RackHudCamera>,
        ),
    >,
    mut roots: Query<(&PlacedAsset, &mut AssetState)>,
    mut calendar: ResMut<Calendar>,
) {
    keys.reset(KeyCode::KeyE);
    keys.reset(KeyCode::Escape);
    if review.rack_busy() {
        return;
    }
    let elapsed = review.now();
    let times = [0.8, 1.5, 2.1, 2.7, 3.3, 3.9, 4.5, 5.2, 6.4, 7.7];
    if review.stage >= times.len() || elapsed < times[review.stage] {
        return;
    }
    if review.stage == 0 && !review.saw_flash {
        return;
    }
    match review.stage {
        0 | 3 | 5 => keys.press(KeyCode::KeyE),
        1 => {
            for mut camera in &mut camera {
                let target = Vec3::new(-1.2, 0.8, 4.);
                *camera = camera.looking_at(target, Vec3::Y);
            }
        }
        2 => {
            for mut camera in &mut camera {
                camera.translation = Vec3::new(2., 1.7, 4.);
                *camera = camera.looking_at(Vec3::new(2., 0.8, 3.), Vec3::Y);
            }
        }
        4 => {
            for mut camera in &mut camera {
                camera.translation = Vec3::new(-4., 1.7, 2.2);
                *camera = camera.looking_at(Vec3::new(-4., 0.3, 1.), Vec3::Y);
            }
        }
        6 => {
            for (asset, mut state) in &mut roots {
                if asset.asset.starts_with("scrub_oak_") || asset.asset.starts_with("desert_") {
                    state.growth = 1.;
                    state.light_quality = 1.;
                }
            }
        }
        7 => {
            calendar.elapsed += std::time::Duration::from_secs(61);
            for mut camera in &mut camera {
                camera.translation = Vec3::new(0., 1.7, 6.);
                *camera = camera.looking_at(Vec3::new(3., 1.5, -1.), Vec3::Y);
            }
        }
        8 => {
            for mut camera in &mut camera {
                camera.translation = Vec3::new(0., 2., 80.);
                *camera = camera.looking_at(Vec3::new(3., 1.5, -1.), Vec3::Y);
            }
        }
        9 => {
            keys.press(KeyCode::Escape);
            for mut camera in &mut camera {
                camera.translation = Vec3::new(0., 1.7, 6.);
                *camera = camera.looking_at(Vec3::new(3., 1.5, -1.), Vec3::Y);
            }
        }
        _ => {}
    }
    write(
        &review,
        serde_json::json!({"event":"action","stage":review.stage,"seconds":elapsed}),
    );
    review.stage += 1;
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Evidence samples independent gameplay and rendering stores.
fn observe(
    mut commands: Commands,
    mut review: ResMut<Review>,
    sources: Res<WorldSources>,
    player: Res<PlayerInventory>,
    roots: Query<(&PlacedAsset, Option<&ResourceNode>, &Visibility)>,
    errors: Query<&PrefabError>,
    players: Query<&AnimationPlayer>,
    nodes: Query<(&Name, &Transform)>,
    cameras: Query<
        &bevy::camera::RenderTarget,
        (
            With<Camera3d>,
            Without<super::machine_config::RackHudCamera>,
        ),
    >,
    mut hover: MessageReader<AssetHoverEvent>,
    mut used: MessageReader<AssetUsed>,
    mut rack: MessageReader<super::machine_config::MachineConfigurationOpened>,
    mut configured: MessageReader<super::rack::MachineConfigured>,
    screens: Res<super::controls::ScreenMode>,
    flat: Query<(&super::prefab_presentation::AssetPart, &Transform)>,
) {
    let elapsed = review.now();
    review.hovers += hover.read().count();
    review.rack_configured |= configured.read().next().is_some();
    review.uses += used.read().count();
    review.rack_opened |= rack.read().next().is_some();
    review.rack_loaded |=
        screens.open.is_some() && nodes.iter().any(|(name, _)| name.as_str() == "RailTop");
    let phases: Vec<_> = players
        .iter()
        .flat_map(|player| {
            player
                .playing_animations()
                .map(|(_, animation)| animation.seek_time())
        })
        .collect();
    if phases.iter().any(|phase| *phase > 0.01) {
        review.saw_animation = true;
    }
    if roots.iter().any(|(asset, node, visible)| {
        asset.asset == "lightning_bug"
            && node.is_some_and(|node| !node.collected)
            && *visible == Visibility::Visible
    }) {
        review.saw_flash = true;
    }
    let standing = flat.iter().any(|(part, transform)| {
        part.flattened
            && matches!(part.name.as_str(), "Standing" | "ReedClump_Far")
            && transform.scale.length_squared() > 0.1
    }) || nodes.iter().any(|(name, transform)| {
        name.as_str() == "Standing" && transform.scale.length_squared() > 0.1
    });
    let leaves = flat
        .iter()
        .filter(|(part, transform)| {
            part.flattened
                && part
                    .name
                    .strip_prefix("Foliage_")
                    .is_some_and(|index| index.parse::<usize>().is_ok())
                && transform.scale.length_squared() > 0.1
        })
        .count()
        + nodes
            .iter()
            .filter(|(name, transform)| {
                name.as_str()
                    .strip_prefix("Foliage_")
                    .is_some_and(|index| index.parse::<usize>().is_ok())
                    && transform.scale.length_squared() > 0.1
            })
            .count();
    let far = flat.iter().any(|(part, transform)| {
        part.flattened
            && part.name == "ScrubOak_Far_Foliage_Node"
            && transform.scale.length_squared() > 0.1
    }) || nodes.iter().any(|(name, transform)| {
        name.as_str() == "ScrubOak_Far_Foliage_Node" && transform.scale.length_squared() > 0.1
    });
    if review.stage == 9 && far {
        review.far_seen = true;
    }
    if elapsed >= review.next {
        write(
            &review,
            serde_json::json!({"event":"sample","seconds":elapsed,"assets":roots.iter().count(),"sources":sources.0.len(),"errors":errors.iter().map(|error|error.0.clone()).collect::<Vec<_>>(),"square":player.items.quantity("square_wave"),"reeds":player.items.quantity("reed"),"standing":standing,"leaves":leaves,"far_leaf":far,"phases":phases,"hovers":review.hovers,"uses":review.uses}),
        );
        review.next = elapsed + 0.25;
    }
    let capture = if elapsed > 1.3 && !review.captured[0] {
        Some((0, "initial.png"))
    } else if review.stage >= 5 && !review.captured[1] {
        Some((1, "burbler-hover.png"))
    } else if review.rack_loaded
        && review.rack_step == 0
        && review.rack_wait < 0.1
        && !review.captured[5]
    {
        Some((5, "rack-open.png"))
    } else if review.rack_step >= RACK_STEPS && !review.captured[4] {
        Some((4, "rack-patched.png"))
    } else if review.stage >= 8 && !review.captured[2] {
        Some((2, "regrown.png"))
    } else if review.stage >= 10 && elapsed > 9.2 && !review.captured[3] {
        Some((3, "lod-return.png"))
    } else {
        None
    };
    if let Some((index, name)) = capture
        && let Ok(target) = cameras.single()
    {
        review.captured[index] = true;
        commands
            .spawn(bevy::render::view::screenshot::Screenshot(target.clone()))
            .observe(bevy::render::view::screenshot::save_to_disk(
                review.path.join(name),
            ));
    }
    if review.stage == 8 && elapsed > 5.8 && !review.near_pass {
        review.near_pass = roots.iter().count() == 7
            && sources.0.len() == 4
            && errors.is_empty()
            // Two from the bug, one mounted in the burbler's rack as its Square VCO.
            && player.items.quantity("square_wave") == 1
            && player.items.quantity("reed") == 6
            && standing
            && leaves > 0
            && review.saw_animation
            && review.saw_flash
            && review.hovers >= 3
            && review.uses >= 1;
        write(
            &review,
            serde_json::json!({"event":"near_check","success":review.near_pass,"seconds":elapsed,"saw_flash":review.saw_flash,"saw_animation":review.saw_animation}),
        );
    }
    if review.stage == 10 && elapsed > 9.2 {
        let success = review.near_pass
            && review.rack_opened
            && review.rack_loaded
            && screens.open.is_none()
            && review.far_seen
            && standing
            && leaves > 0
            && review.rack_configured
            // The comb's two cables are free: the player's ten are untouched.
            && player.items.quantity("patch_cable") == 10
            && errors.is_empty()
            && player.items.quantity("square_wave") == 1
            && player.items.quantity("reed") == 6;
        write(
            &review,
            serde_json::json!({"event":"complete","success":success,"seconds":elapsed,"near_pass":review.near_pass,"rack_opened":review.rack_opened,"rack_loaded":review.rack_loaded,"rack_configured":review.rack_configured,"far_seen":review.far_seen,"saw_animation":review.saw_animation,"saw_flash":review.saw_flash}),
        );
        review.stage = 11;
    }
}

/// The rack steps: real pointer clicks through bevy_picking, on the mount button (UI
/// picking) and on the jacks (mesh picking through the rack camera).
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Clicks need window, both pickers' targets and the rack state.
fn drive_rack(
    mut review: ResMut<Review>,
    time: Res<Time<Real>>,
    windows: Query<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    hud: Query<(&Camera, &GlobalTransform), With<super::machine_config::RackHudCamera>>,
    jacks: Query<(&super::machine_config::JackHit, &GlobalTransform)>,
    plugs: Query<(&super::rack::CombCable, &GlobalTransform), Without<Mesh3d>>,
    buttons: Query<(&bevy::ui::UiGlobalTransform, &ComputedNode), With<super::rack::MountButton>>,
    racks: Query<&super::rack::RackState>,
    held: Res<super::rack::HeldCable>,
    hovers: Res<bevy::picking::hover::HoverMap>,
    names: Query<(
        Option<&Name>,
        Has<super::machine_config::JackHit>,
        Has<Node>,
    )>,
    targets: Query<
        (
            Option<&bevy::camera::RenderTarget>,
            Has<super::machine_config::RackHudCamera>,
        ),
        With<Camera3d>,
    >,
    mut input: MessageWriter<bevy::picking::pointer::PointerInput>,
) {
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    if !review.rack_busy() {
        return;
    }
    review.paused += time.delta_secs();
    let Ok((window_entity, window)) = windows.single() else {
        return;
    };
    // Aim at whatever the game camera draws into: the window when run alone, an image
    // when Jackdaw's Play hosts it.
    let Some(target) = targets
        .iter()
        .find(|(_, hud)| !hud)
        .and_then(|(t, _)| t.cloned())
        .unwrap_or_default()
        .normalize(Some(window_entity))
    else {
        return;
    };
    let at = |position: Vec2| Location {
        target: target.clone(),
        position,
    };
    if review.release {
        // Second frame of a click: lift the button where it went down.
        let position = review.press_at;
        input.write(PointerInput::new(
            PointerId::Mouse,
            at(position),
            PointerAction::Release(PointerButton::Primary),
        ));
        review.release = false;
        return;
    }
    review.rack_wait -= time.delta_secs();
    if review.rack_wait > 0. {
        return;
    }
    let jack = |place: &str, port: &str| {
        let (camera, transform) = hud.single().ok()?;
        let (_, world) = jacks
            .iter()
            .find(|(j, _)| j.0.place == place && j.0.port == port)?;
        // Already in target coordinates: world_to_viewport includes the viewport's offset.
        camera
            .world_to_viewport(transform, world.translation())
            .ok()
    };
    // A hanging cable's plug on the comb (just above its rest point, on the plug body).
    let comb = |index: usize| {
        let (camera, transform) = hud.single().ok()?;
        let (_, at) = plugs.iter().find(|(c, _)| c.0 == index)?;
        camera
            .world_to_viewport(transform, at.translation() + Vec3::Y * 0.03)
            .ok()
    };
    let step = review.rack_step;
    let point = match step {
        0 => buttons
            .iter()
            .next()
            .map(|(t, node)| t.translation * node.inverse_scale_factor()),
        1 => comb(0),
        2 => jack("clock", "gate"),
        3 => jack("volume", "trig"),
        4 => comb(1),
        5 => jack("vco", "out"),
        6 => jack("volume", "in"),
        _ => None,
    };
    if step + 1 < RACK_STEPS {
        // Wait (briefly) for the target to exist: the VCO's jacks appear once it is mounted.
        let Some(position) = point else { return };
        for action in [
            PointerAction::Move { delta: Vec2::ZERO },
            PointerAction::Press(PointerButton::Primary),
        ] {
            input.write(PointerInput::new(PointerId::Mouse, at(position), action));
        }
        review.release = true;
        review.press_at = position;
    }
    let state = racks.iter().next();
    write(
        &review,
        serde_json::json!({"event":"rack_step","step":step,"at":point.map(|p| [p.x, p.y]),
            "window":[window.width(), window.height(), window.scale_factor()],
            "hovered":hovers.get(&PointerId::Mouse).map(|h| h.keys().map(|e| names.get(*e).map(|(n, jack, ui)| format!("{:?} jack={jack} ui={ui}", n.map(Name::as_str))).unwrap_or_default()).collect::<Vec<_>>()),
            "held":held.cable,
            "mounted":state.map(|s| s.mounted.len()),"patched":state.map(|s| s.patched.clone()),"own":state.map(|s| s.own.len())}),
    );
    review.rack_step += 1;
    review.rack_wait = 0.25;
}
