//! The inventory screen on Tab, as in Satisfactory: a HUD panel over the world listing
//! what the player carries (add-inventory-and-goals-hud, inventory-screen spec).
//!
//! Items come from the crafting plugin's `PlayerInventory` and are named by the rules
//! catalog; nothing here names an item. Items the recipe book knows but the player holds
//! none of stay listed, dimmed. Opening and closing go through `controls::ScreenMode`.
use bevy::prelude::*;
use stubby_recipes::SignalUse;

use super::controls::{Screen, ScreenMode};
use super::resources::{PlayerInventory, RecipeCatalog};

const PANEL: Color = Color::srgba(0.035, 0.033, 0.03, 0.94);
const BACKDROP: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);
const TILE: Color = Color::srgba(0.09, 0.085, 0.075, 1.0);
const EDGE: Color = Color::srgb(0.42, 0.29, 0.13);
const AMBER: Color = Color::srgb(1.0, 0.70, 0.31);
const INK: Color = Color::srgb(0.95, 0.94, 0.89);
const MUTED: Color = Color::srgb(0.55, 0.53, 0.49);

pub struct InventoryScreenPlugin;
impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(show_tooltip)
            .add_observer(move_tooltip)
            .add_observer(hide_tooltip)
            .add_systems(Startup, spawn)
            .add_systems(Update, (show, fill).chain());
    }
}

#[derive(Component)]
struct InventoryRoot;
#[derive(Component)]
struct InventoryGrid;

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

/// An item's icon (`assets/ui/icons/<item>.png`, from `tools/icons/make_item_icons.py`).
pub(crate) fn item_icon(server: &AssetServer, item: &str) -> ImageNode {
    ImageNode::new(server.load(format!("ui/icons/{item}.png")))
}

/// A Satisfactory-style inventory slot: the item's icon with its count in the corner,
/// dimmed when the player holds none.
pub(crate) fn icon_slot(
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
pub(crate) struct ItemName(pub String);
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

fn spawn(mut commands: Commands) {
    commands
        .spawn((
            InventoryRoot,
            Visibility::Hidden,
            GlobalZIndex(10),
            BackgroundColor(BACKDROP),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                BackgroundColor(PANEL),
                BorderColor::all(EDGE),
                Node {
                    width: px(760),
                    max_height: percent(80),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(24)),
                    row_gap: px(16),
                    border: UiRect::all(px(1)),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn(text("INVENTORY", 28., INK));
                panel.spawn((
                    InventoryGrid,
                    Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(12),
                        row_gap: px(12),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ));
                panel.spawn(text("Tab / Esc  close", 15., MUTED));
            });
        });
}

fn show(screens: Res<ScreenMode>, mut root: Query<&mut Visibility, With<InventoryRoot>>) {
    let visible = if screens.open == Some(Screen::Inventory) {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut v in &mut root {
        v.set_if_neq(visible);
    }
}

fn uses(item_uses: &[SignalUse]) -> String {
    item_uses
        .iter()
        .map(|u| match u {
            SignalUse::Vco => "VCO",
            SignalUse::Lfo => "LFO",
            SignalUse::Gate => "Gate",
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Rebuild the tiles when the screen opens or the inventory changes while it is open.
fn fill(
    mut commands: Commands,
    screens: Res<ScreenMode>,
    player: Res<PlayerInventory>,
    catalog: Option<Res<RecipeCatalog>>,
    server: Option<Res<AssetServer>>,
    grid: Query<Entity, With<InventoryGrid>>,
    mut was_open: Local<bool>,
) {
    let open = screens.open == Some(Screen::Inventory);
    let opened = open && !*was_open;
    *was_open = open;
    if !open || !(opened || player.is_changed()) {
        return;
    }
    let (Ok(grid), Some(catalog)) = (grid.single(), catalog) else {
        return;
    };
    let mut ids: Vec<&str> = player.items.items().map(|(id, _)| id).collect();
    ids.extend(player.book.items());
    ids.sort_unstable();
    ids.dedup();

    commands.entity(grid).despawn_related::<Children>();
    commands.entity(grid).with_children(|grid| {
        if ids.is_empty() {
            grid.spawn(text(
                "Nothing yet. Listen for a lightning bug and press E to collect.",
                17.,
                MUTED,
            ));
        }
        for id in ids {
            let Some(item) = catalog.0.item(id) else {
                continue;
            };
            let count = player.items.quantity(id);
            let (ink, accent) = if count > 0 {
                (INK, AMBER)
            } else {
                (MUTED, MUTED)
            };
            grid.spawn((
                BackgroundColor(TILE),
                BorderColor::all(if count > 0 { EDGE } else { TILE }),
                Node {
                    width: px(346),
                    min_height: px(96),
                    column_gap: px(12),
                    padding: UiRect::all(px(10)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
            ))
            .with_children(|tile| {
                icon_slot(tile, server.as_deref(), id, &item.name, count, 76.);
                tile.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4),
                    flex_shrink: 1.,
                    ..default()
                })
                .with_children(|words| {
                    words.spawn(text(item.name.clone(), 19., ink));
                    words.spawn(text(format!("x {count}"), 16., accent));
                    if !item.uses.is_empty() {
                        words.spawn(text(uses(&item.uses), 14., accent));
                    }
                    words.spawn(text(item.description.clone(), 13., MUTED));
                });
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_opens_the_inventory_and_lists_held_items_by_catalog_name() {
        let mut app = App::new();
        app.add_plugins(super::super::recipes_plugin::RecipesPlugin::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ScreenMode>()
            .add_plugins(InventoryScreenPlugin)
            .add_systems(PreUpdate, super::super::controls::toggle_screens);
        app.world_mut()
            .resource_mut::<PlayerInventory>()
            .book
            .discover("reed");
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        app.update();
        let texts: Vec<String> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        assert!(texts.iter().any(|t| t == "Reed"), "{texts:?}");
        assert!(texts.iter().any(|t| t == "x 0"), "{texts:?}");
    }
}
