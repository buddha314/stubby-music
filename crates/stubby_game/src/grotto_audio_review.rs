//! Bounded listening pass for the authored echo grotto.
//!
//! `--grotto-audio-review=<directory>` auto-patches the grotto burbler's real rack,
//! waits for its voice to sound, then moves the listener inside, behind the back
//! boulder, and outside the mouth. This is a review driver; normal gameplay does not
//! install it.

use bevy::prelude::*;
use std::{io::Write, path::PathBuf, time::Instant};
use stubby_audio::{Command, ListenerPose, PortRef};
const BURBLER: Vec3 = Vec3::new(69.43934, 0.65, -59.1967);
const VIEWS: [(&str, Vec3); 3] = [
    ("inside", Vec3::new(70.5, 1.6, -64.5)),
    // Place one listener on a ray through boulder 0, then compare at the same
    // source distance through the burbler's spawn-facing mouth.
    ("blocked-outside", Vec3::new(76.89, 1.6, -74.48)),
    ("clear-outside", Vec3::new(69.44, 1.6, -42.2)),
];
const VIEW_SECONDS: f32 = 5.0;

pub struct GrottoAudioReviewPlugin;

#[derive(Resource)]
struct Review {
    directory: PathBuf,
    started: Instant,
    view: usize,
    burbler: Option<Entity>,
    next_sample: f32,
    capture_pending: bool,
    complete_recorded: bool,
}

impl Plugin for GrottoAudioReviewPlugin {
    fn build(&self, app: &mut App) {
        let Some(directory) = std::env::args().find_map(|arg| {
            arg.strip_prefix("--grotto-audio-review=")
                .map(PathBuf::from)
        }) else {
            return;
        };
        app.insert_resource(Review {
            directory,
            started: Instant::now(),
            view: usize::MAX,
            burbler: None,
            next_sample: 0.,
            capture_pending: false,
            complete_recorded: false,
        })
        .add_systems(Startup, start.after(super::spawn_camera))
        .add_systems(Last, arm_burbler)
        .add_systems(Update, advance)
        .add_systems(PostUpdate, capture);
    }
}

fn record(review: &Review, event: serde_json::Value) {
    if let Err(error) = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(review.directory.join("events.jsonl"))?;
        writeln!(file, "{event}")
    })() {
        error!("grotto audio review log: {error}");
    }
}

fn set_view(
    review: &mut Review,
    index: usize,
    cameras: &mut Query<&mut Transform, (With<Camera3d>, Without<stubby_core::HudCamera>)>,
    synth: &super::audio::Synth,
    output: PortRef,
    source: Vec3,
) {
    let Some((name, eye)) = VIEWS.get(index).copied() else {
        return;
    };
    if let Ok(mut camera) = cameras.single_mut() {
        *camera = Transform::from_translation(eye).looking_at(BURBLER, Vec3::Y);
        synth.send(Command::SetListener {
            pose: ListenerPose {
                position: eye.to_array(),
                right: camera.right().to_array(),
                up: camera.up().to_array(),
                ahead: camera.forward().to_array(),
            },
        });
        let offset = camera.rotation.inverse() * (source - eye).normalize();
        synth.send(Command::SetEmitter {
            output,
            offset: offset.to_array(),
            position_world: source.to_array(),
            volume: 0.65,
        });
        review.view = index;
        review.capture_pending = true;
        record(
            review,
            serde_json::json!({
                "event":"view",
                "name":name,
                "listener":eye.to_array(),
                "source":source.to_array(),
                "output_module":output.module.0,
                "elapsed_seconds":review.started.elapsed().as_secs_f32()
            }),
        );
    }
}

