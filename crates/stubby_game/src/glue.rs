//! Glue between systems (split-game-into-systems): the rules that join two systems live
//! here, never inside either.
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy_stubby_audio::{
    Synth,
    module_ids::ModuleIds,
    pulse::{PulseAliases, PulseFollower},
};
use bevy_stubby_rack::machine_config::Profiles;
use bevy_stubby_rack::synth_modules::{ModuleCatalog, ModuleLights, ModuleParams, SynthModule};
use bevy_stubby_recipes::{MineNode, RecipeCatalog, ResourceNode};
use std::collections::BTreeMap;
use stubby_audio::{Command, ModuleKind};
use stubby_core::{AssetClock, AuthoredLevel, PlacedAsset, PrefabError};
use stubby_world::prefabs::{AssetBinder, AssetBinders, Bound, allocate};

pub struct GluePlugin;
impl Plugin for GluePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, segment_cost);
        // A VCO starts with its knob straight up (its module data's default, C#3), not at
        // the bug's pitch, which was shrill (user, 2026-11-18).
        app.add_systems(
            Update,
            (
                interact_points
                    .after(stubby_core::PrefabBinding)
                    .before(stubby_core::InteractionTargeting),
                attach_air_cleaners.after(stubby_core::PrefabBinding),
                hide_collected_finds.after(stubby_core::PrefabBinding),
                report_configured_machines.before(bevy_stubby_recipes::RecipesSystems),
                machine_targets.before(stubby_core::MachineInteractionSet),
            ),
        )
        .add_systems(
            PostUpdate,
            publish_acoustic_scene.after(TransformSystems::Propagate),
        );
    }
}

/// A machine's targets are rules (`machines.json`, add-machine-rules): copy them into the
/// rack's profiles whenever the rules load or reload, so a saved edit reaches a running
/// game. A machine with no row has no targets.
fn machine_targets(
    rules: Option<Res<bevy_stubby_recipes::MachineRules>>,
    profiles: Option<ResMut<Profiles>>,
) {
    let (Some(rules), Some(mut profiles)) = (rules, profiles) else {
        return;
    };
    if !rules.is_changed() && !profiles.is_added() {
        return;
    }
    for (machine, config) in profiles.0.iter_mut() {
        config.targets = rack_targets(rules.of(machine));
    }
}

/// A machine's rule targets as the rack reads them.
pub(crate) fn rack_targets(
    row: Option<&stubby_recipes::machines::MachineDefinition>,
) -> Vec<stubby_rack::Target> {
    row.map(|row| {
        row.targets
            .iter()
            .map(|t| stubby_rack::Target {
                place: t.place.clone(),
                knob: t.knob.clone(),
                note: t.note.clone(),
                value: t.value,
            })
            .collect()
    })
    .unwrap_or_default()
}

/// A rack completed is the `machine_configured` goal condition for that machine's mine
/// ("Build a heartbeat"): the rack and the goals meet here. Unpatching later sends
/// nothing, so a completed chore stays complete.
fn report_configured_machines(
    mut configured: MessageReader<bevy_stubby_rack::rack::MachineConfigured>,
    mines: Query<&MineNode>,
    mut goals: MessageWriter<bevy_stubby_recipes::GoalRequest>,
) {
    for event in configured.read() {
        if let Ok(mine) = mines.get(event.machine) {
            goals.write(bevy_stubby_recipes::GoalRequest(
                stubby_recipes::GoalEvent::ConditionMet(
                    stubby_recipes::Condition::MachineConfigured {
                        machine: mine.mine.clone(),
                    },
                ),
            ));
        }
    }
}

