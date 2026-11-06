//! Laying mud streams from ports, Satisfactory's conveyor (`add-stream-building`).
//!
//! Q enters build mode. Aiming near a free port (a burbler's mouth, a laid stream's end)
//! lights its halo; left click anchors there. The plan then curves from the port to the
//! aim point as whole 4 m segments, shown by the stream hologram; left click lays it if
//! it is valid and affordable, and its end becomes the new anchor. Right click drops the
//! anchor; Escape or Q leaves build mode.
//!
//! Segment and hologram meshes are taken once from their glTFs, baked into the asset's
//! root space, and bent along the plan per segment (design D4).
//!
//! A laid stream carries its burbler's voice in two ways. Bar lines ride the staff, one
//! on each downbeat of the audio clock (`add-mudpit-burbler` 2.7), and the voice is heard
//! from the nearest point of the burbler or its streams (`add-localized-machine-voices`).
use super::audio::{ClockTick, Synth};
use super::controls::{LookCapture, Player, ScreenMode};
use super::machine_config::Profiles;
use super::prefab_presentation::AssetPart;
use super::prefabs::{AssetClock, AssetState, PlacedAsset};
use super::rack::RackEngine;
use super::recipes_plugin::{PlayerInventory, RecipeCatalog};
use bevy::{
    camera::visibility::NoFrustumCulling,
    ecs::message::Messages,
    mesh::VertexAttributeValues,
    platform::collections::HashMap,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use std::time::Instant;
use stubby_audio::Command;

/// The buildable whose cost each segment spends (`assets/recipes/buildables.json`).
const SEGMENT: &str = "stream_segment";
const SEGMENT_LENGTH: f32 = 4.0;
/// How far from a port the aim may be to pick it.
const PORT_REACH: f32 = 3.0;
/// The longest plan from its port, in metres (8 segments).
const MAX_PLAN: f32 = 32.0;
/// Tighter than this and the 3 m-wide banks fold over themselves.
const MIN_RADIUS: f32 = 2.0;
/// The hologram's chevrons scroll at the segment's flow speed, wrapping every 0.5 m.
const FLOW_SPEED: f32 = 1.5;
const CHEVRON_SPACING: f32 = 0.5;
/// Port nodes sit this high above a stream's ground origin.
const PORT_HEIGHT: f32 = 0.07;
const COST_RED: Color = Color::srgb(1., 0.35, 0.27);

pub struct StreamBuildPlugin;
impl Plugin for StreamBuildPlugin {
    fn build(&self, app: &mut App) {
        review::install(app);
        app.init_resource::<StreamBuild>()
            .init_resource::<Templates>()
            .insert_resource(StaffProfile::shipped())
            .add_observer(harvest_template)
            .add_systems(Startup, (load_templates, spawn_cost))
            .add_systems(
                Update,
                review::drive
                    .after(build_input)
                    .before(plan)
                    .run_if(resource_exists::<review::StreamReview>),
            )
            .add_systems(
                Update,
                (build_input, plan, lay, show_hologram, show_ports, show_cost)
                    .chain()
                    .before(super::controls::capture_cursor),
            )
            .add_systems(
                Update,
                (
                    emit_bars.run_if(resource_exists::<Messages<ClockTick>>),
                    ride_bars,
                    place_voices,
                )
                    .chain()
                    .after(lay),
            );
    }
}

/// Where a stream can start: a ground-level origin and a horizontal flow direction.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Port {
    at: Vec3,
    dir: Vec3,
    owner: PortOwner,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PortOwner {
    Burbler(Entity),
    Stream(Entity),
}

#[derive(Resource, Default)]
struct StreamBuild {
    active: bool,
    /// A left click this frame, while the cursor was captured.
    click: bool,
    anchor: Option<Port>,
    hover: Option<Port>,
    plan: Option<Plan>,
}
#[derive(Clone)]
struct Plan {
    path: Path,
    segments: u32,
    affordable: bool,
    valid: bool,
}

/// A laid stream; its far end is a port until another stream is laid from it.
#[derive(Component)]
struct LaidStream {
    end: Port,
    end_used: bool,
    end_halo: Vec<Entity>,
    path: Path,
    /// The length of one bent segment: `path.length() / segments`.
    span: f32,
    /// The burbler whose water this is, through any chain of streams.
    source: Option<Entity>,
    /// How far along the whole chain from the burbler this stream starts.
    start: f32,
}

// ---------- Templates: the glTFs' meshes, baked into root space ----------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Role {
    Body,
    HaloOut,
    RingIn,
    RingOut,
    Chevron(f32),
}
#[derive(Clone)]
struct Piece {
    mesh: Mesh,
    material: Handle<StandardMaterial>,
    role: Role,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Segment,
    Hologram,
}
#[derive(Component)]
struct TemplateRoot(Kind);
#[derive(Resource, Default)]
struct Templates {
    segment: Vec<Piece>,
    hologram: Vec<Piece>,
    /// The hologram's materials tinted per state: (valid, invalid) for each piece.
    tints: Vec<(Handle<StandardMaterial>, Handle<StandardMaterial>)>,
    /// The bar line's parts, centred on the staff (`Bar_0` without its place on it).
    bar: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
    pool: Vec<Vec<(Entity, Handle<Mesh>)>>,
}

fn load_templates(mut commands: Commands, server: Res<AssetServer>) {
    for (kind, path) in [
        (Kind::Segment, "models/streams/stream_segment.glb#Scene0"),
        (Kind::Hologram, "models/streams/stream_hologram.glb#Scene0"),
    ] {
        commands.spawn((
            TemplateRoot(kind),
            WorldAssetRoot(server.load(path)),
            Transform::from_xyz(0., -1000., 0.),
            Visibility::Hidden,
        ));
    }
}

