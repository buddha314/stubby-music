//! The in-world HUD around the reticle and the screen's edges, after Satisfactory's
//! (`HUD 2.png`), in the style guide's look (`stubby_ui::style`):
//!
//! - **The sky and the air**, top left (user, 2026-11-20: no clock time): an arc from
//!   sunrise to sunset with the sun riding it (by night, the moon's from dusk to dawn, in its
//!   pale colour), the simulation speed in amber only when it is not 1x; and an air quality
//!   gauge, an index that falls as the air clears (`CameraLightQuality`), in its band's
//!   colour with the band named.
//! - **Key hints**, bottom right: a keycap over each tool's name, as Satisfactory's cluster.
//! - **Notices**: one line under the interaction prompt that answers what the player just
//!   did (no recorder in hand, not enough reeds), fading after a few seconds.
//!
//! All of it hides while a screen has the controls.
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use stubby_core::{ScreenMode, keys};
use stubby_ui::style::{self, AMBER, BAND, EDGE, INK, MUTED};
use stubby_world::calendar::{CYCLE_SECONDS, Calendar, DAY_SECONDS};
use stubby_world::lighting::CameraLightQuality;

use super::recipes_plugin::PlayerInventory;

/// How long a notice stays, and how long it takes to fade, in seconds.
const NOTICE_S: f32 = 3.5;
const FADE_S: f32 = 0.8;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UiMaterialPlugin::<SunArcMaterial>::default(),
            UiMaterialPlugin::<GaugeMaterial>::default(),
        ))
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (show_sky, show_air, show_notice, hide_under_screens),
        );
    }
}

/// HUD parts that step aside while a screen is open.
#[derive(Component)]
struct WorldHud;
#[derive(Component)]
struct SkyTitle;
#[derive(Component)]
struct SkyEnds(bool); // true: the left end's label
#[derive(Component)]
struct ClockSpeed;
#[derive(Component)]
struct AirValue;
#[derive(Component)]
struct AirBand;

/// The sky card's arc (`assets/shaders/hud_sun_arc.wesl`).
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SunArcMaterial {
    #[uniform(0)]
    arc: SunArc,
}
#[derive(ShaderType, Debug, Clone, Copy)]
struct SunArc {
    /// x: how far along, y: line width, z: disc radius (pixels).
    params: Vec4,
    ink: Vec4,
    dim: Vec4,
    disc: Vec4,
    ground: Vec4,
}
impl UiMaterial for SunArcMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/hud_sun_arc.wesl".into()
    }
}

/// The air quality card's gauge (`assets/shaders/hud_gauge.wesl`).
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct GaugeMaterial {
    #[uniform(0)]
    gauge: Gauge,
}
#[derive(ShaderType, Debug, Clone, Copy)]
struct Gauge {
    /// x: how full, y: ring thickness (pixels).
    params: Vec4,
    track: Vec4,
    fill: Vec4,
}
impl UiMaterial for GaugeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/hud_gauge.wesl".into()
    }
}

fn rgba(color: Color) -> Vec4 {
    let c = LinearRgba::from(color);
    Vec4::new(c.red, c.green, c.blue, c.alpha)
}

/// The sun's disc by day, the moon's by night.
const SUN: Color = Color::srgb(1., 0.75, 0.25);
const MOON: Color = Color::srgb(0.78, 0.86, 1.);

/// Where the sun (or moon) is along its arc, 0..1, and whether it is night.
pub fn sky_progress(elapsed_seconds: f32) -> (f32, bool) {
    let seconds = elapsed_seconds.rem_euclid(CYCLE_SECONDS as f32);
    if seconds < DAY_SECONDS as f32 {
        (seconds / DAY_SECONDS as f32, false)
    } else {
        (
            (seconds - DAY_SECONDS as f32) / (CYCLE_SECONDS - DAY_SECONDS) as f32,
            true,
        )
    }
}