/// Send marked static meshes as a single world-space acoustic scene snapshot.
fn publish_acoustic_scene(
    synth: Option<Res<Synth>>,
    meshes: Res<Assets<Mesh>>,
    changed_occluders: Query<
        (),
        (
            Or<(
                Added<stubby_core::AcousticOccluder>,
                Changed<stubby_core::AcousticOccluder>,
                Changed<Mesh3d>,
                Changed<GlobalTransform>,
            )>,
            With<stubby_core::AcousticOccluder>,
        ),
    >,
    occluders: Query<(
        &stubby_core::AcousticOccluder,
        &Mesh3d,
        &GlobalTransform,
        Option<&Name>,
    )>,
    mut removed: RemovedComponents<stubby_core::AcousticOccluder>,
    mut initialized: Local<bool>,
) {
    let removed_any = removed.read().count() > 0;
    if *initialized && changed_occluders.is_empty() && !removed_any {
        return;
    }
    *initialized = true;
    let Some(synth) = synth else { return };
    let mut geometry = stubby_audio::AcousticSceneGeometry::default();
    let trace = std::env::var_os("STUBBY_AUDIO_SPATIAL_TRACE").is_some();
    for (material, handle, transform, name) in &occluders {
        let Some(mesh) = meshes.get(&handle.0) else {
            continue;
        };
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            continue;
        }
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        let base = geometry.vertices.len() as u32;
        geometry.vertices.extend(positions.iter().map(|position| {
            transform
                .transform_point(Vec3::from_array(*position))
                .to_array()
        }));
        if trace && material.transmission[0] < 0.02 {
            let vertices = &geometry.vertices[base as usize..];
            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            for vertex in vertices {
                for axis in 0..3 {
                    min[axis] = min[axis].min(vertex[axis]);
                    max[axis] = max[axis].max(vertex[axis]);
                }
            }
            eprintln!(
                "AudioNimbus rock mesh name={:?} vertices={} bounds={min:?}..{max:?}",
                name.map(Name::as_str),
                vertices.len(),
            );
        }
        let indices: Vec<u32> = match mesh.indices() {
            Some(Indices::U16(indices)) => indices.iter().map(|&index| u32::from(index)).collect(),
            Some(Indices::U32(indices)) => indices.clone(),
            None => (0..positions.len() as u32).collect(),
        };
        let material_index = geometry.materials.len();
        geometry.materials.push(stubby_audio::AcousticMaterial {
            absorption: material.absorption,
            scattering: material.scattering,
            transmission: material.transmission,
        });
        for triangle in indices.chunks_exact(3) {
            geometry
                .triangles
                .push([base + triangle[0], base + triangle[1], base + triangle[2]]);
            geometry.material_indices.push(material_index);
        }
    }
    synth.set_acoustic_geometry(geometry);
}

/// Placed foliage named by an `air` rule cleans the air (clean-air-by-foliage): the rules
/// (crafting's data) meet the world's placed assets here, so neither system names the other.
#[allow(clippy::type_complexity)] // Assets with and without a cleaner.
fn attach_air_cleaners(
    mut commands: Commands,
    rules: Option<Res<bevy_stubby_recipes::AirRules>>,
    placed: Query<(Entity, &stubby_core::PlacedAsset), Without<stubby_core::AirCleaner>>,
    mut cleaners: Query<(
        Entity,
        &stubby_core::PlacedAsset,
        &mut stubby_core::AirCleaner,
    )>,
    mut unmatched: Local<std::collections::HashSet<Entity>>,
) {
    // A reload retunes existing cleaners in place (keeping how far they have eased) or
    // removes those no rule names any more.
    if let Some(rules) = rules.as_ref().filter(|rules| rules.is_changed()) {
        for (entity, asset, mut cleaner) in &mut cleaners {
            match rules.0.iter().find(|rule| rule.applies_to(&asset.asset)) {
                Some(rule) => {
                    let strength = cleaner.strength;
                    *cleaner = air_cleaner(rule);
                    cleaner.strength = strength;
                }
                None => {
                    commands.entity(entity).remove::<stubby_core::AirCleaner>();
                }
            }
        }
    }
    let Some(rules) = rules else { return };
    if rules.is_changed() {
        // Rules reloaded (or first seen): every asset is matched again.
        unmatched.clear();
    }
    for (entity, asset) in &placed {
        if unmatched.contains(&entity) {
            continue;
        }
        let Some(rule) = rules.0.iter().find(|rule| rule.applies_to(&asset.asset)) else {
            unmatched.insert(entity);
            continue;
        };
        commands.entity(entity).insert(air_cleaner(rule));
    }
}

fn air_cleaner(rule: &stubby_recipes::air::AirCleanerDefinition) -> stubby_core::AirCleaner {
    use stubby_recipes::air::AirFollows as Rule;
    stubby_core::AirCleaner {
        radius_m: rule.radius_m,
        max_strength: rule.strength,
        follows: match rule.follows {
            Rule::Growth => stubby_core::AirFollows::Growth,
            Rule::WaterOrLight => stubby_core::AirFollows::WaterOrLight,
            Rule::Light => stubby_core::AirFollows::Light,
        },
        stops_when_harvested: rule.stops_when_harvested,
        strength: 0.,
    }
}

