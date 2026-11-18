//! The campfire comes alive (add-campfire-craft-bench D1, D3, D7): the rule joining the
//! rack, the audio clock, the streams and the goals, so it lives in the game.
//!
//! - **Its clock is lent** (D7): the nearest running clock within the profile's
//!   `clock_reach_m` (rules: `machines.json`; a sounding burbler, or a flowing stream it feeds) becomes the camp's
//!   `AssetClock`, which the rack's borrowed clock place plays. Every clock is locked to the
//!   master clock, so the camp is on its neighbour's beats.
//! - **It lights on the backbeat** (D3): with the rack sounding and its targets met (the
//!   profile's `configure.targets`: the sequencer's gates on 2 and 4 only), one whole bar
//!   heard in order lights it.
//!   Each struck step throws sparks from the bundle at the instant it is heard.
//! - **It burns only while its clock runs** (D1): with no clock lent it dies to embers and
//!   must be lit again. `fire_lit` is reported once and latches.
//! - **It joins a stream** (add-stream-layers): the nearest stream point within its reach
//!   is its join (`StreamJoin`), and each step its sequencer gates puts an ember note on
//!   the stream's staff there, riding downstream, a layer over the heartbeat.
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy_stubby_rack::rack::{RackEngine, RackKnobs, RackState, RackTargets};
use bevy_stubby_recipes::GoalRequest;
use std::time::Instant;
use stubby_core::{AssetClock, AssetPart, MachineVoice, PlacedAsset};
use stubby_recipes::{Condition, GoalEvent};
use stubby_streams::LaidStream;

const CAMPFIRE: &str = "campfire";
/// Where the sticks meet, above the camp's root: sparks fly from here.
const APEX: Vec3 = Vec3::new(0., 0.9, 0.);
const SPARKS_PER_STRIKE: usize = 7;
const SPARK_LIFE_S: f32 = 0.7;

pub struct CampfirePlugin;
impl Plugin for CampfirePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spark_assets).add_systems(
            Update,
            (
                lend_clocks,
                join_streams,
                strike,
                burn,
                show_fire,
                fly_sparks,
            )
                .chain()
                .after(stubby_core::MachineInteractionSet),
        );
    }
}

/// Where a camp joins a stream: the chain's burbler, how far down the chain, and the
/// stream point itself (add-stream-layers). Its layer starts there and flows downstream.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct StreamJoin {
    pub source: Entity,
    pub join_m: f32,
    pub point: Vec3,
}

/// The colour of the campfire's notes on the staff.
const EMBER: Color = Color::srgb(1.0, 0.42, 0.12);
/// Where across the staff they sit: above the heartbeat's middle line.
const EMBER_STEP: i8 = 2;

/// A camp joins the nearest stream point within its reach (`clock_reach_m`), or none.
fn join_streams(
    mut commands: Commands,
    rules: Option<Res<bevy_stubby_recipes::MachineRules>>,
    camps: Query<(Entity, &GlobalTransform, Option<&StreamJoin>), With<Fire>>,
    streams: Query<&LaidStream>,
) {
    let reach_m = rules
        .as_ref()
        .and_then(|rules| rules.of(CAMPFIRE)?.clock_reach_m)
        .unwrap_or(0.);
    for (camp, at, current) in &camps {
        let at = at.translation();
        let flat = |p: Vec3| Vec2::new(p.x - at.x, p.z - at.z).length();
        let join = streams
            .iter()
            .filter_map(|stream| {
                let source = stream.source?;
                let (join_m, point) = stream.nearest_along(at);
                Some((
                    flat(point),
                    StreamJoin {
                        source,
                        join_m,
                        point,
                    },
                ))
            })
            .filter(|(d, _)| *d <= reach_m)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, join)| join);
        match (current, join) {
            (Some(now), Some(next))
                if now.source == next.source && (now.join_m - next.join_m).abs() < 0.05 => {}
            (_, Some(next)) => {
                commands.entity(camp).insert(next);
            }
            (Some(_), None) => {
                commands.entity(camp).remove::<StreamJoin>();
            }
            (None, None) => {}
        }
    }
}

/// A camp's fire.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fire {
    #[default]
    Unlit,
    Burning,
    /// It burned and its clock stopped: coals, to be lit again.
    Embers,
}

/// How far into a lighting bar a camp is: the next step it needs to hear.
#[derive(Component, Default)]
struct Bar(Backbeat);

/// One whole bar heard in order with the pattern set lights the fire.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
struct Backbeat {
    next: u8,
}
impl Backbeat {
    /// The sequencer moved to `step` (0-based) of a bar of `steps`; `ready` says the rack
    /// sounds with its gates set to the pattern. True when the bar is complete.
    fn hear(&mut self, step: u8, steps: u8, ready: bool) -> bool {
        self.next = if !ready {
            0
        } else if step == self.next {
            self.next + 1
        } else {
            u8::from(step == 0)
        };
        if self.next >= steps {
            self.next = 0;
            return true;
        }
        false
    }
}

