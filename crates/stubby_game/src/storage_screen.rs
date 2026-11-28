//! Storage barrels (add-multiplayer-and-menus 3.3, the user's choice: storage is a built
//! container). E at a barrel opens its screen: what you carry beside what the barrel
//! holds, as Satisfactory's storage container. Clicking a box moves it across, all of it
//! or nothing. Anyone may use any barrel; in a shared world the host decides each move.
use bevy::picking::events::{PointerClick, PointerOut, PointerOver};
use bevy::prelude::*;
use stubby_core::{InteractTarget, Interactable, PlacedAsset, Screen, ScreenMode};
use stubby_ui::icon_slot;
use stubby_ui::style::{AMBER, BACKDROP, EDGE, INK, MUTED, PANEL, TILE};

use super::machine_config::MachineInteraction;
use super::recipes_plugin::{
    PlayerInventory, RecipeCatalog, RecipesSystems, StorageMove, StorageRequest, Store,
};
use stubby_core::MachineInteractionSet;

pub struct StorageScreenPlugin;
impl Plugin for StorageScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn).add_systems(
            Update,
            (
                stores_interactable,
                open_store
                    .after(MachineInteractionSet)
                    .before(RecipesSystems),
                (show, fill).chain(),
            ),
        );
    }
}

/// What E at a barrel says.
pub const PROMPT: &str = "Open the storage barrel";

#[derive(Component)]
struct StorageRoot;
/// The two grids: the player's, and the barrel's.
#[derive(Component)]
struct Grid(StorageMove);
#[derive(Component)]
struct Title;
/// A filled box: clicking it moves its items the grid's way.
#[derive(Component, Clone)]
struct MoveBox {
    item: String,
    quantity: u32,
    direction: StorageMove,
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    )
}

/// A store E can reach, as a machine is.
#[allow(clippy::type_complexity)]
fn stores_interactable(
    stores: Query<Entity, (With<Store>, With<PlacedAsset>, Without<Interactable>)>,
    mut commands: Commands,
) {
    for store in &stores {
        commands.entity(store).insert(Interactable::Machine(store));
    }
}

/// E at a barrel opens it; E again (or Escape) closes it, as a machine's rack.
fn open_store(
    keys: Res<ButtonInput<KeyCode>>,
    target: Res<InteractTarget>,
    stores: Query<(), With<Store>>,
    mut prompt: ResMut<MachineInteraction>,
    mut screens: ResMut<ScreenMode>,
) {
    if let Some(Screen::Storage(store)) = screens.open {
        if !stores.contains(store) || keys.just_pressed(stubby_core::INTERACT) {
            screens.close();
        }
        return;
    }
    if screens.open.is_some() {
        return;
    }
    let Some(store) = target.machine().filter(|m| stores.contains(*m)) else {
        return;
    };
    prompt.target = Some(store);
    prompt.prompt = PROMPT.into();
    if keys.just_pressed(stubby_core::INTERACT) {
        screens.open = Some(Screen::Storage(store));
    }
}

const SLOT_PX: f32 = 72.;

fn spawn(mut commands: Commands) {
    commands
        .spawn((
            StorageRoot,
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
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(24)),
                    row_gap: px(14),
                    border: UiRect::all(px(1)),
                    max_height: percent(85),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((Title, text("STORAGE BARREL", 26., INK)));
                panel.spawn(text(
                    "Click a box to move it across. Anyone may use this barrel.",
                    14.,
                    MUTED,
                ));
                panel
                    .spawn(Node {
                        column_gap: px(32),
                        ..default()
                    })
                    .with_children(|columns| {
                        // Both four rows: the player's 16 boxes four across, a barrel's
                        // 24 six across.
                        for (title, direction, across) in [
                            ("YOU CARRY", StorageMove::Store, 4.),
                            ("IN THE BARREL", StorageMove::Take, 6.),
                        ] {
                            columns
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(10),
                                    ..default()
                                })
                                .with_children(|column| {
                                    column.spawn(text(title, 16., AMBER));
                                    column.spawn((
                                        Grid(direction),
                                        Node {
                                            width: px(across * (SLOT_PX + 10.)),
                                            flex_wrap: FlexWrap::Wrap,
                                            column_gap: px(10),
                                            row_gap: px(10),
                                            overflow: Overflow::scroll_y(),
                                            ..default()
                                        },
                                    ));
                                });
                        }
                    });
                panel.spawn(text("E / Esc  close", 13., MUTED));
            });
        });
}

