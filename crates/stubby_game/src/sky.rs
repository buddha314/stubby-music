use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};

pub fn desert_sky() -> Image {
    let size = 128;
    // Bevy samples cubemaps in left-handed space (world Z is negated).
    let mut data = Vec::with_capacity(size * size * 6 * 8);
    for face in 0..6 {
        for y in 0..size {
            for x in 0..size {
                let u = 2. * (x as f32 + 0.5) / size as f32 - 1.;
                let v = 2. * (y as f32 + 0.5) / size as f32 - 1.;
                let d = match face {
                    0 => Vec3::new(1., -v, -u),
                    1 => Vec3::new(-1., -v, u),
                    2 => Vec3::new(u, 1., v),
                    3 => Vec3::new(u, -1., -v),
                    4 => Vec3::new(u, -v, 1.),
                    _ => Vec3::new(-u, -v, -1.),
                }
                .normalize();
                let horizon = Vec3::new(0.72, 0.43, 0.22);
                let zenith = Vec3::new(0.14, 0.19, 0.25);
                let mut rgb = horizon.lerp(zenith, d.y.max(0.).sqrt());
                if d.y < 0. {
                    rgb = horizon.lerp(Vec3::new(0.13, 0.09, 0.06), (-d.y * 3.).min(1.));
                }
                for c in [rgb.x, rgb.y, rgb.z, 1.] {
                    data.extend_from_slice(&half::f16::from_f32(c).to_le_bytes());
                }
            }
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba16Float,
        RenderAssetUsages::default(),
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image
}

/// The desert sky cubemap never changes (day and night only scale its intensity), but
/// Bevy re-filters a `GeneratedEnvironmentMapLight` every frame (~1.3 ms GPU). Bake it
/// once: when the render world has run the generator's filter passes, drop
/// `GeneratedEnvironmentMapLight` and keep the `EnvironmentMapLight` it produced.
pub struct StaticSkyPlugin;

/// Main-world cameras whose environment map the render world has filtered.
#[derive(Resource, Clone, Default)]
struct BakedEnvironmentMaps(std::sync::Arc<std::sync::Mutex<Vec<Entity>>>);

impl Plugin for StaticSkyPlugin {
    fn build(&self, app: &mut App) {
        use bevy::pbr::generate::filtering_system;
        use bevy::render::{Render, RenderApp, RenderSystems};
        let baked = BakedEnvironmentMaps::default();
        app.insert_resource(baked.clone())
            .add_systems(Update, stop_regenerating);
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
    generated: Query<
        (),
        (
            With<GeneratedEnvironmentMapLight>,
            With<EnvironmentMapLight>,
        ),
    >,
) {
    let Ok(mut list) = baked.0.lock() else { return };
    for entity in list.drain(..) {
        if generated.contains(entity) {
            commands
                .entity(entity)
                .remove::<GeneratedEnvironmentMapLight>();
        }
    }
}