/// The nearest running clock to each camp, lent as its `AssetClock`. A changed lender
/// re-plans the camp's rack.
#[allow(clippy::type_complexity)]
fn lend_clocks(
    mut commands: Commands,
    rules: Option<Res<bevy_stubby_recipes::MachineRules>>,
    new: Query<(Entity, &PlacedAsset), (With<RackState>, Without<Fire>)>,
    mut camps: Query<
        (
            Entity,
            &PlacedAsset,
            &GlobalTransform,
            Option<&AssetClock>,
            &mut RackState,
        ),
        With<Fire>,
    >,
    burblers: Query<(Entity, &GlobalTransform, &AssetClock, &MachineVoice)>,
    names: Query<&PlacedAsset>,
    streams: Query<&LaidStream>,
) {
    // How far a camp reaches for a clock is a rule (add-machine-rules); with no row, none.
    let reach_m = rules
        .as_ref()
        .and_then(|rules| rules.of(CAMPFIRE)?.clock_reach_m)
        .unwrap_or(0.);
    // New camps get a fire to keep.
    for (camp, asset) in &new {
        if asset.asset == CAMPFIRE {
            commands.entity(camp).insert((Fire::Unlit, Bar::default()));
        }
    }
    for (camp, asset, at, current, mut state) in &mut camps {
        if asset.asset != CAMPFIRE {
            continue;
        }
        let at = at.translation();
        let flat = |p: Vec3| Vec2::new(p.x - at.x, p.z - at.z).length();
        let mut best: Option<(f32, AssetClock)> = None;
        let mut consider = |distance: f32, clock: AssetClock| {
            if distance <= reach_m && best.is_none_or(|(d, _)| distance < d) {
                best = Some((distance, clock));
            }
        };
        for (burbler, place, clock, voice) in &burblers {
            let is_camp = names.get(burbler).is_ok_and(|a| a.asset == CAMPFIRE);
            if burbler != camp && !is_camp && voice.sounding {
                consider(flat(place.translation()), *clock);
            }
        }
        for stream in &streams {
            let Some(source) = stream.source else {
                continue;
            };
            if let Ok((_, _, clock, voice)) = burblers.get(source)
                && voice.sounding
            {
                consider(stream.horizontal_distance(at), *clock);
            }
        }
        let lent = best.map(|(_, clock)| clock);
        match (current, lent) {
            (Some(now), Some(next)) if now.engine == next.engine => {
                if now.bpm != next.bpm {
                    commands.entity(camp).insert(next);
                }
            }
            (None, None) => {}
            (_, Some(next)) => {
                commands.entity(camp).insert(next);
                state.set_changed();
            }
            (Some(_), None) => {
                commands.entity(camp).remove::<AssetClock>();
                state.set_changed();
            }
        }
    }
}

/// Heard sequencer steps: strike sparks on the steps whose gate is on, and light on a
/// whole bar heard with the rack's targets met.
#[allow(clippy::type_complexity)]
fn strike(
    mut commands: Commands,
    mut ticks: MessageReader<bevy_stubby_audio::StepTick>,
    mut camps: Query<(
        Entity,
        &RackEngine,
        &RackKnobs,
        &RackTargets,
        &mut Bar,
        &mut Fire,
        Has<AssetClock>,
        Option<&StreamJoin>,
    )>,
    mut pending: Local<Vec<(Entity, Instant)>>,
    mut goals: MessageWriter<GoalRequest>,
    mut layer: MessageWriter<stubby_streams::LayerNote>,
    mut reported: Local<bool>,
) {
    let steps = stubby_audio::SEQUENCER_STEPS as u8;
    for tick in ticks.read() {
        for (camp, engine, knobs, targets, mut bar, mut fire, clocked, join) in &mut camps {
            if engine.place_of(tick.module) != Some("seq") {
                continue;
            }
            let gate = |step: usize| {
                knobs
                    .0
                    .get(&("seq".to_owned(), format!("gate_{}", step + 1)))
                    .is_some_and(|value| *value >= 0.5)
            };
            let ready = engine.sounding && clocked && targets.met;
            if engine.sounding && gate(usize::from(tick.step)) {
                let at = tick.at.unwrap_or_else(Instant::now);
                pending.push((camp, at));
                // Its layer on the stream: a note at the join, riding downstream.
                if let Some(join) = join {
                    layer.write(stubby_streams::LayerNote {
                        source: join.source,
                        join_m: join.join_m,
                        at,
                        step: EMBER_STEP,
                        colour: Some(EMBER),
                    });
                }
            }
            if bar.0.hear(tick.step, steps, ready) && *fire != Fire::Burning {
                *fire = Fire::Burning;
                if !*reported {
                    *reported = true;
                    goals.write(GoalRequest(GoalEvent::ConditionMet(Condition::FireLit {
                        buildable: CAMPFIRE.into(),
                    })));
                }
            }
        }
    }
    let now = Instant::now();
    pending.retain(|(camp, at)| {
        if *at > now {
            return true;
        }
        commands.trigger(Strike(*camp));
        false
    });
}

/// A heard strike at a camp: sparks fly from the bundle.
#[derive(Event)]
struct Strike(Entity);