/// Once a template glTF has spawned, bake every mesh into the asset's root space and
/// drop the instance.
fn harvest_template(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    roots: Query<&TemplateRoot>,
    children: Query<&Children>,
    nodes: Query<(&Transform, Option<&Name>, Option<&ChildOf>)>,
    pieces: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut templates: ResMut<Templates>,
) {
    let root = ready.entity;
    let Ok(TemplateRoot(kind)) = roots.get(root) else {
        return;
    };
    let mut out = Vec::new();
    let mut bar = Vec::new();
    for entity in children.iter_descendants(root) {
        let Ok((mesh, material)) = pieces.get(entity) else {
            continue;
        };
        let Some(source) = meshes.get(&mesh.0) else {
            continue;
        };
        // Walk up to the root: the transform into root space and the named ancestors.
        let mut affine = Mat4::IDENTITY;
        let mut names = Vec::new();
        let mut at = entity;
        while at != root {
            let Ok((transform, name, parent)) = nodes.get(at) else {
                break;
            };
            // Halos and rings ship at scale 0 (hidden); their full size is scale 1.
            let mut transform = *transform;
            if transform.scale.length_squared() < 1e-6 {
                transform.scale = Vec3::ONE;
            }
            // A bar line keeps its height on the staff; `bar_pose` gives the rest.
            if name.is_some_and(|name| name.as_str() == "Bar_0") {
                transform = Transform::from_xyz(0., transform.translation.y, 0.);
            }
            affine = transform.to_matrix() * affine;
            if let Some(name) = name {
                names.push(name.as_str().to_owned());
            }
            let Some(parent) = parent else { break };
            at = parent.parent();
        }
        let under = |node: &str| names.iter().any(|n| n == node);
        // Notes come with a later pass.
        if under("Pulse_0") || under("Port_in_Halo") {
            continue;
        }
        let mut baked = plain_copy(source);
        transform_mesh(&mut baked, affine);
        if under("Bar_0") {
            bar.push((meshes.add(baked), material.0.clone()));
            continue;
        }
        let role = if under("Port_out_Halo") {
            Role::HaloOut
        } else if under("Port_in_Ring") {
            Role::RingIn
        } else if under("Port_out_Ring") {
            Role::RingOut
        } else if under("Flow") {
            Role::Chevron(centre_x(&baked))
        } else {
            Role::Body
        };
        out.push(Piece {
            mesh: baked,
            material: material.0.clone(),
            role,
        });
    }
    match kind {
        Kind::Segment => {
            templates.segment = out;
            templates.bar = bar;
        }
        Kind::Hologram => {
            let valid = hex("#ffd39a");
            let invalid = hex("#ff5a46");
            templates.tints = out
                .iter()
                .map(|piece| {
                    let base = materials.get(&piece.material).cloned().unwrap_or_default();
                    let tint = |color: LinearRgba| {
                        let mut material = base.clone();
                        let alpha = base.base_color.alpha();
                        material.base_color = Color::from(color).with_alpha(alpha);
                        let glow = base
                            .emissive
                            .red
                            .max(base.emissive.green)
                            .max(base.emissive.blue);
                        material.emissive = color * glow;
                        material
                    };
                    (materials.add(tint(valid)), materials.add(tint(invalid)))
                })
                .collect();
            templates.hologram = out;
        }
    }
    commands.entity(root).despawn();
}

/// The mesh's attributes and indices only. The staff's morph targets (its wave) would
/// not bend with it, and without morph weights on the entity they fail to render.
fn plain_copy(source: &Mesh) -> Mesh {
    let mut mesh = Mesh::new(source.primitive_topology(), source.asset_usage);
    for (attribute, values) in source.attributes() {
        mesh.insert_attribute(attribute.clone(), values.clone());
    }
    if let Some(indices) = source.indices() {
        mesh.insert_indices(indices.clone());
    }
    mesh
}

fn hex(text: &str) -> LinearRgba {
    Srgba::hex(text).map_or(LinearRgba::WHITE, LinearRgba::from)
}

fn positions(mesh: &Mesh) -> Option<&Vec<[f32; 3]>> {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        VertexAttributeValues::Float32x3(values) => Some(values),
        _ => None,
    }
}

fn centre_x(mesh: &Mesh) -> f32 {
    positions(mesh).map_or(0., |p| {
        let (lo, hi) = p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), v| {
            (lo.min(v[0]), hi.max(v[0]))
        });
        (lo + hi) / 2.
    })
}

/// Apply an affine transform to a mesh's positions, normals and tangents.
fn transform_mesh(mesh: &mut Mesh, affine: Mat4) {
    let linear = Mat3::from_mat4(affine);
    let normal = linear.inverse().transpose();
    if let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for v in p {
            *v = affine.transform_point3(Vec3::from(*v)).into();
        }
    }
    if let Some(VertexAttributeValues::Float32x3(n)) = mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        for v in n {
            *v = (normal * Vec3::from(*v)).normalize_or_zero().into();
        }
    }
    if let Some(VertexAttributeValues::Float32x4(t)) = mesh.attribute_mut(Mesh::ATTRIBUTE_TANGENT) {
        for v in t {
            let d = (linear * Vec3::new(v[0], v[1], v[2])).normalize_or_zero();
            *v = [d.x, d.y, d.z, v[3]];
        }
    }
}

// ---------- The plan's path: a cubic Bézier with an arc-length table ----------

