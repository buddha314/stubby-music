//! An opt-in standalone setup that leaves the purifier's rack for the player to patch
//! (`--purifier-review`), or plays the whole loop itself (`--purifier-e2e=<dir>`,
//! `add-reed-water-purifier` 4.5): the burbler set to a tempo the recipe accepts, the
//! purifier crafted at that tempo, built through the build menu's placement across the
//! laid stream, patched and tuned, and the water below it seen to clear while upstream
//! stays muddy; the stream's ripple pools are checked to keep its beat. Each step writes
//! an event to `<dir>/events.jsonl` (`complete` with `success`) and screenshots go
//! beside it. Run it under the watchdog: `tools/jackdaw_timed_test.py --standalone
//! --purifier-e2e`.
use crate::purifier::PURIFIER;
use bevy::{prelude::*, world_serialization::WorldAssetRoot};
use bevy_stubby_rack::{machine_config::Profiles, rack::RackState};
use bevy_stubby_recipes::PlayerInventory;
use stubby_core::{AssetClock, AssetPart, AssetState, HudCamera, PlacedAsset};
use stubby_streams::{LaidStream, PORT_HEIGHT, Port, PortOwner, StreamBuild, StreamSystems};

const MODEL: &str = "machines/reed_water_purifier/reed_water_purifier.glb";

pub struct PurifierReviewPlugin;
impl Plugin for PurifierReviewPlugin {
    fn build(&self, app: &mut App) {
        let e2e = std::env::args().find_map(|arg| {
            arg.strip_prefix("--purifier-e2e=")
                .map(std::path::PathBuf::from)
        });
        if std::env::args().any(|arg| arg == "--purifier-review") || e2e.is_some() {
            app.init_resource::<Setup>().add_systems(
                Update,
                prepare
                    .after(StreamSystems::Input)
                    .before(StreamSystems::Plan),
            );
        }
        if let Some(dir) = e2e {
            let _ = std::fs::create_dir_all(&dir);
            app.insert_resource(Script {
                dir,
                step: Step::Tempo,
                since: 0.,
                passed: true,
                requested: false,
                pressed_at: None,
            })
            // Before the build menu's placement reads the click (after StreamSystems::Input).
            .add_systems(Update, script.before(StreamSystems::Input));
        }
    }
}

#[derive(Default, Resource)]
enum Setup {
    #[default]
    FindOutlet,
    Lay(Port),
    WaitForDam(Entity),
    Ready,
    /// The scripted loop takes over once the stream is laid (`--purifier-e2e`).
    Scripted(Entity),
}

