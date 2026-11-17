//! Streams water trees, and trees grow (sync-stream-growth-and-dismantling, the Godot
//! game's watering adapter). This joins three systems, so it is glue: streams
//! (`stubby_streams::LaidStream`), placed trees (`MineNode` and `AssetState`) and goals.
//!
//! A placed mine with `growth` rules is watered while a *flowing* stream (its burbler
//! sounds) passes within `water_reach_m` of it, horizontally; the clearest such stream
//! sets how far it can grow. Its growth rises toward that cap and eases back without
//! water, in simulation time. The presenter already shows growth (buds, leaves, bark),
//! and the oak's air cleaning follows it. Explicit root inlets replace the reach later
//! (`add-tree-inlets-and-pulse`).
//!
//! The goals hear it as state: `tree_watered` holds while any tree of a mine is watered,
//! `tree_grown` holds for the highest level any tree of a mine has reached.
use bevy::prelude::*;
use bevy_stubby_recipes::{GoalRequest, MineNode, RecipeCatalog};
use std::collections::{BTreeMap, BTreeSet};
use stubby_core::{AssetState, MachineVoice};
use stubby_recipes::{Condition, GoalEvent, GrowthLevel};
use stubby_streams::LaidStream;

pub struct WateringPlugin;
impl Plugin for WateringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            water_trees.before(bevy_stubby_recipes::RecipesSystems),
        );
    }
}

/// What the goals last heard: the mines with a watered tree, each mine's highest level.
#[derive(Default)]
struct Reported {
    watered: BTreeSet<String>,
    grown: BTreeMap<String, GrowthLevel>,
}

fn water_trees(
    time: Res<Time<Virtual>>,
    catalog: Option<Res<RecipeCatalog>>,
    mut trees: Query<(&MineNode, &GlobalTransform, &mut AssetState)>,
    streams: Query<&LaidStream>,
    voices: Query<&MachineVoice>,
    mut reported: Local<Reported>,
    mut goals: MessageWriter<GoalRequest>,
) {
    let Some(catalog) = catalog else { return };
    let seconds = time.delta_secs();
    let flowing: Vec<&LaidStream> = streams
        .iter()
        .filter(|stream| {
            stream
                .source
                .and_then(|source| voices.get(source).ok())
                .is_some_and(|voice| voice.sounding)
        })
        .collect();
    let mut watered = BTreeSet::new();
    let mut grown = BTreeMap::<String, GrowthLevel>::new();
    for (mine, transform, mut state) in &mut trees {
        let Some(growth) = catalog.0.mine(&mine.mine).and_then(|m| m.growth.as_ref()) else {
            continue;
        };
        let at = transform.translation();
        let clarity = flowing
            .iter()
            .filter(|stream| stream.horizontal_distance(at) <= growth.water_reach_m)
            .map(|stream| stream.clarity)
            .reduce(f32::max);
        let next = growth.step(state.growth, clarity, seconds);
        if next != state.growth {
            state.growth = next;
        }
        let quality = clarity.unwrap_or(0.);
        if state.water_quality != quality {
            state.water_quality = quality;
        }
        if clarity.is_some() {
            watered.insert(mine.mine.clone());
        }
        if let Some(level) = growth.level(state.growth) {
            let best = grown.entry(mine.mine.clone()).or_insert(level);
            *best = (*best).max(level);
        }
    }
    report(&mut reported, watered, grown, &mut goals);
}