#[derive(Clone, Debug)]
struct Path {
    points: [Vec3; 4],
    /// (arc length, curve parameter), increasing.
    table: Vec<(f32, f32)>,
}
impl Path {
    /// Leave `from` along its direction and curve to `to` (design D3).
    fn new(from: &Port, to: Vec3) -> Self {
        let p0 = from.at;
        let p3 = Vec3::new(to.x, p0.y, to.z);
        let reach = p0.distance(p3);
        let p1 = p0 + from.dir * reach / 3.;
        let d1 = (p3 - p1).normalize_or(from.dir);
        let p2 = p3 - d1 * reach / 3.;
        let points = [p0, p1, p2, p3];
        let mut table = vec![(0., 0.)];
        let mut last = p0;
        let mut length = 0.;
        for i in 1..=128 {
            let t = i as f32 / 128.;
            let p = bezier(&points, t);
            length += p.distance(last);
            last = p;
            table.push((length, t));
        }
        Self { points, table }
    }
    fn length(&self) -> f32 {
        self.table.last().map_or(0., |(s, _)| *s)
    }
    fn parameter(&self, s: f32) -> f32 {
        let i = self
            .table
            .partition_point(|(len, _)| *len < s)
            .clamp(1, self.table.len() - 1);
        let (s0, t0) = self.table[i - 1];
        let (s1, t1) = self.table[i];
        t0 + (t1 - t0) * ((s - s0) / (s1 - s0).max(1e-6)).clamp(0., 1.)
    }
    /// Position and horizontal flow direction at arc length `s`, extended straight past
    /// either end.
    fn at(&self, s: f32) -> (Vec3, Vec3) {
        let length = self.length();
        let clamped = s.clamp(0., length);
        let t = self.parameter(clamped);
        let p = bezier(&self.points, t);
        let mut d = bezier_tangent(&self.points, t);
        d.y = 0.;
        let d = d.normalize_or(Vec3::X);
        (p + d * (s - clamped), d)
    }
    /// The tightest bend radius along the path.
    fn min_radius(&self) -> f32 {
        let steps = 64;
        let ds = self.length() / steps as f32;
        let mut min = f32::MAX;
        let mut previous = self.at(0.).1;
        for i in 1..=steps {
            let d = self.at(ds * i as f32).1;
            let angle = previous.angle_between(d);
            if angle > 1e-4 {
                min = min.min(ds / angle);
            }
            previous = d;
        }
        min
    }
}
fn bezier(p: &[Vec3; 4], t: f32) -> Vec3 {
    let u = 1. - t;
    p[0] * u * u * u + p[1] * 3. * u * u * t + p[2] * 3. * u * t * t + p[3] * t * t * t
}
fn bezier_tangent(p: &[Vec3; 4], t: f32) -> Vec3 {
    let u = 1. - t;
    (p[1] - p[0]) * 3. * u * u + (p[2] - p[1]) * 6. * u * t + (p[3] - p[2]) * 3. * t * t
}

/// Bend a root-space segment mesh onto the path: segment-local x in [0, 4] maps to arc
/// `start .. start + span`; y stays up and z runs along the side (tangent × up).
fn bend(source: &Mesh, target: &mut Mesh, path: &Path, start: f32, span: f32, shift: f32) {
    let scale = span / SEGMENT_LENGTH;
    let frame = |x: f32| {
        let (p, d) = path.at(start + x * scale);
        (p, Mat3::from_cols(d, Vec3::Y, d.cross(Vec3::Y)))
    };
    let Some(src) = positions(source) else {
        return;
    };
    let mut frames = Vec::with_capacity(src.len());
    let bent: Vec<[f32; 3]> = src
        .iter()
        .map(|v| {
            let (origin, rotation) = frame(v[0] + shift);
            frames.push(rotation);
            (origin + rotation * Vec3::new(0., v[1], v[2])).into()
        })
        .collect();
    target.insert_attribute(Mesh::ATTRIBUTE_POSITION, bent);
    if let Some(VertexAttributeValues::Float32x3(n)) = source.attribute(Mesh::ATTRIBUTE_NORMAL) {
        let n: Vec<[f32; 3]> = n
            .iter()
            .zip(&frames)
            .map(|(v, r)| (*r * Vec3::from(*v)).into())
            .collect();
        target.insert_attribute(Mesh::ATTRIBUTE_NORMAL, n);
    }
    if let Some(VertexAttributeValues::Float32x4(t)) = source.attribute(Mesh::ATTRIBUTE_TANGENT) {
        let t: Vec<[f32; 4]> = t
            .iter()
            .zip(&frames)
            .map(|(v, r)| {
                let d = *r * Vec3::new(v[0], v[1], v[2]);
                [d.x, d.y, d.z, v[3]]
            })
            .collect();
        target.insert_attribute(Mesh::ATTRIBUTE_TANGENT, t);
    }
}

// ---------- Build mode ----------

fn free_ports(
    burblers: &Query<(Entity, &PlacedAsset, &mut AssetState)>,
    outlets: &Query<(&AssetPart, &GlobalTransform)>,
    streams: &Query<&mut LaidStream>,
) -> Vec<Port> {
    let mut ports: Vec<Port> = outlets
        .iter()
        .filter(|(part, _)| part.name == "Outlet_0")
        .filter_map(|(part, global)| {
            let (_, asset, state) = burblers.get(part.root).ok()?;
            (asset.asset == "mudpit_burbler" && !state.stream_connected).then(|| {
                let mut dir = global.right().as_vec3();
                dir.y = 0.;
                Port {
                    at: global.translation() - Vec3::Y * PORT_HEIGHT,
                    dir: dir.normalize_or(Vec3::X),
                    owner: PortOwner::Burbler(part.root),
                }
            })
        })
        .collect();
    ports.extend(streams.iter().filter(|s| !s.end_used).map(|s| s.end));
    ports
}

