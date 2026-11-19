//! Omar's shadow (`add-omar-gloves-and-shadow` D2, D3): a body in his silhouette that
//! stands where the player stands and casts the sun's shadow, on a render layer only the
//! sun sees (`stubby_core::SHADOW_ONLY_LAYER`), so the camera inside it never draws it.
//! It turns with the player's heading (not the look's pitch), swings its legs and arms
//! while walking, crouches with the player, and raises its right arm while the hand holds
//! an item, so the shadow agrees with the gloves.
use bevy::{camera::visibility::RenderLayers, prelude::*};
use stubby_core::SHADOW_ONLY_LAYER;

use crate::controls::Player;
use crate::hand::Hand;

const MODEL: &str = "characters/omar_shadow/omar_shadow.glb#Scene0";
/// Eye height standing (`controls::EYE_HEIGHT`): crouching is the eye below it.
const EYE_HEIGHT: f32 = 1.65;
/// Metres walked per full stride (two steps).
const STRIDE_M: f32 = 1.6;
/// How far legs and arms swing at a walk, radians.
const SWING: f32 = 0.45;
/// The right arm raised, holding the item out in front.
const HOLD_RAISE: f32 = 1.15;

pub(crate) fn add(app: &mut App) {
    app.add_systems(
        Update,
        (spawn_body, shadow_only, follow, pose_limbs).chain(),
    );
}

/// The shadow body.
#[derive(Component, Default)]
pub struct ShadowBody {
    /// Where in the stride it is, radians, and how much it is walking (0 to 1).
    phase: f32,
    walking: f32,
    last: Option<Vec3>,
}

fn spawn_body(
    mut commands: Commands,
    assets: Res<AssetServer>,
    players: Query<(), With<Player>>,
    bodies: Query<(), With<ShadowBody>>,
) {
    if players.is_empty() || !bodies.is_empty() {
        return;
    }
    commands.spawn((
        ShadowBody::default(),
        Name::new("Omar's shadow"),
        WorldAssetRoot(assets.load(MODEL)),
        Transform::default(),
        Visibility::Inherited,
    ));
}

/// Every mesh of the body goes on the shadow-only layer as it loads.
fn shadow_only(
    mut commands: Commands,
    meshes: Query<Entity, Added<Mesh3d>>,
    parents: Query<&ChildOf>,
    bodies: Query<(), With<ShadowBody>>,
) {
    for mesh in &meshes {
        if parents.iter_ancestors(mesh).any(|a| bodies.contains(a)) {
            commands
                .entity(mesh)
                .insert(RenderLayers::layer(SHADOW_ONLY_LAYER));
        }
    }
}

/// Stand where the player stands, facing their heading, crouching with them.
fn follow(
    time: Res<Time>,
    players: Query<(&Transform, &Player), Without<ShadowBody>>,
    mut bodies: Query<(&mut Transform, &mut ShadowBody)>,
) {
    let Ok((eye, player)) = players.single() else {
        return;
    };
    let (yaw, _, _) = eye.rotation.to_euler(EulerRot::YXZ);
    let feet = Vec3::new(eye.translation.x, player.feet(), eye.translation.z);
    let height = (eye.translation.y - player.feet()) / EYE_HEIGHT;
    for (mut transform, mut body) in &mut bodies {
        let moved = body
            .last
            .map_or(0., |last| (feet - last).with_y(0.).length());
        body.last = Some(feet);
        let dt = time.delta_secs().max(1e-4);
        let target = if moved / dt > 0.3 { 1. } else { 0. };
        body.walking += (target - body.walking) * (dt * 8.).min(1.);
        body.phase =
            (body.phase + moved / STRIDE_M * std::f32::consts::TAU) % std::f32::consts::TAU;
        transform.translation = feet;
        transform.rotation = Quat::from_rotation_y(yaw);
        transform.scale = Vec3::new(1., height.clamp(0.5, 1.), 1.);
    }
}

/// Swing the limbs from their pivots; the right arm holds the item up.
fn pose_limbs(
    hand: Res<Hand>,
    bodies: Query<(Entity, &ShadowBody)>,
    children: Query<&Children>,
    mut limbs: Query<(&Name, &mut Transform), Without<ShadowBody>>,
) {
    for (root, body) in &bodies {
        let swing = body.phase.sin() * SWING * body.walking;
        for entity in children.iter_descendants(root) {
            let Ok((name, mut transform)) = limbs.get_mut(entity) else {
                continue;
            };
            // Positive turns about +X carry a hanging limb forward (-Z).
            let angle = match name.as_str() {
                "Omar_Hip_L" => swing,
                "Omar_Hip_R" => -swing,
                "Omar_Shoulder_L" => -swing * 0.8,
                "Omar_Shoulder_R" if hand.0.is_some() => HOLD_RAISE,
                "Omar_Shoulder_R" => swing * 0.8,
                _ => continue,
            };
            transform.rotation = Quat::from_rotation_x(angle);
        }
    }
}
