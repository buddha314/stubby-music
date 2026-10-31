//! Shared sample-clock pulse bus and the lightning bug's flash-only presentation.
use super::audio::{ClockTick, Synth};
use bevy::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, time::Instant};
use stubby_audio::{Command, ModuleId, ModuleKind, PortRef};

pub const LIGHTNING_BUG_CLOCK: u64 = 1;
const BUG_POSITIONS: [Vec3; 3] = [
    Vec3::new(-1., 1.5, 2.),
    Vec3::new(0.5, 2.2, 0.),
    Vec3::new(1.4, 2.8, -2.),
];
fn bug_position(index: usize) -> Vec3 {
    let mut position = BUG_POSITIONS[index];
    position.y += super::terrain::height(Vec2::new(position.x, position.z)) + 0.035;
    position
}

fn bug_clock(index: usize) -> u64 {
    LIGHTNING_BUG_CLOCK + index as u64
}
fn bug_seed(world_seed: u64, index: usize) -> u64 {
    world_seed ^ bug_clock(index).wrapping_mul(0x9e3779b97f4a7c15)
}

/// Every follower gets the same sample timestamp and predicted display instant.
#[derive(Message, Debug, Clone, Copy)]
pub struct SamplePulse {
    pub clock: u64,
    pub sample_time: u64,
    pub at: Instant,
}

/// Attach to future pulse-driven assets as well as the lightning bug.
#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component, Default)]
pub struct PulseFollower {
    pub clock: u64,
}
impl Default for PulseFollower {
    fn default() -> Self {
        Self {
            clock: LIGHTNING_BUG_CLOCK,
        }
    }
}

#[derive(Resource, Default)]
pub struct PulseBus {
    pending: Vec<SamplePulse>,
    latest: HashMap<u64, SamplePulse>,
}
impl PulseBus {
    pub fn queue(&mut self, pulse: SamplePulse) {
        self.pending.push(pulse);
    }
    pub fn advance(&mut self, now: Instant) {
        let latest = &mut self.latest;
        self.pending.retain(|pulse| {
            if pulse.at > now {
                return true;
            }
            let entry = latest.entry(pulse.clock).or_insert(*pulse);
            if pulse.sample_time >= entry.sample_time {
                *entry = *pulse;
            }
            false
        });
    }
    pub fn level(&self, clock: u64, now: Instant, attack: f32, duration: f32) -> f32 {
        self.latest
            .get(&clock)
            .and_then(|pulse| now.checked_duration_since(pulse.at))
            .map_or(0., |age| {
                flash_envelope(age.as_secs_f32(), attack, duration)
            })
    }
}

fn flash_envelope(age: f32, attack: f32, duration: f32) -> f32 {
    if age < 0. || age >= duration {
        0.
    } else if age < attack {
        age / attack
    } else {
        ((duration - age) / (duration - attack)).powi(2)
    }
}

#[derive(Resource, Deserialize)]
struct BugAsset {
    model: String,
    flash_seconds: f32,
    attack_seconds: f32,
    light_lumens: f32,
    light_range_metres: f32,
    clock: ClockSettings,
    click: super::waveform_audio::WaveformSound,
}
#[derive(Deserialize)]
struct ClockSettings {
    min_interval_seconds: f32,
    max_interval_seconds: f32,
}

fn voice_ids(index: usize) -> (ModuleId, ModuleId, ModuleId) {
    let base = 100 + index as u64 * 3;
    (ModuleId(base), ModuleId(base + 1), ModuleId(base + 2))
}
#[derive(Component)]
struct FlashVisual;
#[derive(Component)]
struct FlashLight;
#[derive(Component)]
struct FlashShadow;
#[derive(Component)]
struct BugInstance(usize);

pub struct LightningBugPlugin {
    pub seed: u64,
}
impl Plugin for LightningBugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PulseSeed(self.seed))
            .init_resource::<PulseBus>()
            .register_type::<PulseFollower>()
            .add_message::<SamplePulse>()
            .add_systems(Startup, initialize)
            .add_systems(PostUpdate, remove_collected_bugs)
            .add_systems(
                Update,
                (
                    update_click_gains,
                    poll_audio,
                    animate,
                    capture_pulse_preview,
                )
                    .chain(),
            );
    }
}
#[derive(Resource)]
struct PulseSeed(u64);