/// Where the camera's ray meets the horizontal plane at `height`, at most `MAX_PLAN`
/// from `from` (looking at or above the horizon aims at that limit).
fn aim(camera: &GlobalTransform, height: f32, from: Vec3) -> Vec3 {
    let origin = camera.translation();
    let ray = camera.forward().as_vec3();
    let hit = if ray.y < -0.01 {
        origin + ray * ((height - origin.y) / ray.y)
    } else {
        let flat = Vec3::new(ray.x, 0., ray.z).normalize_or(Vec3::X);
        Vec3::new(origin.x, height, origin.z) + flat * (MAX_PLAN * 2.)
    };
    let offset = Vec3::new(hit.x - from.x, 0., hit.z - from.z);
    let offset = offset.clamp_length_max(MAX_PLAN);
    Vec3::new(from.x + offset.x, height, from.z + offset.z)
}

fn build_input(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    capture: Res<LookCapture>,
    screens: Res<ScreenMode>,
    mut build: ResMut<StreamBuild>,
) {
    // The click that recaptures the cursor is not a build click.
    build.click = capture.captured() && buttons.just_pressed(MouseButton::Left);
    if screens.open.is_some() {
        build.active = false;
    } else if keys.just_pressed(KeyCode::KeyQ) {
        build.active = !build.active;
    } else if keys.just_pressed(KeyCode::Escape) {
        build.active = false;
    } else if build.active && buttons.just_pressed(MouseButton::Right) {
        build.anchor = None;
    }
    if !build.active {
        build.anchor = None;
    }
}

fn plan(
    mut build: ResMut<StreamBuild>,
    cameras: Query<&GlobalTransform, With<Player>>,
    burblers: Query<(Entity, &PlacedAsset, &mut AssetState)>,
    outlets: Query<(&AssetPart, &GlobalTransform)>,
    streams: Query<&mut LaidStream>,
    catalog: Option<Res<RecipeCatalog>>,
    inventory: Option<Res<PlayerInventory>>,
) {
    build.hover = None;
    build.plan = None;
    let Ok(camera) = cameras.single() else { return };
    if !build.active {
        return;
    }
    let ports = free_ports(&burblers, &outlets, &streams);
    // A port taken since it was anchored (another stream) is no longer free.
    if build.anchor.is_some_and(|anchor| !ports.contains(&anchor)) {
        build.anchor = None;
    }
    let Some(anchor) = build.anchor else {
        build.hover = ports
            .iter()
            .map(|port| (port, aim(camera, port.at.y, port.at).distance(port.at)))
            .filter(|(_, distance)| *distance <= PORT_REACH)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(port, _)| *port);
        return;
    };
    let path = Path::new(&anchor, aim(camera, anchor.at.y, anchor.at));
    let length = path.length();
    let segments = ((length / SEGMENT_LENGTH).round() as u32).max(1);
    let affordable = match (&catalog, &inventory) {
        (Some(catalog), Some(inventory)) => catalog
            .0
            .build_shortfall(SEGMENT, segments, &inventory.items)
            .is_ok_and(|short| short.is_empty()),
        _ => false,
    };
    let shape_ok = length >= SEGMENT_LENGTH * 0.75 && path.min_radius() >= MIN_RADIUS;
    build.plan = Some(Plan {
        path,
        segments,
        affordable,
        valid: affordable && shape_ok,
    });
}