fn show(screens: Res<ScreenMode>, mut roots: Query<&mut Visibility, With<StorageRoot>>) {
    let open = matches!(screens.open, Some(Screen::Storage(_)));
    for mut visibility in &mut roots {
        visibility.set_if_neq(if open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// Rebuild both grids when the screen opens or either side changes.
#[allow(clippy::too_many_arguments)]
fn fill(
    mut commands: Commands,
    screens: Res<ScreenMode>,
    player: Res<PlayerInventory>,
    stores: Query<Ref<Store>>,
    grids: Query<(Entity, &Grid)>,
    catalog: Option<Res<RecipeCatalog>>,
    server: Option<Res<AssetServer>>,
    mut shown: Local<Option<Entity>>,
) {
    let Some(Screen::Storage(store)) = screens.open else {
        *shown = None;
        return;
    };
    let Ok(contents) = stores.get(store) else {
        return;
    };
    let opened = *shown != Some(store);
    *shown = Some(store);
    if !(opened || player.is_changed() || contents.is_changed()) {
        return;
    }
    let Some(catalog) = catalog else {
        return;
    };
    for (grid, Grid(direction)) in &grids {
        let inventory = match direction {
            StorageMove::Store => &player.items,
            StorageMove::Take => &contents.0,
        };
        let held: Vec<(String, u32)> = inventory
            .boxes()
            .map(|(item, count)| (item.to_owned(), count))
            .collect();
        commands.entity(grid).despawn_related::<Children>();
        commands.entity(grid).with_children(|grid| {
            for index in 0..inventory.box_count() as usize {
                let mut slot = grid.spawn((
                    BackgroundColor(TILE),
                    BorderColor::all(EDGE),
                    Node {
                        width: px(SLOT_PX),
                        height: px(SLOT_PX),
                        padding: UiRect::all(px(5)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                ));
                if let Some((item, count)) = held.get(index) {
                    let name = catalog
                        .0
                        .item(item)
                        .map_or(item.clone(), |i| i.name.clone());
                    slot.with_children(|slot| {
                        icon_slot(slot, server.as_deref(), item, &name, *count, SLOT_PX - 12.);
                    });
                    slot.insert(MoveBox {
                        item: item.clone(),
                        quantity: *count,
                        direction: *direction,
                    })
                    .observe(hover)
                    .observe(unhover)
                    .observe(move_box);
                }
            }
        });
    }
}

fn hover(over: On<PointerOver>, mut boxes: Query<&mut BorderColor, With<MoveBox>>) {
    if let Ok(mut border) = boxes.get_mut(over.entity) {
        *border = BorderColor::all(AMBER);
    }
}

fn unhover(out: On<PointerOut>, mut boxes: Query<&mut BorderColor, With<MoveBox>>) {
    if let Ok(mut border) = boxes.get_mut(out.entity) {
        *border = BorderColor::all(EDGE);
    }
}

fn move_box(
    click: On<PointerClick>,
    boxes: Query<&MoveBox>,
    screens: Res<ScreenMode>,
    stores: Query<&PlacedAsset, With<Store>>,
    mut requests: MessageWriter<StorageRequest>,
) {
    let (Ok(moved), Some(Screen::Storage(store))) = (boxes.get(click.entity), screens.open) else {
        return;
    };
    let Ok(placed) = stores.get(store) else {
        return;
    };
    requests.write(StorageRequest {
        store: placed.instance.clone(),
        item: moved.item.clone(),
        quantity: moved.quantity,
        direction: moved.direction,
    });
}
