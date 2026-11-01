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
    captured: [bool; 4],
    saw_animation: bool,
    saw_flash: bool,
    near_pass: bool,
    far_seen: bool,
}
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
            captured: [false; 4],
            saw_animation: false,
            saw_flash: false,
            near_pass: false,
            far_seen: false,
        })
        .add_systems(Startup, start.after(super::setup))
        .add_systems(PreUpdate, drive.after(bevy::input::InputSystems))
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
    mut camera: Query<&mut Transform, With<Camera3d>>,
    mut roots: Query<(&PlacedAsset, &mut AssetState)>,
    mut calendar: ResMut<Calendar>,
) {
    keys.reset(KeyCode::KeyE);
    let elapsed = review.start.elapsed().as_secs_f32();
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
                camera.translation = Vec3::new(-4., 1.7, 3.);
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
    cameras: Query<&bevy::camera::RenderTarget, With<Camera3d>>,
    mut hover: MessageReader<AssetHoverEvent>,
    mut used: MessageReader<AssetUsed>,
    flat: Query<(&super::prefab_presentation::AssetPart, &Transform)>,
) {
    let elapsed = review.start.elapsed().as_secs_f32();
    review.hovers += hover.read().count();
    review.uses += used.read().count();
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
            && player.items.quantity("square_wave") == 2
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
            && review.far_seen
            && standing
            && leaves > 0
            && errors.is_empty()
            && player.items.quantity("square_wave") == 2
            && player.items.quantity("reed") == 6;
        write(
            &review,
            serde_json::json!({"event":"complete","success":success,"seconds":elapsed,"near_pass":review.near_pass,"far_seen":review.far_seen,"saw_animation":review.saw_animation,"saw_flash":review.saw_flash}),
        );
        review.stage = 11;
    }
}