fn initialize(world: &mut World) {
    let Ok(asset) = serde_json::from_str::<BugAsset>(include_str!(
        "../../../assets/creatures/lightning_bug/asset.json"
    )) else {
        error!("Invalid lightning bug asset descriptor");
        return;
    };
    if !(asset.attack_seconds > 0. && asset.flash_seconds > asset.attack_seconds) {
        error!("Lightning bug flash must have a positive attack shorter than its duration");
        return;
    }
    let seed = world.resource::<PulseSeed>().0;
    if let Some(synth) = world.get_resource::<Synth>() {
        for index in 0..BUG_POSITIONS.len() {
            synth.send(Command::AddModule {
                id: ModuleId(bug_clock(index)),
                kind: ModuleKind::RandomClock {
                    seed: bug_seed(seed, index),
                    min_interval_s: asset.clock.min_interval_seconds,
                    max_interval_s: asset.clock.max_interval_seconds,
                },
            });
            let (oscillator, envelope, vca) = voice_ids(index);
            for command in super::waveform_audio::voice_commands(
                stubby_recipes::Waveform::Square,
                [oscillator, envelope, vca],
                PortRef::new(ModuleId(bug_clock(index)), "gate"),
                0.,
                asset.click,
            ) {
                synth.send(command);
            }
        }
    }
    let scene = world.resource::<AssetServer>().load(asset.model.clone());
    let shadow_texture = shadow_texture();
    let shadow_texture = world.resource_mut::<Assets<Image>>().add(shadow_texture);
    let shadow_mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Plane3d::default().mesh().size(2.0, 2.0));
    for index in 0..BUG_POSITIONS.len() {
        let position = bug_position(index);
        world.spawn((
            Name::new(format!("Lightning bug {}", index + 1)),
            WorldAssetRoot(scene.clone()),
            PulseFollower {
                clock: bug_clock(index),
            },
            FlashVisual,
            BugInstance(index),
            super::resources::ResourceNode::lightning_bug(index),
            Transform::from_translation(position).with_scale(Vec3::ZERO),
            Visibility::Hidden,
        ));
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgba(0.02, 0.015, 0.01, 0.),
                base_color_texture: Some(shadow_texture.clone()),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
        world.spawn((
            Name::new(format!("Lightning bug {} ground shadow", index + 1)),
            PulseFollower {
                clock: bug_clock(index),
            },
            FlashShadow,
            BugInstance(index),
            Mesh3d(shadow_mesh.clone()),
            MeshMaterial3d(material),
            bevy::light::NotShadowCaster,
            Transform::from_xyz(
                position.x,
                super::terrain::height(Vec2::new(position.x, position.z)) + 0.055,
                position.z,
            ),
        ));
        world.spawn((
            Name::new(format!("Lightning bug {} white flash", index + 1)),
            PulseFollower {
                clock: bug_clock(index),
            },
            FlashLight,
            BugInstance(index),
            PointLight {
                color: Color::WHITE,
                intensity: 0.,
                range: asset.light_range_metres,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(position),
        ));
    }
    world.insert_resource(asset);
}

// Run after collection's Update systems, so no depleted cue survives the frame.
fn remove_collected_bugs(
    mut commands: Commands,
    mut collected: MessageReader<super::resources::ResourceCollected>,
    bugs: Query<(Entity, &BugInstance)>,
    synth: Option<Res<Synth>>,
) {
    for event in collected.read() {
        for index in 0..BUG_POSITIONS.len() {
            if event.instance != format!("lightning_bug-{index}") {
                continue;
            }
            for (entity, instance) in &bugs {
                if instance.0 == index {
                    commands.entity(entity).despawn();
                }
            }
            if let Some(synth) = &synth {
                let (oscillator, envelope, vca) = voice_ids(index);
                // Disconnect the output first, then remove its source nodes.
                for id in [vca, oscillator, envelope, ModuleId(bug_clock(index))] {
                    synth.send(Command::RemoveModule { id });
                }
            }
        }
    }
}

// Gain is listener distance only. The transient and its timing remain in the DSP graph.
fn update_click_gains(
    audio: Option<Res<Synth>>,
    asset: Option<Res<BugAsset>>,
    camera: Query<&Transform, With<Camera3d>>,
    bugs: Query<(&BugInstance, &Transform), With<FlashVisual>>,
) {
    let (Some(audio), Some(asset), Ok(camera)) = (audio, asset, camera.single()) else {
        return;
    };
    for (instance, transform) in &bugs {
        let index = instance.0;
        let distance = transform.translation.distance(camera.translation);
        let gain = asset.click.gain
            * (1. - distance / asset.click.hearing_radius_metres)
                .clamp(0., 1.)
                .powi(2);
        let (_, _, vca) = voice_ids(index);
        audio.send(Command::SetParam {
            module: vca,
            param: "gain",
            value: gain,
        });
    }
}