/// A machine's interaction point (its profile's `Interact` node), pointing at the machine.
#[derive(Component)]
pub struct InteractPoint {
    pub machine: Entity,
}

/// Glue between machines and the player: each placed machine's `Interact` node (named in
/// its profile) is something E can reach while the machine works, and so is the machine's
/// centre, so E reaches it from every side (2026-11-18: the grotto burbler, whose `Interact`
/// node sits on one side of its pool, could not be configured from the other).
fn interact_points(
    mut commands: Commands,
    profiles: Res<Profiles>,
    unreached: Query<(Entity, &PlacedAsset), Without<InteractPoint>>,
    names: Query<(Entity, &Name), Added<Name>>,
    parents: Query<&ChildOf>,
    machines: Query<&PlacedAsset>,
    points: Query<(Entity, &InteractPoint, Has<stubby_core::Interactable>)>,
    broken: Query<(), With<PrefabError>>,
) {
    for (machine, asset) in &unreached {
        if profiles.0.contains_key(&asset.asset) {
            commands.entity(machine).insert(InteractPoint { machine });
        }
    }
    for (entity, name) in &names {
        let Some(machine) = parents.iter_ancestors(entity).find(|a| {
            machines.get(*a).is_ok_and(|m| {
                profiles
                    .0
                    .get(&m.asset)
                    .is_some_and(|c| c.interact.node == name.as_str())
            })
        }) else {
            continue;
        };
        commands.entity(entity).insert(InteractPoint { machine });
    }
    for (entity, point, marked) in &points {
        let working = !broken.contains(point.machine);
        if working && !marked {
            commands
                .entity(entity)
                .insert(stubby_core::Interactable::Machine(point.machine));
        } else if !working && marked {
            commands
                .entity(entity)
                .remove::<stubby_core::Interactable>();
        }
    }
}

/// A stream segment costs what the rules' `stream_segment` buildable says.
fn segment_cost(mut commands: Commands, catalog: Option<Res<RecipeCatalog>>) {
    let cost = catalog
        .and_then(|c| c.0.build_cost("stream_segment", 1).ok())
        .unwrap_or_default();
    commands.insert_resource(stubby_streams::SegmentCost(
        cost.into_iter().map(|s| (s.item, s.quantity)).collect(),
    ));
}

// ---------------------------------------------------------------- placed-asset binders

/// The game's placed assets come alive here, because each joins the world to another
/// system: a bug is life and a crafting source, a module is the rack, a tree or the
/// burbler is a crafting mine (and the burbler a clock), reeds are a crafting source.
pub struct PrefabGluePlugin;
impl Plugin for PrefabGluePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetBinders>()
            .init_resource::<WorldSources>()
            .add_systems(Update, world_sources.after(stubby_core::PrefabBinding));
        app.world_mut().resource_mut::<AssetBinders>().0.extend([
            AssetBinder {
                matches: |_, asset| asset == "lightning_bug",
                install: install_bug,
                retire: |world, entity| {
                    stubby_life::bug::retire_placed_bug(world, entity);
                    if world.get_entity(entity).is_ok() {
                        world.entity_mut(entity).remove::<ResourceNode>();
                    }
                },
            },
            AssetBinder {
                matches: |world, asset| world.resource::<ModuleCatalog>().0.contains_key(asset),
                install: install_module,
                retire: |world, entity| {
                    if world.get_entity(entity).is_ok() {
                        bevy_stubby_rack::synth_modules::unbind_module(world, entity);
                    }
                },
            },
            AssetBinder {
                matches: |_, asset| asset.starts_with("scrub_oak_") || asset == "mudpit_burbler",
                install: install_mine,
                retire: |world, entity| {
                    if world.get_entity(entity).is_ok() {
                        world.entity_mut(entity).remove::<MineNode>();
                    }
                },
            },
            AssetBinder {
                matches: |_, asset| asset.starts_with("reed_clump_"),
                install: install_reeds,
                retire: |world, entity| {
                    if world.get_entity(entity).is_ok() {
                        world.entity_mut(entity).remove::<ResourceNode>();
                    }
                },
            },
            // Streams bind themselves when laid (streams.rs); a placed segment is scenery.
            AssetBinder {
                matches: |_, asset| asset == "snail_shell",
                install: install_shell,
                retire: |world, entity| {
                    if world.get_entity(entity).is_ok() {
                        world.entity_mut(entity).remove::<(ResourceNode, Find)>();
                    }
                },
            },
            AssetBinder {
                matches: |_, asset| asset == "stream_segment",
                install: |_, _, _, _| Ok(()),
                retire: |_, _| {},
            },
            // Scenery for now: the grotto's boulders, the rock formations, desert props (their rock is
            // acoustic through glTF extras; formation openings become high-pass sources in
            // add-high-pass-sources 3.1), and the campfire until its rack and fire come
            // (add-campfire-craft-bench 3.1).
            AssetBinder {
                matches: |_, asset| {
                    asset.starts_with("boulder_")
                        || asset.starts_with("rock_formation_")
                        || asset.starts_with("desert_")
                        || asset.starts_with("homestead_")
                        || asset.starts_with("dancer_")
                        || asset == "ripple_pool"
                        || asset == "campfire"
                        || asset == "cottage"
                        || asset == "vardo"
                        || asset == "cyber_cob"
                        || asset == crate::purifier::PURIFIER
                },
                install: |_, _, _, _| Ok(()),
                retire: |_, _| {},
            },
        ]);
    }
}

