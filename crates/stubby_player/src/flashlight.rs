//! The disco light (add-forensic-flashlight), on B. A disco ball floats beside the
//! player: it spins and throws a ring of narrow violet and cyan beams that sweep the
//! ground. It is not a hand item (user, 2026-11-17): B switches it on and off whatever the
//! hand holds, so the field recorder stays up, and no flashlight model is shown. It throws
//! many small beams whose spots run out to 30 m (user, 2026-11-18: "more, smaller"; a dozen
//! wide ones swept too seldom and pickups blinked). A collectible a beam's heading passes
//! (anything marked `Interactable::Pickup`, no asset kind named) flares into a hologram, an
//! overlay on its body and a beacon above it, and is `Spotlit`, so its owner keeps it in
//! view (a bug between flashes); beams cross each point several times a second, and a
//! hologram fades only when none has for a while.
//! Style: `assets/ui/flashlight.json`.
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, CompareFunction, RenderPipelineDescriptor, ShaderType,
        SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use bevy_stubby_audio::ClockTick;
use stubby_core::{AssetClock, HudCamera, Interactable, ScreenMode};

use crate::controls::Player;
/// Whether the disco light is on. B toggles it (not while a screen is open).
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoLight(pub bool);

// ---------------------------------------------------------------- style

#[derive(serde::Deserialize, Clone, Debug)]
struct StyleFile {
    ball: BallStyle,
    pulse: PulseStyle,
    beams: BeamStyle,
    reveal: RevealStyle,
    hologram: HologramStyle,
    beacon: BeaconStyle,
}
#[derive(serde::Deserialize, Clone, Debug)]
struct BallStyle {
    offset: [f32; 3],
    turns_per_second: f32,
}
/// The ball swells on each heard beat of the nearest clock within `reach_m`.
#[derive(serde::Deserialize, Clone, Debug)]
struct PulseStyle {
    reach_m: f32,
    depth: f32,
    decay_seconds: f32,
}
#[derive(serde::Deserialize, Clone, Debug)]
struct BeamStyle {
    count: usize,
    colors: Vec<[f32; 3]>,
    intensity_lumens: f32,
    reference_tilt_deg: f32,
    width_deg: f32,
    range_m: f32,
    tilt_deg: Vec<f32>,
    /// A second ring thrown upward, tilted above the horizon by `up_tilt_deg` in turn
    /// (user, 2026-11-18: for later use, lighting what is above: canopies, boulders, sky).
    #[serde(default)]
    up_count: usize,
    #[serde(default)]
    up_tilt_deg: Vec<f32>,
    /// The beams' glow in the haze (user, 2026-11-20: the scanner scatters in the fog).
    #[serde(default)]
    scatter: Option<ScatterStyle>,
}
/// Each beam glows along its length as light scattered by the haze: strength = `gain` times
/// the fog's density over `reference_density`, at most `max`. The fog's density follows
/// the air quality, so bleak air shows the shafts and clean air barely does.
#[derive(serde::Deserialize, Clone, Copy, Debug)]
struct ScatterStyle {
    gain: f32,
    reference_density: f32,
    max: f32,
    edge_power: f32,
}
impl ScatterStyle {
    fn strength(&self, density: f32) -> f32 {
        (self.gain * density / self.reference_density.max(1e-6)).clamp(0., self.max)
    }
}
#[derive(serde::Deserialize, Clone, Debug)]
struct RevealStyle {
    width_deg: f32,
    range_m: f32,
    fade_seconds: f32,
}
#[derive(serde::Deserialize, Clone, Debug)]
struct HologramStyle {
    rim: [f32; 3],
    core: [f32; 3],
    glow: f32,
    scanlines_per_m: f32,
    scroll_m_per_s: f32,
    flicker: f32,
}
#[derive(serde::Deserialize, Clone, Debug)]
struct BeaconStyle {
    height_m: f32,
    radius_m: f32,
    diamond_m: f32,
}

/// The flashlight's style (`assets/ui/flashlight.json`).
#[derive(Resource, Clone, Debug)]
pub struct FlashlightStyle(StyleFile);
impl FlashlightStyle {
    pub fn shipped() -> Self {
        Self(
            serde_json::from_str(include_str!("../../../assets/ui/flashlight.json"))
                .expect("flashlight style"),
        )
    }
}

