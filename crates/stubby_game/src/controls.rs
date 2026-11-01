//! Satisfactory-style first-person controls shared with Jackdaw Play.
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub const INTERACT: KeyCode = KeyCode::KeyE;
pub const RECIPE_BOOK: KeyCode = KeyCode::KeyO;
pub const INVENTORY: KeyCode = KeyCode::Tab;
pub const RECORDER: KeyCode = KeyCode::KeyH;
pub const JUMP: KeyCode = KeyCode::Space;
pub const CROUCH: KeyCode = KeyCode::KeyC;
const EYE_HEIGHT: f32 = 1.65;
const CROUCH_HEIGHT: f32 = 1.0;

#[derive(Component)]
pub struct Player {
    feet: f32,
    vertical_speed: f32,
    grounded: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            feet: floor(0., 6.),
            vertical_speed: 0.,
            grounded: true,
        }
    }
}
fn floor(x: f32, z: f32) -> f32 {
    super::terrain::height(Vec2::new(x, z)) + 0.035
}
pub fn spawn_transform() -> Transform {
    Transform::from_xyz(0., floor(0., 6.) + EYE_HEIGHT, 6.)
}
#[derive(Resource)]
pub(crate) struct LookCapture(bool);

/// Full-screen HUD panels. One is open at a time; while one is, the cursor is free and
/// the player neither moves nor looks (add-inventory-and-goals-hud D7). The burbler's
/// rack screen joins this when it lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Inventory,
    MachineConfiguration(Entity),
}
#[derive(Resource, Default)]
pub struct ScreenMode {
    pub open: Option<Screen>,
    closed_this_frame: bool,
}
impl ScreenMode {
    pub(crate) fn close(&mut self) {
        self.open = None;
        self.closed_this_frame = true;
    }
}
pub struct PlayerControlsPlugin;
impl Plugin for PlayerControlsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LookCapture(true))
            .init_resource::<ScreenMode>()
            .add_systems(
                Update,
                (toggle_screens, capture_cursor, move_player).chain(),
            );
    }
}
/// Tab opens and closes the inventory; Escape closes whatever is open.
pub(crate) fn toggle_screens(keys: Res<ButtonInput<KeyCode>>, mut screens: ResMut<ScreenMode>) {
    let was = screens.open;
    if keys.just_pressed(INVENTORY) {
        screens.open = match screens.open {
            Some(Screen::Inventory) => None,
            _ => Some(Screen::Inventory),
        };
    } else if keys.just_pressed(KeyCode::Escape) {
        screens.open = None;
    }
    screens.closed_this_frame = was.is_some() && screens.open.is_none();
}
pub(crate) fn capture_cursor(
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
fn move_player(
    time: Res<Time<Virtual>>,
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<LookCapture>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut players: Query<(&mut Transform, &mut Player)>,
) {
    let delta: Vec2 = motion.read().map(|event| event.delta).sum();
    if !capture.0 {
        return;
    }
    for (mut transform, mut player) in &mut players {
        look(&mut transform, delta);
        step(
            &mut transform,
            &mut player,
            &keys,
            time.delta_secs().min(0.1 * time.relative_speed()),
        );
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
fn step(transform: &mut Transform, player: &mut Player, keys: &ButtonInput<KeyCode>, dt: f32) {
    let crouched = keys.pressed(CROUCH);
    let speed = if crouched {
        2.
    } else if keys.pressed(KeyCode::ShiftLeft) {
        8.
    } else {
        4.
    };
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let mut input = Vec3::ZERO;
    for (key, axis) in [
        (KeyCode::KeyW, Vec3::NEG_Z),
        (KeyCode::KeyS, Vec3::Z),
        (KeyCode::KeyA, Vec3::NEG_X),
        (KeyCode::KeyD, Vec3::X),
    ] {
        if keys.pressed(key) {
            input += axis;
        }
    }
    let direction = Quat::from_rotation_y(yaw) * input.normalize_or_zero();
    if keys.just_pressed(JUMP) && player.grounded {
        player.vertical_speed = 7.;
        player.grounded = false;
    }
    // Bound simulation catch-up at the maximum 64× speed; substep for stable landing.
    let mut remaining = dt.min(6.4);
    while remaining > 0. {
        let dt = remaining.min(1. / 120.);
        remaining -= dt;
        transform.translation += direction * speed * dt;
        let ground = floor(transform.translation.x, transform.translation.z);
        if player.grounded {
            player.feet = ground;
        } else {
            player.vertical_speed -= 20. * dt;
            player.feet += player.vertical_speed * dt;
            if player.feet <= ground {
                player.feet = ground;
                player.vertical_speed = 0.;
                player.grounded = true;
            }
        }
    }
    transform.translation.y = player.feet + if crouched { CROUCH_HEIGHT } else { EYE_HEIGHT };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_loss_releases_cursor_and_click_recaptures() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(LookCapture(true))
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
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(!app.world().resource::<LookCapture>().0);
    }

    #[test]
    fn spawn_and_pitched_walking_stay_grounded() {
        let mut t = spawn_transform();
        let mut p = Player::default();
        assert_eq!(t.translation.y, p.feet + EYE_HEIGHT);
        look(&mut t, Vec2::new(0., 300.));
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        for _ in 0..120 {
            step(&mut t, &mut p, &keys, 1. / 60.);
        }
        assert!((t.translation.z + 2.).abs() < 0.001);
        assert_eq!(
            t.translation.y,
            floor(t.translation.x, t.translation.z) + EYE_HEIGHT
        );
        assert!(p.grounded);
    }
    #[test]
    fn jump_lands_without_auto_repeat_and_crouch_preserves_feet() {
        let mut t = spawn_transform();
        let mut p = Player::default();
        let mut keys = ButtonInput::default();
        keys.press(JUMP);
        step(&mut t, &mut p, &keys, 1. / 60.);
        assert!(!p.grounded);
        assert!(p.feet > floor(0., 6.));
        keys.clear();
        for _ in 0..120 {
            step(&mut t, &mut p, &keys, 1. / 60.);
        }
        assert!(p.grounded);
        keys.press(CROUCH);
        step(&mut t, &mut p, &keys, 1. / 60.);
        assert_eq!(t.translation.y, p.feet + CROUCH_HEIGHT);
        keys.release(CROUCH);
        step(&mut t, &mut p, &keys, 1. / 60.);
        assert_eq!(t.translation.y, p.feet + EYE_HEIGHT);
    }
    #[test]
    fn mouse_turns_heading_and_pitch_without_roll_or_flip() {
        let mut t = spawn_transform();
        look(&mut t, Vec2::new(200., -10000.));
        let (yaw, pitch, roll) = t.rotation.to_euler(EulerRot::YXZ);
        assert!((yaw + 0.6).abs() < 0.001);
        assert!((pitch - 1.54).abs() < 0.001);
        assert!(roll.abs() < 0.001);
    }
}