/// Registered world sources, including infinite mines (which are never E pickups).
#[derive(Resource, Default)]
pub struct WorldSources(pub BTreeMap<String, (Entity, stubby_recipes::SourceKind)>);

/// Every working placed source, by instance: crafting's view of what the world holds.
fn world_sources(
    mut sources: ResMut<WorldSources>,
    nodes: Query<
        (
            Entity,
            &PlacedAsset,
            Option<&ResourceNode>,
            Option<&MineNode>,
        ),
        Without<PrefabError>,
    >,
) {
    let next: BTreeMap<_, _> = nodes
        .iter()
        .filter_map(|(entity, asset, resource, mine)| {
            let source = resource
                .map(ResourceNode::source_object)
                .or_else(|| mine.map(MineNode::source_object))?;
            Some((asset.instance.clone(), (entity, source.kind())))
        })
        .collect();
    if sources.0 != next {
        sources.0 = next;
    }
}

fn install_bug(
    world: &mut World,
    entity: Entity,
    definition: &PlacedAsset,
    binding: &mut Bound,
) -> Result<(), String> {
    let logical = world.get::<PulseFollower>(entity).map(|f| f.clock);
    if let Some(logical) = logical {
        if !world.contains_resource::<AuthoredLevel>() && (1..=3).contains(&logical) {
            return Err(format!("clock {logical} belongs to a legacy opening bug"));
        }
        world
            .resource_mut::<ModuleIds>()
            .claim_clock(logical, &definition.instance)?;
        binding.clock = Some(logical);
    }
    let engine = allocate(world)?;
    let logical = logical.unwrap_or(engine.0);
    binding.clock = Some(logical);
    let voices = [allocate(world)?, allocate(world)?, allocate(world)?];
    binding.ids = vec![engine, voices[0], voices[1], voices[2]];
    world
        .resource_mut::<PulseAliases>()
        .0
        .insert(engine.0, logical);
    let harvested = world
        .get_resource::<bevy_stubby_recipes::PlayerInventory>()
        .is_some_and(|player| player.collected.collected(&definition.instance));
    let node = ResourceNode {
        resource: "lightning_bug".into(),
        instance: definition.instance.clone(),
        collected: harvested
            || world
                .get::<ResourceNode>(entity)
                .is_some_and(|node| node.collected),
    };
    world.entity_mut(entity).insert(node);
    stubby_life::bug::bind_placed_bug(
        world,
        entity,
        &definition.instance,
        logical,
        engine,
        voices,
    )?;
    Ok(())
}

fn install_module(
    world: &mut World,
    entity: Entity,
    definition: &PlacedAsset,
    _binding: &mut Bound,
) -> Result<(), String> {
    let asset = definition.asset.as_str();
    let id = allocate(world)?;
    let mut params = ModuleParams(match asset {
        "clock_lamp" | "burbler_clock" => [("bpm".into(), 120.)].into(),
        "volume" => [("gain".into(), 0.7)].into(),
        _ => [("freq".into(), 261.63)].into(),
    });
    if let Some(previous) = world.get::<ModuleParams>(entity) {
        for (name, value) in &previous.0 {
            if params.0.contains_key(name) {
                params.0.insert(name.clone(), *value);
            }
        }
    }
    world.entity_mut(entity).insert((
        SynthModule {
            id,
            asset: asset.into(),
        },
        params,
        ModuleLights::default(),
    ));
    Ok(())
}

