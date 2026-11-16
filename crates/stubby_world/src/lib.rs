//! The world (split-game-into-systems): the seeded desert (terrain, sky, mountains, light),
//! simulation time, world generation, and the lifecycle and presentation of placed
//! prefabs. Placed assets come alive through binders other systems register; the world
//! itself names no creature, machine or crafting source.
pub mod calendar;
pub mod export;
pub mod lighting;
pub mod prefab_presentation;
pub mod prefabs;
pub mod sky;
pub mod terrain;
pub mod world_generation;

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use terrain::hash;

/// The seeded desert: ground, mountains, sun and sky, time, and placed prefabs.
pub struct DesertWorldPlugin {
    pub seed: u64,
}
impl Plugin for DesertWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sky::SkyPlugin,
            calendar::CalendarPlugin,
            prefabs::PlacedPrefabsPlugin,
        ))
        .insert_resource(WorldSeed(self.seed))
        // The desert's terrain is the ground the player and creatures stand on.
        .insert_resource(stubby_core::GroundHeight(terrain::height))
        .add_systems(Startup, setup_world.in_set(WorldSetup));
    }
}

/// The world's startup: ground, mountains, light and the atmosphere exist after it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSetup;

#[derive(Resource)]
pub struct WorldSeed(pub u64);

pub fn mud_texture(seed: u64) -> (Image, Image) {
    let size = 1024;
    let mut bytes = Vec::with_capacity(size * size * 4);
    let mut heights = Vec::with_capacity(size * size);
    // Periodic Worley cells give fine branching cracks; the primary plates are geometry.
    for y in 0..size {
        for x in 0..size {
            let p = Vec2::new(x as f32, y as f32) * (32. / size as f32);
            let cell = p.floor().as_ivec2();
            let mut nearest = [100_f32; 2];
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let q = cell + IVec2::new(dx, dz);
                    let sx = q.x.rem_euclid(32);
                    let sz = q.y.rem_euclid(32);
                    let site = q.as_vec2() + Vec2::new(hash(sx, sz, seed), hash(sx, sz, seed ^ 81));
                    let d = p.distance(site);
                    if d < nearest[0] {
                        nearest[1] = nearest[0];
                        nearest[0] = d;
                    } else if d < nearest[1] {
                        nearest[1] = d;
                    }
                }
            }
            let edge = ((nearest[1] - nearest[0]) / 0.035).clamp(0., 1.);
            let grain = hash(x as i32, y as i32, seed ^ 111);
            let value = 0.62 + edge * 0.30 + grain * 0.08;
            heights.push(edge * 0.002 + grain * 0.0004);
            for c in [0.62, 0.48, 0.33] {
                bytes.push((c * value * 255.) as u8);
            }
            bytes.push(255);
        }
    }
    let mut img = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    let mut normals = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let h = |dx: usize, dy: usize| heights[(dy % size) * size + dx % size];
            let dx = (h(x + 1, y) - h(x + size - 1, y)) / (32. / size as f32);
            let dy = (h(x, y + 1) - h(x, y + size - 1)) / (32. / size as f32);
            let n = Vec3::new(-dx, -dy, 1.).normalize();
            normals.extend_from_slice(&[
                ((n.x * 0.5 + 0.5) * 255.) as u8,
                ((n.y * 0.5 + 0.5) * 255.) as u8,
                ((n.z * 0.5 + 0.5) * 255.) as u8,
                255,
            ]);
        }
    }
    let mut normal = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        normals,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    normal.sampler = img.sampler.clone();
    (img, normal)
}

fn setup_world(
    mut commands: Commands,
    seed: Res<WorldSeed>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut mediums: ResMut<Assets<bevy::light::atmosphere::ScatteringMedium>>,
) {
    sky::spawn_atmosphere(&mut commands, &mut mediums);
    let (albedo, normal) = mud_texture(seed.0);
    let texture = images.add(albedo);
    let normal = images.add(normal);
    let floor = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        normal_map_texture: Some(normal),
        perceptual_roughness: 0.96,
        ..default()
    });
    for mesh in terrain::ground_chunks(seed.0, 60, 12) {
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(floor.clone())));
    }
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1600., 1600.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.19, 0.13, 0.085),
            perceptual_roughness: 1.,
            ..default()
        })),
        Transform::from_xyz(0., -0.15, 0.),
    ));
    // Mountain rings: one seeded skyline, echoed outward (sky.json `ridges`).
    sky::spawn_ridges(&mut commands, &mut meshes, &mut materials, seed.0);
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.68, 0.72, 0.83),
        brightness: 100.,
        ..default()
    });
    commands.spawn((
        calendar::Sunlight,
        DirectionalLight {
            color: Color::srgb(1., 0.71, 0.43),
            illuminance: 9500.,
            shadow_maps_enabled: true,
            ..default()
        },
        // The atmosphere draws the disc; twice Earth's, as the Godot sky's.
        bevy::light::SunDisk {
            angular_size: 0.019,
            intensity: 1.,
        },
        // Two cascades, not Bevy's four: halves the shadow pass on integrated GPUs
        // (780M: 3.2 -> 1.6 ms). Raise if long dawn/dusk shadows look too soft.
        bevy::light::CascadeShadowConfigBuilder {
            num_cascades: 2,
            maximum_distance: 120.,
            ..default()
        }
        .build(),
        Transform::from_xyz(-48., 7.5, -87.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
