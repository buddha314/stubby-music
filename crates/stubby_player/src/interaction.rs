//! What E acts on: the nearest pickup or machine in reach, found with avian3d's spatial
//! queries (Satisfactory-style interaction, AGENTS.md). Each pickup and each machine's
//! `Interact` point gets a reach marker: a small kinematic sensor of its own that follows
//! it (not a component on the visual itself, whose scale animates down to zero, which a
//! physics body cannot take). Every frame one upright cylinder at the player asks avian
//! which markers it overlaps. Reach is horizontal, as it always was: the
//! cylinder is tall, so a bug hovering at 1.8 m is as near as one on the ground.
use avian3d::prelude::*;
use bevy::prelude::*;
use stubby_core::{HudCamera, InteractTarget, Interactable};

/// Horizontal reach of E, in metres (pickups and machines alike).
pub const REACH_M: f32 = 3.0;
/// How far above and below the eye the reach extends.
const REACH_HEIGHT_M: f32 = 8.0;
/// The sensor every interactable carries: small, so it marks a point, not an area.
const MARKER_RADIUS_M: f32 = 0.25;

pub use stubby_core::InteractionTargeting;

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PhysicsSchedulePlugin>() {
            app.add_plugins(PhysicsPlugins::default());
        }
        app.init_resource::<InteractTarget>().add_systems(
            Update,
            (mark, follow, choose_target)
                .chain()
                .in_set(InteractionTargeting)
                .after(stubby_core::PrefabBinding),
        );
    }
}

/// A reach marker, following `owner` (an entity marked [`Interactable`]).
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

/// Every interactable gets a marker the first time it becomes one. It keeps the marker
/// while unavailable (a collected pickup), so it is not re-marked when it returns.
fn mark(mut commands: Commands, owners: Query<Entity, (With<Interactable>, Without<HasMarker>)>) {
    for entity in &owners {
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
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    owners: Query<(&Interactable, &GlobalTransform)>,
    markers: Query<&ReachMarker>,
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
        // An owner that is not available now (collected, broken) has no Interactable.
        let candidate = owners.get(hit).ok().map(|(what, at)| {
            let target = match *what {
                Interactable::Pickup => InteractTarget::Pickup(hit),
                Interactable::Machine(machine) => InteractTarget::Machine(machine),
            };
            (horizontal(at.translation()), target)
        });
        if let Some((d, t)) = candidate.filter(|(d, _)| *d <= REACH_M * REACH_M)
            && d < best.0
        {
            best = (d, t);
        }
    }
    target.set_if_neq(best.1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .add_plugins(InteractionPlugin)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                16,
            )));
        app.world_mut()
            .spawn((Camera3d::default(), Transform::from_xyz(0., 1.6, 0.)));
        // Avian sets up part of itself in `Plugin::finish`, which `App::run` would call.
        app.finish();
        app.cleanup();
        app
    }

    fn settle(app: &mut App) {
        for _ in 0..10 {
            app.update();
        }
    }

    #[test]
    fn the_nearest_interactable_in_reach_is_the_target() {
        let mut app = app();
        let near = app
            .world_mut()
            .spawn((Interactable::Pickup, Transform::from_xyz(1., 0.5, 0.)))
            .id();
        let machine = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            Interactable::Machine(machine),
            Transform::from_xyz(2., 0.5, 0.),
        ));
        app.world_mut()
            .spawn((Interactable::Pickup, Transform::from_xyz(10., 0.5, 0.)));
        settle(&mut app);
        assert_eq!(
            *app.world().resource::<InteractTarget>(),
            InteractTarget::Pickup(near)
        );

        // No longer available (collected): the machine behind it is next.
        app.world_mut().entity_mut(near).remove::<Interactable>();
        settle(&mut app);
        assert_eq!(
            *app.world().resource::<InteractTarget>(),
            InteractTarget::Machine(machine)
        );
    }
}
