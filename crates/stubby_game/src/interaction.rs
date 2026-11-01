//! What E acts on: the nearest pickup or machine in reach, found with avian3d's spatial
//! queries (Satisfactory-style interaction, AGENTS.md). Each pickup and each machine's
//! `Interact` point gets a reach marker: a small kinematic sensor of its own that follows
//! it (not a component on the visual itself, whose scale animates down to zero, which a
//! physics body cannot take). Every frame one upright cylinder at the player asks avian
//! which markers it overlaps. Reach is horizontal, as it always was: the
//! cylinder is tall, so a bug hovering at 1.8 m is as near as one on the ground.
use avian3d::prelude::*;
use bevy::prelude::*;

use super::{
    machine_config::{Profiles, RackHudCamera},
    prefabs::{PlacedAsset, PrefabError},
    resources::ResourceNode,
};

/// Horizontal reach of E, in metres (pickups and machines alike).
pub const REACH_M: f32 = 3.0;
/// How far above and below the eye the reach extends.
const REACH_HEIGHT_M: f32 = 8.0;
/// The sensor every interactable carries: small, so it marks a point, not an area.
const MARKER_RADIUS_M: f32 = 0.25;

/// The one thing E would act on now.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum InteractTarget {
    #[default]
    None,
    /// A finite pickup (its `ResourceNode` entity).
    Pickup(Entity),
    /// A placed machine (its `PlacedAsset` entity), reached through its `Interact` point.
    Machine(Entity),
}
impl InteractTarget {
    pub fn pickup(self) -> Option<Entity> {
        match self {
            Self::Pickup(e) => Some(e),
            _ => None,
        }
    }
    pub fn machine(self) -> Option<Entity> {
        match self {
            Self::Machine(e) => Some(e),
            _ => None,
        }
    }
}

/// A machine's interaction point, pointing at the machine.
#[derive(Component)]
pub struct InteractPoint {
    pub machine: Entity,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionTargeting;

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PhysicsSchedulePlugin>() {
            app.add_plugins(PhysicsPlugins::default());
        }
        app.init_resource::<InteractTarget>().add_systems(
            Update,
            (mark_pickups, mark_interact_points, follow, choose_target)
                .chain()
                .in_set(InteractionTargeting)
                .after(super::prefabs::PrefabBinding),
        );
    }
}

/// A reach marker, following `owner` (a pickup's `ResourceNode` entity or an `Interact`
/// node, which carries `InteractPoint`).
#[derive(Component)]
struct ReachMarker {
    owner: Entity,
}
/// On an owner: its marker.
#[derive(Component)]
struct HasMarker;

fn spawn_marker(commands: &mut Commands, owner: Entity) {
    commands.spawn((
        ReachMarker { owner },
        RigidBody::Kinematic,
        Collider::sphere(MARKER_RADIUS_M),
        Sensor,
        Transform::default(),
    ));
    commands.entity(owner).insert(HasMarker);
}

fn mark_pickups(
    mut commands: Commands,
    pickups: Query<Entity, (With<ResourceNode>, Without<HasMarker>)>,
) {
    for entity in &pickups {
        spawn_marker(&mut commands, entity);
    }
}

/// Give each placed machine's `Interact` node (named in its profile) a marker.
fn mark_interact_points(
    mut commands: Commands,
    profiles: Res<Profiles>,
    names: Query<(Entity, &Name), Added<Name>>,
    parents: Query<&ChildOf>,
    machines: Query<&PlacedAsset>,
) {
    for (entity, name) in &names {
        let Some((machine, _)) = parents
            .iter_ancestors(entity)
            .find_map(|a| machines.get(a).ok().map(|m| (a, m)))
            .filter(|(_, m)| {
                profiles
                    .0
                    .get(&m.asset)
                    .is_some_and(|c| c.interact.node == name.as_str())
            })
        else {
            continue;
        };
        commands.entity(entity).insert(InteractPoint { machine });
        spawn_marker(&mut commands, entity);
    }
}

/// Markers follow their owners' world position; a marker whose owner is gone goes too.
fn follow(
    mut commands: Commands,
    mut markers: Query<(Entity, &ReachMarker, &mut Transform)>,
    owners: Query<&GlobalTransform>,
) {
    for (marker, ReachMarker { owner }, mut transform) in &mut markers {
        match owners.get(*owner) {
            Ok(at) if at.translation().is_finite() => {
                transform.translation = at.translation();
            }
            Ok(_) => {}
            Err(_) => commands.entity(marker).despawn(),
        }
    }
}

fn choose_target(
    spatial: SpatialQuery,
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<RackHudCamera>)>,
    pickups: Query<(&ResourceNode, &GlobalTransform), Without<PrefabError>>,
    points: Query<(&InteractPoint, &GlobalTransform)>,
    markers: Query<&ReachMarker>,
    broken: Query<(), With<PrefabError>>,
    mut target: ResMut<InteractTarget>,
) {
    let Ok(eye) = cameras.single().map(GlobalTransform::translation) else {
        target.set_if_neq(InteractTarget::None);
        return;
    };
    let horizontal = |p: Vec3| Vec2::new(p.x - eye.x, p.z - eye.z).length_squared();
    let mut best = (f32::INFINITY, InteractTarget::None);
    for hit in spatial.shape_intersections(
        &Collider::cylinder(REACH_M, REACH_HEIGHT_M * 2.),
        eye,
        Quat::IDENTITY,
        &SpatialQueryFilter::default(),
    ) {
        let Ok(ReachMarker { owner: hit }) =
            markers.get(hit).map(|m| ReachMarker { owner: m.owner })
        else {
            continue;
        };
        let candidate = if let Ok((node, at)) = pickups.get(hit) {
            (!node.collected).then(|| (horizontal(at.translation()), InteractTarget::Pickup(hit)))
        } else if let Ok((point, at)) = points.get(hit) {
            (!broken.contains(point.machine)).then(|| {
                (
                    horizontal(at.translation()),
                    InteractTarget::Machine(point.machine),
                )
            })
        } else {
            None
        };
        if let Some((d, t)) = candidate.filter(|(d, _)| *d <= REACH_M * REACH_M)
            && d < best.0
        {
            best = (d, t);
        }
    }
    target.set_if_neq(best.1);
}
