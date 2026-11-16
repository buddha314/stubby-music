//! The compass bar (sync-targeting-and-compass, after Satisfactory and the Godot game's
//! `add-compass-bar`): top centre, cardinal letters, smaller intercardinals and ticks
//! scrolling with the view, a caret at the heading, fading toward its ends. Bearings
//! follow the sky's convention: 0° north (−Z), 90° east (+X, where the sun rises). It
//! hides while a screen has the controls.
use bevy::prelude::*;
use stubby_core::{HudCamera, ScreenMode};

/// The bar's width and how many degrees it spans.
const WIDTH_PX: f32 = 520.;
const SPAN_DEGREES: f32 = 150.;
const PX_PER_DEGREE: f32 = WIDTH_PX / SPAN_DEGREES;

/// Compass bearing of a view direction, 0..360 (0 north = −Z, 90 east = +X).
pub fn bearing(forward: Vec3) -> f32 {
    forward.x.atan2(-forward.z).to_degrees().rem_euclid(360.)
}
/// Signed degrees from `heading` to `bearing`, −180..180 (positive to the right).
pub fn offset(heading: f32, bearing: f32) -> f32 {
    (bearing - heading + 540.).rem_euclid(360.) - 180.
}

/// A mark on the compass at a bearing: a label (text) or a tick or the caret (a filled
/// node). Each is its own top-level UI node, scrolled from the screen's centre by
/// `UiTransform`, so turning never re-lays out the HUD.
#[derive(Component)]
struct Mark {
    /// `None` for the heading caret, which stays at the centre.
    bearing: Option<f32>,
    width_px: f32,
    alpha: f32,
    /// A filled node fades its `BackgroundColor`; a label fades its text. (Every `Node`
    /// has a transparent `BackgroundColor`: fading a label's would draw a dark box.)
    filled: bool,
}

/// The bar's top edge, in pixels from the top of the screen.
const TOP_PX: f32 = 14.;

pub(crate) fn add(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, follow_view);
}

/// A node at the screen's centre line, `top_px` below the bar's top.
fn at_centre(top_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: px(TOP_PX + top_px),
        left: percent(50),
        ..default()
    }
}

fn spawn(mut commands: Commands) {
    let text = |commands: &mut Commands, label: &str, size: f32, bearing: f32, alpha: f32| {
        // Roughly the label's width, to centre it on its bearing.
        let width_px = size * label.len() as f32 * 0.62;
        commands.spawn((
            Mark {
                bearing: Some(bearing),
                width_px,
                alpha,
                filled: false,
            },
            Text::new(label),
            TextFont {
                font_size: bevy::text::FontSize::Px(size),
                ..default()
            },
            TextColor(Color::srgba(1., 1., 1., alpha)),
            at_centre(12. - size * 0.5),
        ));
    };
    for (label, at) in [("N", 0.), ("E", 90.), ("S", 180.), ("W", 270.)] {
        text(&mut commands, label, 18., at, 1.);
    }
    for (label, at) in [("NE", 45.), ("SE", 135.), ("SW", 225.), ("NW", 315.)] {
        text(&mut commands, label, 12., at, 0.75);
    }
    for at in (0..360).step_by(15).filter(|at| at % 45 != 0) {
        commands.spawn((
            Mark {
                bearing: Some(at as f32),
                width_px: 1.,
                alpha: 0.5,
                filled: true,
            },
            Node {
                width: px(1),
                height: px(8),
                ..at_centre(8.)
            },
            BackgroundColor(Color::srgba(1., 1., 1., 0.5)),
        ));
    }
    // The caret marking the heading: a short warm bar under the centre.
    commands.spawn((
        Mark {
            bearing: None,
            width_px: 2.,
            alpha: 0.9,
            filled: true,
        },
        Node {
            width: px(2),
            height: px(7),
            ..at_centre(26.)
        },
        BackgroundColor(Color::srgba(1., 0.85, 0.6, 0.9)),
    ));
}

#[allow(clippy::type_complexity)] // Each mark's transform, visibility and colours.
fn follow_view(
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    screens: Option<Res<ScreenMode>>,
    mut marks: Query<(
        &Mark,
        &mut UiTransform,
        &mut Visibility,
        Option<&mut TextColor>,
        &mut BackgroundColor,
    )>,
) {
    let hidden = screens.is_some_and(|screens| screens.open.is_some());
    let heading = cameras
        .single()
        .map(|camera| bearing(camera.forward().as_vec3()))
        .unwrap_or(0.);
    for (mark, mut transform, mut visibility, text, mut background) in &mut marks {
        let degrees = mark.bearing.map_or(0., |at| offset(heading, at));
        // Fade toward the ends; gone beyond them.
        let edge = (1. - degrees.abs() / (SPAN_DEGREES / 2.)).clamp(0., 1.);
        let alpha = mark.alpha * (edge * 3.).min(1.);
        visibility.set_if_neq(if hidden || alpha <= 0. {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        let x = px(degrees * PX_PER_DEGREE - mark.width_px / 2.);
        if transform.translation.x != x {
            transform.translation.x = x;
        }
        // Write colours only when they change.
        if mark.filled {
            let faded = background.0.with_alpha(alpha);
            if background.0 != faded {
                background.0 = faded;
            }
        } else if let Some(mut color) = text {
            let faded = color.0.with_alpha(alpha);
            if color.0 != faded {
                color.0 = faded;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bearings_follow_the_sky_convention() {
        assert!((bearing(Vec3::NEG_Z) - 0.).abs() < 1e-3);
        assert!((bearing(Vec3::X) - 90.).abs() < 1e-3);
        assert!((bearing(Vec3::Z) - 180.).abs() < 1e-3);
        assert!((bearing(Vec3::NEG_X) - 270.).abs() < 1e-3);
        // Turning from east to north moves N from the left of the bar to the centre.
        assert!((offset(90., 0.) - -90.).abs() < 1e-3);
        assert!((offset(0., 0.)).abs() < 1e-3);
        assert!((offset(350., 10.) - 20.).abs() < 1e-3);
    }
}