#[allow(clippy::too_many_arguments)]
fn lay(
    mut commands: Commands,
    mut build: ResMut<StreamBuild>,
    templates: Res<Templates>,
    mut meshes: ResMut<Assets<Mesh>>,
    catalog: Option<Res<RecipeCatalog>>,
    mut inventory: Option<ResMut<PlayerInventory>>,
    mut burblers: Query<(Entity, &PlacedAsset, &mut AssetState)>,
    mut streams: Query<&mut LaidStream>,
) {
    if !build.active || !build.click {
        return;
    }
    let Some(anchor) = build.anchor else {
        build.anchor = build.hover;
        return;
    };
    let Some(plan) = build.plan.clone().filter(|plan| plan.valid) else {
        return;
    };
    let (Some(catalog), Some(inventory)) = (catalog, inventory.as_mut()) else {
        return;
    };
    if templates.segment.is_empty()
        || catalog
            .0
            .build(SEGMENT, plan.segments, &mut inventory.items)
            .is_err()
    {
        return;
    }
    let span = plan.path.length() / plan.segments as f32;
    let (source, start) = match anchor.owner {
        PortOwner::Burbler(entity) => (Some(entity), 0.),
        PortOwner::Stream(entity) => streams.get(entity).map_or((None, 0.), |upstream| {
            (upstream.source, upstream.start + upstream.path.length())
        }),
    };
    let (end_at, end_dir) = plan.path.at(plan.path.length());
    let mut end_halo = Vec::new();
    let stream = commands
        .spawn((
            Name::new("Mud stream"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    for k in 0..plan.segments {
        let last = k + 1 == plan.segments;
        for piece in &templates.segment {
            if piece.role == Role::HaloOut && !last {
                continue;
            }
            let mut mesh = piece.mesh.clone();
            bend(
                &piece.mesh,
                &mut mesh,
                &plan.path,
                k as f32 * span,
                span,
                0.,
            );
            let halo = piece.role == Role::HaloOut;
            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(piece.material.clone()),
                    Transform::default(),
                    if halo {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    },
                    ChildOf(stream),
                ))
                .id();
            if halo {
                end_halo.push(entity);
            }
        }
    }
    let end = Port {
        at: end_at,
        dir: end_dir,
        owner: PortOwner::Stream(stream),
    };
    commands.entity(stream).insert(LaidStream {
        end,
        end_used: false,
        end_halo,
        path: plan.path.clone(),
        span,
        source,
        start,
    });
    match anchor.owner {
        PortOwner::Burbler(entity) => {
            if let Ok((_, _, mut state)) = burblers.get_mut(entity) {
                state.stream_connected = true;
            }
        }
        PortOwner::Stream(entity) => {
            if let Ok(mut laid) = streams.get_mut(entity) {
                laid.end_used = true;
            }
        }
    }
    // Keep extending from the new end, like dragging on from a conveyor's end.
    build.anchor = Some(end);
}

// ---------- What a laid stream carries: bar lines and the voice ----------

/// The staff and bar settings a stream reads from its profile.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
struct StaffProfile {
    beats_per_bar: u32,
    flow_speed: f32,
    wave_amplitude: f32,
    wavelength: f32,
}
impl StaffProfile {
    fn shipped() -> Self {
        let profile: serde_json::Value = serde_json::from_str(include_str!(
            "../../../assets/streams/stream_segment/asset.json"
        ))
        .expect("stream segment profile");
        let number = |value: &serde_json::Value| value.as_f64().expect("number in profile") as f32;
        Self {
            beats_per_bar: profile["bars"]["beats_per_bar"]["default"]
                .as_u64()
                .expect("beats per bar") as u32,
            flow_speed: number(&profile["notes"]["flow_speed_mps"]),
            wave_amplitude: number(&profile["staff"]["wave"]["amplitude_m"]),
            wavelength: number(&profile["staff"]["wave"]["wavelength_m"]),
        }
    }
}

/// A bar line riding the staff of one burbler's stream.
#[derive(Component)]
struct BarLine {
    machine: Entity,
    /// When its downbeat is heard, from the audio clock.
    at: Instant,
    /// Half a beat in flow time: a bar line sits between the measures.
    lead_seconds: f32,
}

/// How far down the chain a bar line is, `elapsed` seconds after its downbeat.
fn bar_distance(elapsed: f32, lead_seconds: f32, flow_speed: f32) -> f32 {
    (elapsed + lead_seconds) * flow_speed
}

/// A bar line `along` metres down this stream: on the staff's centre line, square to it.
fn bar_pose(path: &Path, span: f32, along: f32, staff: &StaffProfile) -> Transform {
    let k = std::f32::consts::TAU / staff.wavelength;
    // Each bent segment holds the same 4 m of staff, however long the bend made it.
    let x = along.rem_euclid(span) * SEGMENT_LENGTH / span;
    let (origin, dir) = path.at(along);
    let side = dir.cross(Vec3::Y);
    Transform {
        // Blender's sideways wave (+y) is -z in the segment's root space.
        translation: origin - side * staff.wave_amplitude * (k * x).sin(),
        rotation: Quat::from_mat3(&Mat3::from_cols(dir, Vec3::Y, side))
            * Quat::from_rotation_y((staff.wave_amplitude * k * (k * x).cos()).atan()),
        scale: Vec3::ONE,
    }
}

/// One bar line on each downbeat of a sounding burbler that has a stream.
fn emit_bars(
    mut commands: Commands,
    mut ticks: MessageReader<ClockTick>,
    mut beats: Local<HashMap<Entity, u32>>,
    staff: Res<StaffProfile>,
    templates: Res<Templates>,
    machines: Query<(Entity, &AssetClock, &RackEngine)>,
    streams: Query<&LaidStream>,
) {
    for tick in ticks.read() {
        for (machine, clock, engine) in &machines {
            if clock.engine != tick.module {
                continue;
            }
            // Bars start with the sound, and on its first beat.
            if !engine.sounding {
                beats.remove(&machine);
                continue;
            }
            let beat = beats.entry(machine).or_default();
            let downbeat = *beat == 0;
            *beat = (*beat + 1) % staff.beats_per_bar.max(1);
            if !downbeat || !streams.iter().any(|s| s.source == Some(machine)) {
                continue;
            }
            commands
                .spawn((
                    Name::new("Bar line"),
                    BarLine {
                        machine,
                        at: tick.at.unwrap_or_else(Instant::now),
                        lead_seconds: 30. / clock.bpm.max(1.),
                    },
                    Transform::default(),
                    Visibility::Hidden,
                ))
                .with_children(|bar| {
                    for (mesh, material) in &templates.bar {
                        bar.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())));
                    }
                });
        }
    }
}

