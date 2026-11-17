use bevy::prelude::*;

/// The components the world camera needs for the sky (`sync-calendar-and-night-sky`):
/// Bevy's atmosphere, drawn from the sun and moon lights, and its light as the
/// environment map. The map is filtered only when the sky has visibly changed
/// ([`SkyPlugin`]), not every frame.
pub fn camera_sky() -> impl Bundle {
    (
        // The sky is blended over the cleared background by its transmittance, so the
        // clear colour shows through thin air: it must be black, or night stays grey.
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        // The world ends at the mountains (~700 m): spend the aerial LUT's slices there.
        bevy::pbr::AtmosphereSettings {
            aerial_view_lut_max_distance: 2000.,
            ..default()
        },
        bevy::light::AtmosphereEnvironmentMapLight {
            size: UVec2::splat(128),
            ..default()
        },
    )
}

/// Desert dust in the bleak opening (per metre): scatters every colour alike and absorbs
/// blue, as iron-oxide dust does, so the clear sky hazes to brown-grey.
const DUST_SCATTERING: f32 = 14e-6;
const DUST_ABSORPTION: Vec3 = Vec3::new(0.5e-6, 1.2e-6, 3.2e-6);

/// Earth's air with a low layer of desert dust, `storm` 0..1 of it: the windstorm's
/// dust thins as the basin's air is cleaned (`air::BasinAir`), down to a light haze.
pub fn desert_air(storm: f32) -> bevy::light::atmosphere::ScatteringMedium {
    use bevy::light::atmosphere::{Falloff, PhaseFunction, ScatteringMedium, ScatteringTerm};
    let dust = 0.15 + 0.85 * storm.clamp(0., 1.);
    let mut air = ScatteringMedium::earth(256, 256);
    air.terms.push(ScatteringTerm {
        absorption: DUST_ABSORPTION * dust,
        scattering: Vec3::splat(DUST_SCATTERING * dust),
        // Thickness is a fraction of the 100 km atmosphere: ~3 km of dust.
        falloff: Falloff::Exponential { scale: 3. / 100. },
        phase: PhaseFunction::Mie { asymmetry: 0.7 },
    });
    air.with_label("desert_air")
}

/// The sky's air, and the storm it was last built for.
#[derive(Resource)]
pub struct DesertAirMedium {
    handle: Handle<bevy::light::atmosphere::ScatteringMedium>,
    storm: f32,
}

/// The planet the atmosphere belongs to: Earth, under desert air (full storm at first).
pub fn spawn_atmosphere(
    commands: &mut Commands,
    mediums: &mut Assets<bevy::light::atmosphere::ScatteringMedium>,
) {
    let medium = mediums.add(desert_air(1.));
    commands.insert_resource(DesertAirMedium {
        handle: medium.clone(),
        storm: 1.,
    });
    let mut atmosphere = bevy::light::Atmosphere::earth(medium);
    // Dry, pale mud: brighter multiscattering than Earth's average.
    atmosphere.ground_albedo = Vec3::new(0.42, 0.36, 0.30);
    commands.spawn(atmosphere);
}

/// The windstorm follows the basin: rebuild the sky's air when the storm has moved 2%
/// (each rebuild recomputes the atmosphere's medium tables, so not every frame).
fn follow_the_storm(
    basin: Res<crate::air::BasinAir>,
    medium: Option<ResMut<DesertAirMedium>>,
    mut mediums: ResMut<Assets<bevy::light::atmosphere::ScatteringMedium>>,
) {
    let Some(mut medium) = medium else { return };
    let storm = basin.storm();
    if (storm - medium.storm).abs() < 0.02 {
        return;
    }
    if let Some(mut air) = mediums.get_mut(&medium.handle) {
        *air = desert_air(storm);
        medium.storm = storm;
    }
}

/// Stars: a seeded sphere of points that turns about the celestial pole with solar time
/// (`calendar::sky_rotation`) and fades in as the sun goes down.
#[derive(Component)]
pub struct StarField(Handle<StandardMaterial>);
const STAR_RADIUS: f32 = 900.;
const STARS: i32 = 1800;

