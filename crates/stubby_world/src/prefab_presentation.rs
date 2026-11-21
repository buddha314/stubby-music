//! Named-node presentation and interaction hooks shared by authored assets.
use crate::prefabs::{AssetClock, AssetState, PlacedAsset, PrefabError};
use bevy::{
    animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex},
    gltf::{Gltf, GltfExtras},
    prelude::*,
};
use bevy_stubby_audio::Synth;
use bevy_stubby_audio::pulse::PulseBus;
use std::time::Instant;
use stubby_core::PrefabBinding;

#[derive(Resource, Default)]
pub struct HoveredAsset {
    pub entity: Option<Entity>,
    pub instance: String,
}
#[derive(Message, Clone, Debug)]
pub struct AssetHoverEvent {
    pub entity: Entity,
    pub instance: String,
    pub entered: bool,
}
pub use stubby_core::AssetUsed;
#[derive(Component)]
struct HoverLabel;
#[derive(Component)]
struct HoverRing;
pub use stubby_core::AssetPart;
#[derive(Component)]
struct AnimationSource(Handle<Gltf>);
#[derive(Component)]
struct ClockAnimations {
    root: Entity,
    clips: Vec<(AnimationNodeIndex, f32, f32)>,
}
#[derive(Resource)]
struct FloraProfiles {
    oak: serde_json::Value,
    reeds: serde_json::Value,
    props: serde_json::Value,
    burbler: serde_json::Value,
}

pub struct PrefabPresentationPlugin;
impl Plugin for PrefabPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MaterialTint>()
            .register_type::<LoopClip>();
        app.add_systems(
            PostUpdate,
            (
                restore_start_scale.before(bevy::transform::TransformSystems::Propagate),
                mark_acoustic_rock,
            ),
        )
        .init_resource::<HoveredAsset>()
        .add_message::<AssetHoverEvent>()
        .add_message::<AssetUsed>()
        .insert_resource(FloraProfiles {
            oak: serde_json::from_str(include_str!("../../../assets/flora/scrub_oak/asset.json"))
                .expect("oak profile"),
            reeds: serde_json::from_str(include_str!("../../../assets/flora/reeds/asset.json"))
                .expect("reed profile"),
            burbler: serde_json::from_str(include_str!(
                "../../../assets/machines/mudpit_burbler/asset.json"
            ))
            .expect("burbler profile"),
            props: serde_json::from_str(include_str!("../../../assets/props/desert/asset.json"))
                .expect("prop profile"),
        })
        .add_systems(Startup, setup_hover)
        .add_systems(
            Update,
            (
                find_parts,
                animation_sources,
                prepare_animations,
                loop_idle_clips,
                update_hover,
                hover_ring,
                find_materials,
            )
                .chain()
                .after(PrefabBinding)
                .after(stubby_core::MachineInteractionSet),
        )
        .add_systems(
            PostUpdate,
            (
                pose_parts,
                shade_materials,
                seek_animations,
                seek_idle_clips,
            )
                .before(bevy::app::AnimationSystems),
        );
    }
}
/// The name line of what the aim rests on.
#[derive(Component)]
struct HoverTitle;

/// A placed asset's name for the player: its kind without a variant number, in sentence
/// case (`boulder_3` is "Boulder", `mudpit_burbler` "Mudpit burbler").
fn display_name(asset: &str) -> String {
    let kind = asset
        .rsplit_once('_')
        .filter(|(_, n)| n.chars().all(|c| c.is_ascii_digit()))
        .map_or(asset, |(kind, _)| kind)
        .replace('_', " ");
    let mut chars = kind.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn setup_hover(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // What the aim rests on, as Satisfactory names it: the name in the HUD's face with a
    // muted detail line under it, just above the E prompt (the game's `CollectionPopup`,
    // at 55%), so name and action read as one block (user, 2026-11-18).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                // Below the reticle, just above the prompt.
                top: percent(51.8),
                width: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(2),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|block| {
            block.spawn((
                HoverLabel,
                HoverTitle,
                stubby_ui::style::world_label("", 18., stubby_ui::style::INK),
            ));
            block.spawn((
                HoverLabel,
                stubby_ui::style::world_label("", 13., stubby_ui::style::MUTED),
            ));
        });
    commands.spawn((
        HoverRing,
        Mesh3d(meshes.add(Torus::new(0.5, 0.52))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.85, 0.55),
            unlit: true,
            ..default()
        })),
        bevy::light::NotShadowCaster,
        Transform::default(),
        Visibility::Hidden,
    ));
}
/// Parts the exporter wrote at full scale but tagged `start_scale` (hidden at start:
/// dormant leaves, closed blooms, stubble, halos) go back to that scale as they load,
/// before transforms propagate, so they are never drawn grown on their first frame. Their
/// children keep the transforms a zero-scale export would have lost
/// (`sync-blender-asset-contracts`). The presenter reads a zero rest scale as full scale 1.
fn restore_start_scale(mut parts: Query<(&GltfExtras, &mut Transform), Added<GltfExtras>>) {
    for (extras, mut transform) in &mut parts {
        let start = serde_json::from_str::<serde_json::Value>(&extras.value)
            .ok()
            .and_then(|value| value.get("start_scale")?.as_f64());
        if let Some(start) = start {
            transform.scale = Vec3::splat(start as f32);
        }
    }
}

