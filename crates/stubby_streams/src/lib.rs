//! Laying mud streams from ports, Satisfactory's conveyor (`add-stream-building`).
//!
//! The build menu (Q, the game's) starts stream building; Q or Escape leaves it. Aiming near a free port (a burbler's mouth, a laid stream's end)
//! lights its halo; left click anchors there. The plan then curves from the port to the
//! aim point as whole 4 m segments, shown by the stream hologram; left click lays it if
//! it is valid and affordable, and its end becomes the new anchor. Right click drops the
//! anchor; Escape or Q leaves build mode.
//!
//! Outside build mode, F dismantles the aimed segment, refunds it and splits the stream
//! (`dismantle`).
//!
//! Segment and hologram meshes are taken once from their glTFs, baked into the asset's
//! root space, and bent along the plan per segment (design D4).
//!
//! A laid stream carries its burbler's voice in two ways. Bar lines and notes ride the
//! staff, a bar line on each downbeat and a note on each beat of the audio clock
//! (`add-mudpit-burbler` 2.7), and the voice is heard from the nearest point of the
//! burbler or its streams (`add-localized-machine-voices`).
//!
//! A stream accumulates a composition (`add-stream-layers`): the heartbeat's notes start
//! at the burbler's mouth, and a machine joined further down adds its own notes from its
//! join (`LayerNote`), so they ride only downstream of it.
use bevy::{
    camera::visibility::NoFrustumCulling,
    ecs::message::Messages,
    mesh::VertexAttributeValues,
    platform::collections::HashMap,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};
use bevy_stubby_audio::{ClockTick, Synth};
use std::marker::PhantomData;
use std::time::Instant;
use stubby_audio::{Command, ListenerPose};
use stubby_core::ScreenMode;
use stubby_core::{
    AssetClock, AssetPart, AssetState, HudCamera, LookCapture, MachineVoice, PlacedAsset, Stock,
    StreamInlet,
};

mod dismantle;
pub use dismantle::{Dismantle, pick_segment};

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
pub const PORT_HEIGHT: f32 = 0.07;
const COST_RED: Color = Color::srgb(1., 0.35, 0.27);

/// What one stream segment costs, item by item. The game sets it from its rules
/// (`buildables.json`); streams only take it from the player's stock.
#[derive(Resource, Default, Clone)]
pub struct SegmentCost(pub Vec<(String, u32)>);

/// Stream building's steps, for anything that drives it (a scripted review).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamSystems {
    /// Build-mode input (Q, clicks).
    Input,
    /// Planning, laying and showing the plan.
    Plan,
}

/// Laying streams, taking each segment's cost from the stock `S`.
pub struct StreamBuildPlugin<S>(PhantomData<S>);
impl<S> Default for StreamBuildPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<S: Stock> Plugin for StreamBuildPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<SegmentCost>()
            .add_plugins(stubby_ui::ItemTooltipPlugin)
            .init_resource::<StreamBuild>()
            .init_resource::<stubby_core::BuildMode>()
            .init_resource::<Templates>()
            .insert_resource(StaffProfile::shipped())
            .add_observer(harvest_template)
            .add_systems(Startup, (load_templates, spawn_cost))
            .add_systems(
                Update,
                (
                    (build_input, publish_build_mode)
                        .chain()
                        .in_set(StreamSystems::Input),
                    (
                        plan::<S>,
                        lay::<S>,
                        show_hologram,
                        show_ports,
                        show_cost::<S>,
                    )
                        .chain()
                        .in_set(StreamSystems::Plan),
                )
                    .chain()
                    .before(stubby_core::CursorCapture),
            )
            .add_message::<Dismantle>()
            .add_message::<LayerNote>()
            .add_systems(
                Update,
                (
                    dismantle::dismantle_input,
                    dismantle::apply_dismantles::<S>,
                    resolve_sources,
                    emit_bars.run_if(resource_exists::<Messages<ClockTick>>),
                    ride_bars,
                    emit_notes.run_if(resource_exists::<Messages<ClockTick>>),
                    ride_notes,
                    place_voices,
                )
                    .chain()
                    .after(StreamSystems::Plan),
            );
    }
}

