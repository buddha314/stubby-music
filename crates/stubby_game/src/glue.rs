//! Glue between systems (split-game-into-systems): the rules that join two systems live
//! here, never inside either.
use bevy::prelude::*;
use bevy_stubby_rack::machine_config::Profiles;
use stubby_core::{PlacedAsset, PrefabError};

pub struct GluePlugin;
impl Plugin for GluePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy_stubby_rack::rack::KnobDefaults(
            // A Square VCO is made from a lightning bug's square wave: it starts at the
            // bug's pitch.
            [(
                ("SquareVco".to_owned(), "freq".to_owned()),
                stubby_life::bug_click().freq_hz,
            )]
            .into(),
        ))
        .add_systems(
            Update,
            interact_points
                .after(stubby_core::PrefabBinding)
                .before(stubby_core::InteractionTargeting),
        );
    }
}

/// A machine's interaction point (its profile's `Interact` node), pointing at the machine.
#[derive(Component)]
pub struct InteractPoint {
    pub machine: Entity,
}

/// Glue between machines and the player: each placed machine's `Interact` node (named in
/// its profile) is something E can reach while the machine works.
fn interact_points(
    mut commands: Commands,
    profiles: Res<Profiles>,
    names: Query<(Entity, &Name), Added<Name>>,
    parents: Query<&ChildOf>,
    machines: Query<&PlacedAsset>,
    points: Query<(Entity, &InteractPoint, Has<stubby_core::Interactable>)>,
    broken: Query<(), With<PrefabError>>,
) {
    for (entity, name) in &names {
        let Some(machine) = parents.iter_ancestors(entity).find(|a| {
            machines.get(*a).is_ok_and(|m| {
                profiles
                    .0
                    .get(&m.asset)
                    .is_some_and(|c| c.interact.node == name.as_str())
            })
        }) else {
            continue;
        };
        commands.entity(entity).insert(InteractPoint { machine });
    }
    for (entity, point, marked) in &points {
        let working = !broken.contains(point.machine);
        if working && !marked {
            commands
                .entity(entity)
                .insert(stubby_core::Interactable::Machine(point.machine));
        } else if !working && marked {
            commands
                .entity(entity)
                .remove::<stubby_core::Interactable>();
        }
    }
}
