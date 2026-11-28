//! The seeded opening in the running game (add-hybrid-world-placement 2.2, 2.4, 2b).
//!
//! When the authored scene has spawned (from its file, or served by the editor under
//! Play), its objects are read by model and position, the world configuration's rules
//! are resolved around them (`WorldConfig::resolve_opening`), and each placement is
//! spawned as a model source, as a placed prefab is. The prefab bridge then makes it a
//! `PlacedAsset`, and the game's binders bring it alive exactly as an authored one: a
//! burbler gets its seeded tempo and clock, an oak becomes a mine, reeds join the harvest
//! ledger. Its name carries the rule and source cell, so its instance ID is stable for
//! the same seed and rules. Respawning the scene replaces the layout; nothing moves an
//! authored object. `--no-seeded-layout` leaves the scene as authored (reviews).
use bevy::prelude::*;
use jackdaw_runtime::{GltfSource, HiddenInGame, JackdawSceneSpawned};
use stubby_jackdaw::world_generation::{PlacementKey, WorldConfig, model_stem};

pub struct SeededLayoutPlugin;
impl Plugin for SeededLayoutPlugin {
    fn build(&self, app: &mut App) {
        if !std::env::args().any(|arg| arg == "--no-seeded-layout")
            && !app
                .world()
                .contains_resource::<stubby_jackdaw::saves::SkipSeededLayout>()
        {
            app.add_observer(spawn_seeded_layout);
        }
        app.add_systems(Update, (place_stream_features, clear_stream_features));
        if let Some(path) =
            std::env::args().find_map(|arg| arg.strip_prefix("--layout-review=").map(str::to_owned))
        {
            app.insert_resource(LayoutReview(path.into()))
                .add_systems(Update, review_layout);
        }
    }
}

/// A placement the world's rules made, not the level designer. The key is what a designer
/// edit (suppress, move) names; kept for that tooling.
#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct SeededPlacement(pub PlacementKey);

#[allow(clippy::type_complexity)] // Authored sources and the hierarchy above them.
fn spawn_seeded_layout(
    _spawned: On<JackdawSceneSpawned>,
    mut commands: Commands,
    config: Res<WorldConfig>,
    sources: Query<(Entity, &GltfSource), (Without<SeededPlacement>, Without<HiddenInGame>)>,
    hierarchy: Query<(&Transform, Option<&ChildOf>)>,
    previous: Query<Entity, With<SeededPlacement>>,
) {
    for entity in &previous {
        commands.entity(entity).despawn();
    }
    // The rules describe the opening scene; review scenes (`--scene=`) stay as authored.
    if config.scene != WorldConfig::shipped().scene {
        return;
    }
    // The event fires before transforms propagate: compose each source's world position.
    let authored: Vec<(String, Vec3)> = sources
        .iter()
        .filter(|(_, source)| !source.path.starts_with("generated/"))
        .map(|(entity, source)| {
            let mut at = Transform::IDENTITY;
            let mut current = Some(entity);
            while let Some(node) = current {
                let Ok((transform, parent)) = hierarchy.get(node) else {
                    break;
                };
                at = transform.mul_transform(at);
                current = parent.map(ChildOf::parent);
            }
            (model_stem(&source.path).to_owned(), at.translation)
        })
        .collect();
    let layout = match config.resolve_opening(&authored) {
        Ok(layout) => layout,
        Err(error) => {
            error!("seeded opening not generated: {error}");
            return;
        }
    };
    for diagnostic in &layout.diagnostics {
        warn!("seeded opening: {diagnostic}");
    }
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    for record in &layout.placements {
        *counts.entry(record.key.rule.as_str()).or_default() += 1;
        commands.spawn((
            Name::new(format!(
                "{} · seeded {} {}",
                model_stem(&record.asset),
                record.key.rule,
                record.key.cell
            )),
            record.pose.transform(),
            Visibility::default(),
            GltfSource {
                path: record.asset.clone(),
                scene_index: 0,
            },
            SeededPlacement(record.key.clone()),
        ));
    }
    info!(
        "seeded opening (seed {}, {} authored objects kept): {counts:?}",
        config.seed,
        authored.len()
    );
}

/// `--layout-review=<dir>`: once the layout has settled, two noon views without haze:
/// straight down from 240 m over spawn (burblers, groves, rocks; reeds and props cull at
/// 90 to 120 m), and a low oblique view over the nearest seeded burbler's reed beds.
#[derive(Resource)]
struct LayoutReview(std::path::PathBuf);