/// Where a stream can start: a ground-level origin and a horizontal flow direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Port {
    pub at: Vec3,
    pub dir: Vec3,
    pub owner: PortOwner,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortOwner {
    Burbler(Entity),
    Stream(Entity),
}

#[derive(Resource, Default)]
pub struct StreamBuild {
    pub active: bool,
    /// A left click this frame, while the cursor was captured.
    pub click: bool,
    pub anchor: Option<Port>,
    pub hover: Option<Port>,
    pub plan: Option<Plan>,
}
#[derive(Clone)]
pub struct Plan {
    pub path: Path,
    pub segments: u32,
    pub affordable: bool,
    pub valid: bool,
}

/// A laid stream: segments `first..=last` of its plan's path. Its far end is a port
/// until another stream is laid from it. Dismantling a segment splits it into the parts
/// either side of the gap (`dismantle`), all on the same path.
#[derive(Component)]
pub struct LaidStream {
    pub end: Port,
    pub end_used: bool,
    pub end_halo: Vec<Entity>,
    pub path: Path,
    /// The length of one bent segment: the plan's length over its segment count.
    pub span: f32,
    /// The first and last of the plan's segments this stream still has.
    pub first: u32,
    pub last: u32,
    /// The port it was laid from: a burbler's mouth or another stream's end. `None` for
    /// a part cut off downstream of a dismantled segment.
    pub upstream: Option<PortOwner>,
    /// The burbler whose water this is, walking `upstream` (kept by `resolve_sources`).
    pub source: Option<Entity>,
    /// How far along the whole chain from the burbler this stream starts.
    pub start: f32,
    /// How clean its water is: 0 is a burbler's raw mud, 1 is clear. Filters raise it
    /// later (add-fluid-quality-and-viscosity).
    pub clarity: f32,
}
impl LaidStream {
    /// A straight muddy stream from `from` to `to`, with no meshes: for tests and
    /// scripted reviews of what streams do, not for laying.
    pub fn straight(source: Option<Entity>, from: Vec3, to: Vec3) -> Self {
        Self::straight_segments(source, from, to, 1)
    }

    /// The same, in `segments` equal segments.
    pub fn straight_segments(source: Option<Entity>, from: Vec3, to: Vec3, segments: u32) -> Self {
        let port = Port {
            at: from,
            dir: (to - from).normalize_or(Vec3::X),
            owner: PortOwner::Burbler(Entity::PLACEHOLDER),
        };
        let path = Path::new(&port, to);
        let (end_at, end_dir) = path.at(path.length());
        Self {
            end: Port {
                at: end_at,
                dir: end_dir,
                owner: PortOwner::Burbler(Entity::PLACEHOLDER),
            },
            end_used: false,
            end_halo: vec![],
            span: path.length() / segments.max(1) as f32,
            path,
            first: 0,
            last: segments.max(1) - 1,
            upstream: source.map(PortOwner::Burbler),
            source,
            start: 0.,
            clarity: 0.,
        }
    }

    /// Where this stream starts and ends along its path (arc length).
    pub fn start_s(&self) -> f32 {
        self.first as f32 * self.span
    }
    pub fn end_s(&self) -> f32 {
        (self.last + 1) as f32 * self.span
    }
    pub fn length(&self) -> f32 {
        self.end_s() - self.start_s()
    }