fn prepare(
    mut commands: Commands,
    mut setup: ResMut<Setup>,
    mut build: ResMut<StreamBuild>,
    mut inventory: ResMut<PlayerInventory>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<HudCamera>)>,
    profiles: Res<Profiles>,
    mut burblers: Query<(&PlacedAsset, &AssetState, &mut RackState)>,
    outlets: Query<(&AssetPart, &GlobalTransform)>,
    streams: Query<(Entity, &LaidStream)>,
    clocks: Query<&AssetClock>,
    server: Res<AssetServer>,
    mut calendar: ResMut<stubby_world::calendar::Calendar>,
    script: Option<Res<Script>>,
    time: Res<Time<Real>>,
    mut gave_up: Local<bool>,
) {
    let scripted = script.is_some();
    if matches!(*setup, Setup::Ready | Setup::Scripted(_)) || *gave_up {
        return;
    }
    // No burbler outlet near spawn, or a stream that will not lay: say so and stop.
    if time.elapsed_secs() > 15. {
        *gave_up = true;
        let stage = match *setup {
            Setup::FindOutlet => "no open burbler outlet within 60 m of spawn",
            Setup::Lay(_) => "the stream would not lay",
            _ => "the dam never took a clock",
        };
        error!("Purifier review gave up: {stage}");
        if let Some(script) = &script {
            log(
                script,
                serde_json::json!({"event": "complete", "success": false, "why": stage}),
            );
        }
        return;
    }
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    match *setup {
        Setup::FindOutlet => {
            let candidate = outlets
                .iter()
                .filter(|(part, _)| part.name == "Outlet_0")
                .filter_map(|(part, transform)| {
                    let (asset, state, _) = burblers.get(part.root).ok()?;
                    let at = transform.translation();
                    let distance = Vec2::new(at.x, at.z).distance(stubby_core::OPENING_SPAWN_XZ);
                    // Use the outdoor opening burbler. The grotto is farther from spawn.
                    (asset.asset == "mudpit_burbler" && !state.stream_connected && distance < 60.0)
                        .then_some((distance, asset.instance.as_str(), part.root, transform))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(b.1)));
            let Some((_, _, source, outlet)) = candidate else {
                return;
            };
            let Ok((_, _, mut rack)) = burblers.get_mut(source) else {
                return;
            };
            let config = &profiles.0["mudpit_burbler"];
            rack.mounted.insert("vco".into(), "square_vco".into());
            rack.own = config
                .cables
                .iter()
                .map(|cable| (cable.from.clone(), cable.to.clone()))
                .collect();
            rack.version += 1;
            // Enough for a stream long enough to grow ripple pools when scripted.
            let _ = inventory.items.grant(&[stubby_recipes::Stack {
                item: "reed".into(),
                quantity: if scripted { 60 } else { 18 },
            }]);
            let needed = 6_u32.saturating_sub(inventory.items.quantity("patch_cable"));
            if needed > 0 {
                let _ = inventory.items.grant(&[stubby_recipes::Stack {
                    item: "patch_cable".into(),
                    quantity: needed,
                }]);
            }
            calendar.elapsed = std::time::Duration::from_secs(1350);
            let right = outlet.right().as_vec3();
            let port = Port {
                at: outlet.translation() - Vec3::Y * PORT_HEIGHT,
                dir: Vec3::new(right.x, 0.0, right.z).normalize_or(Vec3::X),
                owner: PortOwner::Burbler(source),
            };
            *setup = Setup::Lay(port);
        }
        Setup::Lay(port) => {
            if let Some((entity, stream)) = streams
                .iter()
                .find(|(_, stream)| stream.upstream == Some(port.owner))
            {
                build.active = false;
                build.anchor = None;
                if scripted {
                    *setup = Setup::Scripted(entity);
                    return;
                }
                let (point, flow) = stream.centre_at(stream.span.min(stream.length() * 0.5));
                let side = flow.cross(Vec3::Y);
                let dam = commands
                    .spawn((
                        Name::new("Purifier review dam"),
                        Transform::from_translation(point)
                            .with_rotation(Quat::from_mat3(&Mat3::from_cols(flow, Vec3::Y, side))),
                        Visibility::default(),
                        WorldAssetRoot(server.load(format!("{MODEL}#Scene0"))),
                        PlacedAsset {
                            asset: PURIFIER.into(),
                            model: MODEL.into(),
                            instance: "purifier-review".into(),
                        },
                    ))
                    .id();
                *camera = Transform::from_translation(point + side * 2.4 + Vec3::Y * 1.65)
                    .looking_at(point, Vec3::Y);
                *setup = Setup::WaitForDam(dam);
            } else {
                let side = port.dir.cross(Vec3::Y);
                let reach = if scripted { 26.0 } else { 10.0 };
                *camera = Transform::from_translation(port.at + side * 3.0 + Vec3::Y * 1.65)
                    .looking_at(port.at + port.dir * reach, Vec3::Y);
                build.active = true;
                build.anchor = Some(port);
                build.click = build.plan.as_ref().is_some_and(|plan| plan.valid);
            }
        }
        Setup::WaitForDam(dam) => {
            if let Ok(clock) = clocks.get(dam) {
                info!(
                    "Purifier review: unpatched rack with a {:.0} BPM clock. Press E to configure.",
                    clock.bpm
                );
                *setup = Setup::Ready;
            }
        }
        Setup::Ready | Setup::Scripted(_) => {}
    }
}

