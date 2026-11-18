//! Simulation time is separate from sample-timed music.
use bevy::prelude::*;

pub const DAY_SECONDS: u64 = 2700;
pub const CYCLE_SECONDS: u64 = 3600;
const SPEEDS: [f32; 8] = [0.25, 0.5, 1., 2., 4., 8., 16., 64.];
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
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
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CalendarAdvance;
pub struct CalendarPlugin;
impl Plugin for CalendarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Calendar>()
            .register_type::<Calendar>()
            .insert_resource(sky_profile().brightness)
            .insert_resource(crate::lighting::LightingKeys::shipped())
            .init_resource::<crate::lighting::CameraLightQuality>()
            .add_systems(PreStartup, open_at_authored_time)
            .add_systems(Startup, setup_sun)
            .add_systems(
                PreUpdate,
                advance
                    .in_set(CalendarAdvance)
                    .after(bevy_stubby_audio::MasterClockRead),
            )
            .add_systems(Update, (speed_controls, move_sun).chain());
    }
}
/// The sky's authored settings (`assets/lighting/sky.json`).
#[derive(serde::Deserialize)]
pub struct SkyProfile {
    pub opening_solar_hour: f32,
    pub ridges: crate::sky::RidgeProfile,
    pub clouds: crate::sky::CloudProfile,
    pub brightness: Brightness,
}
/// Physical light levels (`sky.json` `brightness`): the lighting keys colour the light,
/// these set how bright it is. Lighten the night here.
#[derive(serde::Deserialize, Clone, Copy, Resource)]
pub struct Brightness {
    pub sun_lux: f32,
    pub moon_lux: f32,
    pub day_ev100: f32,
    pub night_ev100: f32,
}
pub fn sky_profile() -> SkyProfile {
    serde_json::from_str(include_str!("../../../assets/lighting/sky.json")).expect("sky profile")
}
/// The game opens at the authored solar hour (06:30, above the dawn ridges), with a frame cap.
/// PreStartup, so reviews that set the clock at Startup win.
fn open_at_authored_time(mut calendar: ResMut<Calendar>, mut time: ResMut<Time<Virtual>>) {
    calendar.elapsed =
        std::time::Duration::from_secs_f32(seconds_at_solar_hour(sky_profile().opening_solar_hour));
    set_speed(&mut time, 1.);
}
/// Calendar seconds (within the first cycle) at a solar hour: the inverse of [`solar_hour`].
pub fn seconds_at_solar_hour(hour: f32) -> f32 {
    let day_hours = (hour - 6.).rem_euclid(24.);
    if day_hours < 12. {
        day_hours / 12. * DAY_SECONDS as f32
    } else {
        DAY_SECONDS as f32 + (day_hours - 12.) / 12. * (CYCLE_SECONDS - DAY_SECONDS) as f32
    }
}
/// The day runs on the 1 BPM master clock: each frame the calendar advances by the
/// master-clock time heard since the last (capped at [`MAX_REAL_STEP`]) times the
/// speed, so at 1x the 2700 s day is exactly 45 master beats and the night 15, and every
/// boundary lands on a beat. Without a running engine (tests, no device) it follows
/// Bevy's virtual time, which `set_speed` scales and caps the same way.
pub fn advance(
    time: Res<Time<Virtual>>,
    master: Option<Res<bevy_stubby_audio::MasterClock>>,
    mut last: Local<Option<f64>>,
    mut calendar: ResMut<Calendar>,
) {
    let step = match master.and_then(|master| master.beats()) {
        Some(beats) => {
            let heard = last.map_or(0., |last| {
                bevy_stubby_audio::MasterClock::seconds(beats - last).max(0.)
            });
            *last = Some(beats);
            let speed = if time.is_paused() {
                0.
            } else {
                time.relative_speed_f64()
            };
            std::time::Duration::from_secs_f64(heard.min(MAX_REAL_STEP as f64) * speed)
        }
        None => {
            *last = None;
            time.delta()
        }
    };
    calendar.elapsed = calendar.elapsed.saturating_add(step);
}
/// A frame advances the calendar by at most this much real time times the speed, so a
/// hitch never skips a stretch of day (or a night boundary).
const MAX_REAL_STEP: f32 = 0.1;
pub fn set_speed(time: &mut Time<Virtual>, speed: f32) {
    time.set_max_delta(std::time::Duration::from_secs_f32(MAX_REAL_STEP * speed));
    time.set_relative_speed(speed);
}
/// Speed keys, ignored while a screen (inventory, a rack) owns the keyboard.
fn speed_controls(
    keys: Res<ButtonInput<KeyCode>>,
    screens: Option<Res<stubby_core::ScreenMode>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if screens.is_some_and(|screens| screens.open.is_some()) {
        return;
    }
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
/// The moon's disc brightness (a fraction of a sun of its illuminance): pale by day,
/// bright by night.
const MOON_DISC_DAY: f32 = 0.004;
const MOON_DISC_NIGHT: f32 = 0.03;
/// How much of the keys' fog colour is lit: 0.15 at night (navy-black haze), rising
/// with the sun to 1 once it is high, so a low sun leaves the far haze dim.
fn fog_light(sun: Vec3) -> f32 {
    let t = (sun.y / 0.45).clamp(0., 1.);
    0.15 + 0.85 * t * t * (3. - 2. * t)
}
/// 0 below the horizon, 1 once a body is about 7 degrees up.
fn above_horizon(direction: Vec3) -> f32 {
    let t = (direction.y / 0.12).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
#[allow(clippy::type_complexity)] // Disjoint Bevy light queries.
fn move_sun(
    calendar: Res<Calendar>,
    brightness: Res<Brightness>,
    keys: Res<crate::lighting::LightingKeys>,
    quality: Res<crate::lighting::CameraLightQuality>,
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
    // The art keys colour the light and set the fog; `brightness` sets the levels.
    let key = keys.sample(direction, quality.0);
    let night = keys.sample(Vec3::NEG_Y, quality.0);
    let srgb = |[r, g, b]: [f32; 3]| Color::srgb(r, g, b);
    for (mut transform, mut light) in &mut sun {
        *transform = Transform::from_translation(direction * 900.).looking_at(Vec3::ZERO, Vec3::Y);
        light.illuminance = brightness.sun_lux * daylight;
        light.color = srgb(key.sun_color);
    }
    let moon_direction = moon_direction(elapsed);
    for (mut transform, mut light, disc) in &mut moon {
        *transform =
            Transform::from_translation(moon_direction * 900.).looking_at(Vec3::ZERO, Vec3::Y);
        light.illuminance = brightness.moon_lux * above_horizon(moon_direction);
        // The night key's main light is the moon.
        light.color = srgb(night.sun_color);
        if let Some(mut disc) = disc {
            disc.intensity = MOON_DISC_DAY.lerp(MOON_DISC_NIGHT, 1. - daylight);
        }
    }
    // The sky's own light comes from the atmosphere's environment map; this is a floor.
    ambient.brightness = 5. + 95. * daylight;
    ambient.color = srgb(key.ambient_color);
    for (fog, exposure) in &mut cameras {
        if let Some(mut fog) = fog {
            // The keys give the fog's hue and density; its light follows the sun. Away from a
            // low sun the haze is dim, toward it the haze glows in the sun's colour.
            let lit = fog_light(direction);
            fog.color = srgb(key.fog_color.map(|c| c * lit));
            fog.directional_light_color = srgb(key.sun_color.map(|c| c * 0.6 * daylight));
            fog.directional_light_exponent = 10.;
            fog.falloff = FogFalloff::Exponential {
                density: key.fog_density,
            };
        }
        // The eye adapts a little at night (`brightness`), so the night stays navy.
        if let Some(mut exposure) = exposure {
            exposure.ev100 = brightness.night_ev100.lerp(brightness.day_ev100, daylight);
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
        .insert_resource(sky_profile().brightness)
        .insert_resource(crate::lighting::LightingKeys::shipped())
        .init_resource::<crate::lighting::CameraLightQuality>()
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
            {
                let [r, g, b] = crate::lighting::LightingKeys::shipped()
                    .sample(sun_direction(3150.), 0.)
                    .fog_color;
                Color::srgb(r * 0.15, g * 0.15, b * 0.15)
            }
        );
        // Midnight: the moon is up and lights the desert.
        assert_eq!(
            app.world()
                .get::<DirectionalLight>(moon)
                .unwrap()
                .illuminance,
            sky_profile().brightness.moon_lux
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
    fn opening_sun_clears_ridges_slightly_right_of_view() {
        let profile = sky_profile();
        let sun = sun_direction(seconds_at_solar_hour(profile.opening_solar_hour));
        let bearing = sun.x.atan2(-sun.z).to_degrees().rem_euclid(360.);
        let offset = bearing - stubby_core::OPENING_HEADING_DEGREES;
        assert!(
            (10. ..25.).contains(&offset),
            "opening sun offset: {offset}"
        );
        let highest_ridge = profile
            .ridges
            .base_deg
            .iter()
            .zip(&profile.ridges.amp_deg)
            .map(|(base, amp)| base + amp)
            .fold(0., f32::max);
        assert!(
            sun.y.asin().to_degrees() > highest_ridge,
            "sun must clear the authored skyline"
        );
    }

    #[test]
    fn opening_hour_maps_back_to_calendar_seconds() {
        for hour in [0., 3.5, 6., 12., 17.9, 18., 21.] {
            assert!((solar_hour(seconds_at_solar_hour(hour)) - hour).abs() < 1e-3);
        }
        assert_eq!(seconds_at_solar_hour(6.), 0.);
        assert_eq!(seconds_at_solar_hour(0.), 3150.);
    }
    #[test]
    fn screens_own_the_speed_keys_and_hitches_are_capped() {
        let mut app = App::new();
        app.init_resource::<Time<Virtual>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(stubby_core::ScreenMode {
                open: Some(stubby_core::Screen::Inventory),
                closed_this_frame: false,
            })
            .add_systems(Update, speed_controls);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::BracketRight);
        app.update();
        assert_eq!(app.world().resource::<Time<Virtual>>().relative_speed(), 1.);
        // At 64x a two-second hitch advances the calendar at most 6.4 s.
        let mut time = Time::<Virtual>::default();
        set_speed(&mut time, 64.);
        assert!((time.max_delta().as_secs_f32() - 6.4).abs() < 1e-4);
    }
    #[test]
    fn the_day_runs_on_the_master_clock_times_speed_and_capped() {
        let mut app = App::new();
        app.init_resource::<Time<Virtual>>()
            .init_resource::<Calendar>()
            .insert_resource(bevy_stubby_audio::MasterClock::at_beats(10.))
            .add_systems(Update, advance);
        app.update();
        assert_eq!(app.world().resource::<Calendar>().elapsed.as_secs_f64(), 0.);
        // One frame hears 1/600 beat (0.1 s): the calendar moves 0.1 s at 1x.
        let set = |app: &mut App, beats: f64| {
            app.insert_resource(bevy_stubby_audio::MasterClock::at_beats(beats));
        };
        set(&mut app, 10. + 1. / 600.);
        app.update();
        let elapsed = |app: &App| app.world().resource::<Calendar>().elapsed.as_secs_f64();
        assert!((elapsed(&app) - 0.1).abs() < 1e-6);
        // At 64x, a two-second hitch on the clock advances at most 6.4 s.
        set_speed(&mut app.world_mut().resource_mut::<Time<Virtual>>(), 64.);
        set(&mut app, 10. + 1. / 600. + 2. / 60.);
        app.update();
        assert!((elapsed(&app) - 6.5).abs() < 1e-6);
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
