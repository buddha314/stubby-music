//! The inventory screen on Tab, as in Satisfactory: a HUD panel over the world listing
//! what the player carries (add-inventory-and-goals-hud, inventory-screen spec).
//!
//! Items come from the crafting plugin's `PlayerInventory` and are named by the rules
//! catalog; nothing here names an item. Items the recipe book knows but the player holds
//! none of stay listed, dimmed. Opening and closing go through `controls::ScreenMode`.
use bevy::prelude::*;

use super::controls::{Screen, ScreenMode};
use super::resources::{PlayerInventory, RecipeCatalog};

const PANEL: Color = Color::srgba(0.035, 0.033, 0.03, 0.94);
const BACKDROP: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);
const TILE: Color = Color::srgba(0.09, 0.085, 0.075, 1.0);
const EDGE: Color = Color::srgb(0.42, 0.29, 0.13);
const INK: Color = Color::srgb(0.95, 0.94, 0.89);
const MUTED: Color = Color::srgb(0.55, 0.53, 0.49);

pub struct InventoryScreenPlugin;
impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, inventory_full_notice)
            .add_observer(show_tooltip)
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

/// Satisfactory's "Inventory Full" warning: a red banner under the reticle, with the
/// icon of what did not fit, shown for a moment and faded out. Fires whenever something
/// could not go into the inventory (a collection with no free box).
#[derive(Component)]
struct FullNotice {
    shown: f32,
}
const NOTICE_SECONDS: f32 = 2.2;
const NOTICE_FADE: f32 = 0.5;
const NOTICE_RED: Color = Color::srgb(0.62, 0.13, 0.08);

fn inventory_full_notice(
    mut commands: Commands,
    time: Option<Res<Time<Real>>>,
    server: Option<Res<AssetServer>>,
    mut full: MessageReader<super::recipes_plugin::InventoryFull>,
    screens: Option<Res<ScreenMode>>,
    mut notices: Query<(Entity, &mut FullNotice, &mut BackgroundColor)>,
    mut texts: Query<&mut TextColor, With<FullNoticeText>>,
) {
    if let Some(event) = full.read().last() {
        for (entity, ..) in &notices {
            commands.entity(entity).despawn();
        }
        let mut banner = commands.spawn((
            FullNotice { shown: 0. },
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                // Under the reticle in the world; under a machine's window when one is
                // open (the rack view draws over the screen's UI inside the window).
                top: percent(if screens.as_ref().is_some_and(|s| s.open.is_some()) {
                    90
                } else {
                    58
                }),
                margin: UiRect::left(px(-150)),
                width: px(300),
                padding: UiRect::axes(px(14), px(8)),
                column_gap: px(12),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(NOTICE_RED.with_alpha(0.92)),
            BorderColor::all(Color::srgb(1., 0.7, 0.31)),
            GlobalZIndex(40),
            Pickable::IGNORE,
        ));
        banner.with_children(|banner| {
            if let Some(server) = &server {
                banner.spawn((
                    item_icon(server, &event.item),
                    Node {
                        width: px(36),
                        height: px(36),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
            banner.spawn((
                FullNoticeText,
                text("INVENTORY FULL", 22., Color::WHITE),
                Pickable::IGNORE,
            ));
        });
        return;
    }
    for (entity, mut notice, mut background) in &mut notices {
        notice.shown += time.as_ref().map_or(0., |t| t.delta_secs());
        let fade = ((NOTICE_SECONDS - notice.shown) / NOTICE_FADE).clamp(0., 1.);
        background.0 = NOTICE_RED.with_alpha(0.92 * fade);
        for mut colour in &mut texts {
            colour.0 = Color::WHITE.with_alpha(fade);
        }
        if notice.shown >= NOTICE_SECONDS {
            commands.entity(entity).despawn();
        }
    }
}
#[derive(Component)]
struct FullNoticeText;

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
                    width: px(4. * (SLOT_PX + 12.) + 48.),
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
    // Satisfactory's inventory: 16 boxes, each holding up to 100 of one kind, filled in
    // order by what you hold; the rest are empty. Nothing you hold none of is shown.
    let held: Vec<(&str, u32)> = player.items.boxes().collect();
    commands.entity(grid).despawn_related::<Children>();
    commands.entity(grid).with_children(|grid| {
        for index in 0..stubby_recipes::INVENTORY_BOXES as usize {
            grid.spawn((
                BackgroundColor(TILE),
                BorderColor::all(EDGE),
                Node {
                    width: px(SLOT_PX),
                    height: px(SLOT_PX),
                    padding: UiRect::all(px(6)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
            ))
            .with_children(|slot| {
                if let Some((id, count)) = held.get(index) {
                    let name = catalog
                        .0
                        .item(id)
                        .map_or(id.to_string(), |i| i.name.clone());
                    icon_slot(slot, server.as_deref(), id, &name, *count, SLOT_PX - 14.);
                }
            });
        }
    });
}

const SLOT_PX: f32 = 88.;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_inventory_shows_the_notice_then_it_goes() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin)
            .add_message::<crate::recipes_plugin::InventoryFull>()
            .add_systems(Update, inventory_full_notice);
        app.world_mut()
            .write_message(crate::recipes_plugin::InventoryFull {
                item: "reed".into(),
            });
        app.update();
        app.update();
        let shown = app
            .world_mut()
            .query::<&FullNotice>()
            .iter(app.world())
            .count();
        assert_eq!(shown, 1);
        app.world_mut()
            .query::<&mut FullNotice>()
            .iter_mut(app.world_mut())
            .for_each(|mut n| n.shown = NOTICE_SECONDS);
        app.update();
        app.update();
        let shown = app
            .world_mut()
            .query::<&FullNotice>()
            .iter(app.world())
            .count();
        assert_eq!(shown, 0);
    }

    #[test]
    fn tab_opens_the_inventory_and_shows_only_held_items_by_catalog_name() {
        let mut app = App::new();
        app.add_plugins(crate::recipes_plugin::RecipesPlugin::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ScreenMode>()
            .add_plugins(InventoryScreenPlugin)
            .add_systems(PreUpdate, super::super::controls::toggle_screens);
        app.world_mut()
            .resource_mut::<PlayerInventory>()
            .items
            .grant(&[stubby_recipes::Stack {
                item: "reed".into(),
                quantity: 6,
            }])
            .unwrap();
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        app.update();
        let named: Vec<String> = app
            .world_mut()
            .query::<&ItemName>()
            .iter(app.world())
            .map(|n| n.0.clone())
            .collect();
        // The starter patch cables and the reeds: only held items, named by the catalog.
        assert_eq!(named, ["Patch cable", "Reed"]);
    }
}
