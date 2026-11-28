//! Moving the player: `bevy_ahoy`'s character controller in the fixed step, driven by input
//! actions, as the us-outpost-31 template uses it. Chosen by the user over stubby's own
//! per-frame controller after the movement spike (add-multiplayer-and-menus 2.3,
//! 2026-11-27): a fixed-step controller fed by actions is what network prediction needs.
//!
//! - A body (`MoveBody`) carries ahoy's controller and collider; the camera (the
//!   [`Player`]) follows its eye every frame, so the hand, shadow, flashlight and
//!   interaction keep reading the camera and `Player::feet` as before. Look stays per
//!   frame (`controls::look`); the body moves where the camera faces.
//! - The ground is a function (`GroundHeight`), so a heightfield sampled from it covers
//!   the walkable world; `Solid` colliders (boulders) block and can be stepped on. Every
//!   other collider is ignored, as the interaction markers always were.
//! - Tuned to stubby's feel: walk 4 m/s, sprint 8, crouch 2, a 1.225 m jump, quick to
//!   start and stop. The simulation-speed keys scale the fixed step, so the tuning is
//!   scaled back each frame: the player always moves in real time.
//! - While a screen is open, or the cursor is free, the controls are off.

use avian3d::prelude::*;
use bevy::{app::RunFixedMainLoopSystems, ecs::relationship::Relationship, prelude::*};
use bevy_ahoy::{
    CharacterControllerDerivedProps, CharacterControllerState, CharacterLook, prelude::*,
};
use bevy_enhanced_input::prelude::*;
use stubby_core::{CROUCH, GroundHeight, JUMP, LookCapture, ScreenMode, Solid};

use crate::controls::{CROUCH_HEIGHT, EYE_HEIGHT, Player, floor};

/// Speeds, m/s: crouched, walking, sprinting.
pub const CROUCH_SPEED: f32 = 2.;
pub const WALK_SPEED: f32 = 4.;
pub const SPRINT_SPEED: f32 = 8.;
/// Jump take-off speed (m/s) and gravity (m/s²): a 1.225 m jump.
pub const JUMP_SPEED: f32 = 7.;
pub const GRAVITY: f32 = 20.;

/// The body's collider standing: a cylinder this wide and tall.
const BODY_RADIUS: f32 = 0.3;
const BODY_HEIGHT: f32 = 1.8;
/// The ground collider: the whole backdrop (1600 m square, `stubby_world`), sampled every
/// 4 m. The terrain varies by centimetres over tens of metres, so that is exact enough.
const GROUND_SIZE_M: f32 = 1600.;
const GROUND_SAMPLES: usize = 401;

/// The input context the movement actions belong to: the body.
#[derive(Component, Default)]
pub struct MoveControls;

/// Hold to run.
#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Sprint;

/// The body ahoy moves; `camera` is the [`Player`] that follows its eye.
#[derive(Component)]
pub struct MoveBody {
    pub camera: Entity,
}

