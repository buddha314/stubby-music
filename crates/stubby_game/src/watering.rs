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
//! A watered tree shows it: a small trickle runs over the ground from the stream to its
//! trunk while the water flows (user, 2026-11-18: a watered tree needs a sign of the
//! connection).
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
            (
                water_trees.before(bevy_stubby_recipes::RecipesSystems),
                show_trickles.after(water_trees),
            ),
        );
    }
}

/// What the goals last heard: the mines with a watered tree, each mine's highest level.
#[derive(Default)]
struct Reported {
    watered: BTreeSet<String>,
    grown: BTreeMap<String, GrowthLevel>,
}

/// On a watered tree: the point of the stream that waters it (its nearest).
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct WateredFrom(pub Vec3);

#[allow(clippy::too_many_arguments)]
fn water_trees(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    catalog: Option<Res<RecipeCatalog>>,
    mut trees: Query<(
        Entity,
        &MineNode,
        &GlobalTransform,
        &mut AssetState,
        Option<&WateredFrom>,
    )>,
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
    for (tree, mine, transform, mut state, from) in &mut trees {
        let Some(growth) = catalog.0.mine(&mine.mine).and_then(|m| m.growth.as_ref()) else {
            continue;
        };
        let at = transform.translation();
        let reaching: Vec<&&LaidStream> = flowing
            .iter()
            .filter(|stream| stream.horizontal_distance(at) <= growth.water_reach_m)
            .collect();
        let clarity = reaching
            .iter()
            .map(|stream| stream.clarity)
            .reduce(f32::max);
        // Where the trickle starts: the nearest point of the nearest watering stream.
        let source = reaching
            .iter()
            .map(|stream| stream.nearest_point(at))
            .min_by(|a, b| a.distance_squared(at).total_cmp(&b.distance_squared(at)));
        match (source, from) {
            (Some(point), Some(from)) if from.0.distance(point) < 0.05 => {}
            (Some(point), _) => {
                commands.entity(tree).insert(WateredFrom(point));
            }
            (None, Some(_)) => {
                commands.entity(tree).remove::<WateredFrom>();
            }
            (None, None) => {}
        }
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

/// A trickle's drawn path: from the stream to its tree's trunk.
#[derive(Component)]
struct Trickle {
    tree: Entity,
    from: Vec3,
}

/// How a trickle looks: wet, glossy mud a hand wide at the stream, thinning to the trunk.
const TRICKLE_START_WIDTH_M: f32 = 0.22;
const TRICKLE_END_WIDTH_M: f32 = 0.07;
/// How far it sits above the ground's height (the mud's plates stand proud of it).
const TRICKLE_LIFT_M: f32 = 0.08;
/// Where it stops short of the trunk's centre.
const TRICKLE_TRUNK_GAP_M: f32 = 0.25;

/// A trickle runs from each watered tree's stream to its trunk, and dries up with the water.
fn show_trickles(
    mut commands: Commands,
    trees: Query<(Entity, &WateredFrom, &GlobalTransform)>,
    mut trickles: Query<(Entity, &mut Trickle, &Mesh3d)>,
    ground: Option<Res<stubby_core::GroundHeight>>,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut wet: Local<Option<Handle<StandardMaterial>>>,
) {
    // Headless apps (tests) draw nothing.
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };
    let ground = ground.map(|g| *g).unwrap_or_default();
    let mut shown = BTreeSet::new();
    for (entity, mut trickle, mesh) in &mut trickles {
        let Ok((_, from, at)) = trees.get(trickle.tree) else {
            commands.entity(entity).despawn();
            continue;
        };
        shown.insert(trickle.tree);
        if trickle.from != from.0 {
            trickle.from = from.0;
            if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
                *mesh = trickle_mesh(from.0, at.translation(), &ground);
            }
        }
    }
    let material = wet
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                // Muddy water, wet enough to catch the sky.
                base_color: Color::srgb(0.2, 0.14, 0.09),
                perceptual_roughness: 0.04,
                reflectance: 0.9,
                ..default()
            })
        })
        .clone();
    for (tree, from, at) in &trees {
        if shown.contains(&tree) {
            continue;
        }
        commands.spawn((
            Trickle { tree, from: from.0 },
            Mesh3d(meshes.add(trickle_mesh(from.0, at.translation(), &ground))),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            bevy::light::NotShadowCaster,
        ));
    }
}

/// A thin ribbon on the ground from `from` to just short of `trunk`, meandering a little,
/// in world space.
fn trickle_mesh(from: Vec3, trunk: Vec3, ground: &stubby_core::GroundHeight) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let flat = Vec2::new(trunk.x - from.x, trunk.z - from.z);
    let length = (flat.length() - TRICKLE_TRUNK_GAP_M).max(0.05);
    let along = flat.normalize_or(Vec2::X);
    let side = Vec2::new(-along.y, along.x);
    let steps = ((length / 0.15).ceil() as usize).max(2);
    let mut positions = Vec::with_capacity(steps * 2 + 2);
    for k in 0..=steps {
        let t = k as f32 / steps as f32;
        // A gentle meander, still at both ends.
        let wobble =
            0.08 * (t * std::f32::consts::TAU * 1.5).sin() * (std::f32::consts::PI * t).sin();
        let centre = Vec2::new(from.x, from.z) + along * length * t + side * wobble;
        let half = (TRICKLE_START_WIDTH_M + (TRICKLE_END_WIDTH_M - TRICKLE_START_WIDTH_M) * t) / 2.;
        for edge in [-half, half] {
            let p = centre + side * edge;
            // Above the cracked mud's plates, which stand over the ground's height.
            positions.push([p.x, ground.at(p.x, p.y) + TRICKLE_LIFT_M, p.y]);
        }
    }
    let normals = vec![[0., 1., 0.]; positions.len()];
    let mut indices = Vec::with_capacity(steps * 6);
    for k in 0..steps as u32 {
        let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
        // Wound to face up (counter-clockwise seen from above).
        indices.extend([a, b, c, b, d, c]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
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
        // Its trickle starts from the stream's nearest point, beside the tree.
        let from = app
            .world()
            .get::<WateredFrom>(tree)
            .expect("watered from the stream");
        assert!(from.0.distance(Vec3::new(4., 0., 0.)) < 0.3, "{from:?}");
        // Unpatching the burbler stops the water: the tree eases back, the goals stay done.
        app.world_mut()
            .get_mut::<MachineVoice>(burbler)
            .unwrap()
            .sounding = false;
        run_minutes(&mut app, 1.);
        let eased = growth(&app, tree);
        assert!(eased < 0.3 && eased > 0.2, "{eased}");
        assert!(
            app.world().get::<WateredFrom>(tree).is_none(),
            "the trickle dries up"
        );
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