/// Model nodes whose glTF extras say `"acoustic": "rock"` are acoustic rock: every mesh
/// under them gets `AcousticOccluder::ROCK` as it loads, so it enters the spatial-audio
/// scene (add-echo-grotto; the boulders' near level). No asset kind is named.
fn mark_acoustic_rock(
    mut commands: Commands,
    acoustic_roots: Query<(Entity, &stubby_core::AcousticOccluder), Without<Mesh3d>>,
    nodes: Query<(Entity, &GltfExtras)>,
    children: Query<&Children>,
    all_meshes: Query<(), With<Mesh3d>>,
    meshes: Query<(), (With<Mesh3d>, Without<stubby_core::AcousticOccluder>)>,
    mut trace_counts: Local<std::collections::HashMap<Entity, usize>>,
) {
    let trace = std::env::var_os("STUBBY_AUDIO_SPATIAL_TRACE").is_some();
    for (root, material) in &acoustic_roots {
        for entity in children.iter_descendants(root) {
            if meshes.contains(entity) {
                commands.entity(entity).insert(*material);
            }
        }
    }
    for (node, extras) in &nodes {
        let Some(occluder) = serde_json::from_str::<serde_json::Value>(&extras.value)
            .ok()
            .and_then(|value| {
                value
                    .get("acoustic")
                    .and_then(|v| v.as_str())
                    .and_then(stubby_core::AcousticOccluder::preset)
            })
        else {
            continue;
        };
        let mesh_count = std::iter::once(node)
            .chain(children.iter_descendants(node))
            .filter(|entity| all_meshes.contains(*entity))
            .count();
        if trace && trace_counts.get(&node) != Some(&mesh_count) {
            eprintln!(
                "AudioNimbus acoustic tag node={node:?} extras={} mesh_descendants={mesh_count}",
                extras.value,
            );
            trace_counts.insert(node, mesh_count);
        }
        for entity in std::iter::once(node).chain(children.iter_descendants(node)) {
            if meshes.contains(entity) {
                commands.entity(entity).insert(occluder);
            }
        }
    }
}