/// The air quality index shown for the air's cleanliness (0 bleak .. 1 clean): it falls as
/// the air clears, on the familiar scale where lower is better.
pub const AIR_INDEX_MAX: f32 = 310.;
pub fn air_index(clean: f32) -> u32 {
    ((1. - clean.clamp(0., 1.)) * AIR_INDEX_MAX).round() as u32
}

/// The index's band: its name and colour (the usual bands, muted for the HUD).
pub fn air_band(index: u32) -> (&'static str, Color) {
    match index {
        0..=50 => ("Good", Color::srgb(0.31, 0.76, 0.42)),
        51..=100 => ("Moderate", Color::srgb(0.91, 0.79, 0.29)),
        101..=150 => ("Unhealthy (some)", Color::srgb(0.91, 0.57, 0.23)),
        151..=200 => ("Unhealthy", Color::srgb(0.85, 0.31, 0.25)),
        201..=300 => ("Very unhealthy", Color::srgb(0.6, 0.35, 0.66)),
        _ => ("Hazardous", Color::srgb(0.54, 0.16, 0.23)),
    }
}

fn card<'a>(parent: &'a mut ChildSpawnerCommands, width: f32) -> EntityCommands<'a> {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: px(width),
            padding: UiRect::axes(px(7), px(5)),
            row_gap: px(1),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        BackgroundColor(BAND),
        BorderColor::all(EDGE),
        Pickable::IGNORE,
    ))
}
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

