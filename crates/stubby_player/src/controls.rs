//! Satisfactory-style first-person controls shared with Jackdaw Play: look (per frame),
//! the screens' keys and mouse capture. Moving is `crate::movement` (bevy_ahoy in the
//! fixed step); the camera carrying [`Player`] follows the body it moves.
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use stubby_core::{GroundHeight, INVENTORY, Screen, ScreenMode};

pub const EYE_HEIGHT: f32 = 1.65;
pub const CROUCH_HEIGHT: f32 = 1.0;

/// The camera the player sees through. `crate::movement` keeps it at the eye of the body
/// it moves, and `feet` where that body stands.
#[derive(Component)]
pub struct Player {
    pub(crate) feet: f32,
}
impl Player {
    /// The height of the ground under the player's feet.
    pub fn feet(&self) -> f32 {
        self.feet
    }
    /// A player standing at the spawn point on `ground`.
    pub fn on(ground: &GroundHeight) -> Self {
        Self {
            feet: floor(
                ground,
                stubby_core::OPENING_SPAWN_XZ.x,
                stubby_core::OPENING_SPAWN_XZ.y,
            ),
        }
    }
}
pub(crate) fn floor(ground: &GroundHeight, x: f32, z: f32) -> f32 {
    ground.at(x, z) + 0.035
}
pub fn spawn_transform(ground: &GroundHeight) -> Transform {
    let spawn = stubby_core::OPENING_SPAWN_XZ;
    Transform::from_xyz(
        spawn.x,
        floor(ground, spawn.x, spawn.y) + EYE_HEIGHT,
        spawn.y,
    )
    .with_rotation(Quat::from_rotation_y(
        -stubby_core::OPENING_HEADING_DEGREES.to_radians(),
    ))
}
pub use stubby_core::LookCapture;

pub struct PlayerControlsPlugin;
impl Plugin for PlayerControlsPlugin {
    fn build(&self, app: &mut App) {
        crate::compass::add(app);
        crate::movement::add(app);
        app.init_resource::<GroundHeight>()
            .insert_resource(LookCapture(true))
            .init_resource::<ScreenMode>()
            .add_systems(
                Update,
                (
                    toggle_screens.in_set(stubby_core::ScreenInput),
                    capture_cursor.in_set(stubby_core::CursorCapture),
                    look_around,
                )
                    .chain(),
            );
    }
}
/// Tab opens and closes the inventory; Escape closes whatever is open. In the world, the
/// first Escape frees the cursor (`capture_cursor`) and a second opens the pause menu,
/// Satisfactory's escape menu (user, 2026-11-17).
pub fn toggle_screens(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Option<Res<LookCapture>>,
    mut screens: ResMut<ScreenMode>,
) {
    // Without a look capture (tests, tools) the cursor counts as captured.
    let captured = capture.is_none_or(|capture| capture.captured());
    let was = screens.open;
    if was.is_some_and(Screen::is_menu) {
        screens.closed_this_frame = false;
        return;
    }
    if keys.just_pressed(INVENTORY) && screens.open != Some(Screen::Pause) {
        screens.open = match screens.open {
            Some(Screen::Inventory) => None,
            _ => Some(Screen::Inventory),
        };
    } else if keys.just_pressed(KeyCode::Escape) {
        screens.open = match screens.open {
            None if !captured => Some(Screen::Pause),
            _ => None,
        };
    }
    screens.closed_this_frame = was.is_some() && screens.open.is_none();
}
pub fn capture_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    screens: Res<ScreenMode>,
    mut capture: ResMut<LookCapture>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let Ok((window, mut cursor)) = windows.single_mut() else {
        return;
    };
    if screens.open.is_some() || !window.focused {
        capture.0 = false;
    } else if screens.closed_this_frame || buttons.just_pressed(MouseButton::Left) {
        capture.0 = true;
    } else if keys.just_pressed(KeyCode::Escape) {
        capture.0 = false;
    }
    cursor.visible = !capture.0;
    cursor.grab_mode = if capture.0 {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}
