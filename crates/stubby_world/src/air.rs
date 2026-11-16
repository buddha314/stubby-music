//! Clean air from foliage (clean-air-by-foliage). Each placed [`AirCleaner`] cleans
//! the air within its radius as strongly as its life allows; light quality at any point
//! is computed from the cleaners around it, with nothing stored per place. The camera's
//! light quality blends the lighting keys; each placed asset's drives its bloom; the
//! basin's average drives the windstorm (for now, the dust in the sky).
use bevy::prelude::*;
use stubby_core::{AirCleaner, AirFollows, AssetState, PlacedAsset};

/// Seconds (of simulation time) for a cleaner to follow most of a change in its life.
const EASE_SECONDS: f32 = 6.;
/// Light quality is resampled this often (real seconds) at assets and the camera.
const SAMPLE_SECONDS: f32 = 0.1;
/// The basin inside the mountain rings, sampled for the windstorm.
pub const BASIN_RADIUS_M: f32 = 450.;

/// How clean the basin's air is on average, 0..1. The windstorm follows it.
#[derive(Resource, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Resource)]
pub struct BasinAir {
    pub clean: f32,
}
impl BasinAir {
    /// The windstorm's strength: full in a fouled basin, gone in a clean one.
    pub fn storm(&self) -> f32 {
        1. - self.clean
    }
}

pub struct AirPlugin;
impl Plugin for AirPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BasinAir>()
            .register_type::<BasinAir>()
            .register_type::<crate::lighting::CameraLightQuality>()
            .add_systems(
                Update,
                (ease_cleaners, sample_light_quality, sample_basin)
                    .chain()
                    .after(stubby_core::PrefabBinding),
            );
    }
}

/// How strongly a cleaner should clean, given its asset's life.
pub fn target_strength(cleaner: &AirCleaner, state: &AssetState, harvested: bool) -> f32 {
    if cleaner.stops_when_harvested && harvested {
        return 0.;
    }
    let life = match cleaner.follows {
        AirFollows::Growth => state.growth,
        AirFollows::WaterOrLight => state.water_quality.max(state.light_quality),
        AirFollows::Light => state.light_quality,
    };
    cleaner.max_strength * life.clamp(0., 1.)
}

/// Light quality at a point: `1 - Π(1 - strengthᵢ · falloff(dᵢ / radiusᵢ))` over the
/// cleaners, by horizontal distance (a column of air), with a smoothstep falloff. Groves
/// add up and never pass 1; beyond every radius it is 0.
pub fn light_quality_at<'a>(
    point: Vec3,
    cleaners: impl IntoIterator<Item = (Vec3, &'a AirCleaner)>,
) -> f32 {
    let mut fouled = 1.;
    for (at, cleaner) in cleaners {
        let distance = point.xz().distance(at.xz());
        if distance >= cleaner.radius_m || cleaner.strength <= 0. {
            continue;
        }
        let t = 1. - distance / cleaner.radius_m;
        fouled *= 1. - cleaner.strength.clamp(0., 1.) * t * t * (3. - 2. * t);
    }
    1. - fouled
}

fn ease_cleaners(
    time: Res<Time<Virtual>>,
    mut cleaners: Query<(&mut AirCleaner, &AssetState, Has<stubby_core::Harvested>)>,
) {
    let follow = 1. - (-time.delta_secs() / EASE_SECONDS).exp();
    for (mut cleaner, state, harvested) in &mut cleaners {
        let target = target_strength(&cleaner, state, harvested);
        let next = cleaner.strength + (target - cleaner.strength) * follow;
        if (next - cleaner.strength).abs() > 1e-5 {
            cleaner.strength = next;
        }
    }
}

#[allow(clippy::type_complexity)] // Placed assets, cleaners and the world camera.
fn sample_light_quality(
    time: Res<Time<Real>>,
    mut since: Local<f32>,
    cleaners: Query<(Entity, &GlobalTransform, &AirCleaner)>,
    mut assets: Query<(Entity, &GlobalTransform, &mut AssetState), With<PlacedAsset>>,
    cameras: Query<&GlobalTransform, (With<Camera3d>, With<bevy::pbr::AtmosphereSettings>)>,
    mut camera_quality: ResMut<crate::lighting::CameraLightQuality>,
) {
    *since += time.delta_secs();
    if *since < SAMPLE_SECONDS {
        return;
    }
    *since = 0.;
    let all: Vec<(Entity, Vec3, &AirCleaner)> = cleaners
        .iter()
        .map(|(entity, at, cleaner)| (entity, at.translation(), cleaner))
        .collect();
    for (entity, at, mut state) in &mut assets {
        // Not its own cleaning: a bush cannot keep itself bloomed.
        let others = all
            .iter()
            .filter(|(other, ..)| *other != entity)
            .map(|(_, at, cleaner)| (*at, *cleaner));
        let quality = light_quality_at(at.translation(), others);
        if (quality - state.light_quality).abs() > 1e-4 {
            state.light_quality = quality;
        }
    }
    if let Ok(camera) = cameras.single() {
        let quality =
            light_quality_at(camera.translation(), all.iter().map(|(_, at, c)| (*at, *c)));
        camera_quality.0 = quality;
    }
}

/// The basin's average light quality, over a grid inside the mountain rings, once a
/// second (real time).
fn sample_basin(
    time: Res<Time<Real>>,
    mut since: Local<Option<f32>>,
    cleaners: Query<(&GlobalTransform, &AirCleaner)>,
    mut basin: ResMut<BasinAir>,
) {
    let elapsed = since.get_or_insert(1.);
    *elapsed += time.delta_secs();
    if *elapsed < 1. {
        return;
    }
    *elapsed = 0.;
    let all: Vec<(Vec3, &AirCleaner)> = cleaners
        .iter()
        .map(|(at, cleaner)| (at.translation(), cleaner))
        .collect();
    let clean = basin_clean(&all);
    if (clean - basin.clean).abs() > 1e-4 {
        basin.clean = clean;
    }
}

