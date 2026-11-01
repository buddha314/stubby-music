//! Simulation time is separate from sample-timed music.
use bevy::prelude::*;
use stubby_recipes::ResourceTime;

pub const DAY_SECONDS: u64 = 2700;
pub const CYCLE_SECONDS: u64 = 3600;
const SPEEDS: [f32; 8] = [0.25, 0.5, 1., 2., 4., 8., 16., 64.];
#[derive(Resource, Default)]
pub struct Calendar {
    pub(super) elapsed: std::time::Duration,
}
impl Calendar {
    pub fn resource_time(&self) -> ResourceTime {
        let seconds = self.elapsed.as_secs();
        ResourceTime {
            elapsed_millis: self.elapsed.as_millis().min(u64::MAX as u128) as u64,
            nights_started: seconds / CYCLE_SECONDS
                + u64::from(seconds % CYCLE_SECONDS >= DAY_SECONDS),
        }
    }
    pub fn is_night(&self) -> bool {
        self.elapsed.as_secs() % CYCLE_SECONDS >= DAY_SECONDS
    }
}
#[derive(Component)]
struct CalendarHud;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CalendarAdvance;
pub struct CalendarPlugin;
impl Plugin for CalendarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Calendar>()
            .add_systems(Startup, (setup, setup_sun))
            .add_systems(PreUpdate, advance.in_set(CalendarAdvance))
            .add_systems(Update, (speed_controls, hud, move_sun).chain());
    }
}
fn setup(mut commands: Commands) {
    commands.spawn((
        CalendarHud,
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(16.),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            right: px(20),
            top: px(20),
            ..default()
        },
    ));
}
pub(super) fn advance(time: Res<Time<Virtual>>, mut calendar: ResMut<Calendar>) {
    calendar.elapsed = calendar.elapsed.saturating_add(time.delta());
}
pub(super) fn set_speed(time: &mut Time<Virtual>, speed: f32) {
    // Bevy caps the scaled delta, so preserve a quarter-second real catch-up limit.
    time.set_max_delta(std::time::Duration::from_secs_f32(0.25 * speed));
    time.set_relative_speed(speed);
}
fn speed_controls(keys: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    let current = time.relative_speed();
    if keys.just_pressed(KeyCode::BracketRight) {
        set_speed(
            &mut time,
            SPEEDS
                .into_iter()
                .find(|speed| *speed > current)
                .unwrap_or(64.),
        );
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        set_speed(
            &mut time,
            SPEEDS
                .into_iter()
                .rev()
                .find(|speed| *speed < current)
                .unwrap_or(0.25),
        );
    }
    if keys.just_pressed(KeyCode::Backslash) {
        set_speed(&mut time, 1.);
    }
}
fn hud(
    calendar: Res<Calendar>,
    time: Res<Time<Virtual>>,
    mut labels: Query<&mut Text, With<CalendarHud>>,
) {
    for mut label in &mut labels {
        **label = format!(
            "{} | {:.2}x  [ / ] speed | \\ reset",
            if calendar.is_night() { "Night" } else { "Day" },
            time.relative_speed()
        );
    }
}
/// East +X, north -Z. Equinox declination zero at central California latitude.
pub fn sun_direction(elapsed_seconds: f32) -> Vec3 {
    let seconds = elapsed_seconds.rem_euclid(CYCLE_SECONDS as f32);
    let hour_angle = if seconds < DAY_SECONDS as f32 {
        -std::f32::consts::FRAC_PI_2 + seconds / DAY_SECONDS as f32 * std::f32::consts::PI
    } else {
        std::f32::consts::FRAC_PI_2
            + (seconds - DAY_SECONDS as f32) / (CYCLE_SECONDS - DAY_SECONDS) as f32
                * std::f32::consts::PI
    };
    let latitude = 37_f32.to_radians();
    Vec3::new(
        -hour_angle.sin(),
        latitude.cos() * hour_angle.cos(),
        latitude.sin() * hour_angle.cos(),
    )
}
#[derive(Component)]
pub struct Sunlight;
#[derive(Component)]
struct SunDisc;
fn setup_sun(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        SunDisc,
        Mesh3d(meshes.add(Sphere::new(7.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(18000., 10500., 4500.),
            unlit: true,
            fog_enabled: false,
            ..default()
        })),
        bevy::light::NotShadowCaster,
        Transform::default(),
    ));
}
#[allow(clippy::type_complexity)] // Disjoint Bevy transform queries.
fn move_sun(
    calendar: Res<Calendar>,
    mut sun: Query<
        (&mut Transform, &mut DirectionalLight),
        (With<Sunlight>, Without<SunDisc>, Without<Camera3d>),
    >,
    mut discs: Query<&mut Transform, (With<SunDisc>, Without<Sunlight>, Without<Camera3d>)>,
    mut cameras: Query<
        (
            &Transform,
            &mut bevy::light::Skybox,
            &mut GeneratedEnvironmentMapLight,
            Option<&mut DistanceFog>,
        ),
        (With<Camera3d>, Without<SunDisc>, Without<Sunlight>),
    >,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let direction = sun_direction((calendar.elapsed.as_secs_f64() % CYCLE_SECONDS as f64) as f32);
    let daylight = (direction.y / 0.12).clamp(0., 1.);
    let daylight = daylight * daylight * (3. - 2. * daylight);
    for (mut transform, mut light) in &mut sun {
        *transform = Transform::from_translation(direction * 900.).looking_at(Vec3::ZERO, Vec3::Y);
        light.illuminance = 9500. * daylight;
        light.color = Color::srgb(1., 0.71 + 0.22 * daylight, 0.43 + 0.4 * daylight);
    }
    ambient.brightness = 5. + 95. * daylight;
    for (camera, mut sky, mut environment, fog) in &mut cameras {
        if let Some(mut fog) = fog {
            fog.color = Color::srgb(
                0.015 + 0.545 * daylight,
                0.018 + 0.412 * daylight,
                0.03 + 0.3 * daylight,
            );
        }
        sky.brightness = 20. + 1480. * daylight;
        environment.intensity = 20. + 1480. * daylight;
        for mut disc in &mut discs {
            disc.translation = camera.translation + direction * 900.;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_sun_and_sunlight_share_calendar_direction() {
        let mut app = App::new();
        app.insert_resource(Calendar {
            elapsed: std::time::Duration::from_secs(1350),
        })
        .init_resource::<GlobalAmbientLight>()
        .add_systems(Update, move_sun);
        let sun = app
            .world_mut()
            .spawn((Sunlight, DirectionalLight::default(), Transform::default()))
            .id();
        let disc = app.world_mut().spawn((SunDisc, Transform::default())).id();
        let camera = Vec3::new(10., 2., -5.);
        let camera_entity = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Transform::from_translation(camera),
                bevy::light::Skybox::default(),
                GeneratedEnvironmentMapLight::default(),
                DistanceFog::default(),
            ))
            .id();
        app.update();
        let direction = sun_direction(1350.);
        let transform = app.world().get::<Transform>(sun).unwrap();
        assert!(Vec3::from(transform.forward()).distance(-direction) < 0.0001);
        assert!(
            app.world()
                .get::<Transform>(disc)
                .unwrap()
                .translation
                .distance(camera + direction * 900.)
                < 0.001
        );
        assert_eq!(
            app.world()
                .get::<DirectionalLight>(sun)
                .unwrap()
                .illuminance,
            9500.
        );
        app.world_mut().resource_mut::<Calendar>().elapsed = std::time::Duration::from_secs(3150);
        app.update();
        assert_eq!(
            app.world()
                .get::<DirectionalLight>(sun)
                .unwrap()
                .illuminance,
            0.
        );
        assert_eq!(
            app.world().get::<DistanceFog>(camera_entity).unwrap().color,
            Color::srgb(0.015, 0.018, 0.03)
        );
    }
    #[test]
    fn controls_change_virtual_speed_and_reset() {
        let mut app = App::new();
        app.init_resource::<Time<Virtual>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, speed_controls);
        for (key, expected) in [
            (KeyCode::BracketLeft, 0.5),
            (KeyCode::BracketRight, 1.),
            (KeyCode::BracketRight, 2.),
            (KeyCode::Backslash, 1.),
        ] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            assert_eq!(
                app.world().resource::<Time<Virtual>>().relative_speed(),
                expected
            );
        }
    }
    #[test]
    fn scaled_and_paused_time_drive_calendar() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(100),
            ))
            .init_resource::<Calendar>()
            .add_systems(Update, advance);
        app.update();
        set_speed(&mut app.world_mut().resource_mut::<Time<Virtual>>(), 4.);
        app.update();
        assert_eq!(
            app.world()
                .resource::<Calendar>()
                .resource_time()
                .elapsed_millis,
            400
        );
        app.world_mut().resource_mut::<Time<Virtual>>().pause();
        app.update();
        assert_eq!(
            app.world()
                .resource::<Calendar>()
                .resource_time()
                .elapsed_millis,
            400
        );
    }

    #[test]
    fn spring_sun_rises_east_peaks_south_and_sets_west() {
        assert!(sun_direction(0.).distance(Vec3::X) < 0.0001);
        let noon = sun_direction(1350.);
        assert!((noon.y.asin().to_degrees() - 53.).abs() < 0.001);
        assert!(noon.z > 0. && noon.x.abs() < 0.0001);
        assert!(sun_direction(2700.).distance(Vec3::NEG_X) < 0.0001);
        assert!(sun_direction(3150.).y < 0.);
        assert!(sun_direction(3599.99).distance(sun_direction(0.)) < 0.001);
    }
    #[test]
    fn counts_night_boundaries_including_skipped_days() {
        for (seconds, nights, night) in [
            (0, 0, false),
            (2699, 0, false),
            (2700, 1, true),
            (3600, 1, false),
            (6300, 2, true),
            (13500, 4, true),
        ] {
            let calendar = Calendar {
                elapsed: std::time::Duration::from_secs(seconds),
            };
            assert_eq!(calendar.resource_time().nights_started, nights);
            assert_eq!(calendar.is_night(), night);
        }
    }
}