fn arm_burbler(
    mut review: ResMut<Review>,
    profiles: Res<super::machine_config::Profiles>,
    machines: Query<(Entity, &stubby_core::PlacedAsset, &GlobalTransform)>,
    mut racks: Query<&mut super::rack::RackState>,
) {
    if review.burbler.is_some() {
        return;
    }
    let Some(config) = profiles.0.get("mudpit_burbler") else {
        return;
    };
    let machine = machines
        .iter()
        .filter(|(_, asset, _)| asset.asset == "mudpit_burbler")
        .min_by(|(_, _, a), (_, _, b)| {
            a.translation()
                .distance_squared(BURBLER)
                .total_cmp(&b.translation().distance_squared(BURBLER))
        })
        .map(|(entity, _, _)| entity);
    let Some(machine) = machine else {
        return;
    };
    let Ok(mut rack) = racks.get_mut(machine) else {
        return;
    };
    let mounted = rack.mounted.insert("vco".into(), "square_vco".into());
    let patched: Vec<_> = config
        .cables
        .iter()
        .map(|cable| (!cable.optional).then(|| (cable.from.clone(), cable.to.clone())))
        .collect();
    let changed = mounted.as_deref() != Some("square_vco") || rack.patched != patched;
    if changed {
        rack.patched = patched;
        rack.version = rack.version.wrapping_add(1);
    }
    review.burbler = Some(machine);
    record(
        &review,
        serde_json::json!({
            "event":"burbler_autopatched",
            "machine":machine.to_bits(),
            "changed":changed,
            "complete":rack.complete(config)
        }),
    );
}

fn start(
    mut review: ResMut<Review>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<stubby_core::HudCamera>)>,
) {
    if let Err(error) = std::fs::create_dir_all(&review.directory) {
        error!("grotto audio review directory: {error}");
    }
    if let Err(error) = std::fs::File::create(review.directory.join("events.jsonl")) {
        error!("grotto audio review log: {error}");
    }
    review.started = Instant::now();
    if let Ok(mut camera) = cameras.single_mut() {
        camera.translation = VIEWS[0].1;
        camera.look_at(BURBLER, Vec3::Y);
    }
}

fn advance(
    mut review: ResMut<Review>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<stubby_core::HudCamera>)>,
    synth: Res<super::audio::Synth>,
    level: Res<super::audio::WorldLevel>,
    voices: Query<(&stubby_core::MachineVoice, &GlobalTransform)>,
) {
    let Some(machine) = review.burbler else {
        return;
    };
    let Ok((voice, transform)) = voices.get(machine) else {
        return;
    };
    if !voice.sounding {
        return;
    }
    let Some(output) = voice.output else {
        return;
    };
    let source = transform.translation() + Vec3::Y * 0.65;
    if review.view == usize::MAX {
        review.started = Instant::now();
        set_view(&mut review, 0, &mut cameras, &synth, output, source);
        return;
    }
    let elapsed = review.started.elapsed().as_secs_f32();
    let target_view = (elapsed / VIEW_SECONDS) as usize;
    if target_view < VIEWS.len() && target_view != review.view {
        set_view(
            &mut review,
            target_view,
            &mut cameras,
            &synth,
            output,
            source,
        );
    }
    if elapsed >= review.next_sample {
        record(
            &review,
            serde_json::json!({
                "event":"level",
                "view":VIEWS.get(review.view).map(|view| view.0),
                "peak_dbfs":level.peak_db,
                "silent":!level.peak_db.is_finite(),
                "elapsed_seconds":elapsed
            }),
        );
        review.next_sample = elapsed + 0.05;
    }
    if elapsed >= VIEW_SECONDS * VIEWS.len() as f32 && !review.complete_recorded {
        record(
            &review,
            serde_json::json!({"event":"complete", "elapsed_seconds":elapsed}),
        );
        review.next_sample = f32::INFINITY;
        review.complete_recorded = true;
    }
}

fn capture(
    mut commands: Commands,
    mut review: ResMut<Review>,
    cameras: Query<&bevy::camera::RenderTarget, (With<Camera3d>, Without<stubby_core::HudCamera>)>,
) {
    if !review.capture_pending || review.view >= VIEWS.len() {
        return;
    }
    if review
        .started
        .elapsed()
        .as_secs_f32()
        .rem_euclid(VIEW_SECONDS)
        < 0.75
    {
        return;
    }
    let Ok(target) = cameras.single() else {
        return;
    };
    let (name, _) = VIEWS[review.view];
    review.capture_pending = false;
    commands
        .spawn(bevy::render::view::screenshot::Screenshot(target.clone()))
        .observe(bevy::render::view::screenshot::save_to_disk(
            review.directory.join(format!("{name}.png")),
        ));
}