    /// Points along this stream's own part of the path, about `step` metres apart.
    fn samples(&self, step: f32) -> impl Iterator<Item = Vec3> + '_ {
        let (from, length) = (self.start_s(), self.length());
        let steps = (length / step).ceil().max(1.) as u32;
        (0..=steps).map(move |k| self.path.at(from + length * k as f32 / steps as f32).0)
    }

    /// The horizontal distance from `at` to the nearest point of this stream.
    pub fn horizontal_distance(&self, at: Vec3) -> f32 {
        let nearest = self.nearest_point(at);
        Vec2::new(nearest.x - at.x, nearest.z - at.z).length()
    }

    /// The point of this stream horizontally nearest `at`, and how far down the whole
    /// chain from the burbler's mouth it is (`add-stream-layers`: a machine's join).
    pub fn nearest_along(&self, at: Vec3) -> (f32, Vec3) {
        let (from, length) = (self.start_s(), self.length());
        let steps = (length / 0.25).ceil().max(1.) as u32;
        (0..=steps)
            .map(|k| {
                let s = length * k as f32 / steps as f32;
                (self.start + s, self.path.at(from + s).0)
            })
            .min_by(|a, b| {
                let d = |p: &Vec3| Vec2::new(p.x - at.x, p.z - at.z).length_squared();
                d(&a.1).total_cmp(&d(&b.1))
            })
            .unwrap_or((self.start, at))
    }

    /// The point of this stream (on its centre line) horizontally nearest `at`.
    pub fn nearest_point(&self, at: Vec3) -> Vec3 {
        self.samples(0.25)
            .min_by(|a, b| {
                let d = |p: &Vec3| Vec2::new(p.x - at.x, p.z - at.z).length_squared();
                d(a).total_cmp(&d(b))
            })
            .unwrap_or(at)
    }
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
    /// A note's parts (head, stem, halo), centred on the staff (`Pulse_0`, likewise).
    note: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
    pool: Vec<Vec<(Entity, Handle<Mesh>)>>,
}