/// The scrub oak's stream inlet: its profile's `stream_inlet` ring.
fn scrub_oak_inlet() -> stubby_core::StreamInlet {
    static INLET: std::sync::LazyLock<stubby_core::StreamInlet> = std::sync::LazyLock::new(|| {
        let profile: serde_json::Value =
            serde_json::from_str(include_str!("../../../assets/flora/scrub_oak/asset.json"))
                .expect("scrub oak profile");
        let ring = &profile["stream_inlet"];
        stubby_core::StreamInlet {
            radius_m: ring["radius_m"].as_f64().expect("stream_inlet.radius_m") as f32,
            snap_m: ring["snap_m"].as_f64().expect("stream_inlet.snap_m") as f32,
        }
    });
    *INLET
}

fn install_mine(
    world: &mut World,
    entity: Entity,
    definition: &PlacedAsset,
    binding: &mut Bound,
) -> Result<(), String> {
    let asset = definition.asset.as_str();
    let mine = if asset == "mudpit_burbler" {
        "mudpit_burbler"
    } else {
        "scrub_oak"
    };
    world
        .resource::<RecipeCatalog>()
        .0
        .source(mine, &definition.instance)
        .map_err(|error| error.to_string())?;
    world.entity_mut(entity).insert(MineNode {
        mine: mine.into(),
        instance: definition.instance.clone(),
    });
    if mine == "scrub_oak" {
        // Streams snap alongside the tree, never through it (add-tree-inlets).
        world.entity_mut(entity).insert(scrub_oak_inlet());
    }
    if asset == "mudpit_burbler" {
        let id = allocate(world)?;
        let seed = world.resource::<stubby_world::WorldSeed>().0;
        let hash = definition.instance.bytes().fold(seed, |hash, byte| {
            hash.wrapping_mul(1099511628211) ^ u64::from(byte)
        });
        // A seeded tempo on the 24 BPM grid, 48 to 192 (user, 2026-11-17: tempos are
        // multiples of 24 for now, which keeps 120 and 144).
        let bpm = stubby_recipes::BPM_STEP as f32 * (2 + hash % 7) as f32;
        world.entity_mut(entity).insert((
            AssetClock {
                engine: id,
                clock: id.0,
                bpm,
            },
            PulseFollower { clock: id.0 },
        ));
        if let Some(synth) = world.get_resource::<Synth>() {
            synth.send(Command::AddModule {
                id,
                kind: ModuleKind::Clock { bpm },
            });
        }
        binding.clock = Some(id.0);
        binding.ids.push(id);
    }
    Ok(())
}

fn install_reeds(
    world: &mut World,
    entity: Entity,
    definition: &PlacedAsset,
    _binding: &mut Bound,
) -> Result<(), String> {
    let harvested = world
        .get_resource::<bevy_stubby_recipes::PlayerInventory>()
        .is_some_and(|player| player.collected.collected(&definition.instance));
    world.entity_mut(entity).insert(ResourceNode {
        resource: "reed_clump".into(),
        instance: definition.instance.clone(),
        collected: harvested,
    });
    Ok(())
}

/// A find: a pickup that never returns (a snail shell), hidden once collected.
#[derive(Component)]
pub struct Find;

/// The shell's profile (`assets/creatures/snail_shell/asset.json`, from its generator).
#[derive(serde::Deserialize)]
struct ShellProfile {
    world_scale: f32,
    size_metres: ShellSize,
}
#[derive(serde::Deserialize)]
struct ShellSize {
    height: f32,
}
/// How far a shell lying on the mud leans from upright (as the Godot game lays them).
const SHELL_LIE_DEGREES: f32 = 72.;

/// The pose of a shell placed at `at`: shown at its profile's world scale, lying on its
/// side at an angle seeded by its instance, resting on the ground.
fn shell_pose(instance: &str, at: Vec3, ground: &stubby_core::GroundHeight) -> Transform {
    let profile: ShellProfile = serde_json::from_str(include_str!(
        "../../../assets/creatures/snail_shell/asset.json"
    ))
    .expect("snail shell profile");
    let hash = instance.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(1099511628211)
    });
    let yaw = (hash % 3600) as f32 / 3600. * std::f32::consts::TAU;
    let size = profile.size_metres.height * profile.world_scale;
    Transform {
        translation: Vec3::new(at.x, ground.at(at.x, at.z) + size * 0.25, at.z),
        rotation: Quat::from_rotation_y(yaw)
            * Quat::from_rotation_x(SHELL_LIE_DEGREES.to_radians()),
        scale: Vec3::splat(profile.world_scale),
    }
}

