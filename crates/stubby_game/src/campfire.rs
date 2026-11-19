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
//!   the stream's staff there, riding downstream, a layer over the heartbeat. Its rack
//!   inherits the stream's voice (a fixed VCO at the burbler's pitch, patched), and its
//!   voice is heard from the stream downstream of the join.
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
                inherit_voice,
                strike,
                burn,
                show_fire,
                fly_sparks,
                ghost_staff,
                update_chains,
                draw_in_notes,
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
                // Its layer is heard from the stream downstream of the join, too.
                commands.entity(camp).insert((
                    next,
                    stubby_streams::HeardDownstream {
                        source: next.source,
                        from_m: next.join_m,
                    },
                ));
            }
            (Some(_), None) => {
                commands
                    .entity(camp)
                    .remove::<(StreamJoin, stubby_streams::HeardDownstream)>();
            }
            (None, None) => {}
        }
    }
}

/// A joined camp's rack shows the stream's voice as it arrives (add-stream-layers): its
/// inherited VCO plays at the burbler VCO's pitch, following it when the burbler's turns.
fn inherit_voice(
    mut camps: Query<(&StreamJoin, &mut RackKnobs), With<Fire>>,
    burblers: Query<&RackKnobs, Without<Fire>>,
) {
    for (join, mut knobs) in &mut camps {
        let Some(pitch) = burblers
            .get(join.source)
            .ok()
            .and_then(|k| k.0.get(&("vco".to_owned(), "freq".to_owned())))
            .copied()
        else {
            continue;
        };
        let key = ("voice".to_owned(), "freq".to_owned());
        if knobs.0.get(&key) != Some(&pitch) {
            knobs.0.insert(key, pitch);
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
                commands.entity(camp).insert(LayerPlaying(at));
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

// ------------------------------------------------------------------ the join, seen

/// A camp's ghostly staff (add-stream-layers D4): while its layer plays, the stream's five
/// staff lines are drawn translucent from the join along the ground into the fire's base,
/// as air is pulled into a fire, then drift up from it like heat and fade.
#[derive(Component)]
struct GhostStaff(Entity);

const GHOST_LINES: i32 = 5;
const GHOST_SPACING_M: f32 = 0.12;
const GHOST_LINE_M: f32 = 0.014;
const GHOST_RISE_M: f32 = 3.2;

#[allow(clippy::type_complexity)]
fn ghost_staff(
    mut commands: Commands,
    time: Res<Time>,
    camps: Query<(Entity, &GlobalTransform, Option<&StreamJoin>, &RackEngine), With<Fire>>,
    mut staffs: Query<(Entity, &GhostStaff, &Mesh3d)>,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
) {
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };
    let t = time.elapsed_secs();
    let playing = |camp: Entity| {
        camps
            .get(camp)
            .ok()
            .filter(|(_, _, join, engine)| join.is_some() && engine.sounding)
    };
    let mut shown = Vec::new();
    for (entity, GhostStaff(camp), mesh) in &mut staffs {
        match playing(*camp) {
            Some((_, at, Some(join), _)) => {
                shown.push(*camp);
                if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
                    *mesh = ghost_staff_mesh(join.point, at.translation(), t);
                }
            }
            _ => commands.entity(entity).despawn(),
        }
    }
    let material = material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.86, 0.62, 0.5),
                emissive: LinearRgba::rgb(1.6, 1.1, 0.6),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                ..default()
            })
        })
        .clone();
    for (camp, at, join, engine) in &camps {
        let (Some(join), true) = (join, engine.sounding) else {
            continue;
        };
        if shown.contains(&camp) {
            continue;
        }
        commands.spawn((
            Name::new("Ghost staff"),
            GhostStaff(camp),
            Mesh3d(meshes.add(ghost_staff_mesh(join.point, at.translation(), t))),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            NotShadowCaster,
            bevy::camera::visibility::NoFrustumCulling,
        ));
    }
}

/// The ghost staff's path, as air is drawn into a fire (user, 2026-11-18): from the stream
/// it hugs the ground, pulled in to the fire's base, enters at the bottom, then drifts up
/// like rising heat, wavering and leaning more the higher it goes. Each point carries how
/// far along it is (0..1) and how far apart the lines are there (they converge as they are
/// drawn in, and spread as they rise).
fn ghost_path(join: Vec3, fire: Vec3, t: f32) -> Vec<(Vec3, f32, f32)> {
    let start = join + Vec3::Y * 0.05;
    let base = fire + Vec3::Y * 0.1;
    let (ground, rise) = (18, 36);
    let across = Vec3::new(base.x - start.x, 0., base.z - start.z).normalize_or(Vec3::X);
    let side = across.cross(Vec3::Y).normalize_or(Vec3::Z);
    let mut points = Vec::with_capacity(ground + rise + 1);
    for i in 0..=ground {
        let u = i as f32 / ground as f32;
        // Drawn in faster near the fire, just above the ground.
        let pull = u * u * (3. - 2. * u);
        // Over the stream's bank (it stands proud of the water), then down into the fire.
        let p = start.lerp(base, pull) + Vec3::Y * 0.45 * (std::f32::consts::PI * u).sin();
        points.push((p, 0.35 * u, 1. - 0.55 * u));
    }
    for i in 1..=rise {
        let u = i as f32 / rise as f32;
        let h = GHOST_RISE_M * u;
        // Heat leans on with the draw and wavers, slow and growing with height.
        let lean = across * 0.5 * u.powf(1.5);
        let waver =
            side * (0.18 * u * (t * 0.6 + u * 5.).sin() + 0.08 * u * (t * 1.3 + u * 11.).sin());
        points.push((
            base + Vec3::Y * h + lean + waver,
            0.35 + 0.65 * u,
            0.45 + 1.4 * u,
        ));
    }
    points
}

