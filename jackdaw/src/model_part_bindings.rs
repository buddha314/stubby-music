//! Game animation metadata for the render parts provided by Jackdaw's LOD tools.
//! Does not generate, select, replace, or switch any model/LOD geometry.
use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
};
use jackdaw_runtime::LodPart;
use jackdaw_scene_types::{model_import::ModelLodIndex, render_assets::load_model};
use std::collections::HashMap;
use stubby_jackdaw::{
    prefab_presentation::{AssetPart, is_state_node},
    prefabs::PlacedAsset,
};

pub struct ModelPartBindingsPlugin;
#[derive(Clone)]
struct Metadata {
    name: String,
    extras: serde_json::Value,
    origin: Vec3,
    rotation: Quat,
    sway: Option<(Vec3, f32)>,
}
#[derive(Resource, Default)]
struct Profiles {
    models: HashMap<String, Handle<Gltf>>,
    parts: HashMap<AssetId<Gltf>, HashMap<AssetId<Mesh>, Vec<Metadata>>>,
}
impl Plugin for ModelPartBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Profiles>()
            .add_systems(Update, bind_parts);
    }
}
fn node_frame(
    id: AssetId<GltfNode>,
    nodes: &Assets<GltfNode>,
    parents: &HashMap<AssetId<GltfNode>, AssetId<GltfNode>>,
) -> Transform {
    let local = nodes
        .get(id)
        .map_or(Transform::default(), |node| node.transform);
    parents.get(&id).map_or(local, |parent| {
        node_frame(*parent, nodes, parents).mul_transform(local)
    })
}
fn metadata(
    gltf: &Gltf,
    nodes: &Assets<GltfNode>,
    meshes: &Assets<GltfMesh>,
) -> Option<HashMap<AssetId<Mesh>, Vec<Metadata>>> {
    let all: Vec<_> = gltf
        .nodes
        .iter()
        .map(|handle| nodes.get(handle).map(|node| (handle.id(), node)))
        .collect::<Option<_>>()?;
    let parents: HashMap<_, _> = all
        .iter()
        .flat_map(|(id, node)| node.children.iter().map(move |child| (child.id(), *id)))
        .collect();
    let blooms = all
        .iter()
        .filter_map(|(_, node)| {
            node.name
                .strip_prefix("Bloom_")
                .and_then(|index| index.parse::<usize>().ok())
        })
        .max()
        .map_or(1, |index| index + 1);
    let mut result: HashMap<_, Vec<_>> = HashMap::new();
    for (id, node) in &all {
        let Some(mesh) = node.mesh.as_ref() else {
            continue;
        };
        let mesh = meshes.get(mesh)?;
        let mut current = Some(*id);
        let mut control = None;
        let mut sway = None;
        while let Some(id) = current {
            let ancestor = nodes.get(id)?;
            if control.is_none() && is_state_node(&ancestor.name) {
                control = Some(ancestor);
            }
            if sway.is_none() {
                let index = ancestor
                    .name
                    .strip_prefix("Limb_")
                    .or_else(|| ancestor.name.strip_prefix("Sway_"))
                    .and_then(|index| index.parse::<f32>().ok());
                if let Some(index) = index {
                    sway = Some((node_frame(id, nodes, &parents).translation, index));
                }
            }
            current = parents.get(&id).copied();
        }
        let mut extras = control
            .and_then(|node| node.extras.as_ref())
            .and_then(|extras| serde_json::from_str::<serde_json::Value>(&extras.value).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        extras["runtime_bloom_count"] = serde_json::json!(blooms);
        let frame = node_frame(*id, nodes, &parents);
        let info = Metadata {
            name: control.map_or_else(String::new, |node| node.name.clone()),
            extras,
            origin: frame.translation,
            rotation: frame.rotation,
            sway,
        };
        for primitive in &mesh.primitives {
            result
                .entry(primitive.mesh.id())
                .or_default()
                .push(info.clone());
        }
    }
    Some(result)
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Read engine render parts and the source glTF's animation metadata.
fn bind_parts(
    mut commands: Commands,
    server: Res<AssetServer>,
    imports: Option<Res<ModelLodIndex>>,
    gltfs: Res<Assets<Gltf>>,
    nodes: Option<Res<Assets<GltfNode>>>,
    meshes: Option<Res<Assets<GltfMesh>>>,
    parts: Query<(Entity, &Mesh3d, &Transform), (With<LodPart>, Without<AssetPart>)>,
    parents: Query<&ChildOf>,
    roots: Query<&PlacedAsset>,
    mut profiles: ResMut<Profiles>,
    mut changes: MessageReader<AssetEvent<Gltf>>,
) {
    if changes.read().any(|event| {
        matches!(
            event,
            AssetEvent::Modified { .. } | AssetEvent::Removed { .. }
        )
    }) {
        profiles.parts.clear();
    }
    let (Some(nodes), Some(meshes)) = (nodes, meshes) else {
        return;
    };
    for (entity, mesh, rest) in &parts {
        let Some(root) = parents
            .iter_ancestors(entity)
            .find(|root| roots.contains(*root))
        else {
            continue;
        };
        let Ok(asset) = roots.get(root) else {
            continue;
        };
        if !(asset.asset.starts_with("scrub_oak_")
            || asset.asset.starts_with("reed_clump_")
            || asset.asset.starts_with("desert_"))
        {
            continue;
        }
        let Some(path) = mesh.0.path() else {
            continue;
        };
        let file = path.clone_owned().without_label().to_string();
        let handle = profiles
            .models
            .entry(file.clone())
            .or_insert_with(|| load_model(&server, imports.as_deref(), file))
            .clone();
        if let std::collections::hash_map::Entry::Vacant(entry) = profiles.parts.entry(handle.id())
        {
            let Some(gltf) = gltfs.get(&handle) else {
                continue;
            };
            let Some(info) = metadata(gltf, &nodes, &meshes) else {
                continue;
            };
            entry.insert(info);
        }
        // Generated levels retain the original primitive label after LodN/.
        // Only map that label to animation metadata; Jackdaw owns the mesh itself.
        let original = path
            .label()
            .and_then(|label| label.split_once('/'))
            .filter(|(prefix, _)| {
                prefix
                    .strip_prefix("Lod")
                    .is_some_and(|index| index.parse::<usize>().is_ok())
            })
            .and_then(|(_, label)| {
                server.get_handle::<Mesh>(&path.clone_owned().with_label(label.to_owned()))
            });
        let key = original.as_ref().map_or(mesh.0.id(), Handle::id);
        let Some(options) = profiles
            .parts
            .get(&handle.id())
            .and_then(|parts| parts.get(&key))
        else {
            continue;
        };
        let Some(info) = options.iter().min_by(|a, b| {
            a.origin
                .distance_squared(rest.translation)
                .total_cmp(&b.origin.distance_squared(rest.translation))
        }) else {
            continue;
        };
        let mut rest = *rest;
        if !rest.rotation.is_finite() {
            rest.rotation = info.rotation;
        }
        commands.entity(entity).insert(AssetPart {
            root,
            name: info.name.clone(),
            rest,
            extras: info.extras.clone(),
            flattened: true,
            sway: info.sway,
        });
    }
}
