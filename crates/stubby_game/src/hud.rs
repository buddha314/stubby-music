//! The in-world HUD around the reticle and the screen's edges, after Satisfactory's
//! (`HUD 2.png`), in the style guide's look (`stubby_ui::style`):
//!
//! - **The clock**: a quiet readout under the compass, the time and Day or Night, and the
//!   simulation speed in amber only when it is not 1x.
//! - **Key hints**, bottom right: a keycap over each tool's name, as Satisfactory's cluster.
//! - **Notices**: one line under the interaction prompt that answers what the player just
//!   did (no recorder in hand, not enough reeds), fading after a few seconds.
//!
//! All of it hides while a screen has the controls.
use bevy::prelude::*;
use stubby_core::{ScreenMode, keys};
use stubby_ui::style::{self, AMBER, BAND, INK, MUTED};
use stubby_world::calendar::{Calendar, solar_hour};

use super::recipes_plugin::PlayerInventory;

/// How long a notice stays, and how long it takes to fade, in seconds.
const NOTICE_S: f32 = 3.5;
const FADE_S: f32 = 0.8;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (show_clock, show_notice, hide_under_screens));
    }
}

/// HUD parts that step aside while a screen is open.
#[derive(Component)]
struct WorldHud;
#[derive(Component)]
struct ClockTime;
#[derive(Component)]
struct ClockPart;
#[derive(Component)]
struct ClockSpeed;
#[derive(Component)]
struct Notice;

/// The tools the key hints list, left to right.
fn hints() -> [(String, &'static str); 6] {
    [
        (style::key_name(keys::BUILD), "Build"),
        (style::key_name(keys::RECORDER), "Recorder"),
        (style::key_name(keys::FLASHLIGHT), "Scanner"),
        (style::key_name(keys::INVENTORY), "Inventory"),
        (style::key_name(keys::RECIPE_BOOK), "Recipes"),
        ("[ ]".into(), "Time"),
    ]
}

fn spawn(mut commands: Commands) {
    // The clock, centred under the compass.
    commands
        .spawn((
            WorldHud,
            Name::new("Clock"),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                top: px(50),
                width: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Baseline,
                column_gap: px(6),
                ..default()
            },
        ))
        .with_children(|clock| {
            clock.spawn((ClockTime, style::world_label("06:30", 15., INK)));
            clock.spawn((ClockPart, style::world_label("DAY", 11., MUTED)));
            clock.spawn((ClockSpeed, style::world_label("", 12., AMBER)));
        });
    // The key hints, bottom right.
    commands
        .spawn((
            WorldHud,
            Name::new("Key hints"),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                right: px(20),
                bottom: px(20),
                column_gap: px(6),
                ..default()
            },
        ))
        .with_children(|row| {
            for (key, name) in hints() {
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(5),
                        min_width: px(64),
                        padding: UiRect::axes(px(6), px(6)),
                        border: UiRect::bottom(px(2)),
                        ..default()
                    },
                    BackgroundColor(BAND),
                    BorderColor::all(style::EDGE),
                    Pickable::IGNORE,
                ))
                .with_children(|tile| {
                    style::keycap(tile, &key, 13.);
                    tile.spawn(style::label(name, 12., INK));
                });
            }
        });
    // The notice line, under the interaction prompt.
    commands
        .spawn((
            WorldHud,
            Name::new("Notice"),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                top: percent(61),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_child((Notice, style::world_label("", 16., INK)));
}

fn show_clock(
    calendar: Option<Res<Calendar>>,
    time: Res<Time<Virtual>>,
    mut times: Query<&mut Text, (With<ClockTime>, Without<ClockPart>, Without<ClockSpeed>)>,
    mut parts: Query<&mut Text, (With<ClockPart>, Without<ClockSpeed>)>,
    mut speeds: Query<&mut Text, With<ClockSpeed>>,
) {
    let Some(calendar) = calendar else { return };
    let hour = solar_hour(calendar.elapsed.as_secs_f32());
    let clock = format!("{:02}:{:02}", hour as u32, (hour.fract() * 60.) as u32);
    let part = if calendar.is_night() { "NIGHT" } else { "DAY" };
    let speed = time.relative_speed();
    let speed = if (speed - 1.).abs() < 1e-3 {
        String::new()
    } else {
        format!("{speed}x")
    };
    for (mut text, value) in times
        .iter_mut()
        .map(|t| (t, clock.as_str()))
        .chain(parts.iter_mut().map(|t| (t, part)))
        .chain(speeds.iter_mut().map(|t| (t, speed.as_str())))
    {
        if text.0 != value {
            text.0 = value.to_owned();
        }
    }
}

/// The player's latest notice (`PlayerInventory::feedback`), shown when it changes and
/// faded out after `NOTICE_S`.
fn show_notice(
    time: Res<Time<Real>>,
    player: Option<Res<PlayerInventory>>,
    mut notices: Query<(&mut Text, &mut TextColor), With<Notice>>,
    mut shown: Local<(String, f32)>,
) {
    let Some(player) = player else { return };
    let now = time.elapsed_secs();
    if player.feedback != shown.0 {
        *shown = (player.feedback.clone(), now);
    }
    let age = now - shown.1;
    let alpha = if shown.0.is_empty() {
        0.
    } else {
        ((NOTICE_S - age) / FADE_S).clamp(0., 1.)
    };
    // A faded notice is cleared, or its shadow would linger.
    let line = if alpha > 0. { shown.0.as_str() } else { "" };
    for (mut text, mut color) in &mut notices {
        if text.0 != line {
            text.0 = line.to_owned();
        }
        color.0 = INK.with_alpha(alpha);
    }
}

fn hide_under_screens(screens: Res<ScreenMode>, mut parts: Query<&mut Visibility, With<WorldHud>>) {
    let hidden = screens.open.is_some();
    for mut visibility in &mut parts {
        visibility.set_if_neq(if hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
}
