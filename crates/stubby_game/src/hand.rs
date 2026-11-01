//! What the player holds in the hand. The hand holds one item at a time from
//! `HAND_ITEMS`; each item has a key that brings it up or puts it away, and a model
//! and pose in the lower right of the view. Only the field recorder exists so far
//! (add-field-recorder): H.
use bevy::{light::NotShadowCaster, prelude::*};

use super::controls::{Player, RECORDER, ScreenMode};

/// An item the hand can hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandItem {
    FieldRecorder,
}

struct HandItemDef {
    item: HandItem,
    key: KeyCode,
    model: &'static str,
    pose: fn() -> Transform,
}

const HAND_ITEMS: &[HandItemDef] = &[HandItemDef {
    item: HandItem::FieldRecorder,
    key: RECORDER,
    model: "models/equipment/field_recorder.glb#Scene0",
    // About a forearm's length out, windguard leaning forward to where it listens.
    pose: || {
        Transform::from_xyz(0.17, -0.165, -0.42).with_rotation(
            Quat::from_rotation_y(0.35) * Quat::from_rotation_x(-0.7) * Quat::from_rotation_z(0.12),
        )
    },
}];

/// The item in the hand, if any. The hand starts empty.
#[derive(Resource, Default)]
pub struct Hand(pub Option<HandItem>);

/// One per item, spawned on the player camera and shown only while held.
#[derive(Component)]
struct Held(HandItem);

pub struct HandPlugin;
impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hand>()
            .add_systems(Update, (attach, choose, show, fur_casts_no_shadow).chain());
    }
}

fn attach(
    mut commands: Commands,
    assets: Res<AssetServer>,
    players: Query<Entity, With<Player>>,
    held: Query<(), With<Held>>,
) {
    if !held.is_empty() {
        return;
    }
    for player in &players {
        for def in HAND_ITEMS {
            commands.spawn((
                Held(def.item),
                Name::new(format!("Hand: {:?}", def.item)),
                WorldAssetRoot(assets.load(def.model)),
                (def.pose)(),
                Visibility::Hidden,
                ChildOf(player),
            ));
        }
    }
}

/// An item's key brings it up, or puts it away if it is already held. Not while a
/// screen is open.
fn choose(keys: Res<ButtonInput<KeyCode>>, screens: Res<ScreenMode>, mut hand: ResMut<Hand>) {
    if screens.open.is_some() {
        return;
    }
    for def in HAND_ITEMS {
        if keys.just_pressed(def.key) {
            hand.0 = if hand.0 == Some(def.item) {
                None
            } else {
                Some(def.item)
            };
        }
    }
}

fn show(hand: Res<Hand>, mut held: Query<(&Held, &mut Visibility)>) {
    for (item, mut v) in &mut held {
        v.set_if_neq(if hand.0 == Some(item.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// The recorder's stacked fur shells would shadow each other black (its profile's
/// `windguard.shells_cast_shadows`), so only the sock under them casts.
fn fur_casts_no_shadow(
    mut commands: Commands,
    meshes: Query<(Entity, Option<&Name>, Option<&ChildOf>), Added<Mesh3d>>,
    names: Query<&Name>,
) {
    let shell = |n: &Name| n.as_str().starts_with("FieldRecorder_FurShell");
    for (entity, name, parent) in &meshes {
        let parent_shell = parent
            .and_then(|p| names.get(p.parent()).ok())
            .is_some_and(shell);
        if name.is_some_and(shell) || parent_shell {
            commands.entity(entity).insert(NotShadowCaster);
        }
    }
}