// ---------------------------------------------------------------- the beams' geometry

/// The beams' directions in the ball's own frame: spread evenly round it, each tilted
/// below the horizon by the next of `tilts_deg` in turn, so spots land near and far.
pub fn beam_directions(count: usize, tilts_deg: &[f32]) -> Vec<Vec3> {
    (0..count)
        .map(|i| {
            let yaw = i as f32 / count as f32 * std::f32::consts::TAU;
            let tilt = tilts_deg
                .get(i % tilts_deg.len().max(1))
                .copied()
                .unwrap_or(20.)
                .to_radians();
            Vec3::new(yaw.sin() * tilt.cos(), -tilt.sin(), -yaw.cos() * tilt.cos())
        })
        .collect()
}

/// The upward ring's directions: spread round the ball between the downward beams' headings
/// (offset by half a step), each tilted above the horizon by the next of `tilts_deg`.
pub fn upward_beam_directions(count: usize, tilts_deg: &[f32]) -> Vec<Vec3> {
    let half_step = Quat::from_rotation_y(std::f32::consts::PI / count.max(1) as f32);
    beam_directions(count, tilts_deg)
        .into_iter()
        .map(|d| half_step * Vec3::new(d.x, -d.y, d.z))
        .collect()
}

/// Whether any beam from `origin` along `beams` (world directions) passes `point` within
/// `width_deg` round the ball (its heading, at any height: a beam's spots run out to its
/// range, and a bug in the air above them is caught too), within `range_m`.
pub fn touched(origin: Vec3, beams: &[Vec3], point: Vec3, width_deg: f32, range_m: f32) -> bool {
    let to = point - origin;
    let flat = Vec2::new(to.x, to.z);
    to.length() <= range_m
        && flat.length() > 1e-3
        && beams.iter().any(|beam| {
            let heading = Vec2::new(beam.x, beam.z);
            heading.length() > 1e-3 && heading.angle_to(flat).abs().to_degrees() <= width_deg
        })
}

/// A beam's brightness: `intensity` at `reference_tilt_deg` below the horizon, scaled with
/// the square of how much farther its spot lands, so far spots read as bright as near ones.
pub fn beam_intensity(intensity: f32, tilt_deg: f32, reference_tilt_deg: f32) -> f32 {
    let reach = |tilt: f32| 1. / tilt.to_radians().tan().max(1e-3);
    intensity * (reach(tilt_deg) / reach(reference_tilt_deg)).powi(2)
}

/// A hologram's strength after `dt`: full while a beam touches it, else fading out over
/// `fade_seconds`.
pub fn flare(strength: f32, touched: bool, dt: f32, fade_seconds: f32) -> f32 {
    if touched {
        1.
    } else {
        (strength - dt / fade_seconds.max(1e-3)).max(0.)
    }
}

// ---------------------------------------------------------------- the hologram material

#[derive(ShaderType, Clone, Copy, Debug)]
pub struct HologramUniform {
    pub rim: Vec4,
    pub core: Vec4,
    /// x: strength 0..1, y: scanlines per metre, z: scroll metres per second, w: flicker.
    pub params: Vec4,
}