fn find_parts(
    mut commands: Commands,
    nodes: Query<(Entity, &Name, &Transform, Option<&GltfExtras>), Without<AssetPart>>,
    parents: Query<&ChildOf>,
    roots: Query<(), With<PlacedAsset>>,
) {
    for (entity, name, transform, extras) in &nodes {
        if roots.contains(entity) {
            continue;
        }
        let Some(root) = parents
            .iter_ancestors(entity)
            .find(|ancestor| roots.contains(*ancestor))
        else {
            continue;
        };
        let rest = *transform;
        commands.entity(entity).insert(AssetPart {
            root,
            name: name.as_str().into(),
            rest,
            flattened: false,
            sway: None,
            extras: {
                let mut value: serde_json::Value = extras
                    .and_then(|extras| serde_json::from_str(&extras.value).ok())
                    .unwrap_or_else(|| serde_json::json!({}));
                let count = nodes
                    .iter()
                    .filter(|(child, name, _, _)| {
                        indexed(name.as_str(), "Bloom_")
                            && parents.iter_ancestors(*child).any(|parent| parent == root)
                    })
                    .count();
                value["runtime_bloom_count"] = serde_json::json!(count);
                value
            },
        });
    }
}
fn indexed(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix)
        .is_some_and(|index| index.parse::<usize>().is_ok())
}
pub fn is_state_node(name: &str) -> bool {
    ["Dead_", "Bud_", "Foliage_", "Bloom_"]
        .iter()
        .any(|prefix| indexed(name, prefix))
        || matches!(
            name,
            "Standing"
                | "Stubble"
                | "ReedClump_Far"
                | "ScrubOak_Far_Foliage_Node"
                | "ScrubOak_LOD_Far_Foliage_Node"
        )
}
fn smooth(value: f32, from: f32, to: f32) -> f32 {
    let value = ((value - from) / (to - from).max(0.0001)).clamp(0., 1.);
    value * value * (3. - 2. * value)
}
fn ramp(profile: &serde_json::Value, key: &str, value: f32) -> f32 {
    let bounds = &profile["growth"]["ramps"][key];
    smooth(
        value,
        bounds[0].as_f64().unwrap_or(0.) as f32,
        bounds[1].as_f64().unwrap_or(1.) as f32,
    )
}
fn full_scale(rest: Vec3) -> Vec3 {
    if rest.length_squared() < 0.000001 {
        Vec3::ONE
    } else {
        rest
    }
}
fn node_pose(
    asset: &str,
    state: &AssetState,
    collected: bool,
    part: &AssetPart,
    profiles: &FloraProfiles,
    elapsed: f32,
    seed: u64,
) -> Transform {
    let mut pose = part.rest;
    let name = part.name.as_str();
    let growth = state.growth.clamp(0., 1.);
    if asset.starts_with("scrub_oak_") {
        let factor = if indexed(name, "Dead_") {
            Some(1. - ramp(&profiles.oak, "dead_leaves_fall", growth))
        } else if indexed(name, "Bud_") {
            Some(
                (0.4 + 0.6 * ramp(&profiles.oak, "buds_swell", growth))
                    * (1. - ramp(&profiles.oak, "buds_open", growth)),
            )
        } else if indexed(name, "Foliage_")
            || (name == "ScrubOak_Far_Foliage_Node" || name == "ScrubOak_LOD_Far_Foliage_Node")
        {
            Some(ramp(&profiles.oak, "leaf_out", growth))
        } else if indexed(name, "Bloom_") {
            let k = name
                .trim_start_matches("Bloom_")
                .parse::<f32>()
                .unwrap_or(0.);
            let from = part.extras["bloom_from"]
                .as_f64()
                .map(|value| value as f32)
                .unwrap_or(
                    0.72 + k * 0.25
                        / part.extras["runtime_bloom_count"]
                            .as_f64()
                            .unwrap_or(1.)
                            .max(1.) as f32,
                );
            Some(smooth(growth, from, from + 0.03))
        } else {
            None
        };
        if let Some(factor) = factor {
            pose.scale =
                full_scale(part.rest.scale) * factor / if indexed(name, "Bud_") { 0.4 } else { 1. };
        }
    } else if asset.starts_with("reed_clump_") {
        if name == "Standing" {
            pose.scale = part.rest.scale * if collected { 0. } else { 1. };
        }
        if name == "ReedClump_Far" {
            pose.scale = part.rest.scale * if collected { 0. } else { 1. };
        }
        if name == "Stubble" {
            pose.scale = full_scale(part.rest.scale) * if collected { 1. } else { 0. };
        }
    } else if asset.starts_with("desert_") && indexed(name, "Bloom_") {
        let from = part.extras["from"].as_f64().unwrap_or(0.2) as f32;
        let to = part.extras["to"].as_f64().unwrap_or(0.8) as f32;
        pose.scale = full_scale(part.rest.scale) * smooth(state.light_quality, from, to);
    }
    let profile = if asset.starts_with("scrub_oak_") {
        &profiles.oak
    } else if asset.starts_with("reed_clump_") {
        &profiles.reeds
    } else {
        &profiles.props
    };
    if name.starts_with("Limb_") || name.starts_with("Sway_") {
        let max = profile["sway"]["max_deg"].as_f64().unwrap_or(0.) as f32;
        let index = name
            .rsplit('_')
            .next()
            .and_then(|part| part.parse::<f32>().ok())
            .unwrap_or(0.);
        let phase = (seed % 1000) as f32 / 1000. * std::f32::consts::TAU;
        pose.rotation = part.rest.rotation
            * Quat::from_rotation_x((elapsed * 1.2 + phase + index).sin() * max.to_radians());
    }
    pose
}
#[allow(clippy::type_complexity)] // Roots combine presentation state with optional source and clock roles.
fn pose_parts(
    time: Res<Time>,
    profiles: Res<FloraProfiles>,
    roots: Query<
        (
            &PlacedAsset,
            &AssetState,
            Has<stubby_core::Harvested>,
            Option<&AssetClock>,
        ),
        Without<PrefabError>,
    >,
    bus: Res<PulseBus>,
    synth: Option<Res<Synth>>,
    mut parts: Query<(&AssetPart, &mut Transform)>,
) {
    for (part, mut transform) in &mut parts {
        let Ok((asset, state, resource, clock)) = roots.get(part.root) else {
            continue;
        };
        // A part tagged `shown_while: beat` shows only while its placement keeps a clock (a
        // ripple pool's rings, while its stream sounds); otherwise it is still and hidden.
        if part.extras["shown_while"] == "beat" {
            transform.scale = full_scale(part.rest.scale) * f32::from(u8::from(clock.is_some()));
            continue;
        }
        // A stream port's halo (burbler outlet, stream ends) ships hidden and appears
        // only while the player is trying to connect a stream there.
        if part.extras["role"] == "port_highlight" {
            transform.scale =
                full_scale(part.rest.scale) * f32::from(u8::from(state.port_highlight));
            continue;
        }
        if asset.asset == "mudpit_burbler" {
            if part.name.starts_with("Wisp_")
                || part.name.starts_with("Steam_")
                || part.name.starts_with("Mote_")
            {
                let elapsed = clock.and_then(|clock| {
                    synth
                        .as_ref()
                        .and_then(|synth| synth.thread())
                        .and_then(|thread| thread.sample_rate())
                        .and_then(|rate| bus.elapsed(clock.clock, Instant::now(), rate))
                });
                *transform = part.rest;
                if let (Some(elapsed), Some(clock)) = (elapsed, clock) {
                    let index = part
                        .name
                        .rsplit('_')
                        .next()
                        .and_then(|part| part.parse::<f64>().ok())
                        .unwrap_or(0.);
                    let beat = 60. / f64::from(clock.bpm);
                    if part.name.starts_with("Mote_") {
                        transform.translation.y += ((elapsed + index) * 1.5).sin() as f32 * 0.04;
                    } else {
                        let life = part.extras["life_s"].as_f64().unwrap_or(1.8);
                        let cycle = (life / beat).ceil() * beat;
                        let age = (elapsed - index * beat).rem_euclid(cycle);
                        let fraction = (age / life).clamp(0., 1.) as f32;
                        transform.translation.y +=
                            part.extras["rise_m"].as_f64().unwrap_or(2.2) as f32 * fraction;
                        transform.scale = part.rest.scale * (1. - fraction).max(0.).sqrt();
                    }
                } else {
                    transform.scale = Vec3::ZERO;
                }
            }
            continue;
        }
        if !(asset.asset.starts_with("scrub_oak_")
            || asset.asset.starts_with("reed_clump_")
            || asset.asset.starts_with("desert_"))
        {
            continue;
        }
        let seed = asset
            .instance
            .bytes()
            .fold(0xcbf29ce484222325_u64, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(1099511628211)
            });
        *transform = node_pose(
            &asset.asset,
            state,
            resource,
            part,
            &profiles,
            time.elapsed_secs(),
            seed,
        );
        if part.flattened
            && let Some((pivot, index)) = part.sway
        {
            let profile = if asset.asset.starts_with("scrub_oak_") {
                &profiles.oak
            } else {
                &profiles.reeds
            };
            let max = profile["sway"]["max_deg"].as_f64().unwrap_or(0.) as f32;
            let phase = (seed % 1000) as f32 / 1000. * std::f32::consts::TAU;
            let rotation = Quat::from_rotation_x(
                (time.elapsed_secs() * 1.2 + phase + index).sin() * max.to_radians(),
            );
            transform.translation = pivot + rotation * (transform.translation - pivot);
            transform.rotation = rotation * transform.rotation;
        }
    }
}
#[allow(clippy::type_complexity)] // Select roots not yet carrying an animation source.
fn animation_sources(
    mut commands: Commands,
    assets: Res<AssetServer>,
    roots: Query<(Entity, &PlacedAsset), (With<AssetClock>, Without<AnimationSource>)>,
) {
    for (entity, asset) in &roots {
        commands
            .entity(entity)
            .insert(AnimationSource(assets.load(asset.model.clone())));
    }
}
#[allow(clippy::too_many_arguments)] // glTF assets and their ECS animation players are separate stores.
fn prepare_animations(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    players: Query<Entity, (With<AnimationPlayer>, Without<ClockAnimations>)>,
    parents: Query<&ChildOf>,
    roots: Query<&AnimationSource, With<AssetClock>>,
    parts: Query<&AssetPart>,
) {
    for player in &players {
        let Some(root) = std::iter::once(player)
            .chain(parents.iter_ancestors(player))
            .find(|root| roots.contains(*root))
        else {
            continue;
        };
        let source = roots.get(root).expect("root matched");
        let Some(gltf) = gltfs.get(&source.0) else {
            continue;
        };
        let mut named: Vec<_> = gltf
            .named_animations
            .iter()
            // One-beat clips, played on the placement's clock: the burbler's `Bubble_<n>_Burble`
            // and any model's `<name>_Beat` (a ripple pool's rings).
            .filter(|(name, _)| {
                (name.starts_with("Bubble_") && name.ends_with("_Burble"))
                    || name.ends_with("_Beat")
            })
            .collect();
        named.sort_by_key(|(name, _)| *name);
        if named.is_empty() {
            continue;
        }
        let (graph, indices) =
            AnimationGraph::from_clips(named.iter().map(|(_, clip)| (*clip).clone()));
        let binding = named
            .into_iter()
            .zip(indices)
            .map(|((name, handle), index)| {
                let node = name.trim_end_matches("_Burble").trim_end_matches("_Beat");
                let division = parts
                    .iter()
                    .find(|part| part.root == root && part.name == node)
                    .and_then(|part| part.extras["division"].as_f64())
                    .unwrap_or(1.) as f32;
                let duration = clips.get(handle).map_or(1., AnimationClip::duration);
                (index, division, duration)
            })
            .collect();
        commands.entity(player).insert((
            AnimationGraphHandle(graphs.add(graph)),
            ClockAnimations {
                root,
                clips: binding,
            },
        ));
    }
}
/// A placement's own colour for one of its model's materials, by name (a Dancer's skin tone
/// on `Dancer_Skin`). sRGB, 0..1.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct MaterialTint {
    pub material: String,
    pub srgb: [f32; 3],
}

