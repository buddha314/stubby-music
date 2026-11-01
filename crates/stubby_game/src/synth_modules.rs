//! Module assets bound to the synth. Placing a module asset adds the real
//! `stubby_audio` module behind it; its parameters drive the engine; and its lights,
//! display and knobs follow the engine. The clock's lamp and GATE LED flash on the
//! engine's sample-stamped ticks (through the shared pulse bus), so they keep time
//! with whatever BPM the module is set to.
//!
//! Logic and visuals are separate: `ModuleLights` holds each module's light levels
//! (0..1), computed from the engine every frame, and a visual pass applies them to the
//! spawned model's materials, segment nodes, knob nodes and point light.
//!
//! `--module-demo` places a clock and a square VCO in front of spawn.

use super::audio::Synth;
use super::pulse::{PulseBus, PulseBusPlugin, PulseReceive};
use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;
use stubby_audio::{Command, ModuleId, ModuleKind, PortRef};

/// Profiles embedded at build time, as the lightning bug's is, so the game and the
/// Jackdaw adapter read identical data.
const PROFILES: [&str; 7] = [
    include_str!("../../../assets/modules/square_vco/asset.json"),
    include_str!("../../../assets/modules/saw_vco/asset.json"),
    include_str!("../../../assets/modules/triangle_vco/asset.json"),
    include_str!("../../../assets/modules/sine_vco/asset.json"),
    include_str!("../../../assets/modules/clock_lamp/asset.json"),
    include_str!("../../../assets/modules/burbler_clock/asset.json"),
    include_str!("../../../assets/modules/volume/asset.json"),
];

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleProfile {
    pub id: String,
    pub model: String,
    pub module_kind: String,
    #[serde(default)]
    pub params: Vec<ParamProfile>,
    #[serde(default)]
    pub lights: Vec<LightProfile>,
    #[serde(default)]
    pub lamp: Option<LampProfile>,
    #[serde(default)]
    pub display: Option<DisplayProfile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParamProfile {
    pub param: String,
    pub min: f32,
    pub max: f32,
    pub taper: String,
    pub rotation_degrees: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LightProfile {
    pub light: String,
    pub rest_strength: f32,
    pub peak_strength: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LampProfile {
    pub node: String,
    pub material: String,
    pub attack_seconds: f32,
    pub flash_seconds: f32,
    pub rest_strength: f32,
    pub peak_strength: f32,
    pub light_lumens: f32,
    pub light_range_metres: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisplayProfile {
    pub shows: String,
    pub digits: usize,
    pub digit_segments: HashMap<String, String>,
}

/// Every module profile, by asset id.
#[derive(Resource)]
pub struct ModuleCatalog(pub HashMap<String, ModuleProfile>);

impl Default for ModuleCatalog {
    fn default() -> Self {
        Self(
            PROFILES
                .iter()
                .map(|json| {
                    let profile: ModuleProfile = serde_json::from_str(json)
                        .unwrap_or_else(|e| panic!("module profile: {e}"));
                    (profile.id.clone(), profile)
                })
                .collect(),
        )
    }
}

/// A placed module: which engine module it is and which asset shows it.
#[derive(Component, Debug, Clone)]
pub struct SynthModule {
    pub id: ModuleId,
    pub asset: String,
}

/// The module's parameter values, by name (e.g. `freq`, `bpm`). Change these to drive
/// the engine; the knobs and display follow.
#[derive(Component, Debug, Clone, Default)]
pub struct ModuleParams(pub HashMap<String, f32>);

/// Light levels (0..1) computed from the engine each frame: `lamp` for the clock light,
/// and one per panel LED by light name (`gate`, `phase`, `out`).
#[derive(Component, Debug, Clone, Default)]
pub struct ModuleLights {
    pub lamp: f32,
    pub leds: HashMap<String, f32>,
}

/// The engine module kind behind a placed module, once added.
#[derive(Component, Debug, Clone, Copy)]
struct EngineModule(ModuleKind);

/// Parameter values last sent to the engine.
#[derive(Component, Default)]
struct SentParams(HashMap<String, f32>);

/// The spawned model's parts, found once its scene is ready. Glowing materials are
/// per-instance copies, so two modules flash independently.
#[derive(Component, Default)]
struct ModuleParts {
    lamp: Vec<(Handle<StandardMaterial>, LinearRgba)>,
    leds: HashMap<String, Vec<(Handle<StandardMaterial>, LinearRgba)>>,
    segments: HashMap<(usize, char), Entity>,
    knobs: HashMap<String, (Entity, Quat)>,
    point_light: Option<Entity>,
}

#[derive(Default)]
pub struct SynthModulesPlugin {
    /// Place a clock and a square VCO in front of spawn.
    pub showcase: bool,
}

// Reserved showcase IDs; shared allocator migration is tracked in task 2.1.
const SHOWCASE_CLOCK: ModuleId = ModuleId(8_001);
const SHOWCASE_SQUARE: ModuleId = ModuleId(8_002);
const SHOWCASE_ENV: ModuleId = ModuleId(8_003);
const SHOWCASE_VCA: ModuleId = ModuleId(8_004);

#[derive(Resource, Default)]
struct ShowcaseAudio {
    patched: bool,
}

#[derive(SystemSet, Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ModuleEngine;
/// A second view of an existing module, such as a machine's HUD; owns no audio node.
#[derive(Component)]
pub(crate) struct ModuleView;
pub(crate) fn unbind_module(world: &mut World, entity: Entity) {
    if let Some(light) = world
        .get::<ModuleParts>(entity)
        .and_then(|parts| parts.point_light)
    {
        world.despawn(light);
    }
    world.entity_mut(entity).remove::<(
        SynthModule,
        EngineModule,
        SentParams,
        ModuleParts,
        ModuleLights,
    )>();
}
impl Plugin for SynthModulesPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PulseBusPlugin>() {
            app.add_plugins(PulseBusPlugin);
        }
        app.init_resource::<ModuleCatalog>().add_systems(
            Update,
            (
                (add_to_engine, send_param_changes, remove_from_engine).in_set(ModuleEngine),
                audition_showcase,
                update_lights.after(PulseReceive),
                (find_parts, apply_visuals).chain(),
            )
                .chain(),
        );
        if self.showcase || std::env::args().any(|a| a == "--module-demo") {
            app.init_resource::<ShowcaseAudio>()
                .add_systems(Startup, place_showcase);
        }
    }
}

/// Place a module asset in the world, bound to engine module `id`.
pub fn spawn_module(
    commands: &mut Commands,
    assets: &AssetServer,
    catalog: &ModuleCatalog,
    asset: &str,
    id: ModuleId,
    transform: Transform,
    params: &[(&str, f32)],
) -> Option<Entity> {
    let profile = catalog.0.get(asset)?;
    Some(
        commands
            .spawn((
                WorldAssetRoot(assets.load(profile.model.clone())),
                transform,
                SynthModule {
                    id,
                    asset: asset.to_owned(),
                },
                ModuleParams(params.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect()),
                ModuleLights::default(),
                Name::new(format!("Module {asset} #{}", id.0)),
            ))
            .id(),
    )
}

fn place_showcase(mut commands: Commands, assets: Res<AssetServer>, catalog: Res<ModuleCatalog>) {
    spawn_module(
        &mut commands,
        &assets,
        &catalog,
        "clock_lamp",
        SHOWCASE_CLOCK,
        Transform::from_xyz(-1.3, -0.12, 2.2),
        &[("bpm", 120.0)],
    );
    spawn_module(
        &mut commands,
        &assets,
        &catalog,
        "square_vco",
        SHOWCASE_SQUARE,
        Transform::from_xyz(-0.2, -0.12, 2.0),
        &[("freq", 110.0)],
    );
}

/// Audition the *placed* clock and square oscillator. The lamp and audible burst
/// share one clock module, rather than two independent clocks at the same BPM.
fn showcase_patch() -> [Command; 6] {
    [
        Command::AddModule {
            id: SHOWCASE_ENV,
            kind: ModuleKind::Envelope {
                attack_s: 0.002,
                decay_s: 0.12,
            },
        },
        Command::AddModule {
            id: SHOWCASE_VCA,
            kind: ModuleKind::Vca { gain: 0.15 },
        },
        Command::Connect {
            from: PortRef::new(SHOWCASE_CLOCK, "gate"),
            to: PortRef::new(SHOWCASE_ENV, "gate"),
        },
        Command::Connect {
            from: PortRef::new(SHOWCASE_ENV, "out"),
            to: PortRef::new(SHOWCASE_VCA, "cv"),
        },
        Command::Connect {
            from: PortRef::new(SHOWCASE_SQUARE, "out"),
            to: PortRef::new(SHOWCASE_VCA, "in"),
        },
        Command::Connect {
            from: PortRef::new(SHOWCASE_VCA, "out"),
            to: PortRef::output(),
        },
    ]
}
fn audition_showcase(
    state: Option<ResMut<ShowcaseAudio>>,
    synth: Option<Res<Synth>>,
    modules: Query<&SynthModule, With<EngineModule>>,
) {
    let (Some(mut state), Some(synth)) = (state, synth) else {
        return;
    };
    if !synth.is_running() {
        return;
    }
    let ready = modules.iter().any(|module| module.id == SHOWCASE_CLOCK)
        && modules.iter().any(|module| module.id == SHOWCASE_SQUARE);
    if ready && !state.patched {
        for command in showcase_patch() {
            synth.send(command);
        }
        state.patched = true;
    } else if !ready && state.patched {
        for id in [SHOWCASE_VCA, SHOWCASE_ENV] {
            synth.send(Command::RemoveModule { id });
        }
        state.patched = false;
    }
}

/// The engine module for a profile, with its starting parameters.
fn engine_kind(profile: &ModuleProfile, params: &ModuleParams) -> Option<ModuleKind> {
    let get = |name: &str, default: f32| params.0.get(name).copied().unwrap_or(default);
    Some(match profile.module_kind.as_str() {
        "SineVco" => ModuleKind::SineVco {
            freq_hz: get("freq", 261.63),
        },
        "SquareVco" => ModuleKind::SquareVco {
            freq_hz: get("freq", 261.63),
        },
        "Clock" => ModuleKind::Clock {
            bpm: get("bpm", 120.0),
        },
        "Vca" => ModuleKind::Vca {
            gain: get("gain", 0.7),
        },
        // Saw and triangle oscillators don't exist in the engine yet.
        _ => return None,
    })
}

fn add_to_engine(
    mut commands: Commands,
    synth: Option<Res<Synth>>,
    catalog: Res<ModuleCatalog>,
    added: Query<(Entity, &SynthModule, &ModuleParams), (Added<SynthModule>, Without<ModuleView>)>,
) {
    for (entity, module, params) in &added {
        let Some(profile) = catalog.0.get(&module.asset) else {
            warn!("no module profile {:?}", module.asset);
            continue;
        };
        let Some(kind) = engine_kind(profile, params) else {
            warn!(
                "{} has no engine module yet; it will show but not sound",
                profile.module_kind
            );
            continue;
        };
        if let Some(synth) = &synth {
            synth.send(Command::AddModule {
                id: module.id,
                kind,
            });
        }
        commands
            .entity(entity)
            .insert((EngineModule(kind), SentParams(params.0.clone())));
    }
}

fn send_param_changes(
    synth: Option<Res<Synth>>,
    mut changed: Query<
        (&SynthModule, &EngineModule, &ModuleParams, &mut SentParams),
        Changed<ModuleParams>,
    >,
) {
    for (module, engine, params, mut sent) in &mut changed {
        for (name, &value) in &params.0 {
            if sent.0.get(name) == Some(&value) {
                continue;
            }
            // The engine names its parameters with `&'static str`; take it from the kind.
            if let (Some(synth), Some(spec)) = (&synth, engine.0.param(name)) {
                synth.send(Command::SetParam {
                    module: module.id,
                    param: spec.name,
                    value,
                });
            }
            sent.0.insert(name.clone(), value);
        }
    }
}

fn remove_from_engine(
    synth: Option<Res<Synth>>,
    mut removed: RemovedComponents<EngineModule>,
    mut ids: Local<HashMap<Entity, ModuleId>>,
    live: Query<(Entity, &SynthModule), With<EngineModule>>,
) {
    for entity in removed.read() {
        if let (Some(id), Some(synth)) = (ids.remove(&entity), &synth) {
            synth.send(Command::RemoveModule { id });
        }
    }
    for (entity, module) in &live {
        ids.insert(entity, module.id);
    }
}

/// Fraction of the current beat elapsed, from the last tick and the tempo.
fn beat_phase(since_tick_seconds: f32, bpm: f32) -> f32 {
    let beat = 60.0 / bpm.max(1.0);
    (since_tick_seconds / beat).fract()
}

fn update_lights(
    bus: Res<PulseBus>,
    catalog: Res<ModuleCatalog>,
    synth: Option<Res<Synth>>,
    mut modules: Query<(
        &SynthModule,
        &ModuleParams,
        &mut ModuleLights,
        Has<EngineModule>,
    )>,
) {
    let now = Instant::now();
    let sounding = synth.as_ref().is_some_and(|s| s.is_running());
    for (module, params, mut lights, engine) in &mut modules {
        let Some(profile) = catalog.0.get(&module.asset) else {
            continue;
        };
        if let Some(lamp) = &profile.lamp {
            // The flash envelope of the latest engine tick that has reached its instant.
            let gate = bus.level(module.id.0, now, lamp.attack_seconds, lamp.flash_seconds);
            lights.lamp = gate;
            for led in &profile.lights {
                let level = match led.light.as_str() {
                    "gate" => gate,
                    "phase" => bus.last(module.id.0).map_or(0.0, |pulse| {
                        let since = now.saturating_duration_since(pulse.at).as_secs_f32();
                        let bpm = params.0.get("bpm").copied().unwrap_or(120.0);
                        1.0 - beat_phase(since, bpm)
                    }),
                    _ => 0.0,
                };
                lights.leds.insert(led.light.clone(), level);
            }
        } else {
            // Oscillator LEDs show the module is live. They follow output level once
            // the rack publishes meters (add-rack-and-spatial-audio task 3.2).
            let on = if engine && sounding { 0.5 } else { 0.0 };
            for led in &profile.lights {
                lights.leds.insert(led.light.clone(), on);
            }
        }
    }
}

/// Once a module's scene has spawned, find its glowing materials, display segments
/// and knobs, and give it per-instance copies of the glowing materials.
#[allow(clippy::too_many_arguments)] // a Bevy system's parameters
fn find_parts(
    mut commands: Commands,
    catalog: Res<ModuleCatalog>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    modules: Query<(Entity, &SynthModule), Without<ModuleParts>>,
    children: Query<&Children>,
    names: Query<&Name>,
    meshes: Query<(&MeshMaterial3d<StandardMaterial>, Option<&GltfMaterialName>)>,
    transforms: Query<&Transform>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    for (root, module) in &modules {
        let Some(profile) = catalog.0.get(&module.asset) else {
            continue;
        };
        let all: Vec<Entity> = children.iter_descendants(root).collect();
        if !all.iter().any(|&e| meshes.contains(e)) {
            continue; // scene not spawned yet
        }
        let mut parts = ModuleParts::default();
        let own =
            |commands: &mut Commands, entity: Entity, materials: &mut Assets<StandardMaterial>| {
                let (handle, _) = meshes.get(entity).ok()?;
                let copy = materials.get(&handle.0)?.clone();
                let base = copy.emissive;
                let handle = materials.add(copy);
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(handle.clone()));
                Some((handle, base))
            };
        let lamp_material = profile.lamp.as_ref().map(|l| l.material.as_str());
        for &entity in &all {
            if let (Ok((_, Some(material))), Some(lamp)) = (meshes.get(entity), lamp_material)
                && material.0 == lamp
                && let Some(m) = own(&mut commands, entity, materials)
            {
                parts.lamp.push(m);
            }
            let Ok(name) = names.get(entity) else {
                continue;
            };
            let name = name.as_str();
            if let Some(light) = name.strip_prefix("Light_") {
                for child in children.iter_descendants(entity) {
                    if let Some(m) = own(&mut commands, child, materials) {
                        parts.leds.entry(light.to_owned()).or_default().push(m);
                    }
                }
            } else if let Some(rest) = name.strip_prefix("Display_d") {
                let mut it = rest.split('_');
                if let (Some(d), Some(s)) = (it.next(), it.next())
                    && let (Ok(d), Some(s)) = (d.parse(), s.chars().next())
                {
                    parts.segments.insert((d, s), entity);
                }
            } else if let Some(param) = name.strip_prefix("Knob_")
                && let Ok(t) = transforms.get(entity)
            {
                parts.knobs.insert(param.to_owned(), (entity, t.rotation));
            } else if Some(name) == profile.lamp.as_ref().map(|l| l.node.as_str())
                && let Some(lamp) = &profile.lamp
            {
                let light = commands
                    .spawn((
                        PointLight {
                            color: Color::srgb(1.0, 0.8, 0.6),
                            intensity: 0.0,
                            range: lamp.light_range_metres,
                            shadow_maps_enabled: false,
                            ..default()
                        },
                        Transform::default(),
                        ChildOf(entity),
                    ))
                    .id();
                parts.point_light = Some(light);
            }
        }
        commands.entity(root).insert(parts);
    }
}

/// Which segments light for a value on a display of `digits` digits, right-aligned.
fn lit_segments(value: f32, digits: usize, table: &HashMap<String, String>) -> Vec<(usize, char)> {
    let max = 10u32.pow(digits as u32) - 1;
    let text = format!(
        "{:>width$}",
        (value.round().max(0.0) as u32).min(max),
        width = digits
    );
    text.chars()
        .enumerate()
        .flat_map(|(d, ch)| {
            table
                .get(&ch.to_string())
                .map(|segs| segs.chars().map(move |s| (d, s)).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .collect()
}

/// Knob angle (radians, clockwise positive) for a parameter value.
fn knob_angle(value: f32, p: &ParamProfile) -> f32 {
    let v = value.clamp(p.min, p.max);
    let norm = if p.taper == "log" && p.min > 0.0 {
        (v / p.min).ln() / (p.max / p.min).ln()
    } else {
        (v - p.min) / (p.max - p.min)
    };
    (norm - 0.5) * p.rotation_degrees.to_radians()
}

fn apply_visuals(
    catalog: Res<ModuleCatalog>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    modules: Query<(&SynthModule, &ModuleParams, &ModuleLights, &ModuleParts)>,
    mut visibility: Query<&mut Visibility>,
    mut transforms: Query<&mut Transform>,
    mut point_lights: Query<&mut PointLight>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    for (module, params, lights, parts) in &modules {
        let Some(profile) = catalog.0.get(&module.asset) else {
            continue;
        };
        // Materials are exported at peak strength; scale between rest and peak.
        if let Some(lamp) = &profile.lamp {
            let strength =
                lamp.rest_strength + (lamp.peak_strength - lamp.rest_strength) * lights.lamp;
            for (handle, base) in &parts.lamp {
                if let Some(mut m) = materials.get_mut(handle) {
                    m.emissive = *base * (strength / lamp.peak_strength);
                }
            }
            if let Some(light) = parts.point_light
                && let Ok(mut light) = point_lights.get_mut(light)
            {
                light.intensity = lamp.light_lumens * lights.lamp;
            }
        }
        for led in &profile.lights {
            let level = lights.leds.get(&led.light).copied().unwrap_or(0.0);
            let strength = led.rest_strength + (led.peak_strength - led.rest_strength) * level;
            for (handle, base) in parts.leds.get(&led.light).into_iter().flatten() {
                if let Some(mut m) = materials.get_mut(handle) {
                    m.emissive = *base * (strength / led.peak_strength);
                }
            }
        }
        if let Some(display) = &profile.display
            && let Some(&value) = params.0.get(&display.shows)
        {
            let lit = lit_segments(value, display.digits, &display.digit_segments);
            for (key, &entity) in &parts.segments {
                if let Ok(mut v) = visibility.get_mut(entity) {
                    let want = if lit.contains(key) {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                    v.set_if_neq(want);
                }
            }
        }
        for p in &profile.params {
            if let (Some(&value), Some(&(entity, base))) =
                (params.0.get(&p.param), parts.knobs.get(&p.param))
                && let Ok(mut t) = transforms.get_mut(entity)
            {
                // Local +Z points out of the panel, so clockwise as seen is negative.
                t.rotation = base * Quat::from_rotation_z(-knob_angle(value, p));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{AudioOutput, SynthAudioPlugin};
    use std::time::Duration;

    #[test]
    fn showcase_audition_waits_for_sources_and_retires_its_patch() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SynthAudioPlugin {
                output: AudioOutput::Offline,
                demo: false,
            })
            .add_plugins(SynthModulesPlugin { showcase: false });
        app.update();
        assert!(
            !app.world().contains_resource::<ShowcaseAudio>(),
            "normal startup is not an audition"
        );
        app.init_resource::<ShowcaseAudio>();
        let clock = app
            .world_mut()
            .spawn((
                SynthModule {
                    id: SHOWCASE_CLOCK,
                    asset: "clock_lamp".into(),
                },
                ModuleParams(HashMap::from([("bpm".into(), 120.)])),
                ModuleLights::default(),
            ))
            .id();
        app.update();
        assert!(!app.world().resource::<ShowcaseAudio>().patched);
        app.world_mut().spawn((
            SynthModule {
                id: SHOWCASE_SQUARE,
                asset: "square_vco".into(),
            },
            ModuleParams(HashMap::from([("freq".into(), 110.)])),
            ModuleLights::default(),
        ));
        app.update();
        assert!(app.world().resource::<ShowcaseAudio>().patched);
        app.world_mut().despawn(clock);
        app.update();
        assert!(!app.world().resource::<ShowcaseAudio>().patched);
    }

    #[test]
    fn showcase_sound_uses_the_lamps_clock_and_is_silent_between_beats() {
        use stubby_audio::{Engine, EngineConfig, Event};
        let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
        engine
            .send(Command::AddModule {
                id: SHOWCASE_CLOCK,
                kind: ModuleKind::Clock { bpm: 120. },
            })
            .unwrap();
        engine
            .send(Command::AddModule {
                id: SHOWCASE_SQUARE,
                kind: ModuleKind::SquareVco { freq_hz: 110. },
            })
            .unwrap();
        for command in showcase_patch() {
            engine.send(command).unwrap();
        }
        engine.flush().unwrap();
        let mut samples = vec![0.; 48_000 * 2];
        renderer.render(&mut samples);
        let mut ticks = Vec::new();
        engine.drain_events(&mut ticks);
        assert_eq!(
            ticks,
            vec![
                Event::ClockTick {
                    module: SHOWCASE_CLOCK,
                    sample_time: 0
                },
                Event::ClockTick {
                    module: SHOWCASE_CLOCK,
                    sample_time: 24_000
                },
            ]
        );
        for start in [0, 24_000] {
            assert!(
                samples[(start + 100) * 2..(start + 2400) * 2]
                    .iter()
                    .any(|sample| sample.abs() > 0.05)
            );
            assert!(
                samples[(start + 12_000) * 2..(start + 23_000) * 2]
                    .iter()
                    .all(|sample| sample.abs() < 0.00001)
            );
        }
    }

    #[test]
    fn every_shipped_profile_parses() {
        let catalog = ModuleCatalog::default();
        for id in [
            "square_vco",
            "saw_vco",
            "triangle_vco",
            "sine_vco",
            "clock_lamp",
        ] {
            assert!(catalog.0.contains_key(id), "missing {id}");
        }
        assert!(catalog.0["clock_lamp"].lamp.is_some());
        assert_eq!(catalog.0["clock_lamp"].display.as_ref().unwrap().digits, 3);
    }

    #[test]
    fn display_right_aligns_and_clamps() {
        let table = ModuleCatalog::default().0["clock_lamp"]
            .display
            .clone()
            .unwrap()
            .digit_segments;
        let lit = lit_segments(120.0, 3, &table);
        // "120": 1 = bc, 2 = abdeg, 0 = abcdef.
        assert_eq!(lit.iter().filter(|(d, _)| *d == 0).count(), 2);
        assert_eq!(lit.iter().filter(|(d, _)| *d == 1).count(), 5);
        assert_eq!(lit.iter().filter(|(d, _)| *d == 2).count(), 6);
        // "  7": leading digits dark.
        let seven = lit_segments(7.0, 3, &table);
        assert!(seven.iter().all(|(d, _)| *d == 2));
        // Values beyond the display clamp to 999.
        assert_eq!(lit_segments(5000.0, 3, &table).len(), 3 * 6);
    }

    #[test]
    fn knob_tapers() {
        let lin = ParamProfile {
            param: "bpm".into(),
            min: 0.0,
            max: 200.0,
            taper: "linear".into(),
            rotation_degrees: 300.0,
        };
        assert!(
            (knob_angle(100.0, &lin)).abs() < 1e-6,
            "midpoint is straight up"
        );
        assert!((knob_angle(200.0, &lin) - 150f32.to_radians()).abs() < 1e-5);
        let log = ParamProfile {
            min: 1.0,
            max: 10_000.0,
            taper: "log".into(),
            ..lin
        };
        assert!(
            knob_angle(100.0, &log).abs() < 1e-5,
            "log midpoint is the geometric mean"
        );
    }

    /// Run frames for `seconds`. Returns the scheduled instant of each engine tick that
    /// reached the module's light, and how many separate flashes the light showed.
    fn run_clock(app: &mut App, entity: Entity, seconds: f32) -> (Vec<Instant>, usize) {
        let (mut ticks, mut flashes, mut prev, mut last_sample) = (Vec::new(), 0, 0.0, None);
        let end = Instant::now() + Duration::from_secs_f32(seconds);
        while Instant::now() < end {
            app.update();
            if let Some(pulse) = app.world().resource::<PulseBus>().last(42)
                && last_sample != Some(pulse.sample_time)
            {
                last_sample = Some(pulse.sample_time);
                ticks.push(pulse.at);
            }
            let level = app.world().get::<ModuleLights>(entity).unwrap().lamp;
            if prev <= 0.5 && level > 0.5 {
                flashes += 1;
            }
            prev = level;
            std::thread::sleep(Duration::from_millis(2));
        }
        (ticks, flashes)
    }

    fn spacing(times: &[Instant]) -> Vec<f32> {
        times
            .windows(2)
            .map(|w| w[1].duration_since(w[0]).as_secs_f32())
            .collect()
    }

    #[test]
    fn clock_light_flashes_in_time_with_its_bpm() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SynthAudioPlugin {
                output: AudioOutput::Offline,
                demo: false,
            })
            .add_plugins(SynthModulesPlugin { showcase: false });
        let clock = app
            .world_mut()
            .spawn((
                SynthModule {
                    id: ModuleId(42),
                    asset: "clock_lamp".into(),
                },
                ModuleParams(HashMap::from([("bpm".to_owned(), 120.0)])),
                ModuleLights::default(),
            ))
            .id();

        // The flashes are scheduled on the engine's ticks, so they sit exactly one beat
        // apart in time (up to the audio clock's snapshot jitter), and the light flashes
        // once per tick. Frame timing doesn't enter into it.
        let (ticks, flashes) = run_clock(&mut app, clock, 2.3);
        let at_120 = spacing(&ticks);
        assert!(at_120.len() >= 3, "too few ticks: {at_120:?}");
        assert!(
            at_120.iter().all(|s| (s - 0.5).abs() < 0.015),
            "120 BPM spacing {at_120:?}"
        );
        assert!(
            flashes.abs_diff(ticks.len()) <= 1,
            "{flashes} flashes for {} ticks",
            ticks.len()
        );

        // Turn the tempo up: the engine follows, and so does the light.
        app.world_mut()
            .get_mut::<ModuleParams>(clock)
            .unwrap()
            .0
            .insert("bpm".into(), 240.0);
        let _settle = run_clock(&mut app, clock, 0.6);
        let (ticks, flashes) = run_clock(&mut app, clock, 1.6);
        let at_240 = spacing(&ticks);
        assert!(at_240.len() >= 4, "too few ticks: {at_240:?}");
        assert!(
            at_240.iter().all(|s| (s - 0.25).abs() < 0.015),
            "240 BPM spacing {at_240:?}"
        );
        assert!(
            flashes.abs_diff(ticks.len()) <= 1,
            "{flashes} flashes for {} ticks",
            ticks.len()
        );
    }
}