/// Additive, unlit, no depth test: shows through grass and reeds
/// (`assets/shaders/hologram.wesl`).
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct HologramMaterial {
    #[uniform(0)]
    pub hologram: HologramUniform,
}
impl Material for HologramMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/hologram.wesl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(
        _: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_compare = Some(CompareFunction::Always);
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

fn hologram(style: &HologramStyle, strength: f32) -> HologramMaterial {
    let glow = |[r, g, b]: [f32; 3]| {
        let c = LinearRgba::from(Color::srgb(r, g, b)) * style.glow;
        Vec4::new(c.red, c.green, c.blue, 1.)
    };
    HologramMaterial {
        hologram: HologramUniform {
            rim: glow(style.rim),
            core: glow(style.core),
            params: Vec4::new(
                strength,
                style.scanlines_per_m,
                style.scroll_m_per_s,
                style.flicker,
            ),
        },
    }
}

// ---------------------------------------------------------------- the beams' glow

#[derive(ShaderType, Clone, Copy, Debug)]
pub struct BeamUniform {
    pub colour: Vec4,
    /// x: strength, y: edge sharpness.
    pub params: Vec4,
}

/// Additive, depth-tested, two-sided, no depth write: a beam's glow in the haze
/// (`assets/shaders/scanner_beam.wesl`).
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BeamMaterial {
    #[uniform(0)]
    pub beam: BeamUniform,
}
impl Material for BeamMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/scanner_beam.wesl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(
        _: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

/// The glow materials, one per beam colour, whose strength follows the fog.
#[derive(Resource)]
struct BeamGlow(Vec<Handle<BeamMaterial>>);

/// An open cone of unit length along -Z (a spotlight's forward) and unit radius at its far
/// end, its apex at the origin; uv.y runs 0 at the apex to 1 at the end.
pub fn unit_cone(segments: u32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    for k in 0..=segments {
        let a = std::f32::consts::TAU * k as f32 / segments as f32;
        let (s, c) = a.sin_cos();
        // The side's outward normal: radial, tilted back toward the apex a little.
        let normal = Vec3::new(c, s, 1.).normalize();
        positions.push([0., 0., 0.]);
        normals.push(normal.to_array());
        uvs.push([k as f32 / segments as f32, 0.]);
        positions.push([c, s, -1.]);
        normals.push(normal.to_array());
        uvs.push([k as f32 / segments as f32, 1.]);
    }
    for k in 0..segments {
        let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
        indices.extend([a, b, d, a, d, c]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// The beams' glow follows the fog the camera sees: its density is the air quality's.
fn scatter_in_fog(
    style: Res<FlashlightStyle>,
    glow: Option<Res<BeamGlow>>,
    mut materials: Option<ResMut<Assets<BeamMaterial>>>,
    fogs: Query<&DistanceFog, (With<Camera3d>, Without<HudCamera>)>,
    mut shown: Local<f32>,
) {
    let (Some(glow), Some(materials), Some(scatter)) =
        (glow, materials.as_mut(), style.0.beams.scatter)
    else {
        return;
    };
    let density =
        fogs.iter()
            .find_map(|fog| match fog.falloff {
                FogFalloff::Exponential { density }
                | FogFalloff::ExponentialSquared { density } => Some(density),
                _ => None,
            })
            .unwrap_or(0.);
    let strength = scatter.strength(density);
    if (strength - *shown).abs() < 1e-3 {
        return;
    }
    *shown = strength;
    for handle in &glow.0 {
        if let Some(mut material) = materials.get_mut(handle) {
            material.beam.params.x = strength;
        }
    }
}

// ---------------------------------------------------------------- the ball

/// The disco ball: placed above the player's head to the left when switched on, it stays
/// at that place in the world while on (user, 2026-11-16), spinning about the vertical as a
/// hanging ball does.
#[derive(Component)]
struct DiscoBall {
    angle: f32,
    beams: Vec<Vec3>,
    /// Whether it has been placed since it was switched on.
    placed: bool,
    /// The last beat heard from the clock it follows, and the next one, which the
    /// engine reports a little before it is heard.
    heard: Option<std::time::Instant>,
    next: Option<std::time::Instant>,
}

/// The ball's size on a beat: `1 + depth` as the beat is heard, settling back to 1 over
/// `decay` seconds (an exponential ease, so it never snaps).
pub fn beat_scale(since_beat: f32, depth: f32, decay: f32) -> f32 {
    if since_beat < 0. {
        return 1.;
    }
    1. + depth * (-since_beat / decay.max(1e-3) * 3.).exp()
}

/// Where the ball appears: `offset` (right, up, back) from the eye, turned to the way the
/// player faces on the ground (the view's pitch does not tilt it).
pub fn ball_place(eye: Vec3, forward: Vec3, offset: Vec3) -> Vec3 {
    let heading = Quat::from_rotation_y(forward.x.atan2(forward.z) + std::f32::consts::PI);
    eye + heading * offset
}
/// A collectible lit by the ball: its strength, material, overlays and beacon.
#[derive(Component)]
struct Revealed {
    strength: f32,
    material: Handle<HologramMaterial>,
    /// Each dressed mesh and its overlay. Meshes that arrive later (a level of detail
    /// placed as the player nears) are dressed as they come.
    overlays: Vec<(Entity, Entity)>,
    beacon: Entity,
}
#[derive(Component)]
struct Overlay;
#[derive(Component)]
struct Beacon {
    diamond: Entity,
}
#[derive(Resource)]
struct BeaconMeshes {
    column: Handle<Mesh>,
    diamond: Handle<Mesh>,
}

pub(crate) fn add(app: &mut App) {
    app.init_resource::<DiscoLight>()
        .add_systems(Update, toggle);
    // Headless apps (tests) have no renderer for the ball, its lights or its holograms.
    if !app.is_plugin_added::<bevy::render::RenderPlugin>() {
        return;
    }
    app.add_plugins((
        MaterialPlugin::<HologramMaterial>::default(),
        MaterialPlugin::<BeamMaterial>::default(),
    ))
    .insert_resource(FlashlightStyle::shipped())
    .add_systems(Startup, setup)
    .add_systems(Update, (follow_view, reveal, fade_holograms).chain())
    .add_systems(Update, scatter_in_fog)
    .add_systems(
        Update,
        pulse_on_beats
            .after(follow_view)
            .run_if(resource_exists::<bevy::ecs::message::Messages<ClockTick>>),
    );
}

/// Near a clock the ball keeps its time: it follows the nearest clock within reach and
/// swells on each of its beats as the beat is heard (the audio clock's predicted instant,
/// never the frame), then settles. Away from every clock it holds still.
fn pulse_on_beats(
    mut ticks: MessageReader<ClockTick>,
    style: Res<FlashlightStyle>,
    clocks: Query<(&AssetClock, &GlobalTransform)>,
    mut balls: Query<(&mut DiscoBall, &mut Transform)>,
) {
    let now = std::time::Instant::now();
    let pulse = &style.0.pulse;
    let ticks: Vec<ClockTick> = ticks.read().copied().collect();
    for (mut ball, mut transform) in &mut balls {
        let at = transform.translation;
        let nearest = clocks
            .iter()
            .map(|(clock, place)| (clock.engine, place.translation().distance(at)))
            .filter(|(_, distance)| *distance <= pulse.reach_m)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(engine, _)| engine);
        for tick in &ticks {
            if Some(tick.module) == nearest {
                if let Some(next) = ball.next.take().filter(|next| *next <= now) {
                    ball.heard = Some(next);
                }
                ball.next = Some(tick.at.unwrap_or(now));
            }
        }
        if let Some(next) = ball.next.filter(|next| *next <= now) {
            ball.heard = Some(next);
            ball.next = None;
        }
        if nearest.is_none() {
            ball.heard = None;
            ball.next = None;
        }
        let scale = ball.heard.map_or(1., |heard| {
            beat_scale(
                now.duration_since(heard).as_secs_f32(),
                pulse.depth,
                pulse.decay_seconds,
            )
        });
        let scale = Vec3::splat(scale);
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
}

/// Room in the GPU's light clusters for the ball's many spotlights, from the start: Bevy
/// grows these lists when they overflow, but lighting can be wrong for a few frames when it
/// does (it warned of it the first time the ball lit near the camera, 2026-11-18).
const CLUSTER_Z_SLICES: usize = 4096;
const CLUSTER_INDICES: usize = 262_144;

fn setup(
    mut commands: Commands,
    style: Res<FlashlightStyle>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut beam_materials: Option<ResMut<Assets<BeamMaterial>>>,
    clusters: Option<ResMut<bevy::light::cluster::GlobalClusterSettings>>,
) {
    if let Some(mut clusters) = clusters
        && let Some(gpu) = clusters.gpu_clustering.as_mut()
    {
        gpu.initial_z_slice_list_capacity = gpu.initial_z_slice_list_capacity.max(CLUSTER_Z_SLICES);
        gpu.initial_index_list_capacity = gpu.initial_index_list_capacity.max(CLUSTER_INDICES);
    }
    let style = &style.0;
    commands.insert_resource(BeaconMeshes {
        column: meshes.add(Cylinder::new(style.beacon.radius_m, style.beacon.height_m)),
        diamond: meshes.add(
            Sphere::new(style.beacon.diamond_m)
                .mesh()
                .ico(0)
                .expect("ico"),
        ),
    });
    let mut beams = beam_directions(style.beams.count, &style.beams.tilt_deg);
    beams.extend(upward_beam_directions(
        style.beams.up_count,
        &style.beams.up_tilt_deg,
    ));
    let ball = commands
        .spawn((
            DiscoBall {
                angle: 0.,
                beams: beams.clone(),
                placed: false,
                heard: None,
                next: None,
            },
            Name::new("Disco ball"),
            Transform::default(),
            Visibility::Hidden,
        ))
        .id();
    commands.spawn((
        WorldAssetRoot(assets.load("equipment/disco_ball/disco_ball.glb#Scene0")),
        Transform::default(),
        ChildOf(ball),
    ));
    let width = style.beams.width_deg.to_radians();
    // The glow in the haze: one cone mesh, one material per beam colour.
    let glow = style.beams.scatter.and_then(|scatter| {
        let materials = beam_materials.as_mut()?;
        let handles: Vec<_> = style
            .beams
            .colors
            .iter()
            .map(|&[r, g, b]| {
                let c = LinearRgba::from(Color::srgb(r, g, b));
                materials.add(BeamMaterial {
                    beam: BeamUniform {
                        colour: Vec4::new(c.red, c.green, c.blue, 1.),
                        params: Vec4::new(0., scatter.edge_power, 0., 0.),
                    },
                })
            })
            .collect();
        Some((meshes.add(unit_cone(24)), handles))
    });
    if let Some((_, handles)) = &glow {
        commands.insert_resource(BeamGlow(handles.clone()));
    }
    let reach = style.beams.range_m;
    let radius = reach * width.tan();
    for (i, direction) in beams.iter().enumerate() {
        let [r, g, b] = style.beams.colors[i % style.beams.colors.len().max(1)];
        let tilt = (-direction.y).asin().to_degrees();
        // Downward spots land farther the shallower they are; upward ones keep the base.
        let intensity = if direction.y < 0. {
            beam_intensity(
                style.beams.intensity_lumens,
                tilt,
                style.beams.reference_tilt_deg,
            )
        } else {
            style.beams.intensity_lumens
        };
        commands
            .spawn((
                SpotLight {
                    color: Color::srgb(r, g, b),
                    intensity,
                    range: style.beams.range_m,
                    // Crisp spots, as a mirror ball's are.
                    inner_angle: width * 0.85,
                    outer_angle: width,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::default().looking_to(*direction, Vec3::Y),
                ChildOf(ball),
            ))
            .with_children(|beam| {
                if let Some((cone, handles)) = &glow {
                    beam.spawn((
                        Mesh3d(cone.clone()),
                        MeshMaterial3d(handles[i % handles.len()].clone()),
                        Transform::from_scale(Vec3::new(radius, radius, reach)),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                }
            });
    }
}

/// B switches the disco light on or off. Not while a screen is open.
fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    screens: Option<Res<ScreenMode>>,
    mut light: ResMut<DiscoLight>,
) {
    if keys.just_pressed(stubby_core::FLASHLIGHT)
        && !screens.is_some_and(|screens| screens.open.is_some())
    {
        light.0 = !light.0;
    }
}

/// Show the ball while the disco light is on (and no screen is open). Switching it on
/// places it above the head to the left; it stays there while on, and spins.
#[allow(clippy::type_complexity)] // The ball and the eye.
fn follow_view(
    time: Res<Time<Real>>,
    light: Res<DiscoLight>,
    screens: Option<Res<ScreenMode>>,
    style: Res<FlashlightStyle>,
    eyes: Query<&GlobalTransform, (With<Player>, Without<HudCamera>)>,
    mut balls: Query<(&mut DiscoBall, &mut Transform, &mut Visibility)>,
) {
    let on = light.0 && !screens.is_some_and(|screens| screens.open.is_some());
    let Ok(eye) = eyes.single() else { return };
    for (mut ball, mut transform, mut visibility) in &mut balls {
        visibility.set_if_neq(if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if !on {
            ball.placed = false;
            continue;
        }
        if !ball.placed {
            ball.placed = true;
            transform.translation = ball_place(
                eye.translation(),
                eye.forward().as_vec3(),
                Vec3::from(style.0.ball.offset),
            );
        }
        ball.angle = (ball.angle
            + style.0.ball.turns_per_second * std::f32::consts::TAU * time.delta_secs())
        .rem_euclid(std::f32::consts::TAU);
        transform.rotation = Quat::from_rotation_y(ball.angle);
    }
}

/// Overlay each mesh below `entity` not yet in `dressed` with the hologram.
fn dress(
    commands: &mut Commands,
    entity: Entity,
    material: &Handle<HologramMaterial>,
    dressed: &mut Vec<(Entity, Entity)>,
    children: &Query<&Children>,
    meshes: &Query<&Mesh3d, Without<Overlay>>,
) {
    for child in children.iter_descendants(entity) {
        let Ok(mesh) = meshes.get(child) else {
            continue;
        };
        if dressed.iter().any(|(m, _)| *m == child) {
            continue;
        }
        let overlay = commands
            .spawn((
                Overlay,
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                NotShadowCaster,
                NotShadowReceiver,
                ChildOf(child),
            ))
            .id();
        dressed.push((child, overlay));
    }
}

/// While the light is on, show every collectible within reach; start a hologram on newly
/// shown ones.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn reveal(
    mut commands: Commands,
    time: Res<Time<Real>>,
    style: Res<FlashlightStyle>,
    beacon_meshes: Res<BeaconMeshes>,
    mut materials: ResMut<Assets<HologramMaterial>>,
    balls: Query<(&DiscoBall, &Transform, &Visibility)>,
    pickups: Query<(
        Entity,
        &Interactable,
        &GlobalTransform,
        Option<&mut Revealed>,
    )>,
    children: Query<&Children>,
    mesh_query: Query<&Mesh3d, Without<Overlay>>,
) {
    let style = &style.0;
    let Ok((ball, at, visibility)) = balls.single() else {
        return;
    };
    let beams: Vec<Vec3> = ball.beams.iter().map(|b| at.rotation * *b).collect();
    let lit = *visibility != Visibility::Hidden;
    let dt = time.delta_secs();
    let mut pickups = pickups;
    for (entity, what, place, revealed) in &mut pickups {
        if *what != Interactable::Pickup {
            continue;
        }
        let hit = lit
            && touched(
                at.translation,
                &beams,
                place.translation(),
                style.reveal.width_deg,
                style.reveal.range_m,
            );
        match revealed {
            Some(mut revealed) => {
                let strength = flare(revealed.strength, hit, dt, style.reveal.fade_seconds);
                if strength != revealed.strength {
                    revealed.strength = strength;
                    if let Some(mut material) = materials.get_mut(&revealed.material) {
                        material.hologram.params.x = strength;
                    }
                }
                if hit {
                    let material = revealed.material.clone();
                    dress(
                        &mut commands,
                        entity,
                        &material,
                        &mut revealed.overlays,
                        &children,
                        &mesh_query,
                    );
                }
            }
            None if hit => {
                let material = materials.add(hologram(&style.hologram, 1.));
                let mut overlays = Vec::new();
                dress(
                    &mut commands,
                    entity,
                    &material,
                    &mut overlays,
                    &children,
                    &mesh_query,
                );
                let diamond = commands
                    .spawn((
                        Mesh3d(beacon_meshes.diamond.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(0., style.beacon.height_m + style.beacon.diamond_m, 0.),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();
                let column = commands
                    .spawn((
                        Mesh3d(beacon_meshes.column.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(0., style.beacon.height_m / 2., 0.),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();
                let beacon = commands
                    .spawn((
                        Beacon { diamond },
                        Transform::from_translation(place.translation()),
                        Visibility::Inherited,
                    ))
                    .add_children(&[column, diamond])
                    .id();
                commands.entity(entity).insert((
                    stubby_core::Spotlit,
                    Revealed {
                        strength: 1.,
                        material,
                        overlays,
                        beacon,
                    },
                ));
            }
            None => {}
        }
    }
}

/// Beacons follow their collectible and turn; holograms that have faded out, or whose
/// collectible was taken, are removed.
#[allow(clippy::type_complexity)] // Revealed collectibles, their beacons and diamonds.
fn fade_holograms(
    mut commands: Commands,
    time: Res<Time>,
    revealed: Query<(Entity, &Revealed, &GlobalTransform, Option<&Interactable>)>,
    mut beacons: Query<(&Beacon, &mut Transform), Without<Overlay>>,
    mut diamonds: Query<&mut Transform, (Without<Beacon>, Without<Overlay>)>,
) {
    for (entity, hologram, place, available) in &revealed {
        if hologram.strength <= 0. || available.is_none() {
            for (_, overlay) in &hologram.overlays {
                commands.entity(*overlay).try_despawn();
            }
            commands.entity(hologram.beacon).try_despawn();
            commands
                .entity(entity)
                .remove::<(Revealed, stubby_core::Spotlit)>();
            continue;
        }
        if let Ok((beacon, mut transform)) = beacons.get_mut(hologram.beacon) {
            transform.translation = place.translation();
            if let Ok(mut diamond) = diamonds.get_mut(beacon.diamond) {
                diamond.rotation = Quat::from_rotation_y(time.elapsed_secs() * 1.6);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_beams_scatter_less_as_the_air_clears() {
        let style = FlashlightStyle::shipped();
        let scatter = style.0.beams.scatter.expect("the shipped beams scatter");
        let bleak = scatter.strength(0.007);
        let clean = scatter.strength(0.0015);
        assert!(bleak > 0. && clean < bleak * 0.3);
        assert_eq!(scatter.strength(0.), 0.);
    }

    #[test]
    fn b_toggles_the_disco_light_and_the_recorder_stays_in_hand() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ScreenMode>()
            .init_resource::<crate::hand::Hand>();
        add(&mut app);
        let press = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(stubby_core::FLASHLIGHT);
            keys.clear();
            keys.press(stubby_core::FLASHLIGHT);
            app.update();
        };
        let on = |app: &App| app.world().resource::<DiscoLight>().0;
        assert!(!on(&app), "the game opens with the light off");
        press(&mut app);
        assert!(on(&app));
        assert_eq!(
            app.world().resource::<crate::hand::Hand>().0,
            Some(crate::hand::HandItem::FieldRecorder),
            "the recorder stays in hand"
        );
        press(&mut app);
        assert!(!on(&app));
        app.world_mut().resource_mut::<ScreenMode>().open = Some(stubby_core::Screen::Inventory);
        press(&mut app);
        assert!(!on(&app), "not while a screen is open");
    }

    #[test]
    fn beams_ring_the_ball_tilted_down_near_and_far() {
        let beams = beam_directions(12, &[14., 26.]);
        assert_eq!(beams.len(), 12);
        for (i, beam) in beams.iter().enumerate() {
            assert!((beam.length() - 1.).abs() < 1e-5);
            let below = (-beam.y).asin().to_degrees();
            assert!((below - if i % 2 == 0 { 14. } else { 26. }).abs() < 1e-3);
        }
        // Evenly round the ball: opposite beams point opposite ways horizontally.
        assert!(beams[0].xz().normalize().dot(beams[6].xz().normalize()) < -0.999);
    }
    #[test]
    fn the_upward_ring_points_above_the_horizon_between_the_downward_beams() {
        let down = beam_directions(8, &[14.]);
        let up = upward_beam_directions(8, &[25., 50.]);
        for (i, beam) in up.iter().enumerate() {
            let above = beam.y.asin().to_degrees();
            assert!((above - if i % 2 == 0 { 25. } else { 50. }).abs() < 1e-3);
        }
        // Half a step round from each downward beam's heading.
        let step = std::f32::consts::TAU / 8.;
        let between = down[0].xz().angle_to(up[0].xz()).abs();
        assert!((between - step / 2.).abs() < 1e-4, "{between}");
    }
    #[test]
    fn a_beam_lights_what_it_crosses_within_range() {
        let origin = Vec3::new(0., 1.5, 0.);
        let beam = (Vec3::new(0., -1., -4.)).normalize();
        let on_spot = origin + beam * 10.;
        assert!(touched(origin, &[beam], on_spot, 7., 30.));
        assert!(
            !touched(origin, &[beam], origin + beam * 40., 7., 30.),
            "out of range"
        );
        let aside = origin + Quat::from_rotation_y(0.4) * beam * 10.;
        assert!(!touched(origin, &[beam], aside, 7., 30.), "between beams");
        // A bug in the air over the beam's spots is caught too.
        let above = origin + Vec3::new(0., 0.5, -12.);
        assert!(touched(origin, &[beam], above, 7., 30.), "at any height");
        // Far spots are brighter, so they read as bright as near ones.
        assert!(beam_intensity(1., 4.4, 14.) > 9. * beam_intensity(1., 14., 14.));
    }
    #[test]
    fn holograms_flare_then_fade() {
        assert_eq!(flare(0.2, true, 0.1, 1.2), 1.);
        let after = flare(1., false, 0.6, 1.2);
        assert!((after - 0.5).abs() < 1e-5);
        assert_eq!(flare(0.1, false, 0.6, 1.2), 0.);
    }
    #[test]
    fn the_ball_appears_in_the_upper_left_corner_of_a_level_view_whatever_the_pitch() {
        let eye = Vec3::new(2., 1.7, 5.);
        let offset = Vec3::from(FlashlightStyle::shipped().0.ball.offset);
        // Facing north (-Z), looking level: left is -X, up is +Y, ahead is -Z.
        let at = ball_place(eye, Vec3::NEG_Z, offset);
        let to = at - eye;
        // Matched to the user's reference: 33 degrees left, 17 up, 0.95 m away, so a
        // level 16:9 view (36 degrees each side, 22.5 up) cuts it at the left and top.
        let left = (-to.x).atan2(-to.z).to_degrees();
        let up = to.y.atan2(Vec2::new(to.x, to.z).length()).to_degrees();
        assert!((30.0..36.0).contains(&left), "{left} degrees left");
        assert!((14.0..20.0).contains(&up), "{up} degrees up");
        assert!(
            (to.length() - 0.95).abs() < 0.05,
            "0.95 m away: {}",
            to.length()
        );
        // Looking down at the ground does not move it: only the heading counts.
        let down = Vec3::new(0., -0.8, -0.6).normalize();
        assert!(ball_place(eye, down, offset).distance(at) < 1e-4);
        // Facing east (+X), left is north (-Z).
        let east = ball_place(eye, Vec3::X, offset);
        assert!(east.z < eye.z && east.x > eye.x, "{east}");
    }

    #[test]
    fn the_ball_swells_on_a_heard_beat_and_settles() {
        assert_eq!(
            beat_scale(-0.1, 0.06, 0.2),
            1.,
            "not before the beat is heard"
        );
        assert!((beat_scale(0., 0.06, 0.2) - 1.06).abs() < 1e-6);
        assert!(beat_scale(0.1, 0.06, 0.2) < 1.03);
        assert!(
            beat_scale(0.4, 0.06, 0.2) < 1.001,
            "settled well before the next beat"
        );
    }
    #[test]
    fn the_shipped_style_loads() {
        let style = FlashlightStyle::shipped().0;
        assert!(!style.beams.colors.is_empty());
        // The shallowest beam's spot, from a ball about 2 m up, lands out at the reveal's
        // reach, and its light carries that far.
        let shallowest = style
            .beams
            .tilt_deg
            .iter()
            .copied()
            .fold(f32::MAX, f32::min);
        let farthest = 2. / shallowest.to_radians().tan();
        assert!(farthest >= style.reveal.range_m * 0.85, "{farthest} m");
        assert!(style.beams.range_m >= farthest);
    }
}
