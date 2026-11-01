//! Opt-in, bounded-session review driver. Normal gameplay never installs these systems.
use super::{
    calendar::{Calendar, DAY_SECONDS, set_speed},
    resources::{PlayerInventory, ResourceNode},
};
use bevy::prelude::*;
use std::{io::Write, path::PathBuf, time::Instant};

pub struct ReplenishmentReviewPlugin;
#[derive(Resource)]
struct Review {
    directory: PathBuf,
    started: Instant,
    stage: usize,
    next: f32,
    sample_at: f32,
    captures: [bool; 3],
}
impl Plugin for ReplenishmentReviewPlugin {
    fn build(&self, app: &mut App) {
        let Some(directory) = std::env::args().find_map(|arg| {
            arg.strip_prefix("--replenishment-review=")
                .map(PathBuf::from)
        }) else {
            return;
        };
        app.insert_resource(Review {
            directory,
            started: Instant::now(),
            stage: 0,
            next: 1.,
            sample_at: 0.,
            captures: [false; 3],
        })
        .add_systems(Startup, start.after(super::setup))
        .add_systems(
            PreUpdate,
            drive
                .after(bevy::input::InputSystems)
                .before(super::calendar::CalendarAdvance),
        )
        .add_systems(PostUpdate, observe.after(super::pulse::BugEffects));
    }
}
fn record(review: &Review, value: serde_json::Value) {
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(review.directory.join("events.jsonl"))?;
        writeln!(file, "{value}")
    })();
    if let Err(error) = result {
        error!("replenishment review evidence: {error}");
    }
}
fn start(
    mut review: ResMut<Review>,
    mut calendar: ResMut<Calendar>,
    mut time: ResMut<Time<Virtual>>,
    mut cameras: Query<
        &mut Transform,
        (
            With<Camera3d>,
            Without<super::machine_config::RackHudCamera>,
        ),
    >,
) {
    if let Err(error) = std::fs::create_dir_all(&review.directory) {
        error!("review directory: {error}");
    }
    if let Err(error) = std::fs::File::create(review.directory.join("events.jsonl")) {
        error!("review log: {error}");
    }
    review.started = Instant::now();
    calendar.elapsed = std::time::Duration::from_secs(DAY_SECONDS - 300);
    set_speed(&mut time, 64.);
    for mut camera in &mut cameras {
        camera.translation.x = 1.;
        camera.translation.z = 3.;
        *camera = camera.looking_to(Vec3::new(-1., 0.15, -0.2), Vec3::Y);
    }
    record(
        &review,
        serde_json::json!({"event":"started", "speed":64, "day_seconds":DAY_SECONDS, "seconds_before_night":300}),
    );
}
fn drive(
    mut review: ResMut<Review>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    calendar: Res<Calendar>,
    player: Res<PlayerInventory>,
    nodes: Query<&ResourceNode>,
) {
    // Release only the review's own injected keys, once per frame.
    for key in [
        KeyCode::KeyE,
        KeyCode::BracketLeft,
        KeyCode::BracketRight,
        KeyCode::Backslash,
    ] {
        keys.reset(key);
    }
    let elapsed = review.started.elapsed().as_secs_f32();
    if elapsed < review.next {
        return;
    }
    let Some(bug) = nodes.iter().find(|node| node.instance == "lightning_bug-0") else {
        return;
    };
    let quantity = player.items.quantity("square_wave");
    let action = match review.stage {
        0 => Some(KeyCode::KeyE),
        1 if bug.collected && quantity == 2 => {
            record(
                &review,
                serde_json::json!({"event":"depleted", "quantity":quantity}),
            );
            review.stage += 1;
            None
        }
        2 if calendar.is_night() && !bug.collected => {
            record(
                &review,
                serde_json::json!({"event":"returned", "quantity":quantity, "no_free_items":quantity == 2}),
            );
            review.stage += 1;
            review.next = elapsed + 3.;
            None
        }
        3 => Some(KeyCode::KeyE),
        4 if bug.collected && quantity == 4 => {
            record(
                &review,
                serde_json::json!({"event":"recollected", "quantity":quantity}),
            );
            review.stage += 1;
            review.next = elapsed + 0.4;
            None
        }
        5 => Some(KeyCode::BracketLeft),
        6 => Some(KeyCode::BracketRight),
        7 => Some(KeyCode::Backslash),
        8 => Some(KeyCode::BracketLeft),
        9 => Some(KeyCode::Backslash),
        _ => None,
    };
    if let Some(key) = action {
        keys.press(key);
        record(
            &review,
            serde_json::json!({"event":"input", "key":format!("{key:?}"), "stage":review.stage, "real_seconds":elapsed}),
        );
        review.stage += 1;
        review.next = elapsed + 0.4;
    }
}
fn observe(
    mut commands: Commands,
    mut review: ResMut<Review>,
    calendar: Res<Calendar>,
    time: Res<Time<Virtual>>,
    player: Res<PlayerInventory>,
    nodes: Query<(&ResourceNode, &Visibility)>,
    cameras: Query<
        &bevy::camera::RenderTarget,
        (
            With<Camera3d>,
            Without<super::machine_config::RackHudCamera>,
        ),
    >,
) {
    let elapsed = review.started.elapsed().as_secs_f32();
    if elapsed >= review.sample_at {
        let nodes: Vec<_> = nodes.iter().map(|(node, visibility)| serde_json::json!({"instance":node.instance,"collected":node.collected,"visible":*visibility == Visibility::Visible})).collect();
        record(
            &review,
            serde_json::json!({"event":"sample", "real_seconds":elapsed,"night":calendar.is_night(),"speed":time.relative_speed(),"quantity":player.items.quantity("square_wave"),"nodes":nodes}),
        );
        review.sample_at = elapsed + 0.2;
    }
    let capture = if elapsed > 2. && !review.captures[0] {
        Some((0, "sun-before-dusk.png"))
    } else if review.stage == 2 && !review.captures[1] {
        Some((1, "depleted.png"))
    } else if review.stage == 3
        && !review.captures[2]
        && nodes.iter().any(|(node, visibility)| {
            node.instance == "lightning_bug-0" && *visibility == Visibility::Visible
        })
    {
        Some((2, "returned-at-night.png"))
    } else {
        None
    };
    if let Some((index, name)) = capture
        && let Ok(target) = cameras.single()
    {
        review.captures[index] = true;
        commands
            .spawn(bevy::render::view::screenshot::Screenshot(target.clone()))
            .observe(bevy::render::view::screenshot::save_to_disk(
                review.directory.join(name),
            ));
    }
    if review.stage == 10 {
        record(
            &review,
            serde_json::json!({"event":"complete", "success":player.items.quantity("square_wave") == 4 && time.relative_speed() == 1., "real_seconds":elapsed}),
        );
        review.stage = 11;
    }
}
