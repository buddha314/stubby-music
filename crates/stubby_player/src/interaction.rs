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
        crate::outline::add(app);
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

/// Within reach, the interactable closest to the view axis inside this cone wins; with
/// none inside it, the nearest (sync-targeting-and-compass).
pub const AIM_CONE_DEGREES: f32 = 20.;

fn choose_target(
    spatial: SpatialQuery,
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    owners: Query<(&Interactable, &GlobalTransform)>,
    markers: Query<&ReachMarker>,
    mut target: ResMut<InteractTarget>,
) {
    let Ok(camera) = cameras.single() else {
        target.set_if_neq(InteractTarget::None);
        return;
    };
    let eye = camera.translation();
    let look = camera.forward().as_vec3();
    let horizontal = |p: Vec3| Vec2::new(p.x - eye.x, p.z - eye.z).length_squared();
    let off_axis = |p: Vec3| look.angle_between(p - eye).to_degrees();
    let mut candidates = Vec::new();
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
            && let Ok((_, at)) = owners.get(hit)
        {
            candidates.push((off_axis(at.translation()), d, t));
        }
    }
    target.set_if_neq(pick(&candidates));
}

/// From `(degrees off the view axis, horizontal distance², target)`: the most central
/// inside the aim cone, else the nearest.
fn pick(candidates: &[(f32, f32, InteractTarget)]) -> InteractTarget {
    let aimed = candidates
        .iter()
        .filter(|(angle, ..)| *angle <= AIM_CONE_DEGREES)
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let nearest = || candidates.iter().min_by(|a, b| a.1.total_cmp(&b.1));
    aimed
        .or_else(nearest)
        .map_or(InteractTarget::None, |(_, _, target)| *target)
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
    fn aim_beats_distance_inside_the_cone_and_nearest_wins_outside() {
        let shell = InteractTarget::Pickup(Entity::from_raw_u32(1).unwrap());
        let reed = InteractTarget::Pickup(Entity::from_raw_u32(2).unwrap());
        // The shell is aimed at, the reed slightly nearer but off to the side.
        assert_eq!(pick(&[(4., 2.0, shell), (35., 1.5, reed)]), shell);
        // Nothing in the cone: the nearest.
        assert_eq!(pick(&[(40., 2.0, shell), (35., 1.5, reed)]), reed);
        assert_eq!(pick(&[]), InteractTarget::None);
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