fn poll_audio(
    mut ticks: MessageReader<ClockTick>,
    mut bus: ResMut<PulseBus>,
    mut pulses: MessageWriter<SamplePulse>,
) {
    for tick in ticks.read() {
        if let Some(at) = tick.at {
            let pulse = SamplePulse {
                clock: tick.module.0,
                sample_time: tick.sample_time,
                at,
            };
            bus.queue(pulse);
            pulses.write(pulse);
        }
    }
    bus.advance(Instant::now());
}

fn shadow_texture() -> Image {
    let size = 64;
    let mut data = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let radius = Vec2::new(
                (x as f32 + 0.5) / size as f32 * 2. - 1.,
                (y as f32 + 0.5) / size as f32 * 2. - 1.,
            )
            .length();
            let alpha = (1. - radius * radius).clamp(0., 1.);
            data.extend_from_slice(&[255, 255, 255, (alpha * 255.) as u8]);
        }
    }
    Image::new(
        bevy::render::render_resource::Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}

fn animate(
    bus: Res<PulseBus>,
    asset: Option<Res<BugAsset>>,
    mut visuals: Query<(&PulseFollower, &mut Transform, &mut Visibility), With<FlashVisual>>,
    mut lights: Query<(&PulseFollower, &mut PointLight), With<FlashLight>>,
    shadows: Query<(&PulseFollower, &MeshMaterial3d<StandardMaterial>), With<FlashShadow>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(asset) = asset else {
        return;
    };
    let now = Instant::now();
    for (follower, mut transform, mut visibility) in &mut visuals {
        let level = bus.level(
            follower.clock,
            now,
            asset.attack_seconds,
            asset.flash_seconds,
        );
        *visibility = if level > 0. {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.scale = Vec3::splat(level.sqrt());
    }
    for (follower, handle) in &shadows {
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.base_color.set_alpha(
                0.95 * bus.level(
                    follower.clock,
                    now,
                    asset.attack_seconds,
                    asset.flash_seconds,
                ),
            );
        }
    }
    for (follower, mut light) in &mut lights {
        light.intensity = asset.light_lumens
            * bus.level(
                follower.clock,
                now,
                asset.attack_seconds,
                asset.flash_seconds,
            );
    }
}

#[derive(Default)]
struct PreviewCapture {
    rest: bool,
    flash: bool,
}

