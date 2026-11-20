//! Collecting from a model's sockets, for every model alike (user, 2026-11-20: "collection
//! should be an abstract effort, not written for each object").
//!
//! A socket is a leaf node of a placed model (no children); one whose glTF extras name a
//! `resource` that the rules define becomes a pickup of that resource, through the same
//! path as any other: a `ResourceNode` E reaches, yields, a recipe-book entry and the
//! resource's own replenishment. Its instance is the placement's instance and the socket's
//! name (`rock_formation_5-…/Opening_1`), so every opening or pool is its own source.
//! Art decides what is collectible by tagging the socket; nothing here names a kind.
//! (Restored sockets of flattened models carry their extras too: `model_part_bindings`.)
use bevy::{gltf::GltfExtras, prelude::*};
use bevy_stubby_recipes::{PlayerInventory, RecipeCatalog, ResourceNode};
use stubby_core::PlacedAsset;

pub struct CollectiblesPlugin;
impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            collectible_sockets
                .after(stubby_core::PrefabBinding)
                .before(stubby_core::InteractionTargeting),
        );
    }
}

/// A node already looked at, whether it was collectible or not.
#[derive(Component)]
struct SocketChecked;

#[allow(clippy::type_complexity)] // Leaf nodes with extras, not yet looked at.
fn collectible_sockets(
    mut commands: Commands,
    catalog: Res<RecipeCatalog>,
    player: Option<Res<PlayerInventory>>,
    nodes: Query<
        (Entity, &GltfExtras, Option<&Name>),
        (
            Without<Children>,
            Without<ResourceNode>,
            Without<SocketChecked>,
        ),
    >,
    parents: Query<&ChildOf>,
    placed: Query<&PlacedAsset>,
) {
    for (entity, extras, name) in &nodes {
        // Wait until the node hangs under its placement (scenes and sockets arrive late).
        let Some(root) = parents.iter_ancestors(entity).find(|a| placed.contains(*a)) else {
            continue;
        };
        commands.entity(entity).insert(SocketChecked);
        let Some(resource) = serde_json::from_str::<serde_json::Value>(&extras.value)
            .ok()
            .and_then(|v| {
                v.get("resource")
                    .and_then(|r| r.as_str())
                    .map(str::to_owned)
            })
            .filter(|id| catalog.0.resource(id).is_some())
        else {
            continue;
        };
        let Ok(asset) = placed.get(root) else {
            continue;
        };
        let instance = format!("{}/{}", asset.instance, name.map_or("socket", Name::as_str));
        let collected = player
            .as_ref()
            .is_some_and(|player| player.collected.collected(&instance));
        debug!("collectible socket: {instance} gives {resource}");
        commands.entity(entity).insert(ResourceNode {
            resource,
            instance,
            collected,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tagged leaf sockets become pickups, each its own source; nothing else does.
    #[test]
    fn tagged_sockets_become_pickups_of_their_resource() {
        let mut app = App::new();
        app.insert_resource(RecipeCatalog(stubby_recipes::shipped_catalog().unwrap()))
            .add_systems(Update, collectible_sockets);
        let world = app.world_mut();
        let root = world
            .spawn(PlacedAsset {
                asset: "rock_formation_5".into(),
                model: "props/rocks/rock_formation_5/rock_formation_5.glb".into(),
                instance: "rock_formation_5-1234".into(),
            })
            .id();
        let socket = |world: &mut World, name: &str, extras: &str| {
            world
                .spawn((
                    Name::new(name.to_owned()),
                    GltfExtras { value: extras.into() },
                    ChildOf(root),
                ))
                .id()
        };
        let a = socket(world, "Opening_0", r#"{"resource":"rock_opening"}"#);
        let b = socket(world, "Opening_1", r#"{"resource":"rock_opening","opening":1}"#);
        // An item, not a resource (the reeds' `Interact` names `reed`): not a pickup here.
        let item = socket(world, "Interact", r#"{"resource":"reed"}"#);
        // A node with children (a model's root names its resource too): not a socket.
        let parent = socket(world, "RipplePool_Root", r#"{"resource":"ripple_pool"}"#);
        world.spawn((Name::new("Child"), ChildOf(parent)));
        app.update();
        app.update();
        let world = app.world();
        let node = |e: Entity| world.get::<ResourceNode>(e);
        assert_eq!(node(a).unwrap().resource, "rock_opening");
        assert_eq!(node(a).unwrap().instance, "rock_formation_5-1234/Opening_0");
        assert_eq!(node(b).unwrap().instance, "rock_formation_5-1234/Opening_1");
        assert!(!node(a).unwrap().collected);
        assert!(node(item).is_none() && node(parent).is_none());
    }
}