/// Tell the goals what started and stopped holding since last time.
fn report(
    reported: &mut Reported,
    watered: BTreeSet<String>,
    grown: BTreeMap<String, GrowthLevel>,
    goals: &mut MessageWriter<GoalRequest>,
) {
    let tree_watered = |mine: &String| Condition::TreeWatered { mine: mine.clone() };
    for mine in watered.difference(&reported.watered) {
        goals.write(GoalRequest(GoalEvent::ConditionMet(tree_watered(mine))));
    }
    for mine in reported.watered.difference(&watered) {
        goals.write(GoalRequest(GoalEvent::ConditionCleared(tree_watered(mine))));
    }
    let tree_grown = |mine: &String, level: GrowthLevel| Condition::TreeGrown {
        mine: mine.clone(),
        level,
    };
    for (mine, level) in &reported.grown {
        if grown.get(mine) != Some(level) {
            goals.write(GoalRequest(GoalEvent::ConditionCleared(tree_grown(
                mine, *level,
            ))));
        }
    }
    for (mine, level) in &grown {
        if reported.grown.get(mine) != Some(level) {
            goals.write(GoalRequest(GoalEvent::ConditionMet(tree_grown(
                mine, *level,
            ))));
        }
    }
    reported.watered = watered;
    reported.grown = grown;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_stubby_recipes::Goals;
    use stubby_recipes::{GoalDefinition, Scale};

    /// A tree 2 m from a stream from a burbler, in an app with the crafting plugin and
    /// fixture goals: "camp" is "water" (tree watered) then "bud" (tree budding).
    fn app(sounding: bool) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy_stubby_recipes::RecipesPlugin::default(),
        ))
        .add_plugins(WateringPlugin);
        let camp = GoalDefinition {
            id: "camp".into(),
            name: "Camp".into(),
            description: String::new(),
            scale: Scale::Major,
            part_of: None,
            requires: vec![],
            condition: None,
            after: vec![],
            unlocks: vec![],
        };
        let step = |id: &str, condition| GoalDefinition {
            id: id.into(),
            scale: Scale::Minor,
            part_of: Some("camp".into()),
            condition: Some(condition),
            ..camp.clone()
        };
        let water = step(
            "water",
            Condition::TreeWatered {
                mine: "scrub_oak".into(),
            },
        );
        let bud = step(
            "bud",
            Condition::TreeGrown {
                mine: "scrub_oak".into(),
                level: GrowthLevel::Budding,
            },
        );
        app.world_mut().resource_mut::<Goals>().rules.goals = vec![camp, water, bud];
        let burbler = app
            .world_mut()
            .spawn(MachineVoice {
                output: None,
                sounding,
                hearing_radius_metres: 30.,
            })
            .id();
        app.world_mut().spawn(LaidStream::straight(
            Some(burbler),
            Vec3::ZERO,
            Vec3::X * 8.,
        ));
        let tree = app
            .world_mut()
            .spawn((
                MineNode {
                    mine: "scrub_oak".into(),
                    instance: "scrub_oak-0".into(),
                },
                GlobalTransform::from_translation(Vec3::new(4., 0., 2.)),
                AssetState::default(),
            ))
            .id();
        (app, burbler, tree)
    }

    /// Water for `minutes` of simulation time, then let the goals hear it.
    fn run_minutes(app: &mut App, minutes: f32) {
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(std::time::Duration::from_secs_f32(minutes * 60.));
        app.world_mut().run_system_once(water_trees).unwrap();
        app.update();
    }

    fn growth(app: &App, tree: Entity) -> f32 {
        app.world().get::<AssetState>(tree).unwrap().growth
    }

    #[test]
    fn mud_grows_a_watered_oak_to_budding_and_the_goals_hear_it() {
        let (mut app, burbler, tree) = app(true);
        run_minutes(&mut app, 2.);
        assert!(
            (growth(&app, tree) - 0.3).abs() < 1e-3,
            "mud reaches budding, no further"
        );
        let done = |app: &App, id: &str| app.world().resource::<Goals>().progress.is_complete(id);
        assert!(done(&app, "water") && done(&app, "bud") && done(&app, "camp"));
        // Unpatching the burbler stops the water: the tree eases back, the goals stay done.
        app.world_mut()
            .get_mut::<MachineVoice>(burbler)
            .unwrap()
            .sounding = false;
        run_minutes(&mut app, 1.);
        let eased = growth(&app, tree);
        assert!(eased < 0.3 && eased > 0.2, "{eased}");
        assert!(done(&app, "water"));
    }

    #[test]
    fn a_quiet_stream_waters_nothing() {
        let (mut app, _, tree) = app(false);
        run_minutes(&mut app, 2.);
        assert_eq!(growth(&app, tree), 0.);
        assert!(
            !app.world()
                .resource::<Goals>()
                .progress
                .is_complete("water")
        );
    }
}
