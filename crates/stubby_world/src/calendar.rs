//! Simulation time is separate from sample-timed music.
use bevy::prelude::*;

pub const DAY_SECONDS: u64 = 2700;
pub const CYCLE_SECONDS: u64 = 3600;
const SPEEDS: [f32; 8] = [0.25, 0.5, 1., 2., 4., 8., 16., 64.];
#[derive(Resource, Default)]
pub struct Calendar {
    /// Simulation time since the world began; reviews set it to jump the clock.
    pub elapsed: std::time::Duration,
}
impl Calendar {
    /// Simulation time since the world began.
    pub fn elapsed(&self) -> std::time::Duration {
        self.elapsed
    }
    /// How many nights have begun.
    pub fn nights_started(&self) -> u64 {
        let seconds = self.elapsed.as_secs();
        seconds / CYCLE_SECONDS + u64::from(seconds % CYCLE_SECONDS >= DAY_SECONDS)
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
pub fn advance(time: Res<Time<Virtual>>, mut calendar: ResMut<Calendar>) {
    calendar.elapsed = calendar.elapsed.saturating_add(time.delta());
}
pub fn set_speed(time: &mut Time<Virtual>, speed: f32) {
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
/// Central California, on the equinox.
const LATITUDE: f32 = 37. * std::f32::consts::PI / 180.;
/// The moon trails the sun by this hour angle (a waning gibbous): it rises after dusk
/// and is still setting in the west when the game opens at sunrise.
const MOON_LAG: f32 = 195. * std::f32::consts::PI / 180.;
const MOON_DECLINATION: f32 = 8. * std::f32::consts::PI / 180.;
/// The sun's hour angle in radians (0 at solar noon, positive west). The one solar-time
/// mapping: the 2700 s day covers 06:00-18:00 and the 900 s night 18:00-06:00.
pub fn hour_angle(elapsed_seconds: f32) -> f32 {
    let seconds = elapsed_seconds.rem_euclid(CYCLE_SECONDS as f32);
    if seconds < DAY_SECONDS as f32 {
        -std::f32::consts::FRAC_PI_2 + seconds / DAY_SECONDS as f32 * std::f32::consts::PI
    } else {
        std::f32::consts::FRAC_PI_2
            + (seconds - DAY_SECONDS as f32) / (CYCLE_SECONDS - DAY_SECONDS) as f32
                * std::f32::consts::PI
    }
}
/// Solar time of day in hours, 0..24.
pub fn solar_hour(elapsed_seconds: f32) -> f32 {
    (12. + hour_angle(elapsed_seconds).to_degrees() / 15.).rem_euclid(24.)
}
/// World direction (east +X, up +Y, north -Z) of a point on the sky at a declination and
/// hour angle (radians).
pub fn horizontal(declination: f32, hour_angle: f32) -> Vec3 {
    let up =
        LATITUDE.sin() * declination.sin() + LATITUDE.cos() * declination.cos() * hour_angle.cos();
    let north =
        LATITUDE.cos() * declination.sin() - LATITUDE.sin() * declination.cos() * hour_angle.cos();
    Vec3::new(-declination.cos() * hour_angle.sin(), up, -north)
}
pub fn sun_direction(elapsed_seconds: f32) -> Vec3 {
    horizontal(0., hour_angle(elapsed_seconds))
}
/// The moon is fixed among the stars, so it turns with them.
pub fn moon_direction(elapsed_seconds: f32) -> Vec3 {
    horizontal(MOON_DECLINATION, hour_angle(elapsed_seconds) - MOON_LAG)
}
/// How the star sphere is turned: about the celestial pole, by the sun's hour angle, so
/// a star fixed at `horizontal(d, h)` at solar noon stands at `horizontal(d, h + H)`.
pub fn sky_rotation(elapsed_seconds: f32) -> Quat {
    let pole = Vec3::new(0., LATITUDE.sin(), -LATITUDE.cos());
    Quat::from_axis_angle(pole, -hour_angle(elapsed_seconds))
}
#[derive(Component)]
pub struct Sunlight;
/// The moon's light: cool, dim, no shadows. The atmosphere draws its disc and scatters
/// its light, so a moonlit sky is faintly blue.
#[derive(Component)]
pub struct Moonlight;
/// Moonlight at full height, in lux: enough to read silhouettes, still night.
const MOON_LUX: f32 = 100.;
fn setup_sun(mut commands: Commands) {
    commands.spawn((
        Moonlight,
        DirectionalLight {
            // The Godot sky's moonlight colour.
            color: Color::srgb(0.57, 0.78, 0.96),
            illuminance: 0.,
            shadow_maps_enabled: false,
            ..default()
        },
        // Stylised: about four times the real moon, as the Godot sky drew it.
        bevy::light::SunDisk {
            angular_size: 0.035,
            intensity: MOON_DISC_DAY,
        },
        Transform::default(),
    ));
}
const DAY_EV100: f32 = 10.5;
const NIGHT_EV100: f32 = 8.5;
/// The moon's disc brightness (a fraction of a sun of its illuminance): pale by day,
/// bright by night.
const MOON_DISC_DAY: f32 = 0.004;
const MOON_DISC_NIGHT: f32 = 0.03;
/// 0 below the horizon, 1 once a body is about 7 degrees up.
fn above_horizon(direction: Vec3) -> f32 {
    let t = (direction.y / 0.12).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
#[allow(clippy::type_complexity)] // Disjoint Bevy light queries.
fn move_sun(
    calendar: Res<Calendar>,
    mut sun: Query<(&mut Transform, &mut DirectionalLight), (With<Sunlight>, Without<Moonlight>)>,
    mut moon: Query<
        (
            &mut Transform,
            &mut DirectionalLight,
            Option<&mut bevy::light::SunDisk>,
        ),
        (With<Moonlight>, Without<Sunlight>),
    >,
    mut cameras: Query<
        (
            Option<&mut DistanceFog>,
            Option<&mut bevy::camera::Exposure>,
        ),
        With<bevy::pbr::AtmosphereSettings>,
    >,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let elapsed = (calendar.elapsed.as_secs_f64() % CYCLE_SECONDS as f64) as f32;
    let direction = sun_direction(elapsed);
    let daylight = above_horizon(direction);
    for (mut transform, mut light) in &mut sun {
        *transform = Transform::from_translation(direction * 900.).looking_at(Vec3::ZERO, Vec3::Y);
        light.illuminance = 9500. * daylight;
        light.color = Color::srgb(1., 0.71 + 0.22 * daylight, 0.43 + 0.4 * daylight);
    }
    let moon_direction = moon_direction(elapsed);
    for (mut transform, mut light, disc) in &mut moon {
        *transform =
            Transform::from_translation(moon_direction * 900.).looking_at(Vec3::ZERO, Vec3::Y);
        light.illuminance = MOON_LUX * above_horizon(moon_direction);
        if let Some(mut disc) = disc {
            disc.intensity = MOON_DISC_DAY.lerp(MOON_DISC_NIGHT, 1. - daylight);
        }
    }
    // The sky's own light now comes from the atmosphere's environment map; this is a floor.
    ambient.brightness = 5. + 95. * daylight;
    for (fog, exposure) in &mut cameras {
        if let Some(mut fog) = fog {
            fog.color = Color::srgb(
                0.015 + 0.545 * daylight,
                0.018 + 0.412 * daylight,
                0.03 + 0.3 * daylight,
            );
        }
        // The eye adapts a little at night: two stops, so moonlit ground reads and the
        // sky is deep navy rather than black (the art keys go further, to ev -2).
        if let Some(mut exposure) = exposure {
            exposure.ev100 = NIGHT_EV100.lerp(DAY_EV100, daylight);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sun_and_moon_lights_follow_calendar_directions() {
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
        let moon = app
            .world_mut()
            .spawn((Moonlight, DirectionalLight::default(), Transform::default()))
            .id();
        let camera_entity = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                bevy::pbr::AtmosphereSettings::default(),
                DistanceFog::default(),
                bevy::camera::Exposure::default(),
            ))
            .id();
        app.update();
        let direction = sun_direction(1350.);
        let transform = app.world().get::<Transform>(sun).unwrap();
        assert!(Vec3::from(transform.forward()).distance(-direction) < 0.0001);
        let transform = app.world().get::<Transform>(moon).unwrap();
        assert!(Vec3::from(transform.forward()).distance(-moon_direction(1350.)) < 0.0001);
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
        // Midnight: the moon is up and lights the desert.
        assert_eq!(
            app.world()
                .get::<DirectionalLight>(moon)
                .unwrap()
                .illuminance,
            MOON_LUX
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
    fn moon_and_stars_turn_together_with_solar_time() {
        // The sky rotation carries a fixed star to where the hour angle puts it.
        for seconds in [0., 1350., 2900., 3400.] {
            let turned = sky_rotation(seconds) * horizontal(0.4, 1.1);
            let expected = horizontal(0.4, 1.1 + hour_angle(seconds));
            assert!(
                turned.distance(expected) < 1e-4,
                "{seconds}: {turned} vs {expected}"
            );
        }
        // Night runs 15 degrees of hour angle per solar hour: 225 s is three hours.
        assert!((solar_hour(DAY_SECONDS as f32 + 225.) - 21.).abs() < 1e-3);
        assert!(moon_direction(DAY_SECONDS as f32).angle_between(moon_direction(2925.)) > 0.35);
        // The game opens at sunrise with the moon setting, low in the west.
        let opening = moon_direction(0.);
        assert!(
            opening.x < -0.5 && opening.y > 0.1 && opening.y < 0.4,
            "{opening}"
        );
        // It is up through the middle of the night.
        assert!(moon_direction(DAY_SECONDS as f32 + 450.).y > 0.5);
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
            assert_eq!(calendar.nights_started(), nights);
            assert_eq!(calendar.is_night(), night);
        }
    }
}
