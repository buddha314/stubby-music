//! Runtime lifecycle for assets authored in Jackdaw. No editor dependency.
use super::{
    audio::Synth,
    module_ids::ModuleIds,
    pulse::{PulseAliases, PulseBus, PulseFollower},
    resources::{MineNode, RecipeCatalog, ResourceNode},
    synth_modules::{ModuleCatalog, ModuleLights, ModuleParams, SynthModule},
};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};
use stubby_audio::{Command, ModuleId, ModuleKind};

/// Insert before DesertPlugin when a level provides its own placements.
#[derive(Resource)]
pub struct AuthoredLevel;
#[derive(Component, Reflect, Clone, Debug, Default, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct PlacedAsset {
    pub asset: String,
    pub model: String,
    pub instance: String,
}
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default)]
pub struct AssetState {
    pub growth: f32,
    pub water_quality: f32,
    pub light_quality: f32,
    pub stream_connected: bool,
    pub port_highlight: bool,
}
#[derive(Component, Clone, Debug)]
pub struct PrefabError(pub String);
#[derive(Component, Clone, Copy)]
pub struct AssetClock {
    pub engine: ModuleId,
    pub clock: u64,
    pub bpm: f32,
}
#[derive(Clone)]
struct Binding {
    definition: PlacedAsset,
    ids: Vec<ModuleId>,
    clock: Option<u64>,
    authored_clock: Option<u64>,
    rest_visibility: Visibility,
    module: bool,
}
/// Registered world sources, including infinite mines (which are never E pickups).
#[derive(Resource, Default)]
pub struct WorldSources(pub BTreeMap<String, (Entity, stubby_recipes::SourceKind)>);
#[derive(Resource, Default)]
struct Bindings(HashMap<Entity, Binding>);
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PrefabBinding;
pub struct PlacedPrefabsPlugin;
impl Plugin for PlacedPrefabsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModuleIds>()
            .init_resource::<Bindings>()
            .init_resource::<WorldSources>()
            .init_resource::<PulseAliases>()
            .register_type::<PlacedAsset>()
            .register_type::<AssetState>()
            .add_systems(
                Update,
                bind_runtime
                    .in_set(PrefabBinding)
                    .before(super::pulse::PulseReceive)
                    .before(super::synth_modules::ModuleEngine),
            )
            .add_plugins(super::prefab_presentation::PrefabPresentationPlugin);
    }
}
/// Stable authored identity: no entity IDs, process RNG or wall clock.
pub fn identity(asset: &str, hierarchy: &[(String, Transform)]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in asset
        .bytes()
        .chain(hierarchy.iter().flat_map(|(name, transform)| {
            let values = transform
                .translation
                .to_array()
                .into_iter()
                .chain(transform.rotation.to_array())
                .chain(transform.scale.to_array());
            name.as_bytes()
                .to_vec()
                .into_iter()
                .chain(values.flat_map(|v| v.to_bits().to_le_bytes()))
        }))
    {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    format!("{asset}-{hash:016x}")
}
pub fn global_transform(world: &World, entity: Entity) -> GlobalTransform {
    let local = world.get::<Transform>(entity).copied().unwrap_or_default();
    if let Some(parent) = world.get::<ChildOf>(entity) {
        global_transform(world, parent.parent()).mul_transform(local)
    } else {
        GlobalTransform::from(local)
    }
}
fn fail(world: &mut World, entity: Entity, reason: String) {
    if world
        .get::<PrefabError>(entity)
        .is_none_or(|previous| previous.0 != reason)
    {
        error!("Prefab refused: {reason}");
    }
    world
        .entity_mut(entity)
        .insert((PrefabError(reason), Visibility::Hidden));
}
fn retire(world: &mut World, entity: Entity, binding: Binding) {
    if let Some(synth) = world.get_resource::<Synth>() {
        for id in binding.ids.iter().rev() {
            synth.send(Command::RemoveModule { id: *id });
        }
    }
    if let Some(clock) = binding.clock {
        world.resource_mut::<ModuleIds>().release_clock(clock);
        world.resource_mut::<PulseBus>().forget(clock);
        let engines: Vec<_> = world
            .resource::<PulseAliases>()
            .0
            .iter()
            .filter(|(_, logical)| **logical == clock)
            .map(|(engine, _)| *engine)
            .collect();
        for engine in engines {
            world.resource_mut::<PulseAliases>().0.remove(&engine);
        }
    }
    super::pulse::retire_placed_bug(world, entity);
    if world.get_entity(entity).is_ok() {
        world.entity_mut(entity).insert(binding.rest_visibility);
        if binding.module {
            super::synth_modules::unbind_module(world, entity);
        }
        if world
            .get::<PulseFollower>(entity)
            .map(|follower| follower.clock)
            == binding.clock
        {
            if let Some(clock) = binding.authored_clock {
                world.entity_mut(entity).insert(PulseFollower { clock });
            } else {
                world.entity_mut(entity).remove::<PulseFollower>();
            }
        }
        world
            .entity_mut(entity)
            .remove::<(AssetClock, ResourceNode, MineNode)>();
    }
}
fn bind_runtime(world: &mut World) {
    let candidates: HashMap<_, _> = world
        .query::<(Entity, &PlacedAsset)>()
        .iter(world)
        .map(|(entity, asset)| (entity, asset.clone()))
        .collect();
    let unchanged = {
        let bindings = world.resource::<Bindings>();
        bindings.0.len() == candidates.len()
            && candidates.iter().all(|(entity, definition)| {
                bindings.0.get(entity).is_some_and(|binding| {
                    binding.definition == *definition
                        && (binding.clock.is_none()
                            || world
                                .get::<PulseFollower>(*entity)
                                .map(|follower| follower.clock)
                                == binding.clock)
                })
            })
    };
    if unchanged {
        return;
    }
    let mut counts = HashMap::<String, usize>::new();
    for asset in candidates.values() {
        *counts.entry(asset.instance.clone()).or_default() += 1;
    }
    let previous = std::mem::take(&mut world.resource_mut::<Bindings>().0);
    let mut retained = HashMap::new();
    for (entity, binding) in previous {
        let keep = candidates.get(&entity).is_some_and(|definition| {
            *definition == binding.definition
                && counts.get(&definition.instance) == Some(&1)
                && (binding.clock.is_none()
                    || world
                        .get::<PulseFollower>(entity)
                        .map(|follower| follower.clock)
                        == binding.clock)
        });
        if keep {
            retained.insert(entity, binding);
        } else {
            retire(world, entity, binding);
        }
    }
    world.resource_mut::<WorldSources>().0.clear();
    let mut ordered: Vec<_> = candidates.into_iter().collect();
    ordered.sort_by(|(_, a), (_, b)| a.instance.cmp(&b.instance));
    for (entity, definition) in ordered {
        if definition.instance.is_empty() || counts[&definition.instance] != 1 {
            fail(
                world,
                entity,
                format!(
                    "duplicate or empty instance ID {:?}; give each placed prefab a unique instance",
                    definition.instance
                ),
            );
            continue;
        }
        if let std::collections::hash_map::Entry::Vacant(entry) = retained.entry(entity) {
            match install(world, entity, &definition) {
                Ok(binding) => {
                    if world.get::<PrefabError>(entity).is_some() {
                        world
                            .entity_mut(entity)
                            .remove::<PrefabError>()
                            .insert(Visibility::Visible);
                    }
                    entry.insert(binding);
                }
                Err(reason) => {
                    fail(world, entity, reason);
                    continue;
                }
            }
        }
        let source = world
            .get::<ResourceNode>(entity)
            .map(ResourceNode::source_object)
            .or_else(|| world.get::<MineNode>(entity).map(MineNode::source_object));
        if let Some(source) = source {
            world
                .resource_mut::<WorldSources>()
                .0
                .insert(definition.instance, (entity, source.kind()));
        }
    }
    world.resource_mut::<Bindings>().0 = retained;
}
fn allocate(world: &mut World) -> Result<ModuleId, String> {
    world
        .resource_mut::<ModuleIds>()
        .allocate()
        .ok_or_else(|| "engine module IDs exhausted".into())
}
fn install(world: &mut World, entity: Entity, definition: &PlacedAsset) -> Result<Binding, String> {
    let asset = definition.asset.as_str();
    let mut binding = Binding {
        definition: definition.clone(),
        ids: Vec::new(),
        clock: None,
        authored_clock: world
            .get::<PulseFollower>(entity)
            .map(|follower| follower.clock),
        rest_visibility: if world.get::<PrefabError>(entity).is_some() {
            Visibility::Visible
        } else {
            world.get::<Visibility>(entity).copied().unwrap_or_default()
        },
        module: false,
    };
    let global = global_transform(world, entity);
    world.entity_mut(entity).insert(global);
    if asset == "lightning_bug" {
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
            .get_resource::<super::resources::PlayerInventory>()
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
        super::pulse::bind_placed_bug(world, entity, logical, engine, voices)?;
    } else if world.resource::<ModuleCatalog>().0.contains_key(asset) {
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
        binding.module = true;
    } else if asset.starts_with("scrub_oak_") || asset == "mudpit_burbler" {
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
        if asset == "mudpit_burbler" {
            let id = allocate(world)?;
            let seed = world.resource::<super::WorldSeed>().0;
            let hash = definition.instance.bytes().fold(seed, |hash, byte| {
                hash.wrapping_mul(1099511628211) ^ u64::from(byte)
            });
            let bpm = 40. + (hash % 141) as f32;
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
    } else if asset.starts_with("reed_clump_") {
        let harvested = world
            .get_resource::<super::resources::PlayerInventory>()
            .is_some_and(|player| player.collected.collected(&definition.instance));
        world.entity_mut(entity).insert(ResourceNode {
            resource: "reed_clump".into(),
            instance: definition.instance.clone(),
            collected: harvested,
        });
    } else if !asset.starts_with("desert_") && asset != "stream_segment" {
        return Err(format!("unknown placed asset {asset}"));
    }
    if world.get::<AssetState>(entity).is_none() {
        world.entity_mut(entity).insert(AssetState::default());
    }
    Ok(binding)
}

#[cfg(test)]
mod tests {
    use super::super::{
        audio::{AudioOutput, SynthAudioPlugin},
        pulse::{LightningBugPlugin, SamplePulse},
        resources::{PlayerInventory, ResourceRecipesPlugin},
        synth_modules::SynthModulesPlugin,
    };
    use super::*;
    use std::time::{Duration, Instant};
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
        .init_asset::<bevy::animation::graph::AnimationGraph>()
        .init_asset::<bevy::world_serialization::WorldAsset>()
        .insert_resource(AuthoredLevel)
        .insert_resource(super::super::WorldSeed(42))
        .init_resource::<super::super::calendar::Calendar>()
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
                    model: format!("models/{asset}.glb"),
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
        assert_eq!(
            app.world()
                .get::<GlobalTransform>(bug)
                .unwrap()
                .translation(),
            Vec3::new(32., 2., 13.)
        );
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
        app.world_mut()
            .spawn((Camera3d::default(), Transform::from_xyz(100., 1.65, 0.)));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("reed"),
            6
        );
        assert!(app.world().get::<ResourceNode>(reed).unwrap().collected);
        app.update();
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("reed"),
            6
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyE);
        app.world_mut()
            .resource_mut::<super::super::calendar::Calendar>()
            .elapsed = Duration::from_secs(60);
        app.update();
        assert!(!app.world().get::<ResourceNode>(reed).unwrap().collected);
        assert_eq!(
            app.world()
                .resource::<PlayerInventory>()
                .items
                .quantity("reed"),
            6
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
            12
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
    #[test]
    fn burbler_replacement_retires_its_owned_clock_before_binding_a_bug() {
        let mut app = super::tests::app();
        let source = super::tests::placed(
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
        let mut app = super::tests::app();
        let bug = super::tests::placed(
            &mut app,
            "lightning_bug",
            "reload-bug",
            Transform::default(),
        );
        app.update();
        app.update();
        let old = app.world().resource::<Bindings>().0[&bug].ids[0];
        app.world_mut().get_mut::<PlacedAsset>(bug).unwrap().model =
            "models/other/lightning_bug.glb".into();
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