/// Five thin translucent bands along the path, the staff's lines, side by side; they
/// fade toward the top, and a brighter pulse drifts from the stream up into the air.
fn ghost_staff_mesh(join: Vec3, fire: Vec3, t: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let path = ghost_path(join, fire, t);
    let across = Vec3::new(fire.x - join.x, 0., fire.z - join.z).normalize_or(Vec3::X);
    // Lines sit side by side across the direction of travel.
    let side = across.cross(Vec3::Y).normalize_or(Vec3::Z);
    let (mut positions, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    for line in 0..GHOST_LINES {
        let base = positions.len() as u32;
        for (k, (p, along, spread)) in path.iter().enumerate() {
            let offset = side * (line - GHOST_LINES / 2) as f32 * GHOST_SPACING_M * spread;
            // A slow pulse, drifting at the speed of rising air.
            let pulse = (1. - ((along - (t * 0.22).fract()) * 5.).abs()).max(0.);
            let alpha = (0.3 + 0.7 * pulse) * (1. - along).powf(0.7);
            for edge in [-1., 1.] {
                positions.push((*p + offset + side * edge * GHOST_LINE_M / 2.).to_array());
                colours.push([1., 1., 1., alpha]);
            }
            if k > 0 {
                let i = base + 2 * k as u32;
                indices.extend([i - 2, i - 1, i, i - 1, i + 1, i]);
            }
        }
    }
    let normals = vec![[0., 1., 0.]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

// ------------------------------------------------------------------ the chain, updated

/// When a camp last played a gated step: its layer is playing, not just patched.
#[derive(Component, Clone, Copy)]
struct LayerPlaying(Instant);

/// How long after its last gated step a camp still counts as playing (two bars at the
/// slowest tempo, 24 BPM, would be 20 s; a few seconds keeps the heartbeat from going
/// quiet for long when the camp stops).
const PLAYING_FOR_S: f32 = 6.;

/// How long a note takes from the join, along the ghost staff, into the fire and up.
const DRAW_IN_S: f32 = 2.6;

/// While a joined camp's layer plays, it updates its chain (add-stream-layers D8, user
/// 2026-11-18: the campfire updates the stream rather than adding to it): from its join
/// the chain plays the camp's version, the burbler's own voice falls silent, and the
/// heartbeat's notes leave the staff there to be drawn into the fire.
fn update_chains(
    mut commands: Commands,
    camps: Query<(&StreamJoin, &RackEngine, Option<&LayerPlaying>), With<Fire>>,
    burblers: Query<(Entity, Option<&stubby_streams::ChainUpdated>), With<MachineVoice>>,
) {
    let mut updates: Vec<(Entity, f32)> = Vec::new();
    let now = Instant::now();
    for (join, engine, playing) in &camps {
        // Only a camp actually playing its steps updates the chain: a patched camp whose
        // sequencer is not stepping must not silence the heartbeat (user, 2026-11-18:
        // the stream went quiet).
        let playing = playing
            .is_some_and(|p| now.saturating_duration_since(p.0).as_secs_f32() < PLAYING_FOR_S);
        if !engine.sounding || !playing {
            continue;
        }
        match updates
            .iter_mut()
            .find(|(source, _)| *source == join.source)
        {
            // The first update down the chain wins.
            Some((_, at)) => *at = at.min(join.join_m),
            None => updates.push((join.source, join.join_m)),
        }
    }
    for (burbler, current) in &burblers {
        let wanted = updates
            .iter()
            .find(|(source, _)| *source == burbler)
            .map(|(_, at_m)| stubby_streams::ChainUpdated { at_m: *at_m });
        match (current, wanted) {
            (Some(now), Some(next)) if *now == next => {}
            (_, Some(next)) => {
                commands.entity(burbler).insert(next);
            }
            (Some(_), None) => {
                commands
                    .entity(burbler)
                    .remove::<stubby_streams::ChainUpdated>();
            }
            (None, None) => {}
        }
    }
}

/// A note that reached a camp's join follows the ghost staff, as air is drawn into the
/// fire: over the bank, into the fire's base, then up with the heat, fading.
#[allow(clippy::type_complexity)]
fn draw_in_notes(
    mut commands: Commands,
    time: Res<Time>,
    camps: Query<(&StreamJoin, &GlobalTransform), With<Fire>>,
    mut notes: Query<(
        Entity,
        &stubby_streams::StaffNote,
        &stubby_streams::DivertedNote,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    let now = Instant::now();
    let t = time.elapsed_secs();
    for (entity, note, diverted, mut transform, mut visibility) in &mut notes {
        let u = now.duration_since(diverted.at).as_secs_f32() / DRAW_IN_S;
        let camp = camps
            .iter()
            .filter(|(join, _)| join.source == note.source)
            .min_by(|a, b| a.0.join_m.total_cmp(&b.0.join_m));
        let (Some((join, fire)), true) = (camp, u < 1.) else {
            commands.entity(entity).despawn();
            continue;
        };
        let path = ghost_path(join.point, fire.translation(), t);
        let at = path
            .windows(2)
            .find(|w| u <= w[1].1)
            .map(|w| {
                let span = (w[1].1 - w[0].1).max(1e-4);
                w[0].0.lerp(w[1].0, ((u - w[0].1) / span).clamp(0., 1.))
            })
            .unwrap_or_else(|| path.last().map_or(fire.translation(), |p| p.0));
        transform.translation = at;
        transform.scale = Vec3::splat((1. - u).max(0.05).sqrt());
        *visibility = Visibility::Visible;
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
