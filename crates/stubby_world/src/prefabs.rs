//! Runtime lifecycle for assets authored in Jackdaw. No editor dependency.
use bevy::prelude::*;
use bevy_stubby_audio::pulse::{PulseAliases, PulseBus, PulseFollower};
use bevy_stubby_audio::{Synth, module_ids::ModuleIds};
use std::collections::HashMap;
use stubby_audio::{Command, ModuleId};
use stubby_core::PrefabBinding;

/// Insert before DesertPlugin when a level provides its own placements.
pub use stubby_core::AuthoredLevel;
pub use stubby_core::{AssetClock, PlacedAsset, PrefabError};
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default)]
pub struct AssetState {
    pub growth: f32,
    pub water_quality: f32,
    pub light_quality: f32,
    pub stream_connected: bool,
    pub port_highlight: bool,
}
/// One bound placed asset: what it was bound as and what the lifecycle cleans up.
#[derive(Clone)]
pub struct Binding {
    definition: PlacedAsset,
    /// Engine modules its binder added.
    pub ids: Vec<ModuleId>,
    clock: Option<u64>,
    authored_clock: Option<u64>,
    rest_visibility: Visibility,
    /// Undoes what the asset's binder installed.
    retire: fn(&mut World, Entity),
}

/// What a binder installed that the lifecycle cleans up for it: engine modules it added
/// (removed, newest first) and the logical clock it claimed.
#[derive(Default)]
pub struct Bound {
    pub ids: Vec<ModuleId>,
    pub clock: Option<u64>,
}

/// How one kind of placed asset comes alive. Registered by whichever system, or the
/// game's glue, owns that kind; the lifecycle itself names no asset kinds.
#[derive(Clone, Copy)]
pub struct AssetBinder {
    pub matches: fn(&World, &str) -> bool,
    pub install: fn(&mut World, Entity, &PlacedAsset, &mut Bound) -> Result<(), String>,
    pub retire: fn(&mut World, Entity),
}

/// Every binder, tried in order; the first that matches an asset binds it.
#[derive(Resource, Default, Clone)]
pub struct AssetBinders(pub Vec<AssetBinder>);
/// Every bound placed asset.
#[derive(Resource, Default)]
pub struct Bindings(pub HashMap<Entity, Binding>);
pub struct PlacedPrefabsPlugin;
impl Plugin for PlacedPrefabsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModuleIds>()
            .init_resource::<Bindings>()
            .init_resource::<AssetBinders>()
            .init_resource::<PulseAliases>()
            .register_type::<PlacedAsset>()
            .register_type::<AssetState>()
            .add_systems(
                Update,
                bind_runtime
                    .in_set(PrefabBinding)
                    .before(bevy_stubby_audio::pulse::PulseReceive),
            )
            .add_plugins(crate::prefab_presentation::PrefabPresentationPlugin);
        // The desert's props are scenery: placed and presented, nothing to bind.
        app.world_mut()
            .resource_mut::<AssetBinders>()
            .0
            .push(AssetBinder {
                matches: |_, asset| asset.starts_with("desert_"),
                install: |_, _, _, _| Ok(()),
                retire: |_, _| {},
            });
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
    (binding.retire)(world, entity);
    if world.get_entity(entity).is_ok() {
        world.entity_mut(entity).insert(binding.rest_visibility);
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
        world.entity_mut(entity).remove::<AssetClock>();
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
    }
    world.resource_mut::<Bindings>().0 = retained;
}
/// A fresh engine module ID, for binders.
pub fn allocate(world: &mut World) -> Result<ModuleId, String> {
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
        retire: |_, _| {},
    };
    let global = global_transform(world, entity);
    world.entity_mut(entity).insert(global);
    let binders = world.resource::<AssetBinders>().0.clone();
    let binder = binders
        .iter()
        .find(|binder| (binder.matches)(world, asset))
        .ok_or_else(|| format!("unknown placed asset {asset}"))?;
    let mut bound = Bound::default();
    (binder.install)(world, entity, definition, &mut bound)?;
    binding.ids = bound.ids;
    binding.clock = bound.clock;
    binding.retire = binder.retire;
    if world.get::<AssetState>(entity).is_none() {
        world.entity_mut(entity).insert(AssetState::default());
    }
    Ok(binding)
}