// Optional verification captures use the real audio envelope, never a test pulse.
fn capture_pulse_preview(
    mut commands: Commands,
    time: Res<Time>,
    bus: Res<PulseBus>,
    asset: Option<Res<BugAsset>>,
    mut captured: Local<PreviewCapture>,
) {
    if time.elapsed_secs() < 8. || captured.flash {
        return;
    }
    let Some(prefix) =
        std::env::args().find_map(|arg| arg.strip_prefix("--pulse-preview=").map(str::to_owned))
    else {
        return;
    };
    let Some(asset) = asset else {
        return;
    };
    let now = Instant::now();
    let level = (0..BUG_POSITIONS.len())
        .map(|index| {
            bus.level(
                bug_clock(index),
                now,
                asset.attack_seconds,
                asset.flash_seconds,
            )
        })
        .fold(0., f32::max);
    let suffix = if !captured.rest && level == 0. {
        captured.rest = true;
        Some("rest")
    } else if captured.rest && !captured.flash && level > 0.25 {
        captured.flash = true;
        Some("flash")
    } else {
        None
    };
    if let Some(suffix) = suffix {
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(format!(
                "{prefix}_{suffix}.png"
            )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn collecting_removes_only_that_bugs_model_light_and_shadow() {
        let mut app = App::new();
        app.add_message::<super::super::resources::ResourceCollected>()
            .add_systems(PostUpdate, remove_collected_bugs);
        let mut removed = Vec::new();
        for _ in 0..3 {
            let root = app.world_mut().spawn(BugInstance(0)).id();
            app.world_mut().spawn(ChildOf(root));
            removed.push(root);
        }
        let other = app.world_mut().spawn(BugInstance(1)).id();
        app.world_mut()
            .write_message(super::super::resources::ResourceCollected {
                instance: "lightning_bug-0".into(),
                yields: Vec::new(),
            });
        app.update();
        for entity in removed {
            assert!(app.world().get_entity(entity).is_err());
        }
        assert!(app.world().get_entity(other).is_ok());
        assert_eq!(
            app.world_mut()
                .query::<&ChildOf>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn bugs_hover_one_to_three_metres_above_plate_rims() {
        for index in 0..BUG_POSITIONS.len() {
            let position = bug_position(index);
            let ground = super::super::terrain::height(Vec2::new(position.x, position.z)) + 0.035;
            assert!((1.0..=3.0).contains(&(position.y - ground)));
        }
    }

    #[test]
    fn shared_audio_ticks_reach_followers_with_original_sample_time() {
        let mut app = App::new();
        app.add_message::<ClockTick>()
            .add_message::<SamplePulse>()
            .init_resource::<PulseBus>()
            .add_systems(Update, poll_audio);
        let at = Instant::now();
        app.world_mut().write_message(ClockTick {
            module: ModuleId(2),
            sample_time: 48_000,
            at: Some(at),
        });
        app.update();
        let bus = app.world().resource::<PulseBus>();
        assert_eq!(bus.latest[&2].sample_time, 48_000);
        assert_eq!(bus.latest[&2].at, at);
        assert!(!bus.latest.contains_key(&1));
    }

    #[test]
    fn followers_share_timestamp_and_flash_has_exactly_invisible_rest() {
        let at = Instant::now();
        let mut bus = PulseBus::default();
        assert_eq!(bus.level(1, at, 0.015, 0.18), 0.);
        bus.queue(SamplePulse {
            clock: 1,
            sample_time: 48000,
            at: at + Duration::from_secs(1),
        });
        bus.advance(at);
        assert_eq!(bus.level(1, at, 0.015, 0.18), 0.);
        let now = at + Duration::from_millis(1015);
        bus.advance(now);
        let followers = [PulseFollower::default(), PulseFollower::default()];
        let a = bus.level(followers[0].clock, now, 0.015, 0.18);
        let b = bus.level(followers[1].clock, now, 0.015, 0.18);
        assert_eq!(a, b);
        assert!((a - 1.).abs() < 1e-5);
        assert_eq!(
            bus.level(1, at + Duration::from_millis(1181), 0.015, 0.18),
            0.
        );
        assert_eq!(bus.level(99, now, 0.015, 0.18), 0.);
    }
    #[test]
    fn each_bug_has_its_own_reproducible_clock() {
        use stubby_audio::{PulseClock, RandomClock};
        let mut sequences = Vec::new();
        for index in 0..BUG_POSITIONS.len() {
            let mut clock = RandomClock::new(bug_seed(42, index), 10, 40);
            let sequence: Vec<_> = (0..300).filter(|_| clock.advance()).collect();
            assert!(sequences.iter().all(|previous| *previous != sequence));
            assert_eq!(bug_clock(index), index as u64 + 1);
            sequences.push(sequence);
        }
    }

    #[test]
    fn shipped_bug_has_white_emission_and_a_scale_flash_clip() {
        let asset =
            gltf::Gltf::from_slice(include_bytes!("../../../assets/models/lightning_bug.glb"))
                .expect("lightning bug must remain a valid GLB");
        let root = asset
            .nodes()
            .find(|node| node.name() == Some("LightningBug_FlashRoot"))
            .expect("flash root");
        assert_eq!(
            root.transform().decomposed().2,
            [1.; 3],
            "runtime starts from a full-size authored pose"
        );
        let clip = asset
            .animations()
            .find(|clip| clip.name() == Some("white_flash"))
            .expect("named preview animation");
        assert!(
            clip.channels()
                .any(|channel| channel.target().property() == gltf::animation::Property::Scale)
        );
        assert!(
            asset
                .materials()
                .all(|material| material.emissive_factor() == [1.; 3])
        );
        let profile: BugAsset = serde_json::from_str(include_str!(
            "../../../assets/creatures/lightning_bug/asset.json"
        ))
        .unwrap();
        assert_eq!(profile.flash_seconds, 0.3);
        assert!(profile.attack_seconds > 0. && profile.attack_seconds < profile.flash_seconds);
        assert!(profile.clock.min_interval_seconds > profile.flash_seconds);
    }

    #[test]
    fn delayed_event_does_not_rewind_a_follower() {
        let now = Instant::now();
        let mut bus = PulseBus::default();
        bus.queue(SamplePulse {
            clock: 1,
            sample_time: 200,
            at: now,
        });
        bus.queue(SamplePulse {
            clock: 1,
            sample_time: 100,
            at: now - Duration::from_secs(1),
        });
        bus.advance(now);
        assert_eq!(bus.latest[&1].sample_time, 200);
    }
}