fn install_shell(
    world: &mut World,
    entity: Entity,
    definition: &PlacedAsset,
    _binding: &mut Bound,
) -> Result<(), String> {
    let collected = world
        .get_resource::<bevy_stubby_recipes::PlayerInventory>()
        .is_some_and(|player| player.collected.collected(&definition.instance));
    let at = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |transform| transform.translation);
    let pose = world
        .get_resource::<stubby_core::GroundHeight>()
        .map(|ground| shell_pose(&definition.instance, at, ground));
    let mut shell = world.entity_mut(entity);
    shell.insert((
        ResourceNode {
            resource: "empty_snail_shell".into(),
            instance: definition.instance.clone(),
            collected,
        },
        Find,
    ));
    if let Some(pose) = pose {
        shell.insert(pose);
    }
    Ok(())
}

/// A collected find is gone (it never returns).
fn hide_collected_finds(mut finds: Query<(&ResourceNode, &mut Visibility), With<Find>>) {
    for (node, mut visibility) in &mut finds {
        let wanted = if node.collected {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        visibility.set_if_neq(wanted);
    }
}

#[cfg(test)]
mod prefab_tests {
    use super::*;
    use crate::prefabs::PlacedPrefabsPlugin;
    use crate::{
        audio::{AudioOutput, SynthAudioPlugin},
        pulse::LightningBugPlugin,
        resources::{PlayerInventory, ResourceRecipesPlugin},
        synth_modules::SynthModulesPlugin,
    };
    use bevy_stubby_audio::pulse::PulseBus;
    use bevy_stubby_audio::pulse::SamplePulse;
    use std::time::{Duration, Instant};
    use stubby_world::prefabs::*;
    pub(super) fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::transform::TransformPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_asset::<StandardMaterial>()
        .init_asset::<bevy::gltf::Gltf>()
        .init_asset::<AnimationClip>()
        .init_asset::<bevy::animation::graph::BlendGraph>()
        .init_asset::<bevy::world_serialization::WorldAsset>()
        .insert_resource(AuthoredLevel)
        .insert_resource(stubby_world::WorldSeed(42))
        .init_resource::<stubby_world::calendar::Calendar>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins(SynthAudioPlugin {
            output: AudioOutput::Off,
            demo: false,
        })
        .add_plugins((
            LightningBugPlugin { seed: 42 },
            ResourceRecipesPlugin,
            SynthModulesPlugin::default(),
            PlacedPrefabsPlugin,
            PrefabGluePlugin,
        ));
        app
    }
    pub(super) fn placed(
        app: &mut App,
        asset: &str,
        instance: &str,
        transform: Transform,
    ) -> Entity {
        app.world_mut()
            .spawn((
                PlacedAsset {
                    asset: asset.into(),
                    model: format!("test_assets/{asset}/{asset}.glb"),
                    instance: instance.into(),
                },
                transform,
                Visibility::Visible,
            ))
            .id()
    }
    #[test]
    fn placed_bug_binds_once_and_retires_helpers() {
        let mut app = app();
        let parent = app
            .world_mut()
            .spawn(Transform::from_xyz(30., 0., 10.))
            .id();
        let bug = placed(
            &mut app,
            "lightning_bug",
            "authored-bug",
            Transform::from_xyz(2., 2., 3.).with_scale(Vec3::splat(1.7)),
        );
        app.world_mut()
            .entity_mut(bug)
            .insert((ChildOf(parent), PulseFollower { clock: 77 }));
        app.update();
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ResourceNode>()
                .iter(app.world())
                .count(),
            1,
            "no legacy fixed bugs in an authored level"
        );
        assert_eq!(
            app.world_mut()
                .query::<&PointLight>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(app.world().resource::<Bindings>().0[&bug].ids.len(), 4);
        assert_eq!(app.world().resource::<WorldSources>().0.len(), 1);
        let count = app.world().entities().len();
        app.update();
        assert_eq!(app.world().entities().len(), count);
        let now = Instant::now();
        app.world_mut()
            .resource_mut::<PulseBus>()
            .queue(SamplePulse {
                clock: 77,
                sample_time: 1,
                at: now - Duration::from_millis(30),
            });
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(bug).unwrap(),
            Visibility::Visible
        );
        assert!(
            app.world().get::<Transform>(bug).unwrap().scale.x > 0.
                && app.world().get::<Transform>(bug).unwrap().scale.x <= 1.7
        );
        // Each flash hovers at a seeded height within the asset's `hover_metres`.
        let at = app
            .world()
            .get::<GlobalTransform>(bug)
            .unwrap()
            .translation();
        let asset: serde_json::Value = serde_json::from_str(include_str!(
            "../../../assets/creatures/lightning_bug/asset.json"
        ))
        .unwrap();
        let [low, high] = [0, 1].map(|i| asset["hover_metres"][i].as_f64().unwrap() as f32);
        assert_eq!((at.x, at.z), (32., 13.));
        assert!((low..=high).contains(&at.y), "flash height {}", at.y);
        app.world_mut()
            .get_mut::<ResourceNode>(bug)
            .unwrap()
            .collected = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(bug).unwrap(),
            Visibility::Hidden
        );
        app.world_mut().despawn(bug);
        app.update();
        assert!(app.world().resource::<Bindings>().0.is_empty());
        assert!(app.world().resource::<WorldSources>().0.is_empty());
        assert_eq!(
            app.world_mut()
                .query::<&PointLight>()
                .iter(app.world())
                .count(),
            0
        );
        assert!(app.world().resource::<PulseAliases>().0.is_empty());
    }
    #[test]
    fn duplicate_instances_are_quarantined_and_recover_after_correction() {
        let mut app = app();
        let a = placed(&mut app, "reed_clump_0", "duplicate", Transform::default());
        let b = placed(
            &mut app,
            "reed_clump_1",
            "duplicate",
            Transform::from_xyz(1., 0., 0.),
        );
        app.update();
        assert!(app.world().get::<PrefabError>(a).is_some());
        assert!(app.world().get::<PrefabError>(b).is_some());
        assert!(app.world().resource::<WorldSources>().0.is_empty());
        app.world_mut().get_mut::<PlacedAsset>(b).unwrap().instance = "unique".into();
        app.update();
        assert!(app.world().get::<PrefabError>(a).is_none());
        assert!(app.world().get::<PrefabError>(b).is_none());
        assert_eq!(app.world().resource::<WorldSources>().0.len(), 2);
        assert_eq!(
            app.world().get::<ResourceNode>(b).unwrap().instance,
            "unique"
        );
    }
    #[test]
    fn placed_mines_are_registered_without_becoming_pickups() {
        let mut app = app();
        let oak = placed(&mut app, "scrub_oak_0", "oak", Transform::default());
        let burbler = placed(
            &mut app,
            "mudpit_burbler",
            "burbler",
            Transform::from_xyz(10., 0., 0.),
        );
        app.update();
        for entity in [oak, burbler] {
            assert!(app.world().get::<MineNode>(entity).is_some());
            assert!(app.world().get::<ResourceNode>(entity).is_none());
        }
        assert!(app.world().get::<AssetClock>(burbler).is_some());
        assert_eq!(app.world().resource::<WorldSources>().0.len(), 2);
    }
    #[test]
    fn collection_uses_nested_world_positions_and_reeds_replenish() {
        let mut app = app();
        let parent = app
            .world_mut()
            .spawn(Transform::from_xyz(100., 0., 0.))
            .id();
        let reed = placed(
            &mut app,
            "reed_clump_0",
            "reed",
            Transform::from_xyz(1., 0., 0.),
        );
        app.world_mut().entity_mut(reed).insert(ChildOf(parent));
        // The player starts with some reeds (start.json); count what a harvest adds.
        let start = app
            .world()
            .resource::<PlayerInventory>()
            .items
            .quantity("reed");
        // Reach is avian's (interaction.rs); the reed is the target.
        *app.world_mut()
            .resource_mut::<stubby_core::InteractTarget>() =
            stubby_core::InteractTarget::Pickup(reed);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("reed"),
            start + 6
        );
        assert!(app.world().get::<ResourceNode>(reed).unwrap().collected);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("reed"),
            start + 6
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyE);
        app.world_mut()
            .resource_mut::<stubby_world::calendar::Calendar>()
            .elapsed = Duration::from_secs(60);
        app.update();
        assert!(!app.world().get::<ResourceNode>(reed).unwrap().collected);
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("reed"),
            start + 6
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("reed"),
            start + 12
        );
    }
    #[test]
    fn module_reload_gets_fresh_engine_identity_without_transform_changes() {
        let mut app = app();
        let transform = Transform::from_xyz(5., 0., 2.);
        let module = placed(&mut app, "clock_lamp", "module", transform);
        app.update();
        app.update();
        let old = app.world().get::<SynthModule>(module).unwrap().id;
        app.world_mut()
            .get_mut::<PlacedAsset>(module)
            .unwrap()
            .asset = "square_vco".into();
        app.update();
        app.update();
        let new = app.world().get::<SynthModule>(module).unwrap();
        assert_ne!(new.id, old);
        assert_eq!(new.asset, "square_vco");
        assert_eq!(*app.world().get::<Transform>(module).unwrap(), transform);
    }
    #[test]
    fn authored_identity_is_deterministic_and_distinguishes_layout() {
        let a = identity("reed", &[("bed".into(), Transform::from_xyz(1., 0., 2.))]);
        assert_eq!(
            a,
            identity("reed", &[("bed".into(), Transform::from_xyz(1., 0., 2.))])
        );
        assert_ne!(
            a,
            identity("reed", &[("bed".into(), Transform::from_xyz(2., 0., 2.))])
        );
    }
}