fn spawn_stars(
    mut commands: Commands,
    seed: Res<crate::WorldSeed>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use crate::terrain::hash;
    let (mut positions, mut colors, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..STARS {
        let z = 2. * hash(i, 1, seed.0 ^ 0x57a5) - 1.;
        let phi = std::f32::consts::TAU * hash(i, 2, seed.0 ^ 0x57a5);
        let r = (1. - z * z).sqrt();
        let d = Vec3::new(r * phi.cos(), z, r * phi.sin());
        // Few bright stars, many faint ones.
        let magnitude = hash(i, 3, seed.0 ^ 0x57a5).powi(6);
        let size = STAR_RADIUS * (0.0005 + 0.0014 * magnitude);
        let warmth = hash(i, 4, seed.0 ^ 0x57a5);
        let tint = Vec3::new(0.85, 0.9, 1.).lerp(Vec3::new(1., 0.86, 0.7), warmth);
        let brightness = 0.5 + 2.5 * magnitude;
        let (u, v) = d.any_orthonormal_pair();
        let base = positions.len() as u32;
        for (a, b) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] {
            positions.push((d * STAR_RADIUS + (u * a + v * b) * size).to_array());
            colors.push([
                tint.x * brightness,
                tint.y * brightness,
                tint.z * brightness,
                1.,
            ]);
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices));
    let material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        fog_enabled: false,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        StarField(material.clone()),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
        bevy::camera::visibility::NoFrustumCulling,
        Transform::default(),
        Visibility::Hidden,
    ));
}

