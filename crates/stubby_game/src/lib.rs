#[cfg(all(feature = "dev", not(debug_assertions)))]
compile_error!("The dev feature enables dynamic linking; omit it for release builds.");

// The crafting system, as a Bevy plugin. Jackdaw's adapter builds the game against the
// editor's Bevy revision, so there the plugin's source is compiled in as a module.
#[cfg(not(feature = "embedded_recipes_plugin"))]
pub use bevy_stubby_recipes as recipes_plugin;
#[cfg(feature = "embedded_recipes_plugin")]
#[path = "../../bevy_stubby_recipes/src/lib.rs"]
pub mod recipes_plugin;

pub mod audio;
pub mod calendar;
pub mod controls;
pub mod export;
mod hand;
mod inventory_screen;
pub mod machine_config;
pub mod module_ids;
pub mod prefab_presentation;
mod prefab_review;
pub mod prefabs;
pub mod rack;
pub mod pulse;
pub mod resources;
mod review;
mod sky;
pub mod synth_modules;
mod terrain;
mod waveform_audio;

use bevy::{
    asset::RenderAssetUsages,
    camera::Exposure,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    light::Skybox,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use terrain::{Surface, hash};

pub struct DesertPlugin {
    pub seed: u64,
}
impl Plugin for DesertPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            pulse::LightningBugPlugin { seed: self.seed },
            resources::ResourceRecipesPlugin,
            inventory_screen::InventoryScreenPlugin,
            hand::HandPlugin,
            rack::RackPlugin,
            prefabs::PlacedPrefabsPlugin,
            calendar::CalendarPlugin,
            review::ReplenishmentReviewPlugin,
            prefab_review::PrefabReviewPlugin,
            controls::PlayerControlsPlugin,
            machine_config::MachineConfigurationPlugin,
            waveform_audio::WaveformAudioPlugin,
            synth_modules::SynthModulesPlugin::default(),
        ))
        .insert_resource(WorldSeed(self.seed))
        .add_systems(Startup, setup)
        .add_systems(Update, capture_preview);
    }
}

#[derive(Resource)]
pub struct WorldSeed(pub u64);

fn mud_texture(seed: u64) -> (Image, Image) {
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

fn setup(
    mut commands: Commands,
    seed: Res<WorldSeed>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (albedo, normal) = mud_texture(seed.0);
    let texture = images.add(albedo);
    let normal = images.add(normal);
    let floor = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        normal_map_texture: Some(normal),
        perceptual_roughness: 0.96,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(terrain::ground(seed.0, 60))),
        MeshMaterial3d(floor),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1600., 1600.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.19, 0.13, 0.085),
            perceptual_roughness: 1.,
            ..default()
        })),
        Transform::from_xyz(0., -0.15, 0.),
    ));
    // Continuous overlapping ridges keep the basin enclosed without a sawtooth skyline.
    for layer in 0..3 {
        let mut mountains = Surface::default();
        let radius = 480. + layer as f32 * 110.;
        let ridge = |i: i32| {
            let wrapped = i.rem_euclid(192);
            let angle = wrapped as f32 * std::f32::consts::TAU / 192.;
            let h = 8.
                + hash(wrapped / 4, layer, seed.0 ^ 198) * 12.
                + hash(wrapped, layer, seed.0 ^ 991) * 10.;
            Vec3::new(angle.cos() * radius, h, angle.sin() * radius)
        };
        for i in 0..192 {
            let a = ridge(i);
            let b = ridge(i + 1);
            let low_a = Vec3::new(a.x, -3., a.z);
            let low_b = Vec3::new(b.x, -3., b.z);
            mountains.triangle(low_a, a, b, [1.; 4]);
            mountains.triangle(low_a, b, low_b, [1.; 4]);
        }
        commands.spawn((
            Mesh3d(meshes.add(mountains.mesh())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.17, 0.20),
                perceptual_roughness: 1.,
                cull_mode: None,
                ..default()
            })),
        ));
    }
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
        Transform::from_xyz(-48., 7.5, -87.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let sky = images.add(sky::desert_sky());
    commands.spawn((
        GeneratedEnvironmentMapLight {
            environment_map: sky.clone(),
            intensity: 1500.,
            ..default()
        },
        Camera3d::default(),
        bevy::post_process::bloom::Bloom::NATURAL,
        Exposure { ev100: 10.5 },
        bevy::ui::IsDefaultUiCamera,
        controls::Player::default(),
        controls::spawn_transform(),
        Skybox {
            image: Some(sky),
            brightness: 1500.,
            ..default()
        },
        DistanceFog {
            color: Color::srgb(0.56, 0.43, 0.33),
            falloff: FogFalloff::Exponential { density: 0.004 },
            ..default()
        },
    ));
}

// Capture after pipeline warmup; useful for reviewing a seed without desktop tools.
fn capture_preview(mut commands: Commands, time: Res<Time>, mut captured: Local<bool>) {
    if !*captured
        && time.elapsed_secs() > 15.
        && let Some(path) =
            std::env::args().find_map(|s| s.strip_prefix("--screenshot=").map(str::to_owned))
    {
        *captured = true;
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path));
    }
}