/// A burning fire needs its clock: without one it dies to embers.
fn burn(mut camps: Query<(&mut Fire, Has<AssetClock>)>) {
    for (mut fire, clocked) in &mut camps {
        if *fire == Fire::Burning && !clocked {
            *fire = Fire::Embers;
        }
    }
}

/// The model follows the fire: flames while burning, coals as embers, and a warm light
/// that flickers while it burns.
#[allow(clippy::type_complexity)]
fn show_fire(
    mut commands: Commands,
    time: Res<Time>,
    camps: Query<(Entity, &Fire, Option<&Children>)>,
    mut parts: Query<(&AssetPart, &mut Transform)>,
    mut lights: Query<(Entity, &mut PointLight), With<FireLight>>,
) {
    for (part, mut transform) in &mut parts {
        let Ok((_, fire, _)) = camps.get(part.root) else {
            continue;
        };
        let on = match part.name.as_str() {
            "Fire" => *fire == Fire::Burning,
            "Embers" => *fire == Fire::Embers,
            _ => continue,
        };
        let scale = part.rest.scale * f32::from(u8::from(on));
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
    let t = time.elapsed_secs();
    for (camp, fire, children) in &camps {
        let light = children
            .into_iter()
            .flatten()
            .find_map(|child| lights.get(*child).ok().map(|(entity, _)| entity));
        match (fire, light) {
            (Fire::Burning, None) => {
                commands.entity(camp).with_child((
                    FireLight,
                    PointLight {
                        color: Color::srgb(1., 0.62, 0.3),
                        intensity: 60_000.,
                        range: 9.,
                        ..default()
                    },
                    Transform::from_xyz(0., 0.45, 0.),
                ));
            }
            (Fire::Burning, Some(light)) => {
                if let Ok((_, mut point)) = lights.get_mut(light) {
                    let seed = camp.to_bits() as f32 * 0.37;
                    let flicker =
                        0.82 + 0.12 * (t * 9.1 + seed).sin() + 0.06 * (t * 23.7 + seed * 2.).sin();
                    point.intensity = 60_000. * flicker;
                }
            }
            (_, Some(light)) => commands.entity(light).despawn(),
            _ => {}
        }
    }
}

#[derive(Component)]
struct FireLight;

#[derive(Resource)]
struct SparkAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Spark {
    velocity: Vec3,
    age: f32,
}

fn spark_assets(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };
    commands.insert_resource(SparkAssets {
        mesh: meshes.add(Sphere::new(0.018)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0.7, 0.3),
            emissive: LinearRgba::rgb(14., 6., 1.5),
            unlit: true,
            ..default()
        }),
    });
    commands.add_observer(throw_sparks);
}

fn throw_sparks(
    strike: On<Strike>,
    mut commands: Commands,
    assets: Res<SparkAssets>,
    camps: Query<&GlobalTransform>,
    mut seed: Local<u32>,
) {
    let Ok(camp) = camps.get(strike.0) else {
        return;
    };
    let from = camp.transform_point(APEX);
    for _ in 0..SPARKS_PER_STRIKE {
        // A cheap deterministic scatter: sparks are decoration, not world generation.
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let a = (*seed >> 8) as f32 / (1 << 24) as f32 * std::f32::consts::TAU;
        let r = ((*seed >> 4) & 0xff) as f32 / 255.;
        let velocity = Vec3::new(a.cos() * (0.4 + r), 1.6 + 1.4 * r, a.sin() * (0.4 + r));
        commands.spawn((
            Spark { velocity, age: 0. },
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.material.clone()),
            NotShadowCaster,
            Transform::from_translation(from),
        ));
    }
}

fn fly_sparks(
    mut commands: Commands,
    time: Res<Time>,
    mut sparks: Query<(Entity, &mut Spark, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut spark, mut transform) in &mut sparks {
        spark.age += dt;
        if spark.age >= SPARK_LIFE_S {
            commands.entity(entity).despawn();
            continue;
        }
        spark.velocity.y -= 3.0 * dt;
        transform.translation += spark.velocity * dt;
        transform.scale = Vec3::splat(1. - spark.age / SPARK_LIFE_S);
    }
}

#[cfg(test)]
mod tests {
    use super::Backbeat;

    #[test]
    fn a_whole_bar_heard_in_order_lights_the_fire() {
        let mut bar = Backbeat::default();
        // Joining mid-bar waits for the next downbeat.
        assert!(!bar.hear(2, 4, true));
        assert!(!bar.hear(3, 4, true));
        assert!(!bar.hear(0, 4, true));
        assert!(!bar.hear(1, 4, true));
        assert!(!bar.hear(2, 4, true));
        assert!(bar.hear(3, 4, true));
        // Changing the gates mid-bar starts again.
        assert!(!bar.hear(0, 4, true));
        assert!(!bar.hear(1, 4, false));
        assert!(!bar.hear(2, 4, true));
        assert!(!bar.hear(3, 4, true));
        assert_eq!(bar, Backbeat { next: 0 });
    }
}