/// The basin's mean light quality: the clean air integrated over each cleaner's own
/// disc (8 x 8 samples, so even one small plant counts and nothing jumps as plants move),
/// each sample shared among the discs that cover it so groves are not counted twice,
/// divided by the basin's area.
pub fn basin_clean(cleaners: &[(Vec3, &AirCleaner)]) -> f32 {
    const SAMPLES: i32 = 8;
    let mut cleaned = 0.;
    for (centre, cleaner) in cleaners {
        let r = cleaner.radius_m;
        let step = 2. * r / SAMPLES as f32;
        for i in 0..SAMPLES {
            for j in 0..SAMPLES {
                let offset = Vec2::new(-r + (i as f32 + 0.5) * step, -r + (j as f32 + 0.5) * step);
                let point = centre.xz() + offset;
                if offset.length() >= r || point.length() > BASIN_RADIUS_M {
                    continue;
                }
                let covers = cleaners
                    .iter()
                    .filter(|(at, other)| at.xz().distance(point) < other.radius_m)
                    .count()
                    .max(1);
                let quality =
                    light_quality_at(Vec3::new(point.x, 0., point.y), cleaners.iter().copied());
                cleaned += quality * step * step / covers as f32;
            }
        }
    }
    (cleaned / (std::f32::consts::PI * BASIN_RADIUS_M * BASIN_RADIUS_M)).clamp(0., 1.)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tree(strength: f32) -> AirCleaner {
        AirCleaner {
            radius_m: 18.,
            max_strength: 0.9,
            follows: AirFollows::Growth,
            stops_when_harvested: false,
            strength,
        }
    }
    #[test]
    fn a_tree_clears_its_ground_only() {
        let oak = tree(0.9);
        let at = Vec3::ZERO;
        assert!((light_quality_at(at, [(at, &oak)]) - 0.9).abs() < 1e-5);
        assert_eq!(light_quality_at(Vec3::new(50., 0., 0.), [(at, &oak)]), 0.);
        // Height does not matter: a column of air.
        assert!((light_quality_at(Vec3::Y * 10., [(at, &oak)]) - 0.9).abs() < 1e-5);
    }
    #[test]
    fn groves_add_up_but_never_pass_one() {
        let oak = tree(0.9);
        let one = light_quality_at(Vec3::new(5., 0., 0.), [(Vec3::ZERO, &oak)]);
        let grove: Vec<_> = (0..6)
            .map(|i| (Vec3::new(i as f32 * 2., 0., 0.), &oak))
            .collect();
        let many = light_quality_at(Vec3::new(5., 0., 0.), grove.iter().copied());
        assert!(many > one && many <= 1.);
    }
    #[test]
    fn cleaning_follows_life_and_harvest() {
        let mut reed = tree(0.);
        reed.follows = AirFollows::WaterOrLight;
        reed.stops_when_harvested = true;
        let wet = AssetState {
            water_quality: 1.,
            ..default()
        };
        assert!((target_strength(&reed, &wet, false) - 0.9).abs() < 1e-5);
        assert_eq!(target_strength(&reed, &wet, true), 0.);
        assert_eq!(
            target_strength(&tree(0.), &AssetState::default(), false),
            0.
        );
    }
    #[test]
    fn a_dying_grove_fouls_the_air() {
        let mut app = App::new();
        app.init_resource::<Time<Virtual>>()
            .add_systems(Update, ease_cleaners);
        let oak = app
            .world_mut()
            .spawn((
                tree(0.9),
                AssetState::default(), // growth 0: dormant or dead
            ))
            .id();
        for _ in 0..200 {
            app.world_mut()
                .resource_mut::<Time<Virtual>>()
                .advance_by(std::time::Duration::from_millis(100));
            app.update();
        }
        assert!(app.world().get::<AirCleaner>(oak).unwrap().strength < 0.05);
    }
    #[test]
    fn the_basin_is_bleak_until_foliage_fills_it() {
        assert_eq!(basin_clean(&[]), 0.);
        let oak = tree(0.9);
        let one_tree = basin_clean(&[(Vec3::ZERO, &oak)]);
        assert!(one_tree > 0. && one_tree < 0.01, "{one_tree}");
        // Overlapping trees are not counted twice: the disc samples agree with a fine
        // brute-force integral over the same ground.
        let pair = [(Vec3::ZERO, &oak), (Vec3::new(10., 0., 0.), &oak)];
        let mut brute = 0.;
        for i in -100..200 {
            for j in -100..100 {
                let point = Vec3::new(i as f32 * 0.25, 0., j as f32 * 0.25);
                brute += light_quality_at(point, pair) * 0.0625;
            }
        }
        let brute = brute / (std::f32::consts::PI * BASIN_RADIUS_M * BASIN_RADIUS_M);
        let sampled = basin_clean(&pair);
        assert!(
            (sampled - brute).abs() < 0.05 * brute,
            "{sampled} vs {brute}"
        );
        // A basin packed with blooming trees is clean.
        let forest: Vec<_> = (-30..=30)
            .flat_map(|i| (-30..=30).map(move |j| Vec3::new(i as f32 * 15., 0., j as f32 * 15.)))
            .collect();
        let packed: Vec<_> = forest.iter().map(|at| (*at, &oak)).collect();
        assert!(basin_clean(&packed) > 0.85);
    }
}
