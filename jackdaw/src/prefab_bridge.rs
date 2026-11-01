//! Editor-only translation; shared gameplay never depends on Jackdaw types.
use bevy::prelude::*;
use jackdaw_runtime::{GltfSource, HiddenInGame};
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
        let mut path = Vec::new();
        let mut current = Some(entity);
        while let Some(root) = current {
            let Ok((name, transform, parent)) = hierarchy.get(root) else {
                break;
            };
            path.push((name.map_or("", Name::as_str).to_owned(), *transform));
            current = parent.map(ChildOf::parent);
        }
        let instance = previous
            .map(|previous| previous.instance.clone())
            .or_else(|| resource.map(|node| node.instance.clone()))
            .or_else(|| mine.map(|node| node.instance.clone()))
            .unwrap_or_else(|| identity(asset, &path));
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
