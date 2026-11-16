//! Shared HUD widgets (a service crate, split-game-into-systems): Satisfactory-style item
//! icons and slots, and the tooltip that names an item under the pointer. Any system's
//! screen (the inventory, a rack) uses the same ones, so items look alike everywhere.
use bevy::prelude::*;

const PANEL: Color = Color::srgba(0.035, 0.033, 0.03, 0.94);
const EDGE: Color = Color::srgb(0.42, 0.29, 0.13);
const INK: Color = Color::srgb(0.95, 0.94, 0.89);
const MUTED: Color = Color::srgb(0.55, 0.53, 0.49);

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

/// Shows an [`ItemName`] tooltip while the pointer is over a tagged icon.
pub struct ItemTooltipPlugin;
impl Plugin for ItemTooltipPlugin {
    fn build(&self, app: &mut App) {
        if app.world().contains_resource::<TooltipsAdded>() {
            return;
        }
        app.insert_resource(TooltipsAdded)
            .add_observer(show_tooltip)
            .add_observer(move_tooltip)
            .add_observer(hide_tooltip);
    }
}
#[derive(Resource)]
struct TooltipsAdded;

/// An item's icon (`assets/ui/icons/<item>.png`, from `tools/icons/make_item_icons.py`).
pub fn item_icon(server: &AssetServer, item: &str) -> ImageNode {
    ImageNode::new(server.load(format!("ui/icons/{item}.png")))
}

/// A Satisfactory-style inventory slot: the item's icon with its count in the corner,
/// dimmed when the player holds none.
pub fn icon_slot(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    server: Option<&AssetServer>,
    item: &str,
    name: &str,
    count: u32,
    size: f32,
) {
    let mut slot = parent.spawn((
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.,
            ..default()
        },
        ItemName(name.into()),
    ));
    if let Some(server) = server {
        let mut icon = item_icon(server, item);
        if count == 0 {
            icon.color = Color::srgba(1., 1., 1., 0.35);
        }
        slot.insert(icon);
    }
    slot.with_children(|slot| {
        slot.spawn((
            text(
                count.to_string(),
                (size * 0.28).max(12.),
                if count > 0 { INK } else { MUTED },
            ),
            Node {
                position_type: PositionType::Absolute,
                right: px(5),
                bottom: px(2),
                ..default()
            },
            TextShadow::default(),
            Pickable::IGNORE,
        ));
    });
}

/// On an icon: the item's name, shown in a tooltip while the pointer is over it.
#[derive(Component)]
pub struct ItemName(pub String);
#[derive(Component)]
struct Tooltip;

fn tooltip_at(position: Vec2) -> (Val, Val) {
    (px(position.x + 16.), px(position.y + 18.))
}

fn show_tooltip(
    over: On<bevy::picking::events::PointerOver>,
    mut commands: Commands,
    names: Query<&ItemName>,
    old: Query<Entity, With<Tooltip>>,
) {
    let Ok(ItemName(name)) = names.get(over.entity) else {
        return;
    };
    old.iter().for_each(|e| commands.entity(e).despawn());
    let (left, top) = tooltip_at(over.pointer.position);
    commands
        .spawn((
            Tooltip,
            Node {
                position_type: PositionType::Absolute,
                left,
                top,
                padding: UiRect::axes(px(8), px(4)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor::all(EDGE),
            GlobalZIndex(200),
            Pickable::IGNORE,
        ))
        .with_children(|tip| {
            tip.spawn((text(name.clone(), 15., INK), Pickable::IGNORE));
        });
}

fn move_tooltip(
    moved: On<bevy::picking::events::PointerMove>,
    mut tips: Query<&mut Node, With<Tooltip>>,
) {
    let (left, top) = tooltip_at(moved.pointer.position);
    for mut node in &mut tips {
        node.left = left;
        node.top = top;
    }
}

fn hide_tooltip(
    out: On<bevy::picking::events::PointerOut>,
    mut commands: Commands,
    names: Query<(), With<ItemName>>,
    tips: Query<Entity, With<Tooltip>>,
) {
    if names.contains(out.entity) {
        tips.iter().for_each(|e| commands.entity(e).despawn());
    }
}