/// Mouse look, every frame while the cursor is captured. The body moves where the camera
/// faces (`crate::movement`).
fn look_around(
    capture: Res<LookCapture>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let delta: Vec2 = motion.read().map(|event| event.delta).sum();
    if !capture.0 {
        return;
    }
    for mut transform in &mut players {
        look(&mut transform, delta);
    }
}
fn look(transform: &mut Transform, delta: Vec2) {
    let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        yaw - delta.x * 0.003,
        (pitch - delta.y * 0.003).clamp(-1.54, 1.54),
        0.,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    /// A gentle slope, so staying grounded is tested on uneven ground.
    const GROUND: GroundHeight = GroundHeight(|p| 0.05 * p.x - 0.03 * p.y);
    #[test]
    fn focus_loss_releases_cursor_and_click_recaptures() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(LookCapture(true))
            .init_resource::<ScreenMode>()
            .add_systems(Update, capture_cursor);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: false,
                    ..default()
                },
                CursorOptions::default(),
                PrimaryWindow,
            ))
            .id();
        app.update();
        assert!(!app.world().resource::<LookCapture>().0);
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::None
        );
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert!(app.world().resource::<LookCapture>().0);
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::Locked
        );
        // No InputPlugin here: clear last frame's click as Bevy does each frame.
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(!app.world().resource::<LookCapture>().0);
    }

    #[test]
    fn escape_and_tab_leave_the_menus_to_themselves() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(LookCapture(false))
            .init_resource::<ScreenMode>()
            .add_systems(Update, toggle_screens);
        for menu in [Screen::MainMenu, Screen::Loading, Screen::Settings] {
            app.world_mut().resource_mut::<ScreenMode>().open = Some(menu);
            for key in [KeyCode::Escape, INVENTORY] {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.release_all();
                keys.clear();
                keys.press(key);
                app.update();
                assert_eq!(
                    app.world().resource::<ScreenMode>().open,
                    Some(menu),
                    "{key:?} on {menu:?}"
                );
            }
        }
    }

    #[test]
    fn escape_frees_the_cursor_then_opens_the_pause_menu_then_resumes() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(LookCapture(true))
            .init_resource::<ScreenMode>()
            .add_systems(Update, toggle_screens);
        let escape = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.clear();
            keys.press(KeyCode::Escape);
            app.update();
        };
        let open = |app: &App| app.world().resource::<ScreenMode>().open;
        // Looking around: the first Escape only frees the cursor (capture_cursor does).
        escape(&mut app);
        assert_eq!(open(&app), None);
        app.world_mut().resource_mut::<LookCapture>().0 = false;
        // The second opens the pause menu; Tab does not swap it for the inventory.
        escape(&mut app);
        assert_eq!(open(&app), Some(Screen::Pause));
        {
            // No InputPlugin here: clear last frame's keys as Bevy does each frame.
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Escape);
            keys.clear();
            keys.press(INVENTORY);
        }
        app.update();
        assert_eq!(open(&app), Some(Screen::Pause));
        // Escape on the menu resumes.
        escape(&mut app);
        assert_eq!(open(&app), None);
        assert!(app.world().resource::<ScreenMode>().closed_this_frame);
    }

    #[test]
    fn opening_faces_fifteen_degrees_left_of_sunrise() {
        let camera = spawn_transform(&GROUND);
        let heading = crate::compass::bearing(camera.forward().as_vec3());
        assert!((heading - 75.).abs() < 0.001);
        assert!((crate::compass::offset(heading, 90.) - 15.).abs() < 0.001);
    }

    #[test]
    fn mouse_turns_heading_and_pitch_without_roll_or_flip() {
        let mut t = spawn_transform(&GROUND);
        look(&mut t, Vec2::new(200., -10000.));
        let (yaw, pitch, roll) = t.rotation.to_euler(EulerRot::YXZ);
        assert!((yaw + stubby_core::OPENING_HEADING_DEGREES.to_radians() + 0.6).abs() < 0.001);
        assert!((pitch - 1.54).abs() < 0.001);
        assert!(roll.abs() < 0.001);
    }
}