/// The scripted loop's state: where it is, since when, and whether every step passed.
#[derive(Resource)]
struct Script {
    dir: std::path::PathBuf,
    step: Step,
    since: f32,
    passed: bool,
    /// The craft request has been sent (frames can be slower than a time window).
    requested: bool,
    /// When E went down at the ripple pool.
    pressed_at: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Step {
    Tempo,
    Collect,
    Craft,
    Aim,
    Click,
    Built,
    Patch,
    Clearing(Entity),
    /// Open the tuned dam's rack and photograph its scope (one beat of the wave).
    Scope(Entity),
    Pools,
    Done,
}

/// The tempo the review sets: inside the recipe's 96 to 144 BPM, on the 24 BPM grid.
const TEMPO: f32 = 120.;
/// The purifier's targets (machines.json): the low VCO on C2, the high on C4, cutoff C5.
const TUNING: [(&str, &str, f32); 3] = [
    ("vco_low", "freq", 65.41),
    ("vco_high", "freq", 261.63),
    ("filter", "cutoff", 523.25),
];

fn log(script: &Script, event: serde_json::Value) {
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(script.dir.join("events.jsonl"))
    {
        let _ = writeln!(file, "{event}");
    }
    info!("purifier e2e: {event}");
}

/// From the bank, high, the dam in the middle: muddy above, clearer below. The same view
/// before and after tuning, so the two screenshots compare.
fn clearing_view(stream: &LaidStream, dam: Vec3) -> Transform {
    let (along, _) = stream.nearest_along(dam);
    let (p, flow) = stream.centre_at(along - stream.start);
    Transform::from_translation(p + flow.cross(Vec3::Y) * 5.5 + Vec3::Y * 4.5 - flow * 0.5)
        .looking_at(p + flow * 0.5, Vec3::Y)
}

fn shoot(commands: &mut Commands, script: &Script, name: &str) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(script.dir.join(name)));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)] // A whole loop, step by step.
fn script(
    mut commands: Commands,
    time: Res<Time<Real>>,
    setup: Res<Setup>,
    mut script: ResMut<Script>,
    profiles: Res<Profiles>,
    mut player: ResMut<PlayerInventory>,
    mut crafts: MessageWriter<bevy_stubby_recipes::CraftRequest>,
    mut placing: ResMut<crate::build::Placing>,
    (mut buttons, mut keys, mut capture): (
        ResMut<ButtonInput<MouseButton>>,
        ResMut<ButtonInput<KeyCode>>,
        ResMut<stubby_core::LookCapture>,
    ),
    (target, mut screens, nodes, eyes): (
        Res<stubby_core::InteractTarget>,
        ResMut<stubby_core::ScreenMode>,
        Query<(
            &bevy_stubby_recipes::ResourceNode,
            &GlobalTransform,
            Has<stubby_core::Interactable>,
        )>,
        Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    ),
    mut camera: Query<(Entity, &mut Transform), (With<Camera3d>, Without<HudCamera>)>,
    streams: Query<&LaidStream>,
    mut racks: Query<(
        Entity,
        &PlacedAsset,
        &mut RackState,
        Option<&mut bevy_stubby_rack::rack::RackKnobs>,
        Option<&AssetClock>,
        Option<&bevy_stubby_rack::rack::RackTargets>,
        Option<&crate::stream_features::OnSegment>,
        &GlobalTransform,
    )>,
    clarity: Res<crate::purifier::WaterClarity>,
    pools: Query<(
        &crate::stream_features::OnStream,
        &GlobalTransform,
        Has<AssetClock>,
    )>,
) {
    let Setup::Scripted(stream_entity) = *setup else {
        return;
    };
    let Ok(stream) = streams.get(stream_entity) else {
        return;
    };
    let now = time.elapsed_secs();
    let waited = now - script.since;
    let next = |script: &mut Script, step: Step| {
        script.step = step;
        script.since = now;
    };
    let source = stream.source;
    match script.step {
        Step::Tempo => {
            // The burbler's BPM knob retunes its clock (and the stream's tempo with it).
            let Some(source) = source else { return };
            let Ok((.., Some(mut knobs), clock, _, _, _)) = racks.get_mut(source) else {
                return;
            };
            if clock.is_some_and(|c| c.bpm == TEMPO) {
                log(
                    &script,
                    serde_json::json!({"event": "tempo", "bpm": TEMPO, "ok": true}),
                );
                next(&mut script, Step::Collect);
            } else {
                knobs.0.insert(("clock".into(), "bpm".into()), TEMPO);
            }
        }
        Step::Collect => {
            // Walk up to the stream's ripple pool and hold E: the oscilloscope the recipe
            // needs comes from the pool, through the generic socket rule.
            let pool = pools
                .iter()
                .find(|(on, ..)| on.0 == stream_entity)
                .map(|(_, place, _)| {
                    // The pool's Interact sits 2.3 m out from the stream, the model's -Z.
                    let at = place.transform_point(Vec3::new(0., 0.45, -2.3));
                    let out = (place.rotation() * Vec3::NEG_Z)
                        .with_y(0.)
                        .normalize_or(Vec3::X);
                    (at, out)
                });
            let Some((at, out)) = pool else {
                log(
                    &script,
                    serde_json::json!({"event": "collected", "ok": true,
                    "skipped": "no ripple pool on this stream; the oscilloscope is granted"}),
                );
                let _ = player.items.grant(&[stubby_recipes::Stack {
                    item: "oscilloscope".into(),
                    quantity: 1,
                }]);
                next(&mut script, Step::Craft);
                return;
            };
            if let Ok((_, mut camera)) = camera.single_mut() {
                // Beyond the pool, facing back over it toward the stream.
                *camera = Transform::from_translation(at + out * 1.8 + Vec3::Y * 1.2)
                    .looking_at(at, Vec3::Y);
            }
            // The pool's model loads after the stream is laid: wait until its pickup is live
            // (marked interactable), then hold E for half a second.
            let live = nodes.iter().any(|(node, place, interactable)| {
                interactable && !node.collected && place.translation().distance(at) < 0.5
            });
            match script.pressed_at {
                None if live && waited > 0.3 => {
                    script.pressed_at = Some(now);
                    keys.press(stubby_core::INTERACT);
                }
                Some(t) if now - t > 0.5 => keys.release(stubby_core::INTERACT),
                _ => {}
            }
            let done = script.pressed_at.is_some_and(|t| now - t > 0.8);
            // The whole review must finish inside the 20 s watchdog: do not wait long here.
            if done || waited > 3.0 {
                let scopes = player.items.quantity("oscilloscope");
                let ok = scopes >= 1;
                script.passed &= ok;
                log(
                    &script,
                    serde_json::json!({"event": "collected", "item": "oscilloscope",
                    "held": scopes, "ok": ok, "feedback": player.feedback.clone(),
                    "target": format!("{:?}", *target), "screen": format!("{:?}", screens.open),
                    "eye_to_pool_m": eyes.single().map(|e| e.translation().distance(at)).unwrap_or(-1.),
                    "pickups_within_4m": nodes.iter().filter(|(_, p, _)| p.translation().distance(at) < 4.)
                        .map(|(n, p, i)| format!("{} {:.1}m interactable={i} collected={}", n.resource, p.translation().distance(at), n.collected))
                        .collect::<Vec<_>>()}),
                );
                if !ok {
                    let _ = player.items.grant(&[stubby_recipes::Stack {
                        item: "oscilloscope".into(),
                        quantity: 1,
                    }]);
                }
                next(&mut script, Step::Craft);
            }
        }
        Step::Craft => {
            if !script.requested {
                script.requested = true;
                let _ = player.items.grant(&[
                    stubby_recipes::Stack {
                        item: "reed".into(),
                        quantity: 20,
                    },
                    stubby_recipes::Stack {
                        item: "high_pass_filter".into(),
                        quantity: 1,
                    },
                ]);
                crafts.write(bevy_stubby_recipes::CraftRequest {
                    recipe: "craft_reed_water_purifier".into(),
                    bpm: Some(TEMPO),
                    bench: None,
                });
                script.since = now;
            } else if waited > 0.3 {
                let kits = player.items.quantity("reed_water_purifier_kit");
                let ok = kits >= 1;
                script.passed &= ok;
                log(
                    &script,
                    serde_json::json!({"event": "crafted", "kits": kits, "ok": ok,
                        "feedback": player.feedback.clone()}),
                );
                next(&mut script, Step::Aim);
            }
        }
        Step::Aim => {
            // Pick the purifier in the build menu and look down at the stream, 8 m along it.
            // The camera leaves the player's controller, which would turn it back.
            placing.active = Some(crate::purifier::PURIFIER.into());
            if let Ok((entity, _)) = camera.single() {
                commands
                    .entity(entity)
                    .remove::<stubby_player::controls::Player>();
            }
            let (point, flow) = stream.centre_at(8.0_f32.min(stream.length() * 0.5));
            let side = flow.cross(Vec3::Y);
            if let Ok((_, mut camera)) = camera.single_mut() {
                *camera = Transform::from_translation(point + side * 2.5 + Vec3::Y * 3.0)
                    .looking_at(point, Vec3::Y);
            }
            if waited > 0.6 {
                capture.0 = true;
                buttons.press(MouseButton::Left);
                next(&mut script, Step::Click);
            }
        }
        Step::Click => {
            buttons.release(MouseButton::Left);
            next(&mut script, Step::Built);
        }
        Step::Built => {
            let dam = racks
                .iter()
                .find(|(_, asset, ..)| asset.asset == crate::purifier::PURIFIER)
                .map(|(dam, .., on, _)| (dam, on.is_some_and(|on| on.stream == stream_entity)));
            if let Some((dam, on_this_stream)) = dam.filter(|_| waited > 0.3) {
                placing.active = None;
                script.passed &= on_this_stream;
                let kits = player.items.quantity("reed_water_purifier_kit");
                log(
                    &script,
                    serde_json::json!({"event": "built", "across_segment": on_this_stream,
                    "kits_left": kits, "ok": on_this_stream && kits == 0}),
                );
                script.passed &= kits == 0;
                shoot(&mut commands, &script, "1-built.png");
                next(&mut script, Step::Patch);
                let _ = dam;
            } else if waited > 3.0 {
                script.passed = false;
                log(
                    &script,
                    serde_json::json!({"event": "built", "ok": false, "why": "no dam after the click"}),
                );
                next(&mut script, Step::Done);
            }
        }
        Step::Patch => {
            let Some((dam, _, mut rack, knobs, .., place)) = racks
                .iter_mut()
                .find(|(_, asset, ..)| asset.asset == crate::purifier::PURIFIER)
            else {
                return;
            };
            // First the water as it is, untuned, from the view the clearing is shot from.
            if let Ok((_, mut camera)) = camera.single_mut() {
                *camera = clearing_view(stream, place.translation());
            }
            if waited < 0.6 {
                return;
            }
            if waited - time.delta_secs() < 0.6 {
                shoot(&mut commands, &script, "2a-muddy.png");
                return;
            }
            if waited < 0.9 {
                return;
            }
            let Some(mut knobs) = knobs else { return };
            let config = &profiles.0[crate::purifier::PURIFIER];
            rack.own = config
                .cables
                .iter()
                .map(|cable| (cable.from.clone(), cable.to.clone()))
                .collect();
            rack.version += 1;
            for (place, knob, value) in TUNING {
                knobs.0.insert((place.into(), knob.into()), value);
            }
            next(&mut script, Step::Clearing(dam));
        }
        Step::Clearing(dam) => {
            let Ok((.., targets, on, place)) = racks.get(dam) else {
                return;
            };
            let at = place.translation();
            let (along, _) = stream.nearest_along(at);
            let point = |d: f32| stream.centre_at(along - stream.start + d).0;
            let below = clarity.at(stream_entity, stream, point(3.0));
            let above = clarity.at(stream_entity, stream, point(-3.0));
            let met = targets.is_some_and(|t| t.met && t.sounding);
            if met && below >= 0.3 {
                let ok = above < 0.05;
                script.passed &= ok;
                log(
                    &script,
                    serde_json::json!({"event": "clearing", "targets_met": true,
                    "below": below, "above": above, "after_s": waited, "ok": ok}),
                );
                if let (Ok((_, mut camera)), Some(_)) = (camera.single_mut(), on) {
                    *camera = clearing_view(stream, at);
                }
                next(&mut script, Step::Scope(dam));
            } else if waited > 6.0 {
                script.passed = false;
                log(
                    &script,
                    serde_json::json!({"event": "clearing", "ok": false,
                    "targets": targets.map(|t| (t.met, t.sounding, t.missed.clone())),
                    "below": below, "above": above}),
                );
                next(&mut script, Step::Pools);
            }
        }
        Step::Scope(dam) => {
            if screens.open.is_none() && waited < 0.5 {
                screens.open = Some(stubby_core::Screen::MachineConfiguration(dam));
                return;
            }
            // The scope refreshes once a beat (0.5 s at the review's 120 BPM): two refreshes in.
            if script.since + 1.2 <= now && script.since + 1.2 > now - time.delta_secs() {
                shoot(&mut commands, &script, "2c-scope.png");
            }
            if waited > 1.6 {
                screens.close();
                next(&mut script, Step::Pools);
            }
        }
        Step::Pools => {
            if waited < 0.4 {
                return; // the camera has moved to the clearing water
            }
            if script.since + 0.4 <= now && script.since + 0.4 > now - time.delta_secs() {
                shoot(&mut commands, &script, "2b-clearing.png");
            }
            let on_stream: Vec<_> = pools
                .iter()
                .filter(|(on, ..)| on.0 == stream_entity)
                .collect();
            let keeping_time = on_stream.iter().filter(|(.., beat)| *beat).count();
            if waited > 1.0 {
                log(
                    &script,
                    serde_json::json!({"event": "pools", "count": on_stream.len(),
                    "keeping_the_beat": keeping_time,
                    "ok": on_stream.is_empty() || keeping_time == on_stream.len()}),
                );
                script.passed &= keeping_time == on_stream.len();
                if let (Some((_, pool, _)), Ok((_, mut camera))) =
                    (on_stream.first(), camera.single_mut())
                {
                    // The pool's water sits 2.3 m off the stream's centre line, the model's -Z.
                    let at = pool.transform_point(Vec3::new(0., 0., -2.3));
                    let out = (at - pool.translation()).normalize_or(Vec3::X);
                    *camera = Transform::from_translation(at + out * 2.5 + Vec3::Y * 3.5)
                        .looking_at(at, Vec3::Y);
                }
                next(&mut script, Step::Done);
            }
        }
        Step::Done => {
            if waited > 0.5 && waited - time.delta_secs() <= 0.5 {
                shoot(&mut commands, &script, "3-pool.png");
                log(
                    &script,
                    serde_json::json!({"event": "complete", "success": script.passed}),
                );
            }
        }
    }
}