#[cfg(test)]
mod reload_regressions {
    use super::*;
    use stubby_world::prefabs::Bindings;
    #[test]
    fn burbler_replacement_retires_its_owned_clock_before_binding_a_bug() {
        let mut app = super::prefab_tests::app();
        let source = super::prefab_tests::placed(
            &mut app,
            "mudpit_burbler",
            "replacement",
            Transform::default(),
        );
        app.update();
        let old = app.world().get::<AssetClock>(source).unwrap().clock;
        app.world_mut()
            .get_mut::<PlacedAsset>(source)
            .unwrap()
            .asset = "lightning_bug".into();
        app.update();
        assert!(app.world().get::<PrefabError>(source).is_none());
        assert!(app.world().get::<AssetClock>(source).is_none());
        assert!(app.world().get::<MineNode>(source).is_none());
        assert_ne!(app.world().get::<PulseFollower>(source).unwrap().clock, old);
        assert_eq!(
            app.world().get::<ResourceNode>(source).unwrap().resource,
            "lightning_bug"
        );
    }
    #[test]
    fn auto_clock_reload_and_bug_to_module_restore_authored_visibility() {
        let mut app = super::prefab_tests::app();
        let bug = super::prefab_tests::placed(
            &mut app,
            "lightning_bug",
            "reload-bug",
            Transform::default(),
        );
        app.update();
        app.update();
        let old = app.world().resource::<Bindings>().0[&bug].ids[0];
        app.world_mut().get_mut::<PlacedAsset>(bug).unwrap().model =
            "other/lightning_bug/lightning_bug.glb".into();
        app.update();
        assert!(app.world().get::<PrefabError>(bug).is_none());
        assert_ne!(app.world().resource::<Bindings>().0[&bug].ids[0], old);
        app.world_mut().get_mut::<PlacedAsset>(bug).unwrap().asset = "square_vco".into();
        app.update();
        assert!(app.world().get::<ResourceNode>(bug).is_none());
        assert!(app.world().get::<SynthModule>(bug).is_some());
        assert_eq!(
            *app.world().get::<Visibility>(bug).unwrap(),
            Visibility::Visible
        );
    }
}