/// Night factor for the stars: 0 by day, 1 once the sun is 8 degrees down.
pub fn star_visibility(sun: Vec3) -> f32 {
    let t = ((0.05 - sun.y) / 0.19).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

#[allow(clippy::type_complexity)] // Disjoint camera and star transforms.
fn turn_stars(
    calendar: Res<crate::calendar::Calendar>,
    mut stars: Query<(&StarField, &mut Transform, &mut Visibility), Without<Camera3d>>,
    cameras: Query<&Transform, (With<Camera3d>, With<bevy::pbr::AtmosphereSettings>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shown: Local<f32>,
) {
    let elapsed = calendar.elapsed.as_secs_f32();
    let night = star_visibility(crate::calendar::sun_direction(elapsed));
    let Ok(camera) = cameras.single() else { return };
    for (field, mut transform, mut visibility) in &mut stars {
        transform.translation = camera.translation;
        transform.rotation = crate::calendar::sky_rotation(elapsed);
        *visibility = if night > 0. {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if (night - *shown).abs() > 0.01 || (night == 0.) != (*shown == 0.) {
            *shown = night;
            if let Some(mut material) = materials.get_mut(&field.0) {
                material.base_color = Color::linear_rgb(night, night, night);
            }
        }
    }
}

/// The cloud layer (`assets/lighting/sky.json`, `clouds`).
#[derive(serde::Deserialize, Clone)]
pub struct CloudProfile {
    pub coverage: f32,
    pub softness: f32,
    pub repeats: f32,
    pub drift_per_hour: [f32; 2],
    pub opacity: f32,
    pub day_color: [f32; 3],
    pub dusk_color: [f32; 3],
    pub night_color: [f32; 3],
}
#[derive(Component)]
pub struct CloudLayer(Handle<StandardMaterial>);
const CLOUD_RADIUS: f32 = 850.;
const CLOUD_TEXTURE: usize = 256;

/// Seeded, tileable cloud cover: periodic value-noise octaves, thresholded at the
/// authored coverage into soft-edged alpha.
pub fn cloud_alpha(seed: u64, profile: &CloudProfile) -> Vec<u8> {
    use crate::terrain::hash;
    let noise = |x: f32, y: f32, period: i32, salt: u64| {
        let (cx, cy) = (x.floor() as i32, y.floor() as i32);
        let (tx, ty) = (x - cx as f32, y - cy as f32);
        let (tx, ty) = (tx * tx * (3. - 2. * tx), ty * ty * (3. - 2. * ty));
        let at = |i: i32, j: i32| hash(i.rem_euclid(period), j.rem_euclid(period), seed ^ salt);
        let top = at(cx, cy) + (at(cx + 1, cy) - at(cx, cy)) * tx;
        let bottom = at(cx, cy + 1) + (at(cx + 1, cy + 1) - at(cx, cy + 1)) * tx;
        top + (bottom - top) * ty
    };
    let mut alpha = Vec::with_capacity(CLOUD_TEXTURE * CLOUD_TEXTURE);
    for y in 0..CLOUD_TEXTURE {
        for x in 0..CLOUD_TEXTURE {
            let (u, v) = (
                x as f32 / CLOUD_TEXTURE as f32,
                y as f32 / CLOUD_TEXTURE as f32,
            );
            let mut value = 0.;
            let mut weight = 0.5;
            for octave in 0..5 {
                let period = 4 << octave;
                value += weight
                    * noise(
                        u * period as f32,
                        v * period as f32,
                        period,
                        0xc10d + octave,
                    );
                weight *= 0.5;
            }
            let edge = profile.softness.max(1e-3);
            let t = ((value / 0.97 - (1. - profile.coverage)) / edge).clamp(0., 1.);
            alpha.push((t * t * (3. - 2. * t) * 255.) as u8);
        }
    }
    alpha
}

fn spawn_clouds(
    mut commands: Commands,
    seed: Res<crate::WorldSeed>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
    let profile = crate::calendar::sky_profile().clouds;
    let pixels: Vec<u8> = cloud_alpha(seed.0, &profile)
        .into_iter()
        .flat_map(|a| [255, 255, 255, a])
        .collect();
    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: CLOUD_TEXTURE as u32,
            height: CLOUD_TEXTURE as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    // A dome above the horizon; texture coordinates project each direction onto a flat
    // cloud deck, so clouds crowd toward the horizon, and fade out just above it.
    let (rings, sectors) = (24, 72);
    let (mut positions, mut uvs, mut colors, mut indices) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for ring in 0..=rings {
        let elevation = (ring as f32 / rings as f32).powf(1.6) * std::f32::consts::FRAC_PI_2;
        for sector in 0..=sectors {
            let azimuth = sector as f32 / sectors as f32 * std::f32::consts::TAU;
            let d = Vec3::new(
                elevation.cos() * azimuth.cos(),
                elevation.sin(),
                elevation.cos() * azimuth.sin(),
            );
            positions.push((d * CLOUD_RADIUS).to_array());
            let deck = Vec2::new(d.x, d.z) / (d.y + 0.12) * 0.25 * profile.repeats;
            uvs.push(deck.to_array());
            let t = ((d.y - 0.02) / 0.2).clamp(0., 1.);
            colors.push([1., 1., 1., t * t * (3. - 2. * t)]);
        }
    }
    for ring in 0..rings {
        for sector in 0..sectors {
            let a = (ring * (sectors + 1) + sector) as u32;
            let b = a + sectors as u32 + 1;
            indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices));
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(image)),
        unlit: true,
        fog_enabled: false,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        // Drawn after the stars (also centred on the camera), so clouds cover them.
        depth_bias: 1.,
        ..default()
    });
    commands.spawn((
        CloudLayer(material.clone()),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
        bevy::camera::visibility::NoFrustumCulling,
        Transform::default(),
    ));
}