fn spawn(
    mut commands: Commands,
    mut arcs: Option<ResMut<Assets<SunArcMaterial>>>,
    mut gauges: Option<ResMut<Assets<GaugeMaterial>>>,
) {
    // The sky and the air, top left.
    let arc = arcs.as_mut().map(|arcs| {
        arcs.add(SunArcMaterial {
            arc: SunArc {
                params: Vec4::new(0., 1.5, 5., 0.),
                ink: rgba(INK),
                dim: rgba(MUTED.with_alpha(0.45)),
                disc: rgba(SUN),
                ground: rgba(Color::srgba(1., 1., 1., 0.05)),
            },
        })
    });
    let gauge = gauges.as_mut().map(|gauges| {
        gauges.add(GaugeMaterial {
            gauge: Gauge {
                params: Vec4::new(1., 6., 0., 0.),
                track: rgba(Color::srgba(0.8, 0.8, 0.82, 0.22)),
                fill: rgba(air_band(AIR_INDEX_MAX as u32).1),
            },
        })
    });
    commands
        .spawn((
            WorldHud,
            Name::new("Sky and air"),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(20),
                top: px(20),
                column_gap: px(6),
                ..default()
            },
        ))
        .with_children(|column| {
            card(column, 132.).with_children(|sky| {
                sky.spawn(Node {
                    column_gap: px(8),
                    align_items: AlignItems::Baseline,
                    ..default()
                })
                .with_children(|title| {
                    title.spawn((SkyTitle, style::world_label("Day", 10., MUTED)));
                    title.spawn((ClockSpeed, style::world_label("", 10., AMBER)));
                });
                sky.spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|ends| {
                    ends.spawn((SkyEnds(true), style::world_label("Sunrise", 9., INK)));
                    ends.spawn((SkyEnds(false), style::world_label("Sunset", 9., INK)));
                });
                let mut arc_node = sky.spawn((
                    Node {
                        width: px(118.),
                        height: px(36.),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
                if let Some(arc) = arc {
                    arc_node.insert(MaterialNode(arc));
                }
            });
            card(column, 104.).with_children(|air| {
                air.spawn(style::world_label("Air quality", 10., MUTED));
                air.spawn(Node {
                    width: px(90.),
                    height: px(50.),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|holder| {
                    let mut ring = holder.spawn((
                        Node {
                            width: px(84.),
                            height: px(48.),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::End,
                            padding: UiRect::bottom(px(1)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    if let Some(gauge) = gauge {
                        ring.insert(MaterialNode(gauge));
                    }
                    ring.with_children(|ring| {
                        ring.spawn((AirValue, style::world_label("310", 15., INK)));
                        ring.spawn((AirBand, style::world_label("Hazardous", 8., MUTED)));
                    });
                });
            });
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

/// The sky card: where the sun (by night, the moon) is along its arc, the ends' names,
/// and the speed only when it is not 1x. No clock time.
#[allow(clippy::type_complexity)] // The card's texts.
fn show_sky(
    calendar: Option<Res<Calendar>>,
    sim: Option<Res<stubby_core::SimSpeed>>,
    mut arcs: Option<ResMut<Assets<SunArcMaterial>>>,
    nodes: Query<&MaterialNode<SunArcMaterial>>,
    mut titles: Query<&mut Text, (With<SkyTitle>, Without<SkyEnds>, Without<ClockSpeed>)>,
    mut ends: Query<(&mut Text, &SkyEnds), Without<ClockSpeed>>,
    mut speeds: Query<&mut Text, With<ClockSpeed>>,
) {
    let Some(calendar) = calendar else { return };
    let (progress, night) = sky_progress(calendar.elapsed.as_secs_f32());
    if let Some(arcs) = arcs.as_mut() {
        for node in &nodes {
            if let Some(mut material) = arcs.get_mut(&node.0) {
                material.arc.params.x = progress;
                material.arc.disc = rgba(if night { MOON } else { SUN });
            }
        }
    }
    let set = |text: &mut Text, value: &str| {
        if text.0 != value {
            text.0 = value.to_owned();
        }
    };
    for mut title in &mut titles {
        set(&mut title, if night { "Night" } else { "Day" });
    }
    for (mut text, SkyEnds(left)) in &mut ends {
        let name = match (night, *left) {
            (false, true) => "Sunrise",
            (false, false) => "Sunset",
            (true, true) => "Dusk",
            (true, false) => "Dawn",
        };
        set(&mut text, name);
    }
    let speed = sim.map_or(1., |sim| sim.0);
    let speed = if (speed - 1.).abs() < 1e-3 {
        String::new()
    } else {
        format!("{speed}x")
    };
    for mut text in &mut speeds {
        set(&mut text, &speed);
    }
}

/// The air quality card: the index for the air at the camera, its band and colour.
fn show_air(
    quality: Option<Res<CameraLightQuality>>,
    mut gauges: Option<ResMut<Assets<GaugeMaterial>>>,
    nodes: Query<&MaterialNode<GaugeMaterial>>,
    mut values: Query<&mut Text, (With<AirValue>, Without<AirBand>)>,
    mut bands: Query<&mut Text, With<AirBand>>,
    mut shown: Local<Option<u32>>,
) {
    let Some(quality) = quality else { return };
    let index = air_index(quality.0);
    if *shown == Some(index) {
        return;
    }
    *shown = Some(index);
    let (band, colour) = air_band(index);
    if let Some(gauges) = gauges.as_mut() {
        for node in &nodes {
            if let Some(mut material) = gauges.get_mut(&node.0) {
                material.gauge.params.x = index as f32 / AIR_INDEX_MAX;
                material.gauge.fill = rgba(colour);
            }
        }
    }
    for mut text in &mut values {
        text.0 = index.to_string();
    }
    for mut text in &mut bands {
        text.0 = band.to_owned();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sky_runs_sunrise_to_sunset_then_dusk_to_dawn() {
        assert_eq!(sky_progress(0.), (0., false));
        assert_eq!(sky_progress(DAY_SECONDS as f32 / 2.), (0.5, false));
        let (at, night) = sky_progress(DAY_SECONDS as f32 + 450.);
        assert!(night && (at - 0.5).abs() < 1e-6);
        assert_eq!(sky_progress(CYCLE_SECONDS as f32), (0., false));
    }

    #[test]
    fn the_air_index_falls_as_the_air_clears() {
        assert_eq!(air_index(0.), 310);
        assert_eq!(air_band(air_index(0.)).0, "Hazardous");
        assert_eq!(air_band(air_index(0.95)).0, "Good");
        assert!(air_index(0.5) < air_index(0.2));
    }
}
