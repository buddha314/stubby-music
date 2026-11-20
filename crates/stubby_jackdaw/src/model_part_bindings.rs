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
            .add_systems(Update, (bind_parts, restore_sockets, solid_boulders));
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
/// A node whose extras the game reads (a `start_scale`, an `acoustic` material) controls
/// the parts under it, whatever it is named.
fn controls(node: &GltfNode) -> bool {
    node.extras.as_ref().is_some_and(|extras| {
        serde_json::from_str::<serde_json::Value>(&extras.value).is_ok_and(|value| {
            value.get("start_scale").is_some() || value.get("acoustic").is_some()
        })
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
            if control.is_none() && (is_state_node(&ancestor.name) || controls(ancestor)) {
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
            || asset.asset.starts_with("desert_")
            || asset.asset.starts_with("boulder_")
            || asset.asset.starts_with("rock_formation_")
            || asset.asset == "campfire")
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
        // A part exported full-size but tagged `start_scale` (sync-blender-asset-contracts)
        // starts hidden here too; the rest pose keeps its full size for the presenter.
        if let Some(start) = info.extras["start_scale"].as_f64() {
            commands
                .entity(entity)
                .insert(rest.with_scale(rest.scale * start as f32));
        }
        // Acoustic rock (add-echo-grotto): the flattened part joins the spatial-audio scene,
        // as an unflattened model's mesh does through its extras.
        if info.extras["acoustic"].as_str() == Some("rock") {
            commands
                .entity(entity)
                .insert(stubby_core::AcousticOccluder::ROCK);
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

/// On a placed asset whose sockets are back.
#[derive(Component)]
struct SocketsRestored;

/// The model's sockets: childless empties outside its levels of detail (`Interact`,
/// `Sparks`, a light socket), as `(name, frame in the model)`.
fn sockets(
    gltf: &Gltf,
    nodes: &Assets<GltfNode>,
) -> Option<Vec<(String, Transform, Option<bevy::gltf::GltfExtras>)>> {
    let all: Vec<_> = gltf
        .nodes
        .iter()
        .map(|handle| nodes.get(handle).map(|node| (handle.id(), node)))
        .collect::<Option<_>>()?;
    let parents: HashMap<_, _> = all
        .iter()
        .flat_map(|(id, node)| node.children.iter().map(move |child| (child.id(), *id)))
        .collect();
    let in_level = |mut id: AssetId<GltfNode>| {
        while let Some(parent) = parents.get(&id) {
            if nodes.get(*parent).is_some_and(|n| n.name.contains("_LOD")) {
                return true;
            }
            id = *parent;
        }
        false
    };
    Some(
        all.iter()
            .filter(|(id, node)| {
                node.mesh.is_none()
                    && node.children.is_empty()
                    && parents.contains_key(id)
                    && !in_level(*id)
            })
            .map(|(id, node)| {
                (
                    node.name.clone(),
                    node_frame(*id, nodes, &parents),
                    node.extras.clone(),
                )
            })
            .collect(),
    )
}

/// Jackdaw's LOD runtime flattens a level into bare meshes, so a flattened model's
/// sockets never reach the world (2026-11-18: the campfire had no `Interact`, so E reached
/// past it to the burbler). Each placed asset drawn by flattened parts gets its model's
/// sockets back, as named children at their place in the model, once.
#[allow(clippy::type_complexity)]
fn restore_sockets(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    nodes: Option<Res<Assets<GltfNode>>>,
    profiles: Res<Profiles>,
    parts: Query<(&Mesh3d, &AssetPart), With<LodPart>>,
    restored: Query<(), With<SocketsRestored>>,
) {
    let Some(nodes) = nodes else {
        return;
    };
    // `SocketsRestored` lands when the commands apply, after this loop: a model drawn by
    // several flattened meshes would otherwise get its sockets once per mesh (2026-11-20:
    // each rock opening, and the campfire's `Interact`, were doubled or tripled).
    let mut done = std::collections::HashSet::new();
    for (mesh, part) in &parts {
        if !part.flattened || restored.contains(part.root) || done.contains(&part.root) {
            continue;
        }
        let Some(file) = mesh
            .0
            .path()
            .map(|p| p.clone_owned().without_label().to_string())
        else {
            continue;
        };
        let Some(gltf) = profiles.models.get(&file).and_then(|h| gltfs.get(h)) else {
            continue;
        };
        let Some(sockets) = sockets(gltf, &nodes) else {
            continue;
        };
        done.insert(part.root);
        commands
            .entity(part.root)
            .insert(SocketsRestored)
            .with_children(|root| {
                // With their glTF extras, as a scene node has them: what a socket is for
                // (a `resource` to collect, an `action`) travels with it.
                for (name, frame, extras) in sockets {
                    let mut socket = root.spawn((Name::new(name), frame, Visibility::default()));
                    if let Some(extras) = extras {
                        socket.insert(extras);
                    }
                }
            });
    }
}

/// On a placed asset whose collision mesh is built.
#[derive(Component)]
struct CollisionBuilt;

/// Boulders and rock formations are solid (2026-11-18: the player walked through them):
/// each placed one gets static triangle-mesh colliders from its own detailed level's meshes
/// (`<id>_LOD0`), so the grotto's opening and the formations' openings stay open, marked
/// `Solid` for the player's movement.
#[allow(clippy::type_complexity)]
fn solid_boulders(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    nodes: Option<Res<Assets<GltfNode>>>,
    gltf_meshes: Option<Res<Assets<GltfMesh>>>,
    meshes: Res<Assets<Mesh>>,
    profiles: Res<Profiles>,
    parts: Query<(&Mesh3d, &AssetPart), With<LodPart>>,
    roots: Query<&PlacedAsset, Without<CollisionBuilt>>,
) {
    let (Some(nodes), Some(gltf_meshes)) = (nodes, gltf_meshes) else {
        return;
    };
    for (mesh, part) in &parts {
        let Ok(asset) = roots.get(part.root) else {
            continue;
        };
        if !(asset.asset.starts_with("boulder_") || asset.asset.starts_with("rock_formation_")) {
            continue;
        }
        let Some(file) = mesh
            .0
            .path()
            .map(|p| p.clone_owned().without_label().to_string())
        else {
            continue;
        };
        let Some(gltf) = profiles.models.get(&file).and_then(|h| gltfs.get(h)) else {
            continue;
        };
        let Some(colliders) = level_colliders(gltf, &nodes, &gltf_meshes, &meshes, "_LOD0") else {
            continue;
        };
        commands
            .entity(part.root)
            .insert(CollisionBuilt)
            .with_children(|root| {
                for (collider, frame) in colliders {
                    root.spawn((
                        Name::new("Collision mesh"),
                        collider,
                        avian3d::prelude::RigidBody::Static,
                        stubby_core::Solid,
                        frame,
                    ));
                }
            });
    }
}

/// Trimesh colliders for every mesh under the model's node ending in `level`, each with its
/// frame in the model. `None` until every mesh is loaded.
fn level_colliders(
    gltf: &Gltf,
    nodes: &Assets<GltfNode>,
    gltf_meshes: &Assets<GltfMesh>,
    meshes: &Assets<Mesh>,
    level: &str,
) -> Option<Vec<(avian3d::prelude::Collider, Transform)>> {
    let all: Vec<_> = gltf
        .nodes
        .iter()
        .map(|handle| nodes.get(handle).map(|node| (handle.id(), node)))
        .collect::<Option<_>>()?;
    let parents: HashMap<_, _> = all
        .iter()
        .flat_map(|(id, node)| node.children.iter().map(move |child| (child.id(), *id)))
        .collect();
    let under = |mut id: AssetId<GltfNode>| loop {
        if nodes.get(id).is_some_and(|n| n.name.ends_with(level)) {
            return true;
        }
        match parents.get(&id) {
            Some(parent) => id = *parent,
            None => return false,
        }
    };
    let mut out = Vec::new();
    for (id, node) in &all {
        let Some(mesh) = node.mesh.as_ref() else {
            continue;
        };
        if !under(*id) {
            continue;
        }
        let frame = node_frame(*id, nodes, &parents);
        for primitive in &gltf_meshes.get(mesh)?.primitives {
            let mesh = meshes.get(&primitive.mesh)?;
            if let Some(collider) = avian3d::prelude::Collider::trimesh_from_mesh(mesh) {
                out.push((collider, frame));
            }
        }
    }
    (!out.is_empty()).then_some(out)
}