/// Clouds follow the camera, drift with simulation time and take the light's colour:
/// pale by day, warm near sunrise and sunset, dark blue-grey at night.
fn drift_clouds(
    calendar: Res<crate::calendar::Calendar>,
    mut clouds: Query<(&CloudLayer, &mut Transform), Without<Camera3d>>,
    cameras: Query<&Transform, (With<Camera3d>, With<bevy::pbr::AtmosphereSettings>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut profile: Local<Option<CloudProfile>>,
) {
    let profile = profile.get_or_insert_with(|| crate::calendar::sky_profile().clouds);
    let Ok(camera) = cameras.single() else { return };
    let hours = calendar.elapsed.as_secs_f32() / crate::calendar::CYCLE_SECONDS as f32 * 24.;
    let sun = crate::calendar::sun_direction(calendar.elapsed.as_secs_f32());
    let color = cloud_color(profile, sun);
    for (layer, mut transform) in &mut clouds {
        transform.translation = camera.translation;
        if let Some(mut material) = materials.get_mut(&layer.0) {
            material.uv_transform =
                bevy::math::Affine2::from_translation(Vec2::from(profile.drift_per_hour) * hours);
            material.base_color = color;
        }
    }
}

/// The clouds' colour for a sun direction.
pub fn cloud_color(profile: &CloudProfile, sun: Vec3) -> Color {
    let day = ((sun.y + 0.05) / 0.3).clamp(0., 1.);
    let low_sun = (1. - (sun.y.abs() / 0.25)).clamp(0., 1.);
    let lit = Vec3::from(profile.day_color).lerp(Vec3::from(profile.dusk_color), low_sun);
    let rgb = Vec3::from(profile.night_color).lerp(lit, day);
    Color::srgba(rgb.x, rgb.y, rgb.z, profile.opacity)
}

/// Bevy re-filters a `GeneratedEnvironmentMapLight` every frame (~1.3 ms GPU), but the
/// sky changes slowly. Filter it, keep the `EnvironmentMapLight` it produced and drop
/// the generator; filter again once the sun or moon has moved a degree (at 64x, a few
/// times a second; at 1x, every ~15 s) or the sky's brightness has changed.
pub struct SkyPlugin;

/// Main-world cameras whose environment map the render world has filtered.
#[derive(Resource, Clone, Default)]
struct BakedEnvironmentMaps(std::sync::Arc<std::sync::Mutex<Vec<Entity>>>);

/// The generator a camera's map was last filtered with, and the sky it saw.
#[derive(Component)]
struct BakedSky {
    generator: GeneratedEnvironmentMapLight,
    sun: Vec3,
    moon: Vec3,
}
const REBAKE_RADIANS: f32 = 1. * std::f32::consts::PI / 180.;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        use bevy::pbr::generate::filtering_system;
        use bevy::render::{Render, RenderApp, RenderSystems};
        let baked = BakedEnvironmentMaps::default();
        app.insert_resource(baked.clone())
            .add_systems(Startup, (spawn_stars, spawn_clouds))
            .add_systems(
                Update,
                (
                    stop_regenerating,
                    rebake,
                    turn_stars,
                    drift_clouds,
                    follow_the_storm,
                ),
            );
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.insert_resource(baked).add_systems(
                Render,
                mark_filtered
                    .after(filtering_system)
                    .before(RenderSystems::Render),
            );
        }
    }
}

/// Runs after the generator's filter passes: if all of its pipelines are compiled, every
/// generated map with bind groups was filtered this frame.
fn mark_filtered(
    maps: Query<
        &bevy::render::sync_world::MainEntity,
        (
            With<bevy::pbr::generate::GeneratorBindGroups>,
            With<bevy::pbr::generate::RenderEnvironmentMap>,
        ),
    >,
    cache: Res<bevy::render::render_resource::PipelineCache>,
    pipelines: Option<Res<bevy::pbr::generate::GeneratorPipelines>>,
    baked: Res<BakedEnvironmentMaps>,
) {
    let Some(p) = pipelines else { return };
    let ready = [
        p.downsample_first,
        p.downsample_second,
        p.copy,
        p.radiance,
        p.irradiance,
    ]
    .into_iter()
    .all(|id| cache.get_compute_pipeline(id).is_some());
    if !ready || maps.is_empty() {
        return;
    }
    if let Ok(mut list) = baked.0.lock() {
        list.extend(maps.iter().map(|main| main.id()));
    }
}