#[allow(clippy::type_complexity)] // The world camera, not the rack's.
fn review_layout(
    mut commands: Commands,
    time: Res<Time<Real>>,
    review: Res<LayoutReview>,
    mut calendar: ResMut<stubby_jackdaw::calendar::Calendar>,
    mut cameras: Query<(Entity, &Camera, &mut Transform, &mut Projection), With<Camera3d>>,
    seeded: Query<(&SeededPlacement, &GlobalTransform)>,
    mut step: Local<u8>,
) {
    let t = time.elapsed_secs();
    let shots = [(5., 9.), (11., 15.)];
    let Some(&(pose_at, shoot_at)) = shots.get(usize::from(*step / 2)) else {
        return;
    };
    if *step % 2 == 0 && t > pose_at {
        let view = if *step == 0 {
            Transform::from_xyz(10., 240., -10.).looking_at(Vec3::new(10., 0., -10.), Vec3::NEG_Z)
        } else {
            let burbler = seeded
                .iter()
                .filter(|(s, _)| s.0.rule == "burblers")
                .map(|(_, g)| g.translation())
                .min_by(|a, b| a.length().total_cmp(&b.length()))
                .unwrap_or(Vec3::new(39., 0., -6.));
            Transform::from_translation(burbler + Vec3::new(0., 22., 34.))
                .looking_at(burbler, Vec3::Y)
        };
        calendar.elapsed = std::time::Duration::from_secs_f32(
            stubby_jackdaw::calendar::seconds_at_solar_hour(12.),
        );
        for (entity, camera, mut transform, mut projection) in &mut cameras {
            if camera.is_active && camera.order == 0 {
                commands
                    .entity(entity)
                    .remove::<(stubby_jackdaw::controls::Player, DistanceFog)>();
                *transform = view;
                if let Projection::Perspective(p) = projection.as_mut() {
                    p.fov = 70f32.to_radians();
                }
            }
        }
        *step += 1;
    } else if *step % 2 == 1 && t > shoot_at {
        let name = if *step == 1 {
            "overhead.png"
        } else {
            "reed_beds.png"
        };
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(
                review.0.join(name),
            ));
        *step += 1;
    }
}

/// Stream features (ripple pools) along each newly laid stream, at the places the world's
/// rules give each of its segments (`stream_feature_places`), turned to the flow there.
/// They come alive like any placed model; collecting from them is the generic socket rule
/// (`stubby_game::collectibles`), and they keep their stream's time (`OnStream`). A
/// dismantled stream takes its features with it, and the pieces left get theirs again, the
/// same ones, since each segment's places come from its own path and index.
fn place_stream_features(
    mut commands: Commands,
    config: Res<WorldConfig>,
    streams: Query<
        (Entity, &stubby_jackdaw::streams::LaidStream),
        Added<stubby_jackdaw::streams::LaidStream>,
    >,
) {
    if config.stream_features.is_empty() {
        return;
    }
    let Ok(seed) = config.seed_value() else {
        return;
    };
    for (entity, stream) in &streams {
        let mut spawned = 0;
        for k in stream.first..=stream.last {
            let segment = stream.segment_key(k);
            for feature in &config.stream_features {
                let places = stubby_jackdaw::world_generation::stream_feature_places(
                    seed,
                    feature,
                    segment,
                    stream.span,
                );
                for (i, (offset, far_bank)) in places.into_iter().enumerate() {
                    let (at, flow) = stream.path.at(k as f32 * stream.span + offset);
                    // The model's +X runs along the flow; half a turn puts it on the far bank.
                    let yaw =
                        (-flow.z).atan2(flow.x) + if far_bank { std::f32::consts::PI } else { 0. };
                    commands.spawn((
                        Name::new(format!(
                            "{} · {} {segment:016x} {i}",
                            model_stem(&feature.asset),
                            feature.id
                        )),
                        Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw)),
                        Visibility::default(),
                        GltfSource {
                            path: feature.asset.clone(),
                            scene_index: 0,
                        },
                        stubby_jackdaw::stream_features::OnStream(entity),
                    ));
                    spawned += 1;
                }
            }
        }
        debug!(
            "stream features: {spawned} along segments {}..={}",
            stream.first, stream.last
        );
    }
}

/// Features whose stream is gone (dismantled, or replaced by its pieces) go with it.
fn clear_stream_features(
    mut commands: Commands,
    features: Query<(Entity, &stubby_jackdaw::stream_features::OnStream)>,
    streams: Query<(), With<stubby_jackdaw::streams::LaidStream>>,
) {
    for (entity, on) in &features {
        if !streams.contains(on.0) {
            commands.entity(entity).despawn();
        }
    }
}