/// Which of its model's clips a placement loops, in time with the nearest clock as `Idle`
/// is (a Dancer set to `Dance`). Without it, `Idle`.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct LoopClip {
    pub clip: String,
}

/// How far a placed asset looks for a running clock to keep its idle in time with.
pub const IDLE_CLOCK_REACH_M: f32 = 30.0;

/// A model's `Idle` clip plays on a placed asset that has no clock of its own (the cyber-cob
/// by the vardo), in time with the music (user, 2026-11-20): phase-locked to the nearest
/// running clock within `IDLE_CLOCK_REACH_M`, or to the master clock when none is near. Any
/// model with an `Idle` clip comes alive this way; no code per kind.
#[derive(Component)]
struct IdleClip {
    root: Entity,
    node: AnimationNodeIndex,
    duration: f32,
}
/// Marks a player already looked at (with or without an `Idle` clip).
#[derive(Component)]
struct IdleChecked;
/// The model a clock-free placed asset's clips come from, held so it stays loaded.
#[derive(Component)]
struct IdleSource(Handle<Gltf>);

/// Where in an idle clip of `duration` seconds to be after `beats` beats of a clock at
/// `bpm`: one cycle spans a whole number of beats, or a beat holds a whole number of
/// cycles, whichever keeps the clip nearest its authored speed; so idles land on the beat.
pub fn idle_phase(beats: f64, bpm: f64, duration: f32) -> f32 {
    let duration = f64::from(duration.max(0.001));
    let beat = 60. / bpm.max(1e-6);
    let cycles = if beat <= duration {
        beats / (duration / beat).round().max(1.)
    } else {
        beats * (beat / duration).round().max(1.)
    };
    (cycles.rem_euclid(1.) * duration) as f32
}