#[cfg(test)]
mod machine_rule_tests {
    /// Every machine row's targets name a real place and knob in that machine's rack,
    /// within the knob's travel (add-machine-rules D1: checked where both are known).
    #[test]
    fn machine_targets_name_real_rack_knobs() {
        let rules = stubby_recipes::shipped_rules().unwrap();
        let profiles = super::Profiles::default();
        for row in &rules.machines {
            let mut config = profiles
                .0
                .get(&row.machine)
                .unwrap_or_else(|| panic!("{}: no machine `{}`", row.id, row.machine))
                .clone();
            config.targets = super::rack_targets(Some(row));
            let problems = config.target_problems();
            assert!(problems.is_empty(), "{}: {problems:#?}", row.id);
        }
    }

    /// A stream snapped to an oak's inlet runs within the oak's watering reach, so laying
    /// one there waters the tree.
    #[test]
    fn a_stream_on_the_oak_inlet_waters_it() {
        let inlet = super::scrub_oak_inlet();
        let catalog = stubby_recipes::shipped_catalog().expect("rules");
        let reach = catalog
            .mine("scrub_oak")
            .and_then(|mine| mine.growth.as_ref())
            .expect("oak growth")
            .water_reach_m;
        assert!(
            inlet.radius_m > 0. && inlet.radius_m < reach,
            "{inlet:?} vs {reach} m"
        );
    }
}