/// Move bar lines at the water's speed, from the audio clock, not the frame count. One
/// past the end of its chain is gone.
fn ride_bars(
    mut commands: Commands,
    staff: Res<StaffProfile>,
    mut bars: Query<(Entity, &BarLine, &mut Transform, &mut Visibility)>,
    streams: Query<&LaidStream>,
) {
    let now = Instant::now();
    for (entity, bar, mut transform, mut visibility) in &mut bars {
        if now < bar.at {
            continue;
        }
        let elapsed = now.duration_since(bar.at).as_secs_f32();
        let distance = bar_distance(elapsed, bar.lead_seconds, staff.flow_speed);
        let on = streams.iter().find(|s| {
            s.source == Some(bar.machine)
                && (s.start..s.start + s.path.length()).contains(&distance)
        });
        if let Some(stream) = on {
            *transform = bar_pose(&stream.path, stream.span, distance - stream.start, &staff);
            *visibility = Visibility::Visible;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

/// Where a voice is heard from: the machine, or the nearest point of its streams.
fn nearest_source(listener: Vec3, machine: Vec3, streams: &[&Path]) -> Vec3 {
    let mut best = machine;
    for path in streams {
        let steps = (path.length() / 0.5).ceil().max(1.) as u32;
        for k in 0..=steps {
            let (point, _) = path.at(path.length() * k as f32 / steps as f32);
            if point.distance_squared(listener) < best.distance_squared(listener) {
                best = point;
            }
        }
    }
    best
}

/// Silent beyond the radius, full at the source: the same curve as a lightning bug's click.
fn hearing_gain(distance: f32, radius: f32) -> f32 {
    (1. - distance / radius.max(0.001)).clamp(0., 1.).powi(2)
}

/// Tell the engine where each rack's voice is, relative to the player's ears.
fn place_voices(
    synth: Option<Res<Synth>>,
    profiles: Option<Res<Profiles>>,
    camera: Query<
        &GlobalTransform,
        (
            With<Camera3d>,
            Without<super::machine_config::RackHudCamera>,
        ),
    >,
    machines: Query<(Entity, &PlacedAsset, &RackEngine, &GlobalTransform)>,
    streams: Query<&LaidStream>,
) {
    let (Some(synth), Some(profiles), Ok(camera)) = (synth, profiles, camera.single()) else {
        return;
    };
    let (_, ears, listener) = camera.to_scale_rotation_translation();
    for (machine, asset, engine, transform) in &machines {
        let (Some(output), Some(config)) = (engine.voice, profiles.0.get(&asset.asset)) else {
            continue;
        };
        let paths: Vec<&Path> = streams
            .iter()
            .filter(|s| s.source == Some(machine))
            .map(|s| &s.path)
            .collect();
        let source = nearest_source(listener, transform.translation(), &paths);
        synth.send(Command::SetEmitter {
            output,
            offset: (ears.inverse() * (source - listener)).to_array(),
            volume: hearing_gain(source.distance(listener), config.hearing_radius_metres),
        });
    }
}

fn show_hologram(
    mut commands: Commands,
    time: Res<Time>,
    build: Res<StreamBuild>,
    mut templates: ResMut<Templates>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pieces: Query<(&mut Visibility, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let templates = &mut *templates;
    let plan = build.plan.as_ref();
    let wanted = plan.map_or(0, |plan| plan.segments as usize);
    if templates.hologram.is_empty() {
        return;
    }
    while templates.pool.len() < wanted {
        let pool = templates
            .hologram
            .iter()
            .map(|piece| {
                let mesh = meshes.add(piece.mesh.clone());
                let entity = commands
                    .spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(piece.material.clone()),
                        Transform::default(),
                        Visibility::Hidden,
                        NoFrustumCulling,
                        bevy::light::NotShadowCaster,
                    ))
                    .id();
                (entity, mesh)
            })
            .collect();
        templates.pool.push(pool);
    }
    let shift = (time.elapsed_secs() * FLOW_SPEED) % CHEVRON_SPACING;
    for (k, pool) in templates.pool.iter().enumerate() {
        for (index, (entity, handle)) in pool.iter().enumerate() {
            let Ok((mut visibility, mut material)) = pieces.get_mut(*entity) else {
                continue;
            };
            let piece = &templates.hologram[index];
            let shown = plan.is_some_and(|plan| {
                k < plan.segments as usize
                    && match piece.role {
                        Role::RingIn => k == 0,
                        Role::RingOut => k + 1 == plan.segments as usize,
                        _ => true,
                    }
            });
            visibility.set_if_neq(if shown {
                Visibility::Visible
            } else {
                Visibility::Hidden
            });
            let Some(plan) = plan.filter(|_| shown) else {
                continue;
            };
            let (valid, invalid) = &templates.tints[index];
            let tint = if plan.valid { valid } else { invalid };
            if material.0 != *tint {
                material.0 = tint.clone();
            }
            // A chevron scrolls downstream and wraps within its segment.
            let offset = match piece.role {
                Role::Chevron(centre) => {
                    let moved = centre + shift;
                    if moved > SEGMENT_LENGTH {
                        shift - SEGMENT_LENGTH
                    } else {
                        shift
                    }
                }
                _ => 0.,
            };
            let span = plan.path.length() / plan.segments as f32;
            if let Some(mut mesh) = meshes.get_mut(handle) {
                bend(
                    &piece.mesh,
                    &mut mesh,
                    &plan.path,
                    k as f32 * span,
                    span,
                    offset,
                );
            }
        }
    }
}

/// Light the halo of the port being picked or extended from; hide every other.
fn show_ports(
    build: Res<StreamBuild>,
    mut burblers: Query<(Entity, &PlacedAsset, &mut AssetState)>,
    streams: Query<(Entity, &LaidStream)>,
    mut visibility: Query<&mut Visibility>,
) {
    let lit = build.anchor.or(build.hover).map(|port| port.owner);
    for (entity, asset, mut state) in &mut burblers {
        if asset.asset != "mudpit_burbler" {
            continue;
        }
        let on = lit == Some(PortOwner::Burbler(entity));
        if state.port_highlight != on {
            state.port_highlight = on;
        }
    }
    for (entity, stream) in &streams {
        let on = lit == Some(PortOwner::Stream(entity)) && !stream.end_used;
        for halo in &stream.end_halo {
            if let Ok(mut v) = visibility.get_mut(*halo) {
                v.set_if_neq(if on {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                });
            }
        }
    }
}

// ---------- Reed cost under the reticle ----------

#[derive(Component)]
struct CostBox;
#[derive(Component)]
struct CostText;

fn spawn_cost(mut commands: Commands, server: Option<Res<AssetServer>>) {
    commands
        .spawn((
            CostBox,
            Name::new("Stream cost"),
            Visibility::Hidden,
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                // Below the interaction prompt under the reticle.
                top: percent(63),
                margin: UiRect::left(px(-70)),
                width: px(140),
                padding: UiRect::axes(px(10), px(4)),
                column_gap: px(8),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            // Readable over a hologram and the bright mudflat alike.
            BackgroundColor(Color::srgba(0.05, 0.04, 0.03, 0.72)),
        ))
        .with_children(|row| {
            if let Some(server) = &server {
                row.spawn((
                    super::inventory_screen::item_icon(server, "reed"),
                    Node {
                        width: px(32),
                        height: px(32),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
            row.spawn((
                CostText,
                Text::new(""),
                TextFont {
                    font_size: bevy::text::FontSize::Px(20.),
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        });
}

fn show_cost(
    build: Res<StreamBuild>,
    catalog: Option<Res<RecipeCatalog>>,
    inventory: Option<Res<PlayerInventory>>,
    mut boxes: Query<&mut Visibility, With<CostBox>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<CostText>>,
) {
    let segments = build.plan.as_ref().map_or(1, |plan| plan.segments);
    let need = catalog
        .as_ref()
        .and_then(|catalog| catalog.0.build_cost(SEGMENT, segments).ok())
        .and_then(|cost| cost.into_iter().find(|s| s.item == "reed"))
        .map_or(0, |s| s.quantity);
    let have = inventory.as_ref().map_or(0, |i| i.items.quantity("reed"));
    for mut visibility in &mut boxes {
        visibility.set_if_neq(if build.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    for (mut text, mut color) in &mut texts {
        let value = format!("{need} / {have}");
        if text.0 != value {
            text.0 = value;
        }
        let short = build.plan.as_ref().is_some_and(|plan| !plan.affordable) || have < need;
        color.set_if_neq(TextColor(if short { COST_RED } else { Color::WHITE }));
    }
}

/// `--stream-review=<dir>`: a scripted pass over the scene's burbler that plans, lays
/// and fails to afford a stream, saving screenshots and `events.jsonl` to `<dir>`. Run
/// it under the graphical watchdog (`tools/jackdaw_timed_test.py`).
mod review {
    use super::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    use std::path::PathBuf;

    #[derive(Resource)]
    pub(super) struct StreamReview {
        dir: PathBuf,
        step: usize,
    }
    pub(super) fn install(app: &mut App) {
        if let Some(dir) =
            std::env::args().find_map(|arg| arg.strip_prefix("--stream-review=").map(PathBuf::from))
        {
            let _ = std::fs::create_dir_all(&dir);
            app.insert_resource(StreamReview { dir, step: 0 });
        }
    }
    fn log(review: &StreamReview, event: serde_json::Value) {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(review.dir.join("events.jsonl"))
        {
            let _ = writeln!(file, "{event}");
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn drive(
        mut commands: Commands,
        time: Res<Time<Real>>,
        mut review: ResMut<StreamReview>,
        mut build: ResMut<StreamBuild>,
        mut cameras: Query<&mut Transform, With<Player>>,
        outlets: Query<(&AssetPart, &GlobalTransform)>,
        mut inventory: Option<ResMut<PlayerInventory>>,
        streams: Query<&LaidStream>,
        bars: Query<&BarLine>,
        mut racks: Query<&mut RackEngine>,
        mut calendar: ResMut<super::super::calendar::Calendar>,
    ) {
        let Some((_, outlet)) = outlets.iter().find(|(part, _)| part.name == "Outlet_0") else {
            return;
        };
        let at = outlet.translation() - Vec3::Y * PORT_HEIGHT;
        let mut dir = outlet.right().as_vec3();
        dir.y = 0.;
        let dir = dir.normalize_or(Vec3::X);
        let side = dir.cross(Vec3::Y);
        let times = [4., 5., 5.5, 7., 7.5, 8.5, 9.5, 11., 12., 13.];
        let t = time.elapsed_secs();
        let Some(&due) = times.get(review.step) else {
            return;
        };
        if t < due {
            return;
        }
        let shoot = |commands: &mut Commands, review: &StreamReview, name: &str| {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(review.dir.join(name)));
        };
        let Ok(mut camera) = cameras.single_mut() else {
            return;
        };
        let look = |camera: &mut Transform, from: Vec3, to: Vec3| {
            *camera = Transform::from_translation(from).looking_at(to, Vec3::Y);
        };
        match review.step {
            // Stand by the mouth with 18 reeds, enter build mode, aim at the mouth.
            0 => {
                // Midday, so the screenshots show the stream (the game opens at dawn).
                calendar.elapsed = std::time::Duration::from_secs(1350);
                if let Some(inventory) = inventory.as_mut() {
                    let _ = inventory.items.grant(&[stubby_recipes::Stack {
                        item: "reed".into(),
                        quantity: 18,
                    }]);
                }
                build.active = true;
                look(&mut camera, at - dir * 4. + side * 3. + Vec3::Y * 1.7, at);
            }
            1 => {
                shoot(&mut commands, &review, "1-port-hover.png");
                build.click = true;
            }
            // Aim off to the side: a curving two-segment plan, affordable.
            2 => look(
                &mut camera,
                at - dir * 3. + side * 4. + Vec3::Y * 2.2,
                at + dir * 7. + side * 4.,
            ),
            3 => shoot(&mut commands, &review, "2-plan-valid.png"),
            4 => {
                build.click = true;
                // Show bar lines without patching anything: their gate stays silent.
                for mut rack in &mut racks {
                    rack.sounding = true;
                }
            }
            // Plan on from the new end: 6 reeds left, too few.
            5 => {
                if let Some(end) = streams.iter().next().map(|s| s.end) {
                    look(
                        &mut camera,
                        end.at - end.dir * 3. + Vec3::Y * 2.2,
                        end.at + end.dir * 9.,
                    );
                }
            }
            6 => shoot(&mut commands, &review, "3-plan-unaffordable.png"),
            // Leave build mode and look over what was laid, then the join at the mouth.
            7 => {
                build.active = false;
                // From the side at eye height (the controller keeps the player grounded).
                look(
                    &mut camera,
                    at + dir * 3. - side * 7. + Vec3::Y * 1.7,
                    at + dir * 4. + side * 2.,
                );
            }
            8 => shoot(&mut commands, &review, "4-laid.png"),
            9 => {
                // From behind the mouth, over the pool, down the stream.
                look(&mut camera, at - dir * 2.5 + Vec3::Y * 2.2, at + dir * 4.);
                shoot(&mut commands, &review, "5-mouth.png");
            }
            _ => {}
        }
        log(
            &review,
            serde_json::json!({
                "step": review.step,
                "seconds": t,
                "active": build.active,
                "hover": build.hover.is_some(),
                "anchor": build.anchor.is_some(),
                "plan": build.plan.as_ref().map(|p| serde_json::json!({
                    "segments": p.segments, "length": p.path.length(),
                    "valid": p.valid, "affordable": p.affordable,
                })),
                "streams": streams.iter().count(),
                "bars": bars.iter().count(),
                "reeds": inventory.as_ref().map(|i| i.items.quantity("reed")),
            }),
        );
        review.step += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port() -> Port {
        Port {
            at: Vec3::ZERO,
            dir: Vec3::X,
            owner: PortOwner::Stream(Entity::PLACEHOLDER),
        }
    }

    fn staff() -> StaffProfile {
        StaffProfile {
            beats_per_bar: 4,
            flow_speed: 1.5,
            wave_amplitude: 0.07,
            wavelength: 2.0,
        }
    }

    #[test]
    fn the_shipped_profile_gives_the_bars_settings() {
        assert_eq!(StaffProfile::shipped(), staff());
    }

    #[test]
    fn bar_lines_are_a_measure_apart_at_any_tempo() {
        let staff = staff();
        for bpm in [60., 90., 140.] {
            let beat = 60. / bpm;
            let first = bar_distance(
                staff.beats_per_bar as f32 * beat,
                beat / 2.,
                staff.flow_speed,
            );
            let second = bar_distance(0., beat / 2., staff.flow_speed);
            let spacing = staff.beats_per_bar as f32 * staff.flow_speed * beat;
            assert!((first - second - spacing).abs() < 1e-4);
        }
    }

    #[test]
    fn a_bar_line_sits_on_the_staff_square_to_it() {
        // Straight along +X: the pose reduces to the staff's own wave.
        let path = Path::new(&port(), port().at + Vec3::X * 8.);
        let staff = staff();
        let span = 4.;
        let k = std::f32::consts::TAU / staff.wavelength;
        for x in [0.25_f32, 0.5, 1.5, 3.2] {
            let pose = bar_pose(&path, span, x, &staff);
            let on_staff = port().at.z - staff.wave_amplitude * (k * x).sin();
            assert!((pose.translation.x - port().at.x - x).abs() < 0.02);
            assert!((pose.translation.z - on_staff).abs() < 1e-3);
            // A line across the channel turned by the wave's slope, as Blender's bar is.
            let across = pose.rotation * Vec3::Z;
            let yaw = (staff.wave_amplitude * k * (k * x).cos()).atan();
            assert!((across.x - yaw.sin()).abs() < 1e-3);
        }
    }

    #[test]
    fn the_voice_comes_from_the_nearest_point_of_machine_or_stream() {
        let path = Path::new(&port(), port().at + Vec3::X * 20.);
        let machine = port().at - Vec3::X * 2.;
        // Standing by the far end of the stream: heard from there, not the machine.
        let far = port().at + Vec3::new(20., 0., 3.);
        assert!(nearest_source(far, machine, &[&path]).distance(port().at + Vec3::X * 20.) < 0.6);
        // Standing behind the machine: heard from the machine.
        let behind = machine - Vec3::X * 5.;
        assert_eq!(nearest_source(behind, machine, &[&path]), machine);
        assert_eq!(nearest_source(far, machine, &[]), machine);
    }

    #[test]
    fn hearing_fades_to_silence_at_the_radius() {
        assert_eq!(hearing_gain(0., 30.), 1.);
        assert!(hearing_gain(15., 30.) > hearing_gain(25., 30.));
        assert_eq!(hearing_gain(30., 30.), 0.);
        assert_eq!(hearing_gain(50., 30.), 0.);
    }

    #[test]
    fn path_leaves_along_the_port_and_ends_at_the_aim() {
        let path = Path::new(&port(), Vec3::new(8., 0., 6.));
        let (start, d0) = path.at(0.);
        assert!(start.distance(Vec3::ZERO) < 1e-4);
        assert!(d0.distance(Vec3::X) < 1e-3);
        assert!(path.at(path.length()).0.distance(Vec3::new(8., 0., 6.)) < 1e-3);
        // Straight ahead is a straight line; a hairpin behind the port is too tight.
        assert!((Path::new(&port(), Vec3::new(12., 0., 0.)).length() - 12.).abs() < 1e-3);
        assert!(Path::new(&port(), Vec3::new(-3., 0., 1.)).min_radius() < MIN_RADIUS);
    }

    #[test]
    fn bending_a_straight_path_keeps_the_segment_shape() {
        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0., 0., 0.5], [4., 0.1, -0.5], [2., 0., 0.]],
        );
        let path = Path::new(&port(), Vec3::new(8., 0., 0.));
        let mut bent = mesh.clone();
        bend(&mesh, &mut bent, &path, 4., 4., 0.);
        let p = positions(&bent).unwrap();
        assert!(Vec3::from(p[0]).distance(Vec3::new(4., 0., 0.5)) < 1e-3);
        assert!(Vec3::from(p[1]).distance(Vec3::new(8., 0.1, -0.5)) < 1e-3);
    }
}
