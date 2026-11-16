//! The forensic flashlight (add-forensic-flashlight), held on B. Its light is a disco ball
//! floating in front of the player: it spins and throws a ring of narrow violet and cyan
//! beams that sweep the ground. A collectible a beam passes over (anything marked
//! `Interactable::Pickup`, no asset kind named) flares into a hologram, an overlay on its
//! body and a beacon above it, and fades after. Style: `assets/ui/flashlight.json`.
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
use stubby_core::{HudCamera, Interactable, ScreenMode};

use crate::controls::Player;
use crate::hand::{Hand, HandItem};

// ---------------------------------------------------------------- style

#[derive(serde::Deserialize, Clone, Debug)]
struct StyleFile {
    ball: BallStyle,
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
#[derive(serde::Deserialize, Clone, Debug)]
struct BeamStyle {
    count: usize,
    colors: Vec<[f32; 3]>,
    intensity_lumens: f32,
    width_deg: f32,
    range_m: f32,
    tilt_deg: Vec<f32>,
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

/// Whether any beam from `origin` along `beams` (world directions) passes within
/// `width_deg` of `point`, within `range_m`.
pub fn touched(origin: Vec3, beams: &[Vec3], point: Vec3, width_deg: f32, range_m: f32) -> bool {
    let to = point - origin;
    let distance = to.length();
    distance > 1e-3
        && distance <= range_m
        && beams
            .iter()
            .any(|beam| beam.angle_between(to).to_degrees() <= width_deg)
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

// ---------------------------------------------------------------- the ball

/// The disco ball: follows the eye's place and heading (not its pitch, so it spins about
/// the vertical, as a hanging ball does).
#[derive(Component)]
struct DiscoBall {
    angle: f32,
    beams: Vec<Vec3>,
}
/// A collectible lit by the ball: its strength, material, overlays and beacon.
#[derive(Component)]
struct Revealed {
    strength: f32,
    material: Handle<HologramMaterial>,
    overlays: Vec<Entity>,
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
    // Headless apps (tests) have no renderer for the ball, its lights or its holograms.
    if !app.is_plugin_added::<bevy::render::RenderPlugin>() {
        return;
    }
    app.add_plugins(MaterialPlugin::<HologramMaterial>::default())
        .insert_resource(FlashlightStyle::shipped())
        .add_systems(Startup, setup)
        .add_systems(Update, (follow_view, reveal, fade_holograms).chain());
}

fn setup(
    mut commands: Commands,
    style: Res<FlashlightStyle>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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
    let beams = beam_directions(style.beams.count, &style.beams.tilt_deg);
    let ball = commands
        .spawn((
            DiscoBall {
                angle: 0.,
                beams: beams.clone(),
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
    for (i, direction) in beams.iter().enumerate() {
        let [r, g, b] = style.beams.colors[i % style.beams.colors.len().max(1)];
        commands.spawn((
            SpotLight {
                color: Color::srgb(r, g, b),
                intensity: style.beams.intensity_lumens,
                range: style.beams.range_m,
                // Crisp spots, as a mirror ball's are.
                inner_angle: width * 0.85,
                outer_angle: width,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::default().looking_to(*direction, Vec3::Y),
            ChildOf(ball),
        ));
    }
}

/// Show the ball while the flashlight is held (and no screen is open), keep it in front
/// of the eye, and spin it.
#[allow(clippy::type_complexity)] // The ball and the eye.
fn follow_view(
    time: Res<Time<Real>>,
    hand: Res<Hand>,
    screens: Option<Res<ScreenMode>>,
    style: Res<FlashlightStyle>,
    eyes: Query<&GlobalTransform, (With<Player>, Without<HudCamera>)>,
    mut balls: Query<(&mut DiscoBall, &mut Transform, &mut Visibility)>,
) {
    let on = hand.0 == Some(HandItem::Flashlight)
        && !screens.is_some_and(|screens| screens.open.is_some());
    let Ok(eye) = eyes.single() else { return };
    for (mut ball, mut transform, mut visibility) in &mut balls {
        visibility.set_if_neq(if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let forward = eye.forward().as_vec3();
        let heading = Quat::from_rotation_y(forward.x.atan2(forward.z) + std::f32::consts::PI);
        ball.angle = (ball.angle
            + style.0.ball.turns_per_second * std::f32::consts::TAU * time.delta_secs())
        .rem_euclid(std::f32::consts::TAU);
        transform.translation = eye.translation() + heading * Vec3::from(style.0.ball.offset);
        transform.rotation = Quat::from_rotation_y(ball.angle);
    }
}

/// Light the collectibles the beams touch; start a hologram on newly lit ones.
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
    let lit = *visibility != Visibility::Hidden;
    let beams: Vec<Vec3> = ball.beams.iter().map(|b| at.rotation * *b).collect();
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
            }
            None if hit => {
                let material = materials.add(hologram(&style.hologram, 1.));
                let overlays: Vec<Entity> = children
                    .iter_descendants(entity)
                    .filter_map(|child| {
                        mesh_query
                            .get(child)
                            .ok()
                            .map(|mesh| (child, mesh.0.clone()))
                    })
                    .map(|(child, mesh)| {
                        commands
                            .spawn((
                                Overlay,
                                Mesh3d(mesh),
                                MeshMaterial3d(material.clone()),
                                Transform::default(),
                                NotShadowCaster,
                                NotShadowReceiver,
                                ChildOf(child),
                            ))
                            .id()
                    })
                    .collect();
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
                commands.entity(entity).insert(Revealed {
                    strength: 1.,
                    material,
                    overlays,
                    beacon,
                });
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
            for overlay in &hologram.overlays {
                commands.entity(*overlay).try_despawn();
            }
            commands.entity(hologram.beacon).try_despawn();
            commands.entity(entity).remove::<Revealed>();
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
    }
    #[test]
    fn holograms_flare_then_fade() {
        assert_eq!(flare(0.2, true, 0.1, 1.2), 1.);
        let after = flare(1., false, 0.6, 1.2);
        assert!((after - 0.5).abs() < 1e-5);
        assert_eq!(flare(0.1, false, 0.6, 1.2), 0.);
    }
    #[test]
    fn the_shipped_style_loads() {
        let style = FlashlightStyle::shipped();
        assert_eq!(style.0.beams.count, 12);
        assert!(!style.0.beams.colors.is_empty());
    }
}