fn stop_regenerating(
    mut commands: Commands,
    baked: Res<BakedEnvironmentMaps>,
    calendar: Res<crate::calendar::Calendar>,
    generated: Query<&GeneratedEnvironmentMapLight, With<EnvironmentMapLight>>,
) {
    let Ok(mut list) = baked.0.lock() else { return };
    let elapsed = calendar.elapsed.as_secs_f32();
    for entity in list.drain(..) {
        if let Ok(generator) = generated.get(entity) {
            commands
                .entity(entity)
                .insert(BakedSky {
                    generator: generator.clone(),
                    sun: crate::calendar::sun_direction(elapsed),
                    moon: crate::calendar::moon_direction(elapsed),
                })
                .remove::<GeneratedEnvironmentMapLight>();
        }
    }
}

/// Filter the map again once the sky has visibly moved since the last bake.
fn rebake(
    mut commands: Commands,
    calendar: Res<crate::calendar::Calendar>,
    cameras: Query<(Entity, &BakedSky), Without<GeneratedEnvironmentMapLight>>,
) {
    let elapsed = calendar.elapsed.as_secs_f32();
    let sun = crate::calendar::sun_direction(elapsed);
    let moon = crate::calendar::moon_direction(elapsed);
    for (entity, baked) in &cameras {
        if needs_rebake(baked.sun, sun) || needs_rebake(baked.moon, moon) {
            commands.entity(entity).insert(baked.generator.clone());
        }
    }
}

fn needs_rebake(baked: Vec3, now: Vec3) -> bool {
    baked.angle_between(now) > REBAKE_RADIANS
}

/// The mountain rings (`assets/lighting/sky.json`, `ridges`).
#[derive(serde::Deserialize, Clone)]
pub struct RidgeProfile {
    pub rings: usize,
    pub near_radius_m: f32,
    pub ring_spacing_m: f32,
    pub harmonics: Vec<u32>,
    pub roughness: f32,
    pub echo_shift_deg: f32,
    pub base_deg: Vec<f32>,
    pub amp_deg: Vec<f32>,
    pub colors: Vec<[f32; 3]>,
}
/// Segments per ring: one per degree, so an echo shift of whole degrees is exact.
pub const RIDGE_SEGMENTS: usize = 360;

/// The seeded skyline, 0..1, one value per degree: whole-cycle harmonics (so it closes
/// around the ring) with amplitudes falling as 1/k and seeded phases, plus a little
/// smooth seeded roughness.
pub fn skyline(seed: u64, profile: &RidgeProfile) -> Vec<f32> {
    use crate::terrain::hash;
    let raw: Vec<f32> = (0..RIDGE_SEGMENTS)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / RIDGE_SEGMENTS as f32;
            let waves: f32 = profile
                .harmonics
                .iter()
                .enumerate()
                .map(|(k, &n)| {
                    let phase = std::f32::consts::TAU * hash(k as i32, 7, seed ^ 0x51d6e);
                    (n as f32 * angle + phase).sin() / (k + 1) as f32
                })
                .sum();
            // Value noise every 4 degrees, smoothly interpolated, wrapped around the ring.
            let cell = i / 4;
            let t = (i % 4) as f32 / 4.;
            let t = t * t * (3. - 2. * t);
            let cells = (RIDGE_SEGMENTS / 4) as i32;
            let a = hash(cell as i32 % cells, 9, seed ^ 0x51d6e);
            let b = hash((cell as i32 + 1) % cells, 9, seed ^ 0x51d6e);
            waves + profile.roughness * (a + (b - a) * t - 0.5) * 4.
        })
        .collect();
    let (low, high) = raw
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    raw.iter()
        .map(|v| (v - low) / (high - low).max(1e-6))
        .collect()
}