pub(crate) fn add(app: &mut App) {
    if !app.is_plugin_added::<EnhancedInputPlugin>() {
        app.add_plugins(EnhancedInputPlugin);
    }
    app.add_plugins(AhoyPlugins::default())
        .add_input_context::<MoveControls>()
        .add_observer(spawn_body)
        .add_systems(Startup, spawn_ground_collider)
        // Before input is read, so a screen that just opened stops this frame's input.
        .add_systems(
            PreUpdate,
            switch_controls.before(EnhancedInputSystems::Update),
        )
        .add_systems(
            RunFixedMainLoop,
            steer.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(Update, follow_body);
}

/// ahoy's controller, tuned to stubby's feel at simulation speed `speed` (1 = real time)
/// while walking or `running`.
fn tuning(running: bool, speed: f32) -> CharacterController {
    let mut controller = CharacterController::default();
    tune(&mut controller, running, speed);
    controller
}

/// Set only the tuned fields: ahoy keeps state of its own in the controller (its query
/// filter excludes the body itself), and replacing the whole value loses it. Seen
/// 2026-11-27: a replaced controller collided with its own collider and lurched at 28 m/s.
fn tune(controller: &mut CharacterController, running: bool, speed: f32) {
    let s = speed.max(1e-3);
    let defaults = CharacterController::default();
    let walk = if running { SPRINT_SPEED } else { WALK_SPEED };
    // The fixed step runs `s` times faster than real time: velocities shrink by `s`,
    // accelerations by s², rates by `s`, and times grow by `s`, so the motion seen in
    // real time is unchanged (and the jump keeps its height).
    controller.speed = walk / s;
    controller.max_speed = SPRINT_SPEED / s;
    controller.air_speed = defaults.air_speed / s;
    controller.stop_speed = defaults.stop_speed / s;
    controller.max_air_wish_speed = defaults.max_air_wish_speed / s;
    controller.unground_speed = defaults.unground_speed / s;
    controller.crouch_speed_scale = CROUCH_SPEED / walk;
    controller.gravity = GRAVITY / (s * s);
    controller.jump_height = JUMP_SPEED * JUMP_SPEED / (2. * GRAVITY);
    controller.acceleration_hz = 30. / s;
    controller.air_acceleration_hz = defaults.air_acceleration_hz / s;
    controller.friction_hz = 30. / s;
    controller.coyote_time = defaults.coyote_time.mul_f32(s);
    controller.jump_input_buffer = defaults.jump_input_buffer.mul_f32(s);
}

/// The movement actions on `context`: WASD and the left stick, Space, C, Shift.
fn spawn_actions(commands: &mut Commands, context: Entity) {
    commands
        .spawn((
            ActionOf::<MoveControls>::new(context),
            Action::<Movement>::new(),
            DeadZone::default(),
        ))
        .apply_scene(bsn! {
            Bindings [
                {bevy_enhanced_input::preset::cardinal::new(
                    KeyCode::KeyW, KeyCode::KeyD, KeyCode::KeyS, KeyCode::KeyA
                )}
                --
                {bevy_enhanced_input::preset::axial::left_stick()}
            ]
        });
    // One jump per press: ahoy jumps again for as long as jump fires, and stubby's jump
    // never auto-repeated.
    commands
        .spawn((
            ActionOf::<MoveControls>::new(context),
            Action::<Jump>::new(),
            Press::default(),
        ))
        .apply_scene(bsn! { Bindings [ Binding(JUMP) -- Binding(GamepadButton::South) ] });
    commands
        .spawn((
            ActionOf::<MoveControls>::new(context),
            Action::<Crouch>::new(),
        ))
        .apply_scene(bsn! { Bindings [ Binding(CROUCH) -- Binding(GamepadButton::East) ] });
    commands
        .spawn((
            ActionOf::<MoveControls>::new(context),
            Action::<Sprint>::new(),
        ))
        .apply_scene(
            bsn! { Bindings [ Binding(KeyCode::ShiftLeft) -- Binding(GamepadButton::LeftThumb) ] },
        );
}

/// A player appeared: give it a body standing where it stands.
fn spawn_body(
    added: On<Add<Player>>,
    cameras: Query<&Transform>,
    ground: Res<GroundHeight>,
    mut commands: Commands,
) {
    let camera = added.entity;
    let at = cameras
        .get(camera)
        .map(|t| t.translation)
        .unwrap_or_default();
    let feet = floor(&ground, at.x, at.z);
    let body = commands
        .spawn((
            Name::new("Player body"),
            MoveBody { camera },
            MoveControls,
            tuning(false, 1.),
            RigidBody::Kinematic,
            Collider::cylinder(BODY_RADIUS, BODY_HEIGHT),
            CharacterLook::default(),
            TransformInterpolation,
            Transform::from_xyz(at.x, feet + BODY_HEIGHT / 2. + 0.05, at.z),
        ))
        .id();
    spawn_actions(&mut commands, body);
}

/// A static heightfield matching `GroundHeight` (with the floor's 3.5 cm plate offset)
/// over the whole world. Marked `Solid`: the ground is walked on.
fn spawn_ground_collider(ground: Res<GroundHeight>, mut commands: Commands) {
    let n = GROUND_SAMPLES;
    let at = |i: usize| (i as f32 / (n - 1) as f32 - 0.5) * GROUND_SIZE_M;
    // parry's heightfield: rows run along X, columns along Z (the tests pin this).
    let heights: Vec<Vec<f32>> = (0..n)
        .map(|row| (0..n).map(|col| floor(&ground, at(row), at(col))).collect())
        .collect();
    commands.spawn((
        Name::new("Ground collider"),
        RigidBody::Static,
        Solid,
        Collider::heightfield(heights, Vec3::new(GROUND_SIZE_M, 1., GROUND_SIZE_M)),
        Transform::default(),
    ));
}

/// Controls work only in the world: not while a screen is open or the cursor is free.
fn switch_controls(
    capture: Option<Res<LookCapture>>,
    screens: Res<ScreenMode>,
    mut bodies: Query<(Entity, Option<&ContextActivity<MoveControls>>), With<MoveBody>>,
    mut commands: Commands,
) {
    let active = capture.is_none_or(|c| c.0) && screens.open.is_none();
    for (body, activity) in &mut bodies {
        if activity.is_none_or(|a| **a != active) {
            commands
                .entity(body)
                .insert(ContextActivity::<MoveControls>::new(active));
        }
    }
}

/// Before the fixed steps: the body moves where the camera faces, Shift runs, and the
/// tuning follows the simulation speed.
fn steer(
    virtual_time: Res<Time<Virtual>>,
    cameras: Query<&Transform, With<Player>>,
    sprints: Query<(&ActionOf<MoveControls>, &Action<Sprint>)>,
    mut bodies: Query<(
        Entity,
        &MoveBody,
        &mut CharacterLook,
        &mut CharacterController,
    )>,
) {
    let speed = virtual_time.effective_speed();
    for (entity, body, mut look, mut controller) in &mut bodies {
        if let Ok(camera) = cameras.get(body.camera) {
            *look = CharacterLook::from_quat(camera.rotation);
        }
        let running = sprints
            .iter()
            .any(|(of, sprint)| of.get() == entity && **sprint);
        let wanted = tuning(running, speed);
        if controller.speed != wanted.speed || controller.gravity != wanted.gravity {
            tune(&mut controller, running, speed);
        }
    }
}

/// After the body moved: the camera at its eye (lower when crouched), and the player's
/// feet where the body stands.
fn follow_body(
    bodies: Query<
        (
            &MoveBody,
            &Transform,
            &CharacterController,
            Option<&CharacterControllerState>,
            Option<&CharacterControllerDerivedProps>,
        ),
        Without<Player>,
    >,
    mut players: Query<(&mut Transform, &mut Player)>,
) {
    for (body, at, controller, state, derived) in &bodies {
        let Ok((mut camera, mut player)) = players.get_mut(body.camera) else {
            continue;
        };
        let (to_feet, crouched) = match (state, derived) {
            (Some(state), Some(derived)) => (derived.pos_to_feet_dist(state), state.crouching),
            _ => (-BODY_HEIGHT / 2., false),
        };
        // ahoy hovers the body `ground_distance` above what it stands on.
        let feet = at.translation.y + to_feet - controller.ground_distance;
        player.feet = feet;
        camera.translation = Vec3::new(
            at.translation.x,
            feet + if crouched { CROUCH_HEIGHT } else { EYE_HEIGHT },
            at.translation.z,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// A slope, so the ground collider's orientation matters.
    const SLOPE: GroundHeight = GroundHeight(|p| 0.05 * p.x - 0.03 * p.y);
    const TICK: f64 = 1. / 64.;

    /// Movement on a headless app with physics, one fixed step per update.
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            bevy::mesh::MeshPlugin,
            bevy::input::InputPlugin,
            bevy::scene::ScenePlugin,
            PhysicsPlugins::default(),
        ))
        .insert_resource(SLOPE)
        .insert_resource(LookCapture(true))
        .init_resource::<ScreenMode>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            TICK,
        )))
        .insert_resource(Time::<Fixed>::from_seconds(TICK));
        add(&mut app);
        app.finish();
        app.cleanup();
        app
    }

    fn spawn_player(app: &mut App, x: f32, z: f32) -> Entity {
        let feet = floor(&SLOPE, x, z);
        let player = app
            .world_mut()
            .spawn((
                Player { feet: feet + 5. }, // wrong on purpose: the body must set it
                Transform::from_xyz(x, feet + EYE_HEIGHT, z),
            ))
            .id();
        for _ in 0..32 {
            app.update();
        }
        player
    }

    /// Hold (or release, with `None`) action `A`.
    fn hold<A: InputAction>(app: &mut App, value: Option<ActionValue>) {
        let world = app.world_mut();
        let actions: Vec<Entity> = world
            .query_filtered::<Entity, With<Action<A>>>()
            .iter(world)
            .collect();
        for entity in actions {
            let mock = match value {
                Some(value) => ActionMock::new(TriggerState::Fired, value, MockSpan::Manual),
                None => ActionMock::new(
                    TriggerState::None,
                    ActionValue::zero(A::Output::DIM),
                    MockSpan::Manual,
                ),
            };
            world.entity_mut(entity).insert(mock);
        }
    }

    fn run(app: &mut App, updates: usize) {
        for _ in 0..updates {
            app.update();
        }
    }

    fn xz(app: &App, entity: Entity) -> Vec2 {
        let t = app.world().get::<Transform>(entity).unwrap().translation;
        Vec2::new(t.x, t.z)
    }

    fn feet(app: &App, player: Entity) -> f32 {
        app.world().get::<Player>(player).unwrap().feet
    }

    #[test]
    fn the_body_stands_on_the_ground_away_from_spawn() {
        let mut app = app();
        let (x, z) = (40., -30.);
        let player = spawn_player(&mut app, x, z);
        run(&mut app, 96);
        let ground = floor(&SLOPE, x, z);
        assert!(
            (feet(&app, player) - ground).abs() < 0.03,
            "feet {} on {ground}",
            feet(&app, player)
        );
        let eye = app.world().get::<Transform>(player).unwrap().translation.y;
        assert!((eye - feet(&app, player) - EYE_HEIGHT).abs() < 1e-4);
    }

    #[test]
    fn walking_and_running_keep_their_speeds_in_real_time() {
        let mut app = app();
        let player = spawn_player(&mut app, 0., 6.);
        hold::<Movement>(&mut app, Some(Vec2::Y.into()));
        run(&mut app, 32); // up to speed
        let start = xz(&app, player);
        run(&mut app, 64);
        let walked = xz(&app, player).distance(start);
        assert!(
            (walked - WALK_SPEED).abs() < 0.3,
            "walked {walked} m in a second"
        );

        hold::<Sprint>(&mut app, Some(true.into()));
        run(&mut app, 32);
        let start = xz(&app, player);
        run(&mut app, 64);
        let ran = xz(&app, player).distance(start);
        assert!((ran - SPRINT_SPEED).abs() < 0.5, "ran {ran} m in a second");

        // Four times the simulation speed: the player still runs 8 m a real second.
        hold::<Sprint>(&mut app, None);
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .set_relative_speed(4.);
        run(&mut app, 32);
        let start = xz(&app, player);
        run(&mut app, 64);
        let walked = xz(&app, player).distance(start);
        assert!(
            (walked - WALK_SPEED).abs() < 0.3,
            "walked {walked} m in a real second at 4x"
        );
    }

    #[test]
    fn a_jump_reaches_stubbys_height_and_holding_jump_does_not_repeat() {
        let mut app = app();
        let player = spawn_player(&mut app, 0., 6.);
        let ground = feet(&app, player);
        // Space held down, as the keyboard sends it (a mock would skip `Press`).
        app.world_mut()
            .write_message(bevy::input::keyboard::KeyboardInput {
                key_code: JUMP,
                logical_key: bevy::input::keyboard::Key::Space,
                state: bevy::input::ButtonState::Pressed,
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
        let mut peak = ground;
        let mut landings = 0;
        let mut airborne = false;
        for _ in 0..192 {
            app.update();
            let now = feet(&app, player);
            peak = peak.max(now);
            if now > ground + 0.1 {
                airborne = true;
            } else if airborne {
                airborne = false;
                landings += 1;
            }
        }
        let height = peak - ground;
        assert!((height - 1.225).abs() < 0.15, "jumped {height} m");
        assert!(landings <= 1, "held Space jumped {} times", landings + 1);
    }

    #[test]
    fn crouching_lowers_the_eye_and_keeps_the_feet() {
        let mut app = app();
        let player = spawn_player(&mut app, 0., 6.);
        let standing = feet(&app, player);
        hold::<Crouch>(&mut app, Some(true.into()));
        run(&mut app, 16);
        let eye = app.world().get::<Transform>(player).unwrap().translation.y;
        assert!((feet(&app, player) - standing).abs() < 0.03);
        assert!(
            (eye - feet(&app, player) - CROUCH_HEIGHT).abs() < 1e-4,
            "eye {eye}"
        );
    }

    #[test]
    fn a_solid_stops_the_body_and_a_marker_does_not() {
        let mut app = app();
        let player = spawn_player(&mut app, 0., 6.);
        let ground = floor(&SLOPE, 0., 0.);
        // A marker (a sensor) just ahead, then a wall beyond it. Ahead is -Z.
        app.world_mut().spawn((
            Sensor,
            Collider::sphere(0.25),
            Transform::from_xyz(0., ground + 1., 4.),
        ));
        app.world_mut().spawn((
            RigidBody::Static,
            Solid,
            Collider::cuboid(10., 4., 0.5),
            Transform::from_xyz(0., ground + 2., 1.),
        ));
        hold::<Movement>(&mut app, Some(Vec2::Y.into()));
        run(&mut app, 192);
        let z = xz(&app, player).y;
        assert!(
            z > 1.25 + BODY_RADIUS - 0.05,
            "stopped by the wall, not short of it: z {z}"
        );
        assert!(z < 1.25 + BODY_RADIUS + 0.2, "stopped at the wall: z {z}");
    }

    #[test]
    fn controls_are_off_while_a_screen_is_open() {
        let mut app = app();
        let player = spawn_player(&mut app, 0., 6.);
        app.world_mut().resource_mut::<ScreenMode>().open = Some(stubby_core::Screen::Inventory);
        hold::<Movement>(&mut app, Some(Vec2::Y.into()));
        let start = xz(&app, player);
        run(&mut app, 64);
        assert!(xz(&app, player).distance(start) < 0.05);
    }

    #[test]
    fn tuning_scales_back_the_simulation_speed() {
        let real = tuning(false, 1.);
        let fast = tuning(false, 8.);
        assert_eq!(real.speed, WALK_SPEED);
        assert!((fast.speed * 8. - WALK_SPEED).abs() < 1e-5);
        assert!((fast.gravity * 64. - GRAVITY).abs() < 1e-3);
        assert_eq!(fast.jump_height, real.jump_height);
    }
}