#[allow(clippy::type_complexity)] // glTF assets, players and their placed roots.
fn loop_idle_clips(
    mut commands: Commands,
    assets: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    players: Query<
        Entity,
        (
            With<AnimationPlayer>,
            Without<ClockAnimations>,
            Without<IdleChecked>,
        ),
    >,
    parents: Query<&ChildOf>,
    roots: Query<(&PlacedAsset, Option<&IdleSource>, Option<&LoopClip>), Without<AssetClock>>,
) {
    for player in &players {
        let Some(root) = parents.iter_ancestors(player).find(|e| roots.contains(*e)) else {
            continue;
        };
        let (asset, source, chosen) = roots.get(root).expect("root matched");
        let Some(source) = source else {
            commands
                .entity(root)
                .insert(IdleSource(assets.load(asset.model.clone())));
            continue;
        };
        let Some(gltf) = gltfs.get(&source.0) else {
            continue;
        };
        let name = chosen.map_or("Idle", |c| c.clip.as_str());
        let Some(idle) = gltf.named_animations.get(name) else {
            commands.entity(player).insert(IdleChecked);
            continue;
        };
        let duration = clips.get(idle).map_or(1., AnimationClip::duration);
        let (graph, node) = AnimationGraph::from_clip(idle.clone());
        info!(
            "{}: its {name} clip plays in time with the nearest clock",
            asset.instance
        );
        commands.entity(player).insert((
            AnimationGraphHandle(graphs.add(graph)),
            IdleClip {
                root,
                node,
                duration,
            },
            IdleChecked,
        ));
    }
}

#[allow(clippy::type_complexity)] // Clock sources, positions and players are separate stores.
fn seek_idle_clips(
    bus: Res<PulseBus>,
    synth: Option<Res<Synth>>,
    master: Option<Res<bevy_stubby_audio::MasterClock>>,
    clocks: Query<(&AssetClock, &GlobalTransform)>,
    places: Query<&GlobalTransform>,
    mut players: Query<(&IdleClip, &mut AnimationPlayer)>,
) {
    let rate = synth.and_then(|synth| synth.thread().and_then(|thread| thread.sample_rate()));
    let now = Instant::now();
    for (idle, mut player) in &mut players {
        let at = places
            .get(idle.root)
            .map_or(Vec3::ZERO, GlobalTransform::translation);
        // The nearest running clock in reach, as (beats, bpm); else the master clock.
        let near = rate.and_then(|rate| {
            clocks
                .iter()
                .filter_map(|(clock, place)| {
                    let d = place.translation().distance(at);
                    let elapsed = bus.elapsed(clock.clock, now, rate)?;
                    (d <= IDLE_CLOCK_REACH_M).then_some((
                        d,
                        elapsed * f64::from(clock.bpm) / 60.,
                        f64::from(clock.bpm),
                    ))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, beats, bpm)| (beats, bpm))
        });
        let timing = near.or_else(|| {
            master
                .as_ref()
                .and_then(|m| m.beats())
                .map(|beats| (beats, bevy_stubby_audio::MASTER_BPM))
        });
        let active = player.play(idle.node).repeat();
        match timing {
            Some((beats, bpm)) => {
                active
                    .pause()
                    .set_seek_time(idle_phase(beats, bpm, idle.duration));
            }
            // No engine at all (a silent build): let it run free rather than freeze.
            None => {
                active.resume();
            }
        }
    }
}