/// Each ring's elevation in radians, per degree: the skyline echoed `r` shifts round
/// the ring, higher and softer with distance, so a far ring peeks over a near one.
pub fn ridge_elevations(seed: u64, profile: &RidgeProfile) -> Vec<Vec<f32>> {
    let line = skyline(seed, profile);
    let shift = profile.echo_shift_deg.round() as usize;
    (0..profile.rings)
        .map(|ring| {
            (0..RIDGE_SEGMENTS)
                .map(|i| {
                    let f = line
                        [(i + RIDGE_SEGMENTS - (ring * shift) % RIDGE_SEGMENTS) % RIDGE_SEGMENTS];
                    (profile.base_deg[ring] + profile.amp_deg[ring] * f).to_radians()
                })
                .collect()
        })
        .collect()
}

/// The rings as meshes: each segment a quad from below the ground up to the ridge.
pub fn spawn_ridges(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    seed: u64,
) {
    let profile = crate::calendar::sky_profile().ridges;
    let rotation = Quat::from_rotation_y(-stubby_core::OPENING_HEADING_DEGREES.to_radians());
    let spawn = stubby_core::OPENING_SPAWN_XZ;
    let pivot = Vec3::new(spawn.x, 0., spawn.y);
    let frame = Transform::from_translation(pivot - rotation * pivot).with_rotation(rotation);
    for (ring, elevations) in ridge_elevations(seed, &profile).iter().enumerate() {
        let radius = profile.near_radius_m + ring as f32 * profile.ring_spacing_m;
        let point = |i: usize| {
            let angle = (i % RIDGE_SEGMENTS) as f32 * std::f32::consts::TAU / RIDGE_SEGMENTS as f32;
            let height = radius * elevations[i % RIDGE_SEGMENTS].tan();
            Vec3::new(angle.cos() * radius, height, angle.sin() * radius)
        };
        let mut surface = crate::terrain::Surface::default();
        for i in 0..RIDGE_SEGMENTS {
            let (a, b) = (point(i), point(i + 1));
            let (low_a, low_b) = (Vec3::new(a.x, -3., a.z), Vec3::new(b.x, -3., b.z));
            surface.triangle(low_a, a, b, [1.; 4]);
            surface.triangle(low_a, b, low_b, [1.; 4]);
        }
        let [r, g, b] = profile.colors[ring];
        commands.spawn((
            frame,
            Mesh3d(meshes.add(surface.mesh())),
            stubby_core::AcousticOccluder::ROCK,
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
                perceptual_roughness: 1.,
                cull_mode: None,
                ..default()
            })),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn far_rings_echo_the_near_skyline_softer_and_higher() {
        let profile = crate::calendar::sky_profile().ridges;
        let rings = ridge_elevations(42, &profile);
        assert_eq!(rings, ridge_elevations(42, &profile));
        assert_ne!(rings, ridge_elevations(43, &profile));
        let shift = profile.echo_shift_deg.round() as usize;
        let shape = |ring: usize, i: usize| {
            (rings[ring][i].to_degrees() - profile.base_deg[ring]) / profile.amp_deg[ring]
        };
        for i in 0..RIDGE_SEGMENTS {
            assert!((shape(1, (i + shift) % RIDGE_SEGMENTS) - shape(0, i)).abs() < 1e-4);
        }
        // Softer: less spread; the echo's base sits higher.
        let spread = |ring: usize| {
            let (l, h) = rings[ring]
                .iter()
                .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
            h - l
        };
        assert!(spread(2) < spread(1) && spread(1) < spread(0));
        assert!(profile.base_deg[2] > profile.base_deg[0]);
    }
    #[test]
    fn cloud_cover_is_seeded_and_follows_coverage() {
        let profile = crate::calendar::sky_profile().clouds;
        let cover = |coverage: f32, seed: u64| {
            let alpha = cloud_alpha(
                seed,
                &CloudProfile {
                    coverage,
                    ..profile.clone()
                },
            );
            alpha.iter().filter(|a| **a > 128).count() as f32 / alpha.len() as f32
        };
        assert_eq!(cloud_alpha(42, &profile), cloud_alpha(42, &profile));
        assert_ne!(cloud_alpha(42, &profile), cloud_alpha(43, &profile));
        assert!(cover(0.2, 42) < cover(0.5, 42) && cover(0.5, 42) < cover(0.8, 42));
    }
}
