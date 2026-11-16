//! `--sky-review=<dir>`: jump the calendar through dawn, noon, dusk and night, looking at
//! the sun, moon and stars, and save a screenshot of each plus `events.jsonl` to `<dir>`
//! (`sync-calendar-and-night-sky` 4.1). Run it under the graphical watchdog
//! (`tools/jackdaw_timed_test.py --sky`).
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::path::PathBuf;
use stubby_world::calendar::{Calendar, moon_direction, solar_hour, sun_direction};

#[derive(Resource)]
struct SkyReview {
    dir: PathBuf,
    step: usize,
}

/// (calendar seconds, where to look as a direction, screenshot name)
const SHOTS: [(u64, Vec3, &str); 6] = [
    (120, Vec3::new(1., 0.12, 0.), "dawn-east.png"),
    (120, Vec3::new(-1., 0.25, 0.), "dawn-west-moon.png"),
    (1350, Vec3::new(0., 0.45, 1.), "noon-south.png"),
    (2620, Vec3::new(-1., 0.1, 0.), "dusk-west.png"),
    (3150, Vec3::new(0., 0.9, 0.35), "midnight-moon.png"),
    (3420, Vec3::new(0.3, 0.6, -1.), "stars-north.png"),
];
/// Real seconds: the first shot is set up at `START`, then a step every `STEP` (set the
/// sky, then shoot it once the environment map has been filtered again).
const START: f32 = 3.;
const STEP: f32 = 0.7;

fn install(app: &mut App) {
    if let Some(dir) =
        std::env::args().find_map(|arg| arg.strip_prefix("--sky-review=").map(PathBuf::from))
    {
        let _ = std::fs::create_dir_all(&dir);
        app.insert_resource(SkyReview { dir, step: 0 });
    }
}

fn log(review: &SkyReview, event: serde_json::Value) {
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(review.dir.join("events.jsonl"))
    {
        let _ = writeln!(file, "{event}");
    }
}

fn drive(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut review: ResMut<SkyReview>,
    mut calendar: ResMut<Calendar>,
    mut cameras: Query<
        (&mut Transform, &bevy::camera::RenderTarget),
        (With<Camera3d>, Without<stubby_core::HudCamera>),
    >,
    ground: Res<stubby_core::GroundHeight>,
    master: Option<Res<bevy_stubby_audio::MasterClock>>,
    lights: Query<(
        Option<&Name>,
        &DirectionalLight,
        &Visibility,
        &ViewVisibility,
        &GlobalTransform,
    )>,
) {
    if time.elapsed_secs() < START + review.step as f32 * STEP {
        return;
    }
    let Some(&(seconds, look, name)) = SHOTS.get(review.step / 2) else {
        if review.step / 2 == SHOTS.len() {
            log(
                &review,
                serde_json::json!({"event": "complete", "success": true}),
            );
            review.step += 1;
        }
        return;
    };
    if review.step % 2 == 0 {
        calendar.elapsed = std::time::Duration::from_secs(seconds);
        let at = Vec3::new(0., (ground.0)(Vec2::ZERO) + 1.7, 0.);
        for (mut camera, _) in &mut cameras {
            *camera = Transform::from_translation(at).looking_at(at + look, Vec3::Y);
        }
    } else {
        // The camera's own target: under embedded Play there is no primary window.
        let Ok((_, target)) = cameras.single() else {
            return;
        };
        commands
            .spawn(Screenshot(target.clone()))
            .observe(save_to_disk(review.dir.join(name)));
        let elapsed = seconds as f32;
        log(
            &review,
            serde_json::json!({"event": "shot", "name": name, "calendar_s": seconds,
                "solar_hour": solar_hour(elapsed),
                "sun": sun_direction(elapsed).to_array(),
                "moon": moon_direction(elapsed).to_array(),
                "fps": 1. / time.delta_secs().max(1e-6),
                "master_beats": master.as_ref().and_then(|m| m.beats()),
                "lights": lights.iter().map(|(name, light, visibility, view, at)| serde_json::json!({
                    "name": name.map(Name::as_str), "lux": light.illuminance,
                    "visibility": format!("{visibility:?}"), "drawn": view.get(),
                    "toward": (-at.forward().as_vec3()).to_array()})).collect::<Vec<_>>()}),
        );
    }
    review.step += 1;
}

/// The sky review, when `--sky-review=<dir>` asks for it.
pub struct SkyReviewPlugin;
impl Plugin for SkyReviewPlugin {
    fn build(&self, app: &mut App) {
        install(app);
        if app.world().contains_resource::<SkyReview>() {
            // Per-pass GPU time in the log: the atmosphere's cost against the old sky.
            app.add_plugins((
                bevy::render::diagnostic::RenderDiagnosticsPlugin,
                bevy::diagnostic::LogDiagnosticsPlugin::default(),
            ));
        }
        app.add_systems(Update, drive.run_if(resource_exists::<SkyReview>));
    }
}
