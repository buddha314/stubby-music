//! Opt-in tripod review of the production models, never a second LOD selector.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use jackdaw_runtime::{LiveLevels, LodSwitches};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
use stubby_jackdaw::{
    calendar::{Calendar, seconds_at_solar_hour},
    prefabs::{AssetState, PlacedAsset},
};

pub struct LodReviewPlugin;
#[derive(Clone, Copy)]
struct Shot {
    name: &'static str,
    asset: &'static str,
    distance: f32,
    growth: f32,
    harvested: bool,
    scale: f32,
    fov: f32,
}
impl Shot {
    const fn at(name: &'static str, asset: &'static str, distance: f32) -> Self {
        Self {
            name,
            asset,
            distance,
            growth: 0.,
            harvested: false,
            scale: 1.,
            fov: 45.,
        }
    }
}
fn shots(flora: bool) -> Vec<Shot> {
    if flora {
        vec![
            Shot::at("oak-near", "scrub_oak_0", 30.),
            Shot::at("oak-transition", "scrub_oak_0", 45.),
            Shot::at("oak-far", "scrub_oak_0", 65.),
            Shot {
                growth: 1.,
                ..Shot::at("oak-grown-near", "scrub_oak_0", 30.)
            },
            Shot {
                growth: 1.,
                ..Shot::at("oak-grown-far", "scrub_oak_0", 65.)
            },
            Shot {
                growth: 1.,
                ..Shot::at("oak-cull", "scrub_oak_0", 270.)
            },
            Shot::at("reeds-near", "reed_clump_0", 20.),
            Shot::at("reeds-transition", "reed_clump_0", 30.),
            Shot::at("reeds-far", "reed_clump_0", 45.),
            Shot {
                harvested: true,
                ..Shot::at("reeds-harvested", "reed_clump_0", 45.)
            },
            Shot {
                growth: 1.,
                scale: 1.5,
                ..Shot::at("oak-scaled", "scrub_oak_0", 65.)
            },
            Shot {
                growth: 1.,
                fov: 70.,
                ..Shot::at("oak-wide-fov", "scrub_oak_0", 45.)
            },
        ]
    } else {
        vec![
            Shot::at("burbler-near", "mudpit_burbler", 20.),
            Shot::at("burbler-mid", "mudpit_burbler", 50.),
            Shot::at("burbler-far", "mudpit_burbler", 110.),
            Shot::at("burbler-cull", "mudpit_burbler", 300.),
            Shot::at("shell-near", "snail_shell", 1.),
            Shot::at("shell-reach", "snail_shell", 3.),
            Shot::at("shell-far", "snail_shell", 12.),
            Shot::at("shell-cull", "snail_shell", 40.),
            Shot::at("brush-near", "desert_brush_0", 12.),
            Shot::at("brush-far", "desert_brush_0", 60.),
            Shot::at("stump-near", "desert_stump_0", 15.),
            Shot::at("stump-far", "desert_stump_0", 110.),
        ]
    }
}
#[derive(Resource)]
struct Review {
    dir: PathBuf,
    shots: Vec<Shot>,
    step: usize,
    changed: Instant,
    camera: Option<Entity>,
    source: Option<Entity>,
    // Retain the original scale per subject, so testing 1.5x never compounds.
    scales: std::collections::HashMap<Entity, Vec3>,
    captured: bool,
    complete: bool,
}
fn log(review: &Review, value: serde_json::Value) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(review.dir.join("events.jsonl"))
        .expect("LOD review evidence file");
    writeln!(file, "{value}").expect("write LOD review evidence");
}
impl Plugin for LodReviewPlugin {
    fn build(&self, app: &mut App) {
        let Some(dir) =
            std::env::args().find_map(|arg| arg.strip_prefix("--lod-review=").map(PathBuf::from))
        else {
            return;
        };
        let flora = !std::env::args().any(|arg| arg == "--lod-review-set=machines");
        std::fs::create_dir_all(&dir).expect("LOD review directory");
        std::fs::write(dir.join("events.jsonl"), "").expect("fresh LOD review evidence");
        app.insert_resource(Review {
            dir,
            shots: shots(flora),
            step: 0,
            changed: Instant::now(),
            camera: None,
            source: None,
            scales: default(),
            captured: false,
            complete: false,
        })
        .add_systems(Startup, fixtures)
        .add_systems(Last, drive);
    }
}
fn fixtures(mut commands: Commands) {
    for (id, at) in [
        ("desert_brush_0", Vec3::new(-20., 0., -20.)),
        ("desert_stump_0", Vec3::new(-30., 0., -20.)),
    ] {
        commands.spawn((
            Name::new(format!("LOD review {id}")),
            jackdaw_scene_types::GltfSource {
                path: format!("props/desert/{id}/{id}.glb"),
                scene_index: 0,
            },
            Transform::from_translation(at),
            Visibility::Inherited,
        ));
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn drive(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut review: ResMut<Review>,
    mut calendar: ResMut<Calendar>,
    mut cameras: Query<
        (
            Entity,
            &Camera,
            &mut Transform,
            &mut Projection,
            &bevy::camera::RenderTarget,
        ),
        With<Camera3d>,
    >,
    mut roots: Query<
        (
            Entity,
            &PlacedAsset,
            &GlobalTransform,
            &mut Transform,
            Option<&mut AssetState>,
            Option<&LiveLevels>,
            Option<&LodSwitches>,
        ),
        Without<Camera3d>,
    >,
    mut pickups: Query<&mut stubby_jackdaw::resources::ResourceNode>,
    parts: Query<(&Mesh3d, &ViewVisibility)>,
    meshes: Res<Assets<Mesh>>,
    children: Query<&Children>,
) {
    if review.complete || time.elapsed_secs() < 2. {
        return;
    }
    let Some(shot) = review.shots.get(review.step).copied() else {
        log(
            &review,
            serde_json::json!({"event":"complete", "shots":review.shots.len()}),
        );
        review.complete = true;
        return;
    };
    // Diagnostic shots hold noon for consistent geometry/state comparison. The normal
    // opening is still 06:30; no calendar or placement is saved by this review.
    calendar.elapsed = Duration::from_secs_f32(seconds_at_solar_hour(12.));
    if review.camera.is_none() {
        // The world camera is the first one created; rack cameras have positive order.
        review.camera = cameras
            .iter()
            .find(|(_, camera, ..)| camera.is_active && camera.order == 0)
            .map(|(e, ..)| e);
        if let Some(camera) = review.camera {
            commands
                .entity(camera)
                .remove::<stubby_jackdaw::controls::Player>();
        }
    }
    let Some(camera_id) = review.camera else {
        return;
    };
    let Ok((_, _, mut camera, mut projection, target)) = cameras.get_mut(camera_id) else {
        return;
    };
    if review.source.is_none() {
        let mut candidates: Vec<_> = roots
            .iter()
            .filter(|(_, asset, ..)| asset.asset == shot.asset)
            .map(|(entity, ..)| entity)
            .collect();
        candidates.sort();
        let Some(entity) = candidates.first().copied() else {
            return;
        };
        review.source = Some(entity);
        review.changed = Instant::now();
        review.captured = false;
        let (_, _, global, mut local, state, _, _) = roots.get_mut(entity).unwrap();
        let original = *review.scales.entry(entity).or_insert(local.scale);
        local.scale = original * shot.scale;
        if let Some(mut state) = state {
            state.growth = shot.growth;
            state.water_quality = shot.growth;
            state.light_quality = shot.growth;
        }
        if let Ok(mut pickup) = pickups.get_mut(entity) {
            pickup.collected = shot.harvested;
        }
        let height = match shot.asset {
            "scrub_oak_0" => 1.8 * shot.scale,
            "reed_clump_0" => 0.7,
            "mudpit_burbler" => 0.4,
            _ => 0.03,
        };
        let center = global.translation() + Vec3::Y * height;
        *camera = Transform::from_translation(
            center + Vec3::new(-0.1, 0.18, 1.).normalize() * shot.distance,
        )
        .looking_at(center, Vec3::Y);
        if let Projection::Perspective(p) = projection.as_mut() {
            p.fov = shot.fov.to_radians();
        }
        return;
    }
    let held = review.changed.elapsed().as_secs_f32();
    if held > 0.45 && !review.captured {
        let entity = review.source.unwrap();
        let (_, asset, global, _, state, live, switches) = roots.get(entity).unwrap();
        // Animated levels can be scene instances, not flat LodPart entities.
        // Count their mesh descendants too, without double-counting shared nodes.
        let entities: std::collections::HashSet<_> = live
            .into_iter()
            .flat_map(|live| live.nodes())
            .flat_map(|node| std::iter::once(node).chain(children.iter_descendants(node)))
            .collect();
        let visible_triangles: usize = entities
            .iter()
            .filter_map(|entity| parts.get(*entity).ok())
            .filter(|(_, visible)| visible.get())
            .filter_map(|(mesh, _)| meshes.get(&mesh.0))
            .map(|mesh| {
                mesh.indices()
                    .map_or(mesh.count_vertices(), |indices| indices.len())
                    / 3
            })
            .sum();
        log(
            &review,
            serde_json::json!({"event":"shot", "name":shot.name, "asset":asset.asset,
            "distance_m":camera.translation.distance(global.translation()), "fov":shot.fov,
            "placement_scale":global.compute_transform().scale.to_array(),
            "growth":state.map(|s| s.growth), "harvested":pickups.get(entity).ok().map(|s| s.collected),
            "wanted":live.map(LiveLevels::wanted), "ready":live.map(LiveLevels::ready),
            "submitted_triangles":visible_triangles,
            "ranges":switches.map(|s| s.ranges.iter().map(|r| [r.start_margin.start,r.start_margin.end,r.end_margin.start,r.end_margin.end]).collect::<Vec<_>>())}),
        );
        commands
            .spawn(Screenshot(target.clone()))
            .observe(save_to_disk(review.dir.join(format!("{}.png", shot.name))));
        review.captured = true;
    }
    if held > 0.75 && review.captured {
        review.step += 1;
        review.source = None;
    }
}
