//! A goal done is announced (user, 2026-11-18: "we need an indication that a task has been
//! accomplished, like lighting the campfire"), as Satisfactory announces a milestone: a
//! banner drops in at the top of the screen, STEP COMPLETE (or GOAL COMPLETE for a major
//! goal) over the goal's name, with a short rising chime, holds a few seconds and fades.
//! Several at once queue, one after another. The goals hub (upper right) moves on by
//! itself.
use bevy::prelude::*;
use bevy_stubby_audio::Synth;
use bevy_stubby_recipes::{GoalReported, Goals};
use std::collections::VecDeque;
use stubby_recipes::{GoalOutcome, Scale};
use stubby_ui::style::{self, AMBER, INK, PANEL};

pub struct GoalBannerPlugin;
impl Plugin for GoalBannerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Banners>()
            .add_systems(Startup, spawn_banner)
            .add_systems(Update, (load_chime, queue_banners, show_banner).chain());
    }
}

/// The chime (`assets/audio/sfx/goal_chime/`, made by `tools/audio/make_goal_chime.py`).
const CHIME: &[u8] = include_bytes!("../../../assets/audio/sfx/goal_chime/goal_chime.wav");
/// Its sample slot in the engine (the rack's snaps use 1 to 3).
const CHIME_SAMPLE: stubby_audio::SampleId = stubby_audio::SampleId(64);
const CHIME_GAIN: f32 = 0.5;

/// How long a banner shows, and how long it takes to come and go.
const SHOW_S: f32 = 4.;
const IN_S: f32 = 0.25;
const OUT_S: f32 = 0.8;

#[derive(Resource, Default)]
struct Banners {
    waiting: VecDeque<(String, String)>,
    /// The banner up now and when it went up.
    shown: Option<f32>,
}

#[derive(Component)]
struct Banner;
#[derive(Component)]
struct BannerKind;
#[derive(Component)]
struct BannerName;

fn spawn_banner(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Goal banner"),
            Node {
                position_type: PositionType::Absolute,
                top: px(118),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(40),
            Pickable::IGNORE,
            Visibility::Hidden,
            Banner,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(4),
                    padding: UiRect::axes(px(28), px(10)),
                    border: UiRect::axes(px(0), px(2)),
                    min_width: px(320),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(AMBER),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((BannerKind, style::label("", 14., AMBER)));
                panel.spawn((BannerName, style::label("", 24., INK)));
            });
        });
}

/// Load the chime into the engine once its thread is up.
fn load_chime(synth: Option<Res<Synth>>, mut done: Local<bool>) {
    let Some(thread) = synth.as_ref().and_then(|s| s.thread()) else {
        return;
    };
    if *done {
        return;
    }
    *done = true;
    let Ok(mut reader) = hound::WavReader::new(std::io::Cursor::new(CHIME)) else {
        return;
    };
    let spec = reader.spec();
    let scale = 1. / (1u32 << (spec.bits_per_sample.max(1) - 1)) as f32;
    let samples: Vec<f32> = reader
        .samples::<i32>()
        .filter_map(Result::ok)
        .map(|s| s as f32 * scale)
        .collect();
    thread.load_sample(CHIME_SAMPLE, spec.sample_rate, vec![samples]);
}

/// Each goal completed waits its turn: its kind and its name.
fn queue_banners(
    mut reports: MessageReader<GoalReported>,
    goals: Option<Res<Goals>>,
    mut banners: ResMut<Banners>,
) {
    let Some(goals) = goals else {
        reports.clear();
        return;
    };
    for GoalReported(outcome) in reports.read() {
        let GoalOutcome::GoalCompleted(id) = outcome else {
            continue;
        };
        let Some(goal) = goals.rules.goals.iter().find(|g| &g.id == id) else {
            continue;
        };
        let kind = match goal.scale {
            Scale::Major => "GOAL COMPLETE",
            Scale::Minor => "STEP COMPLETE",
        };
        banners.waiting.push_back((kind.into(), goal.name.clone()));
    }
}

#[allow(clippy::type_complexity)]
fn show_banner(
    time: Res<Time<Real>>,
    synth: Option<Res<Synth>>,
    mut banners: ResMut<Banners>,
    mut roots: Query<&mut Visibility, With<Banner>>,
    mut panels: Query<(&mut BackgroundColor, &mut BorderColor)>,
    children: Query<&Children>,
    mut kinds: Query<(&mut Text, &mut TextColor), (With<BannerKind>, Without<BannerName>)>,
    mut names: Query<(&mut Text, &mut TextColor), (With<BannerName>, Without<BannerKind>)>,
    root: Query<Entity, With<Banner>>,
) {
    let now = time.elapsed_secs();
    if banners.shown.is_some_and(|since| now - since > SHOW_S) {
        banners.shown = None;
    }
    if banners.shown.is_none()
        && let Some((kind, name)) = banners.waiting.pop_front()
    {
        banners.shown = Some(now);
        for (mut text, _) in &mut kinds {
            text.0 = kind.clone();
        }
        for (mut text, _) in &mut names {
            text.0 = name.clone();
        }
        if let Some(synth) = &synth {
            synth.send(stubby_audio::Command::PlaySample {
                sample: CHIME_SAMPLE,
                gain: CHIME_GAIN,
            });
        }
    }
    let alpha = banners.shown.map_or(0., |since| {
        let age = now - since;
        (age / IN_S).min((SHOW_S - age) / OUT_S).clamp(0., 1.)
    });
    for mut visibility in &mut roots {
        visibility.set_if_neq(if alpha > 0. {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (_, mut color) in &mut kinds {
        color.0 = AMBER.with_alpha(alpha);
    }
    for (_, mut color) in &mut names {
        color.0 = INK.with_alpha(alpha);
    }
    for entity in root.iter().flat_map(|r| children.iter_descendants(r)) {
        if let Ok((mut background, mut border)) = panels.get_mut(entity) {
            background.0 = PANEL.with_alpha(PANEL.alpha() * alpha);
            *border = BorderColor::all(AMBER.with_alpha(alpha));
        }
    }
}
