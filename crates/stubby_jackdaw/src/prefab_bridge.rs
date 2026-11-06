//! Editor-only translation; shared gameplay never depends on Jackdaw types.
use bevy::prelude::*;
use jackdaw_prefab::components::{IsA, PrefabEntityId};
use jackdaw_runtime::{GltfSource, HiddenInGame};
use jackdaw_scene_types::SceneNodeId;
use stubby_jackdaw::pulse::PulseFollower;
#[path = "model_part_bindings.rs"]
mod model_part_bindings;
use stubby_jackdaw::{
    prefabs::{PlacedAsset, identity},
    resources::{MineNode, ResourceNode},
};
pub struct PrefabBridgePlugin;
impl Plugin for PrefabBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (bridge, removed_sources).chain())
            .add_plugins(model_part_bindings::ModelPartBindingsPlugin);
    }
}
#[allow(clippy::type_complexity)] // glTF sources may carry either finite or mine authoring metadata.
fn bridge(
    mut commands: Commands,
    sources: Query<
        (
            Entity,
            &GltfSource,
            Option<&ResourceNode>,
            Option<&MineNode>,
            Option<&PlacedAsset>,
        ),
        (Changed<GltfSource>, Without<HiddenInGame>),
    >,
    hierarchy: Query<(Option<&Name>, &Transform, Option<&ChildOf>)>,
    placements: Query<(&IsA, &SceneNodeId)>,
    local_ids: Query<&PrefabEntityId>,
    followers: Query<&PulseFollower>,
) {
    for (entity, source, resource, mine, previous) in &sources {
        if previous.is_some_and(|previous| previous.model == source.path) {
            continue;
        }
        let Some(asset) = std::path::Path::new(&source.path)
            .file_stem()
            .and_then(|stem| stem.to_str())
        else {
            continue;
        };
        if !(asset == "lightning_bug"
            || asset.starts_with("scrub_oak_")
            || asset.starts_with("reed_clump_")
            || asset.starts_with("desert_")
            || asset.ends_with("_vco")
            || matches!(
                asset,
                "clock_lamp" | "mudpit_burbler" | "stream_segment" | "burbler_clock" | "volume"
            ))
        {
            if previous.is_some() {
                commands.entity(entity).remove::<PlacedAsset>();
            }
            continue;
        }
        // A prefab's default state is a template, not a global instance/clock.
        // Use Jackdaw's persisted placement ID plus its prefab-local object ID.
        let template_bug = asset == "lightning_bug"
            && resource.is_some_and(|node| node.instance == "lightning_bug-placed-0");
        let mut placement_id = None;
        let mut path = Vec::new();
        let mut current = Some(entity);
        while let Some(root) = current {
            if placement_id.is_none() {
                placement_id = placements.get(root).ok().map(|(_, id)| id.0);
            }
            let Ok((name, transform, parent)) = hierarchy.get(root) else {
                break;
            };
            path.push((name.map_or("", Name::as_str).to_owned(), *transform));
            current = parent.map(ChildOf::parent);
        }
        let instance = previous
            .map(|previous| previous.instance.clone())
            .or_else(|| {
                resource.map(|node| {
                    if template_bug {
                        placement_id.map_or_else(
                            || identity(asset, &path),
                            |id| {
                                format!(
                                    "{asset}-{id:016x}-{:08x}",
                                    local_ids.get(entity).map_or(0, |id| id.0)
                                )
                            },
                        )
                    } else {
                        node.instance.clone()
                    }
                })
            })
            .or_else(|| mine.map(|node| node.instance.clone()))
            .unwrap_or_else(|| identity(asset, &path));
        if previous.is_none()
            && template_bug
            && followers
                .get(entity)
                .is_ok_and(|follower| follower.clock == 1)
        {
            commands.entity(entity).remove::<PulseFollower>();
        }
        commands.entity(entity).insert(PlacedAsset {
            asset: asset.into(),
            model: source.path.clone(),
            instance,
        });
    }
}

fn removed_sources(
    mut commands: Commands,
    mut removed: RemovedComponents<GltfSource>,
    sources: Query<(), With<GltfSource>>,
    placed: Query<(), With<PlacedAsset>>,
) {
    for entity in removed.read() {
        if !sources.contains(entity) && placed.contains(entity) {
            commands.entity(entity).remove::<PlacedAsset>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn repeated_bug_prefabs_use_native_placement_identity_and_independent_clocks() {
        let mut app = App::new();
        app.add_systems(Update, bridge);
        let mut bugs = Vec::new();
        for index in 0..7 {
            let root = app
                .world_mut()
                .spawn((
                    IsA {
                        source: "prefabs/creatures/lightning_bug.bsn".into(),
                        deleted: vec![],
                    },
                    SceneNodeId(1_u64 << 40 | index),
                    Transform::default(),
                ))
                .id();
            let bug = app
                .world_mut()
                .spawn((
                    GltfSource {
                        path: "models/creatures/lightning_bug.glb".into(),
                        scene_index: 0,
                    },
                    ResourceNode {
                        resource: "lightning_bug".into(),
                        instance: "lightning_bug-placed-0".into(),
                        collected: false,
                    },
                    PulseFollower { clock: 1 },
                    PrefabEntityId(1),
                    Transform::from_xyz(0., 1.8, 0.),
                    ChildOf(root),
                ))
                .id();
            bugs.push((root, bug));
        }
        let explicit = app
            .world_mut()
            .spawn((
                GltfSource {
                    path: "models/creatures/lightning_bug.glb".into(),
                    scene_index: 0,
                },
                ResourceNode {
                    resource: "lightning_bug".into(),
                    instance: "review-bug".into(),
                    collected: false,
                },
                PulseFollower { clock: 51 },
                Transform::default(),
            ))
            .id();
        app.update();
        let instances: HashSet<_> = bugs
            .iter()
            .map(|(_, bug)| {
                assert!(
                    app.world().get::<PulseFollower>(*bug).is_none(),
                    "template clock must go through the existing allocator"
                );
                app.world()
                    .get::<PlacedAsset>(*bug)
                    .unwrap()
                    .instance
                    .clone()
            })
            .collect();
        assert_eq!(
            instances.len(),
            7,
            "even coincident copies have distinct saved placement IDs"
        );
        assert_eq!(
            app.world().get::<PlacedAsset>(explicit).unwrap().instance,
            "review-bug"
        );
        assert_eq!(
            app.world().get::<PulseFollower>(explicit).unwrap().clock,
            51
        );
        let (root, bug) = bugs[0];
        let before = app
            .world()
            .get::<PlacedAsset>(bug)
            .unwrap()
            .instance
            .clone();
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation
            .x = 30.;
        app.world_mut().entity_mut(bug).remove::<PlacedAsset>();
        app.world_mut()
            .get_mut::<GltfSource>(bug)
            .unwrap()
            .scene_index = 0;
        app.update();
        assert_eq!(
            app.world().get::<PlacedAsset>(bug).unwrap().instance,
            before
        );
    }
}
