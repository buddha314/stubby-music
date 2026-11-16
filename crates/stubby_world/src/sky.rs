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

/// Earth's air with a low layer of desert dust.
pub fn desert_air() -> bevy::light::atmosphere::ScatteringMedium {
    use bevy::light::atmosphere::{Falloff, PhaseFunction, ScatteringMedium, ScatteringTerm};
    let mut air = ScatteringMedium::earth(256, 256);
    air.terms.push(ScatteringTerm {
        absorption: DUST_ABSORPTION,
        scattering: Vec3::splat(DUST_SCATTERING),
        // Thickness is a fraction of the 100 km atmosphere: ~3 km of dust.
        falloff: Falloff::Exponential { scale: 3. / 100. },
        phase: PhaseFunction::Mie { asymmetry: 0.7 },
    });
    air.with_label("desert_air")
}

/// The planet the atmosphere belongs to: Earth, under desert air.
pub fn spawn_atmosphere(
    commands: &mut Commands,
    mediums: &mut Assets<bevy::light::atmosphere::ScatteringMedium>,
) {
    let medium = mediums.add(desert_air());
    let mut atmosphere = bevy::light::Atmosphere::earth(medium);
    // Dry, pale mud: brighter multiscattering than Earth's average.
    atmosphere.ground_albedo = Vec3::new(0.42, 0.36, 0.30);
    commands.spawn(atmosphere);
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
            .add_systems(Startup, spawn_stars)
            .add_systems(Update, (stop_regenerating, rebake, turn_stars));
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