fn load_templates(mut commands: Commands, server: Res<AssetServer>) {
    for (kind, path) in [
        (
            Kind::Segment,
            "streams/stream_segment/stream_segment.glb#Scene0",
        ),
        (
            Kind::Hologram,
            "streams/stream_hologram/stream_hologram.glb#Scene0",
        ),
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
    let mut note = Vec::new();
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
            // A bar line or note keeps its height on the staff; `bar_pose` gives the rest.
            if name.is_some_and(|name| matches!(name.as_str(), "Bar_0" | "Pulse_0")) {
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
        if under("Port_in_Halo") {
            continue;
        }
        let mut baked = plain_copy(source);
        transform_mesh(&mut baked, affine);
        if under("Pulse_0") {
            note.push((meshes.add(baked), material.0.clone()));
            continue;
        }
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
            templates.note = note;
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
pub struct Path {
    pub points: [Vec3; 4],
    /// (arc length, curve parameter), increasing.
    pub table: Vec<(f32, f32)>,
}
impl Path {
    /// Leave `from` along its direction and curve to `to` (design D3).
    fn new(from: &Port, to: Vec3) -> Self {
        Self::arriving(from, to, None)
    }
    /// The same, arriving at `to` along `end` when given (a stream snapped to an inlet).
    fn arriving(from: &Port, to: Vec3, end: Option<Vec3>) -> Self {
        let p0 = from.at;
        let p3 = Vec3::new(to.x, p0.y, to.z);
        let reach = p0.distance(p3);
        let p1 = p0 + from.dir * reach / 3.;
        let d1 = end.unwrap_or_else(|| (p3 - p1).normalize_or(from.dir));
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
    pub fn length(&self) -> f32 {
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

/// Snap a stream laid from `from` and aimed at `aim` to the nearest inlet whose ring the
/// aim is inside or within `snap_m` of: the point on the ring nearest the aim, and the ring's tangent
/// there that leads away from `from`, so the stream arrives alongside the inlet's owner.
pub fn snap_to_inlet(
    from: Vec3,
    aim: Vec3,
    inlets: impl IntoIterator<Item = (StreamInlet, Vec3)>,
) -> Option<(Vec3, Vec3)> {
    inlets
        .into_iter()
        .filter_map(|(inlet, centre)| {
            let centre = Vec3::new(centre.x, aim.y, centre.z);
            let out = Vec3::new(aim.x - centre.x, 0., aim.z - centre.z);
            // Anywhere inside the ring snaps too: aiming at the trunk means the tree.
            let off_ring = (out.length() - inlet.radius_m).max(0.);
            (off_ring <= inlet.snap_m).then(|| {
                let radial = out.normalize_or((from - centre).with_y(0.).normalize_or(Vec3::X));
                let at = centre + radial * inlet.radius_m;
                let tangent = radial.cross(Vec3::Y);
                let along = if tangent.dot(at - from) >= 0. {
                    tangent
                } else {
                    -tangent
                };
                (off_ring, at, along)
            })
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, at, along)| (at, along))
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
    } else if keys.just_pressed(stubby_core::keys::BUILD) && build.active {
        // Q leaves stream building; opening it is the build menu's (the game's) job.
        build.active = false;
    } else if keys.just_pressed(KeyCode::Escape) {
        build.active = false;
    } else if build.active && buttons.just_pressed(MouseButton::Right) {
        build.anchor = None;
    }
    if !build.active {
        build.anchor = None;
    }
}

/// Whether the stock holds `segments` segments' worth of the cost.
fn affordable(cost: &SegmentCost, stock: &impl Stock, segments: u32) -> bool {
    !cost.0.is_empty()
        && cost
            .0
            .iter()
            .all(|(item, n)| stock.quantity(item) >= n * segments)
}

fn plan<S: Stock>(
    mut build: ResMut<StreamBuild>,
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    burblers: Query<(Entity, &PlacedAsset, &mut AssetState)>,
    outlets: Query<(&AssetPart, &GlobalTransform)>,
    streams: Query<&mut LaidStream>,
    inlets: Query<(&StreamInlet, &GlobalTransform)>,
    cost: Res<SegmentCost>,
    inventory: Option<Res<S>>,
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
    let target = aim(camera, anchor.at.y, anchor.at);
    let snapped = snap_to_inlet(
        anchor.at,
        target,
        inlets.iter().map(|(inlet, at)| (*inlet, at.translation())),
    );
    let path = match snapped {
        Some((at, along)) => Path::arriving(&anchor, at, Some(along)),
        None => Path::new(&anchor, target),
    };
    let length = path.length();
    let segments = ((length / SEGMENT_LENGTH).round() as u32).max(1);
    let affordable = inventory
        .as_ref()
        .is_some_and(|stock| affordable(&cost, stock.as_ref(), segments));
    let shape_ok = length >= SEGMENT_LENGTH * 0.75 && path.min_radius() >= MIN_RADIUS;
    build.plan = Some(Plan {
        path,
        segments,
        affordable,
        valid: affordable && shape_ok,
    });
}

#[allow(clippy::too_many_arguments)]
fn lay<S: Stock>(
    mut commands: Commands,
    mut build: ResMut<StreamBuild>,
    templates: Res<Templates>,
    mut meshes: ResMut<Assets<Mesh>>,
    cost: Res<SegmentCost>,
    mut inventory: Option<ResMut<S>>,
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
    let Some(inventory) = inventory.as_mut() else {
        return;
    };
    if templates.segment.is_empty() || !affordable(&cost, inventory.as_ref(), plan.segments) {
        return;
    }
    for (item, count) in &cost.0 {
        inventory.take(item, count * plan.segments);
    }
    let span = plan.path.length() / plan.segments as f32;
    let start = match anchor.owner {
        PortOwner::Burbler(_) => 0.,
        PortOwner::Stream(entity) => streams
            .get(entity)
            .map_or(0., |upstream| upstream.start + upstream.length()),
    };
    let (_, end) = spawn_stream(
        &mut commands,
        &templates,
        &mut meshes,
        StreamPart {
            path: plan.path.clone(),
            span,
            first: 0,
            last: plan.segments - 1,
            upstream: Some(anchor.owner),
            start,
            end_used: false,
        },
    );
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

/// A stream to spawn: segments `first..=last` of `path`.
struct StreamPart {
    path: Path,
    span: f32,
    first: u32,
    last: u32,
    upstream: Option<PortOwner>,
    start: f32,
    end_used: bool,
}

/// Spawn a laid stream with its bent segment meshes (the end halo only on its last
/// segment). Its source is filled in by `resolve_sources`. Returns it and its end port.
fn spawn_stream(
    commands: &mut Commands,
    templates: &Templates,
    meshes: &mut Assets<Mesh>,
    part: StreamPart,
) -> (Entity, Port) {
    let StreamPart {
        path,
        span,
        first,
        last,
        upstream,
        start,
        end_used,
    } = part;
    let stream = commands
        .spawn((
            Name::new("Mud stream"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    let mut end_halo = Vec::new();
    for k in first..=last {
        for piece in &templates.segment {
            let halo = piece.role == Role::HaloOut;
            if halo && k != last {
                continue;
            }
            let mut mesh = piece.mesh.clone();
            bend(&piece.mesh, &mut mesh, &path, k as f32 * span, span, 0.);
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
    let (end_at, end_dir) = path.at((last + 1) as f32 * span);
    let end = Port {
        at: end_at,
        dir: end_dir,
        owner: PortOwner::Stream(stream),
    };
    commands.entity(stream).insert(LaidStream {
        end,
        end_used,
        end_halo,
        path,
        span,
        first,
        last,
        upstream,
        source: None,
        start,
        clarity: 0.,
    });
    (stream, end)
}

/// Each stream's source is the burbler its `upstream` chain reaches, if any: a part cut
/// off by a dismantle, and everything laid on from it, lies quiet.
fn resolve_sources(mut streams: Query<(Entity, &mut LaidStream)>) {
    let upstream: HashMap<Entity, Option<PortOwner>> = streams
        .iter()
        .map(|(entity, stream)| (entity, stream.upstream))
        .collect();
    for (_, mut stream) in &mut streams {
        let mut at = stream.upstream;
        let mut source = None;
        // Bounded: a chain longer than every stream there is would be a loop.
        for _ in 0..=upstream.len() {
            match at {
                Some(PortOwner::Burbler(burbler)) => {
                    source = Some(burbler);
                    break;
                }
                Some(PortOwner::Stream(entity)) => at = upstream.get(&entity).copied().flatten(),
                None => break,
            }
        }
        if stream.source != source {
            stream.source = source;
        }
    }
}

// ---------- What a laid stream carries: bar lines and the voice ----------

/// The staff and bar settings a stream reads from its profile.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
struct StaffProfile {
    beats_per_bar: u32,
    flow_speed: f32,
    /// Between the staff's lines; a note's step moves it half of this across the staff.
    spacing: f32,
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
            spacing: number(&profile["staff"]["spacing_m"]),
            wave_amplitude: number(&profile["staff"]["wave"]["amplitude_m"]),
            wavelength: number(&profile["staff"]["wave"]["wavelength_m"]),
        }
    }
}

/// A bar line riding the staff of one burbler's stream.
#[derive(Component)]
pub struct BarLine {
    pub machine: Entity,
    /// When its downbeat is heard, from the audio clock.
    pub at: Instant,
    /// Half a beat in flow time: a bar line sits between the measures.
    pub lead_seconds: f32,
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
    machines: Query<(Entity, &AssetClock, &MachineVoice)>,
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
            s.source == Some(bar.machine) && (s.start..s.start + s.length()).contains(&distance)
        });
        if let Some(stream) = on {
            let along = stream.start_s() + distance - stream.start;
            *transform = bar_pose(&stream.path, stream.span, along, &staff);
            *visibility = Visibility::Visible;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

/// A note for the staff from a machine joined to a chain (`add-stream-layers`): it
/// appears at the join and rides downstream, never upstream. The heartbeat's notes come
/// from the burbler's own clock; a joined machine's (the campfire's sequencer) from glue.
#[derive(Message, Clone, Debug)]
pub struct LayerNote {
    /// The chain's burbler.
    pub source: Entity,
    /// How far down the chain the join is, in metres from the burbler's mouth.
    pub join_m: f32,
    /// When the note is heard, from the audio clock.
    pub at: Instant,
    /// Where across the staff it sits: 0 is the middle line, each step half a space.
    pub step: i8,
    /// Its colour, or the staff's own.
    pub colour: Option<Color>,
}

/// On a burbler whose chain a joined machine updates (add-stream-layers D8: the stream
/// plays its latest version, user 2026-11-18): from `at_m` down the chain the joined
/// machine's version plays. The burbler's own voice is silent, and notes of older layers
/// leave the staff there (`DivertedNote`), for the joined machine to take in.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ChainUpdated {
    pub at_m: f32,
}

/// A note that reached an update and left the staff there; whoever updated the chain
/// moves it now (the campfire draws it in and up).
#[derive(Component, Clone, Copy, Debug)]
pub struct DivertedNote {
    /// When it reached the update.
    pub at: Instant,
}

/// A note riding a chain's staff.
#[derive(Component)]
pub struct StaffNote {
    pub source: Entity,
    pub join_m: f32,
    pub at: Instant,
    pub step: i8,
}

/// How far down the chain a note is, `elapsed` seconds after it was heard at its join.
fn note_distance(join_m: f32, elapsed: f32, flow_speed: f32) -> f32 {
    join_m + elapsed * flow_speed
}

/// A note on every beat of a sounding burbler with a stream (the heartbeat, joined at the
/// mouth), and one for each layer note a joined machine sends.
#[allow(clippy::too_many_arguments)]
fn emit_notes(
    mut commands: Commands,
    mut ticks: MessageReader<ClockTick>,
    mut layers: MessageReader<LayerNote>,
    templates: Res<Templates>,
    machines: Query<(Entity, &AssetClock, &MachineVoice)>,
    streams: Query<&LaidStream>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tints: Local<Vec<([u8; 4], Vec<Handle<StandardMaterial>>)>>,
) {
    let mut notes: Vec<LayerNote> = Vec::new();
    for tick in ticks.read() {
        for (machine, clock, voice) in &machines {
            if clock.engine == tick.module
                && voice.sounding
                && streams.iter().any(|s| s.source == Some(machine))
            {
                notes.push(LayerNote {
                    source: machine,
                    join_m: 0.,
                    at: tick.at.unwrap_or_else(Instant::now),
                    step: 0,
                    colour: None,
                });
            }
        }
    }
    notes.extend(layers.read().cloned());
    for note in notes {
        // A coloured layer's parts share one tinted copy of each template material.
        let parts: Vec<_> = match note.colour {
            None => templates.note.clone(),
            Some(colour) => {
                let key = colour.to_srgba().to_u8_array();
                let tinted = match tints.iter().find(|(k, _)| *k == key) {
                    Some((_, tinted)) => tinted.clone(),
                    None => {
                        let tinted: Vec<_> = templates
                            .note
                            .iter()
                            .map(|(_, material)| {
                                let mut m = materials.get(material).cloned().unwrap_or_default();
                                let alpha = m.base_color.alpha();
                                m.base_color = colour.with_alpha(alpha);
                                let glow =
                                    m.emissive.red.max(m.emissive.green).max(m.emissive.blue);
                                m.emissive = colour.to_linear() * glow.max(1.);
                                materials.add(m)
                            })
                            .collect();
                        tints.push((key, tinted.clone()));
                        tinted
                    }
                };
                templates
                    .note
                    .iter()
                    .zip(tinted)
                    .map(|((mesh, _), material)| (mesh.clone(), material))
                    .collect()
            }
        };
        commands
            .spawn((
                Name::new("Staff note"),
                StaffNote {
                    source: note.source,
                    join_m: note.join_m,
                    at: note.at,
                    step: note.step,
                },
                Transform::default(),
                Visibility::Hidden,
            ))
            .with_children(|n| {
                for (mesh, material) in parts {
                    n.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
                }
            });
    }
}

/// Move notes down their chain at the water's speed, from the audio clock; one past the
/// end of its chain is gone.
fn ride_notes(
    mut commands: Commands,
    staff: Res<StaffProfile>,
    mut notes: Query<(Entity, &StaffNote, &mut Transform, &mut Visibility), Without<DivertedNote>>,
    streams: Query<&LaidStream>,
    updated: Query<&ChainUpdated>,
) {
    let now = Instant::now();
    for (entity, note, mut transform, mut visibility) in &mut notes {
        if now < note.at {
            continue;
        }
        let elapsed = now.duration_since(note.at).as_secs_f32();
        let distance = note_distance(note.join_m, elapsed, staff.flow_speed);
        // An older layer's note reaching an update leaves the staff there.
        if let Ok(update) = updated.get(note.source)
            && note.join_m < update.at_m
            && distance >= update.at_m
        {
            commands.entity(entity).insert(DivertedNote { at: now });
            continue;
        }
        let on = streams.iter().find(|s| {
            s.source == Some(note.source) && (s.start..s.start + s.length()).contains(&distance)
        });
        if let Some(stream) = on {
            let along = stream.start_s() + distance - stream.start;
            let mut pose = bar_pose(&stream.path, stream.span, along, &staff);
            let side = pose.rotation * Vec3::Z;
            pose.translation += side * f32::from(note.step) * staff.spacing / 2.;
            *transform = pose;
            *visibility = Visibility::Visible;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

/// Where a voice is heard from: the machine, or the nearest point of its streams.
/// A voice sits this high above the ground, whether heard from a stream (clear of its
/// 0.36 m banks) or from the machine itself (a campfire, a burbler's pool), so the ground,
/// an acoustic occluder with its mud plates proud of it, never cuts it off. Measured
/// (2026-11-18): 6 m off a stream, on the bed it averaged -77 dB; at 0.6 m, -59 dB; at
/// 1.2 m no louder.
const VOICE_ABOVE_BED_M: f32 = 0.6;

/// Counting only the chain from `from_m` down: a joined machine's layer is heard
/// downstream of its join, never upstream (a burbler's own voice from 0).
fn nearest_source_from(
    listener: Vec3,
    machine: Vec3,
    streams: &[&LaidStream],
    from_m: f32,
) -> Vec3 {
    let mut best = machine + Vec3::Y * VOICE_ABOVE_BED_M;
    for stream in streams {
        let (from, length) = (stream.start_s(), stream.length());
        let steps = (length / 0.5).ceil().max(1.) as u32;
        for k in 0..=steps {
            let s = length * k as f32 / steps as f32;
            if stream.start + s < from_m {
                continue;
            }
            // Heard from just above the banks, not from the bed: from the bed, the ground
            // (an acoustic occluder) cut a stream's voice off for anyone beside it
            // (user, 2026-11-18).
            let point = stream.path.at(from + s).0 + Vec3::Y * VOICE_ABOVE_BED_M;
            if point.distance_squared(listener) < best.distance_squared(listener) {
                best = point;
            }
        }
    }
    best
}

/// A joined machine's voice is heard from its chain downstream of its join, as well as
/// from the machine (add-stream-layers: the stream sounds the layers it carries).
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct HeardDownstream {
    /// The chain's burbler.
    pub source: Entity,
    /// The join's distance down the chain.
    pub from_m: f32,
}

/// Silent beyond the radius, full at the source: the same curve as a lightning bug's click.
fn hearing_gain(distance: f32, radius: f32) -> f32 {
    (1. - distance / radius.max(0.001)).clamp(0., 1.).powi(2)
}

/// Tell the engine where each rack's voice is, relative to the player's ears.
fn place_voices(
    synth: Option<Res<Synth>>,
    camera: Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    machines: Query<(
        Entity,
        &MachineVoice,
        &GlobalTransform,
        Option<&HeardDownstream>,
        Has<ChainUpdated>,
    )>,
    streams: Query<&LaidStream>,
) {
    let (Some(synth), Ok(camera)) = (synth, camera.single()) else {
        return;
    };
    let (_, ears, listener) = camera.to_scale_rotation_translation();
    synth.send(Command::SetListener {
        pose: ListenerPose {
            position: listener.to_array(),
            right: camera.right().to_array(),
            up: camera.up().to_array(),
            ahead: camera.forward().to_array(),
        },
    });
    for (machine, voice, transform, downstream, superseded) in &machines {
        let Some(output) = voice.output else {
            continue;
        };
        let (chain, from_m) = downstream.map_or((machine, 0.), |d| (d.source, d.from_m));
        let fed: Vec<&LaidStream> = streams.iter().filter(|s| s.source == Some(chain)).collect();
        let source = nearest_source_from(listener, transform.translation(), &fed, from_m);
        synth.send(Command::SetEmitter {
            output,
            offset: (ears.inverse() * (source - listener)).to_array(),
            position_world: source.to_array(),
            // A chain plays its latest version: an updated burbler's own voice is silent.
            volume: if superseded {
                0.
            } else {
                hearing_gain(source.distance(listener), voice.hearing_radius_metres)
            },
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

fn spawn_cost(mut commands: Commands, server: Option<Res<AssetServer>>, cost: Res<SegmentCost>) {
    let cost_item = cost
        .0
        .first()
        .map(|(item, _)| item.clone())
        .unwrap_or_default();
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
                    stubby_ui::item_icon(server, &cost_item),
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

fn show_cost<S: Stock>(
    build: Res<StreamBuild>,
    cost: Res<SegmentCost>,
    inventory: Option<Res<S>>,
    mut boxes: Query<&mut Visibility, With<CostBox>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<CostText>>,
) {
    let segments = build.plan.as_ref().map_or(1, |plan| plan.segments);
    // The first item of the cost is the one shown (reeds, in the game).
    let (item, per_segment) = cost.0.first().map_or(("", 0), |(i, n)| (i.as_str(), *n));
    let need = per_segment * segments;
    let have = inventory.as_ref().map_or(0, |i| i.quantity(item));
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
            spacing: 0.12,
            wave_amplitude: 0.07,
            wavelength: 2.0,
        }
    }

    #[test]
    fn a_joined_layer_rides_only_downstream_of_its_join() {
        // A note heard at a join 12 m down is never upstream of it, and moves at the water.
        let flow = staff().flow_speed;
        assert_eq!(note_distance(12., 0., flow), 12.);
        assert!((note_distance(12., 2., flow) - 15.).abs() < 1e-5);
        assert_eq!(
            note_distance(0., 0., flow),
            0.,
            "the heartbeat starts at the mouth"
        );
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
        let stream = LaidStream::straight(None, port().at, port().at + Vec3::X * 20.);
        let path = &stream;
        let machine = port().at - Vec3::X * 2.;
        // Standing by the far end of the stream: heard from there, not the machine.
        let far = port().at + Vec3::new(20., 0., 3.);
        let heard = |listener, streams: &[&LaidStream], from_m| {
            nearest_source_from(listener, machine, streams, from_m)
        };
        let end = port().at + Vec3::X * 20. + Vec3::Y * VOICE_ABOVE_BED_M;
        assert!(heard(far, &[path], 0.).distance(end) < 0.6);
        // Above the banks, so the ground beside the stream does not cut it off.
        assert!(heard(far, &[path], 0.).y >= port().at.y + 0.36);
        // Standing behind the machine: heard from the machine.
        let behind = machine - Vec3::X * 5.;
        let above = machine + Vec3::Y * VOICE_ABOVE_BED_M;
        assert_eq!(heard(behind, &[path], 0.), above);
        assert_eq!(heard(far, &[], 0.), above);
        // A layer joined 10 m down is heard from there on, not from upstream of its join.
        let upstream = port().at + Vec3::new(2., 0., 1.);
        let at = heard(upstream, &[path], 10.);
        assert!(at == above || at.x >= port().at.x + 9.9, "{at:?}");
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
    fn a_stream_aimed_at_a_tree_snaps_alongside_it_not_through_it() {
        let inlet = StreamInlet {
            radius_m: 2.2,
            snap_m: 2.,
        };
        let tree = Vec3::new(12., 0., 0.);
        // Aimed straight at the trunk: it ends on the ring, along its tangent.
        let (at, along) = snap_to_inlet(Vec3::ZERO, tree, [(inlet, tree)]).expect("snaps");
        assert!((at.distance(tree) - 2.2).abs() < 1e-3);
        assert!(along.dot(at - tree).abs() < 1e-3, "tangent to the ring");
        let path = Path::arriving(&port(), at, Some(along));
        let closest = (0..=64)
            .map(|i| path.at(path.length() * i as f32 / 64.).0.distance(tree))
            .fold(f32::MAX, f32::min);
        assert!(
            closest > 1.5,
            "the stream stays clear of the trunk ({closest} m)"
        );
        // Aimed well away from it: no snap.
        assert!(snap_to_inlet(Vec3::ZERO, Vec3::new(12., 0., 8.), [(inlet, tree)]).is_none());
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

/// Tell the other systems (the player's outlines) whether build mode has the mouse.
fn publish_build_mode(build: Res<StreamBuild>, mut mode: ResMut<stubby_core::BuildMode>) {
    mode.set_if_neq(stubby_core::BuildMode(build.active));
}