fn seek_animations(
    bus: Res<PulseBus>,
    synth: Option<Res<Synth>>,
    roots: Query<&AssetClock>,
    mut players: Query<(&ClockAnimations, &mut AnimationPlayer)>,
) {
    let Some(rate) = synth.and_then(|synth| synth.thread().and_then(|thread| thread.sample_rate()))
    else {
        return;
    };
    for (animations, mut player) in &mut players {
        let Ok(clock) = roots.get(animations.root) else {
            continue;
        };
        let Some(elapsed) = bus.elapsed(clock.clock, Instant::now(), rate) else {
            continue;
        };
        for &(node, division, duration) in &animations.clips {
            let phase = (elapsed * f64::from(clock.bpm) / 60. / f64::from(division.max(0.01)))
                .rem_euclid(f64::from(duration.max(0.001))) as f32;
            player.play(node).repeat().pause().set_seek_time(phase);
        }
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Independent ECS stores for hover state, target metadata and feedback.
fn update_hover(
    mut hover: ResMut<HoveredAsset>,
    keys: Res<ButtonInput<KeyCode>>,
    cameras: Query<Entity, (With<Camera3d>, Without<stubby_core::HudCamera>)>,
    roots: Query<
        (
            Entity,
            &PlacedAsset,
            Has<stubby_core::Harvested>,
            Option<&AssetClock>,
            Option<&stubby_core::Interactable>,
        ),
        Without<PrefabError>,
    >,
    transforms: bevy::transform::helper::TransformHelper,
    mut events: MessageWriter<AssetHoverEvent>,
    mut uses: MessageWriter<AssetUsed>,
    mut labels: Query<(&mut Text, Has<HoverTitle>), With<HoverLabel>>,
    screens: Option<Res<stubby_core::ScreenMode>>,
    interact: Option<Res<stubby_core::InteractTarget>>,
) {
    // The machine E acts on is the one named and ringed, so the ring, the label and the
    // E prompt never disagree (2026-11-18: a campfire ringed while E configured the
    // burbler behind it). With no machine in reach, the aim picks as before.
    let reached = interact
        .and_then(|target| target.machine())
        .filter(|machine| roots.contains(*machine));
    let camera = cameras
        .single()
        .ok()
        .and_then(|camera| transforms.compute_global_transform(camera).ok());
    let target = camera
        .filter(|_| screens.as_ref().is_none_or(|s| s.open.is_none()))
        .and_then(|camera| {
            let forward = camera.forward();
            roots
                .iter()
                .filter(|(_, asset, resource, _, _)| {
                    !asset.asset.starts_with("desert_") && !*resource
                })
                .filter_map(|(entity, asset, _, _, _)| {
                    let global = transforms.compute_global_transform(entity).ok()?;
                    let mut center = global.translation();
                    if asset.asset != "lightning_bug" {
                        center.y += if asset.asset == "mudpit_burbler" {
                            0.3
                        } else {
                            0.8
                        };
                    }
                    let delta = center - camera.translation();
                    if Vec2::new(delta.x, delta.z).length_squared() > 9. {
                        return None;
                    }
                    let along = delta.dot(*forward);
                    let radius = if asset.asset == "mudpit_burbler"
                        || asset.asset.starts_with("scrub_oak_")
                    {
                        1.5
                    } else {
                        0.7
                    };
                    (along >= 0. && (delta - *forward * along).length() <= radius)
                        .then_some((entity, along))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(entity, _)| entity)
        });
    let target = reached
        .filter(|_| screens.as_ref().is_none_or(|s| s.open.is_none()))
        .or(target);
    if target != hover.entity {
        if let Some(previous) = hover.entity {
            events.write(AssetHoverEvent {
                entity: previous,
                instance: hover.instance.clone(),
                entered: false,
            });
        }
        if let Some(next) = target
            && let Ok((_, asset, _, _, _)) = roots.get(next)
        {
            events.write(AssetHoverEvent {
                entity: next,
                instance: asset.instance.clone(),
                entered: true,
            });
        }
        hover.entity = target;
        hover.instance = target
            .and_then(|entity| roots.get(entity).ok())
            .map(|(_, asset, _, _, _)| asset.instance.clone())
            .unwrap_or_default();
    }
    let selected = target.and_then(|target| roots.get(target).ok());
    let (title, detail) = selected.map_or_else(
        || (String::new(), String::new()),
        |(_, asset, _, _, what)| {
            if what == Some(&stubby_core::Interactable::Pickup) {
                // The E prompt names pickups ("Collect lightning bug").
                (String::new(), String::new())
            } else if asset.asset.starts_with("scrub_oak_") {
                ("Scrub oak".into(), "Infinite lightning-bug mine".into())
            } else {
                (display_name(&asset.asset), String::new())
            }
        },
    );
    for (mut label, is_title) in &mut labels {
        let text = if is_title { &title } else { &detail };
        if &label.0 != text {
            label.0.clone_from(text);
        }
    }
    if keys.just_pressed(stubby_core::INTERACT)
        && let Some((entity, asset, _, _, _)) = selected
        && asset.asset != "mudpit_burbler"
    {
        uses.write(AssetUsed {
            entity,
            instance: asset.instance.clone(),
            asset: asset.asset.clone(),
        });
    }
}

fn hover_ring(
    hover: Res<HoveredAsset>,
    roots: Query<(&PlacedAsset, &GlobalTransform)>,
    mut rings: Query<(&mut Transform, &mut Visibility), With<HoverRing>>,
) {
    let position = hover
        .entity
        .and_then(|entity| roots.get(entity).ok())
        .filter(|(asset, _)| asset.asset != "lightning_bug")
        .map(|(_, global)| global.translation());
    for (mut transform, mut visible) in &mut rings {
        *visible = if position.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some(position) = position {
            transform.translation = Vec3::new(
                position.x,
                crate::terrain::height(Vec2::new(position.x, position.z)) + 0.06,
                position.z,
            );
        }
    }
}

#[derive(Component)]
struct InstanceMaterial {
    root: Entity,
    name: String,
    handle: Handle<StandardMaterial>,
    rest: StandardMaterial,
}
fn find_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            &bevy::gltf::GltfMaterialName,
        ),
        Without<InstanceMaterial>,
    >,
    parents: Query<&ChildOf>,
    roots: Query<(&PlacedAsset, Has<MaterialTint>)>,
) {
    for (entity, handle, name) in &meshes {
        let Some(root) = parents
            .iter_ancestors(entity)
            .find(|root| roots.contains(*root))
        else {
            continue;
        };
        let Ok((asset, tinted)) = roots.get(root) else {
            continue;
        };
        if !(tinted
            || asset.asset.starts_with("scrub_oak_")
            || asset.asset.starts_with("reed_clump_")
            || asset.asset.starts_with("desert_")
            || asset.asset == "mudpit_burbler")
        {
            continue;
        }
        let Some(rest) = materials.get(&handle.0).cloned() else {
            continue;
        };
        let handle = materials.add(rest.clone());
        commands.entity(entity).insert((
            MeshMaterial3d(handle.clone()),
            InstanceMaterial {
                root,
                name: name.0.clone(),
                handle,
                rest,
            },
        ));
    }
}
fn hex(value: &serde_json::Value) -> Option<Vec3> {
    let value = value.as_str()?.strip_prefix('#')?;
    if value.len() != 6 {
        return None;
    }
    Some(
        Vec3::new(
            u8::from_str_radix(&value[0..2], 16).ok()? as f32,
            u8::from_str_radix(&value[2..4], 16).ok()? as f32,
            u8::from_str_radix(&value[4..6], 16).ok()? as f32,
        ) / 255.,
    )
}
fn color_between(colors: &serde_json::Value, from: &str, to: &str, level: f32) -> Option<Color> {
    let color = hex(&colors[from])?.lerp(hex(&colors[to])?, level.clamp(0., 1.));
    Some(Color::srgb(color.x, color.y, color.z))
}
fn shade_materials(
    profiles: Res<FloraProfiles>,
    bus: Res<PulseBus>,
    roots: Query<(
        &PlacedAsset,
        &AssetState,
        Option<&AssetClock>,
        Option<&MaterialTint>,
    )>,
    parts: Query<&InstanceMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for part in &parts {
        let Ok((asset, state, clock, tint)) = roots.get(part.root) else {
            continue;
        };
        let Some(mut material) = materials.get_mut(&part.handle) else {
            continue;
        };
        if let Some(tint) = tint.filter(|tint| tint.material == part.name) {
            let [r, g, b] = tint.srgb;
            material.base_color = Color::srgb(r, g, b);
            continue;
        }
        let color = if asset.asset.starts_with("scrub_oak_") {
            let colors = &profiles.oak["growth"]["colors"];
            if part.name == "ScrubOak_Bark" {
                color_between(
                    colors,
                    "bark_dormant",
                    "bark_alive",
                    ramp(&profiles.oak, "bark_wakes", state.growth),
                )
            } else if part.name == "ScrubOak_Leaf" {
                color_between(
                    colors,
                    "leaf_young",
                    "leaf_mature",
                    ramp(&profiles.oak, "leaf_matures", state.growth),
                )
            } else {
                None
            }
        } else if asset.asset.starts_with("reed_clump_") {
            let colors = &profiles.reeds["greenness"]["colors"];
            let green = state.water_quality.max(state.light_quality);
            if part.name == "ReedClump_Stalk" {
                color_between(colors, "stalk_dry", "stalk_green", green)
            } else if part.name == "ReedClump_Blade" {
                color_between(colors, "blade_dry", "blade_green", green)
            } else {
                None
            }
        } else if asset.asset.starts_with("desert_") {
            let colors = &profiles.props["bloom"]["materials"][&part.name];
            hex(&colors[0]).zip(hex(&colors[1])).map(|(from, to)| {
                let color = from.lerp(to, state.light_quality.clamp(0., 1.));
                Color::srgb(color.x, color.y, color.z)
            })
        } else {
            None
        };
        if let Some(color) = color {
            material.base_color = color;
        }
        if let Some(clock) = clock {
            let profile = &profiles.burbler["lights"][&part.name];
            if !profile.is_null() {
                let rest = profile["rest"].as_f64().unwrap_or(0.) as f32;
                let peak = profile["peak"].as_f64().unwrap_or(1.) as f32;
                let attack = profile["attack_s"].as_f64().unwrap_or(0.04) as f32;
                let decay = profile["decay_s"].as_f64().unwrap_or(0.45) as f32;
                let level = bus.level(clock.clock, Instant::now(), attack, attack + decay);
                // The port halo is only visible while a connection is attempted.
                let strength = if part.name == "Burbler_OutletHalo" {
                    if state.port_highlight { peak } else { 0. }
                } else {
                    rest + (peak - rest) * level
                };
                material.emissive = part.rest.emissive * strength;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_clips_land_on_the_beat() {
        // A 3.33 s idle at 120 BPM (0.5 s beats): seven beats per cycle, restarting on a beat.
        assert_eq!(idle_phase(0.0, 120.0, 3.33), 0.0);
        assert!(idle_phase(7.0, 120.0, 3.33).abs() < 1e-4);
        // At the 1 BPM master clock (60 s beats): eighteen cycles per beat, one on each beat.
        assert!(idle_phase(1.0, 1.0, 3.33).abs() < 1e-3);
        assert!((idle_phase(0.5 / 18.0, 1.0, 3.33) - 1.665).abs() < 1e-3);
    }
    fn profiles() -> FloraProfiles {
        FloraProfiles {
            oak: serde_json::from_str(include_str!("../../../assets/flora/scrub_oak/asset.json"))
                .unwrap(),
            reeds: serde_json::from_str(include_str!("../../../assets/flora/reeds/asset.json"))
                .unwrap(),
            props: serde_json::from_str(include_str!("../../../assets/props/desert/asset.json"))
                .unwrap(),
            burbler: serde_json::from_str(include_str!(
                "../../../assets/machines/mudpit_burbler/asset.json"
            ))
            .unwrap(),
        }
    }
    #[test]
    fn meshes_under_acoustic_rock_extras_become_rock_occluders() {
        let mut app = App::new();
        app.add_systems(Update, mark_acoustic_rock);
        let extras = |value: &str| GltfExtras {
            value: value.into(),
        };
        let rock = app
            .world_mut()
            .spawn(extras(r#"{"acoustic": "rock"}"#))
            .id();
        let plain = app.world_mut().spawn(extras(r#"{"start_scale": 0}"#)).id();
        let plain_mesh = app
            .world_mut()
            .spawn((Mesh3d(Handle::default()), ChildOf(plain)))
            .id();
        app.update();
        // glTF extras may arrive before the mesh descendants finish loading.
        let rock_mesh = app
            .world_mut()
            .spawn((Mesh3d(Handle::default()), ChildOf(rock)))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<stubby_core::AcousticOccluder>(rock_mesh),
            Some(&stubby_core::AcousticOccluder::ROCK)
        );
        assert!(
            app.world()
                .get::<stubby_core::AcousticOccluder>(plain_mesh)
                .is_none()
        );
    }

    #[test]
    fn prefab_acoustic_material_reaches_mesh_children_loaded_later() {
        let mut app = App::new();
        app.add_systems(Update, mark_acoustic_rock);
        let root = app
            .world_mut()
            .spawn(stubby_core::AcousticOccluder::ROCK)
            .id();
        app.update();
        let mesh = app
            .world_mut()
            .spawn((Mesh3d(Handle::default()), ChildOf(root)))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<stubby_core::AcousticOccluder>(mesh),
            Some(&stubby_core::AcousticOccluder::ROCK)
        );
    }

    #[test]
    fn tagged_parts_load_hidden_and_others_keep_their_scale() {
        let mut app = App::new();
        app.add_systems(Update, restore_start_scale);
        let extras = |value: &str| GltfExtras {
            value: value.into(),
        };
        let tagged = app
            .world_mut()
            .spawn((extras(r#"{"start_scale":0.0}"#), Transform::default()))
            .id();
        let plain = app
            .world_mut()
            .spawn((extras(r#"{"other":1}"#), Transform::default()))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Transform>(tagged).unwrap().scale,
            Vec3::ZERO
        );
        assert_eq!(
            app.world().get::<Transform>(plain).unwrap().scale,
            Vec3::ONE
        );
    }

    #[test]
    fn authored_zero_scale_nodes_can_bloom_and_reeds_show_stubble() {
        let profiles = profiles();
        let part = AssetPart {
            root: Entity::PLACEHOLDER,
            name: "Bloom_16".into(),
            rest: Transform::from_scale(Vec3::ZERO),
            extras: serde_json::json!({"runtime_bloom_count":17}),
            flattened: false,
            sway: None,
        };
        let dormant = AssetState::default();
        let grown = AssetState {
            growth: 1.,
            light_quality: 1.,
            ..default()
        };
        assert_eq!(
            node_pose("scrub_oak_0", &dormant, false, &part, &profiles, 0., 42).scale,
            Vec3::ZERO
        );
        assert_eq!(
            node_pose("scrub_oak_0", &grown, false, &part, &profiles, 0., 42).scale,
            Vec3::ONE
        );
        let stubble = AssetPart {
            name: "Stubble".into(),
            ..part
        };
        assert_eq!(
            node_pose("reed_clump_0", &dormant, true, &stubble, &profiles, 0., 42).scale,
            Vec3::ONE
        );
        assert_eq!(
            node_pose("reed_clump_0", &dormant, false, &stubble, &profiles, 0., 42).scale,
            Vec3::ZERO
        );
    }
    #[test]
    fn hover_is_transition_based_and_leaves_when_entity_is_removed() {
        let mut app = App::new();
        app.init_resource::<HoveredAsset>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<AssetHoverEvent>()
            .add_message::<AssetUsed>()
            .add_systems(Update, update_hover);
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::from_xyz(0., 1., 2.).looking_at(Vec3::new(0., 0.8, 0.), Vec3::Y),
        ));
        let root = app
            .world_mut()
            .spawn((
                PlacedAsset {
                    asset: "clock_lamp".into(),
                    instance: "clock".into(),
                    model: "clock.glb".into(),
                },
                Transform::default(),
            ))
            .id();
        app.update();
        let first: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<AssetHoverEvent>>()
            .drain()
            .collect();
        assert_eq!(first.len(), 1);
        assert!(first[0].entered);
        app.update();
        assert!(
            app.world_mut()
                .resource_mut::<Messages<AssetHoverEvent>>()
                .drain()
                .next()
                .is_none()
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        let used: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<AssetUsed>>()
            .drain()
            .collect();
        assert_eq!(used.len(), 1);
        assert_eq!(used[0].instance, "clock");
        app.world_mut().despawn(root);
        app.update();
        let left: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<AssetHoverEvent>>()
            .drain()
            .collect();
        assert_eq!(left.len(), 1);
        assert!(!left[0].entered);
        assert_eq!(left[0].instance, "clock");
    }
    #[test]
    fn audio_elapsed_uses_sample_timestamps_across_skipped_frames() {
        let now = Instant::now();
        let mut bus = PulseBus::default();
        bus.queue(bevy_stubby_audio::pulse::SamplePulse {
            clock: 5,
            sample_time: 100,
            at: now,
        });
        bus.advance(now);
        bus.queue(bevy_stubby_audio::pulse::SamplePulse {
            clock: 5,
            sample_time: 96100,
            at: now + std::time::Duration::from_secs(2),
        });
        bus.advance(now + std::time::Duration::from_secs(2));
        assert!(
            (bus.elapsed(5, now + std::time::Duration::from_millis(2100), 48000)
                .unwrap()
                - 2.1)
                .abs()
                < 0.0001
        );
        bus.forget(5);
        assert!(bus.elapsed(5, now, 48000).is_none());
    }
}
