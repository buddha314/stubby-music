//! The crafting screen (add-campfire-craft-bench 3.3, 2b.3; design D4): Satisfactory's
//! Craft Bench. E at a burning campfire opens it (`stubby_core::CraftBench`): a framed
//! window with a title bar (the bench and the tempo it hears), the recipes on the left,
//! and on the right the selected one: its tempo range against the bench's, each input as
//! `have / need` (amber when enough), the output, a craft button (click makes one, holding
//! repeats, `x N` counts the hold) and a pin for the goals hub. A Rack tab opens the
//! bench's rack, so the music can still be changed; E or Escape closes it. The screen only
//! sends `CraftRequest` with the bench's tempo: the rules live in `stubby_recipes`.
use bevy::picking::events::{PointerClick, PointerOut, PointerOver, PointerPress};
use bevy::prelude::*;
use stubby_core::{AssetClock, PlacedAsset, Screen, ScreenMode};
use stubby_ui::style::{AMBER, BACKDROP, EDGE, INK, MUTED, PANEL, TILE, WARNING};

use super::recipes_plugin::{
    CraftCompleted, CraftRequest, GoalRequest, PlayerInventory, RecipeCatalog,
};

/// Holding the craft button repeats it: after this long, then at this interval.
const REPEAT_AFTER_S: f32 = 0.4;
const REPEAT_EVERY_S: f32 = 0.2;

pub struct CraftingScreenPlugin;
impl Plugin for CraftingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Crafting>()
            .add_systems(Update, (count_crafts, hold_to_craft, rebuild).chain());
    }
}

/// What the screen shows and does: the selected recipe, the filter, a held craft button.
#[derive(Resource, Default)]
struct Crafting {
    selected: Option<String>,
    only_makeable: bool,
    /// Seconds the craft button has been held, while it is.
    held: Option<f32>,
    /// Crafts made in this hold (`x N`).
    made: u32,
}

#[derive(Component)]
struct CraftRoot;
#[derive(Component)]
struct RecipeButton(String);
#[derive(Component)]
struct CraftButton;
#[derive(Component)]
struct PinButton;
#[derive(Component)]
struct RackTab(Entity);
#[derive(Component)]
struct FilterToggle;

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    stubby_ui::style::label(value, size, color)
}

/// The bench open on the screen, if any.
fn bench(screens: &ScreenMode) -> Option<Entity> {
    match screens.open {
        Some(Screen::Crafting(bench)) => Some(bench),
        _ => None,
    }
}

/// The tempo the bench hears: its lent clock's, if it has one.
fn bench_bpm(clocks: &Query<&AssetClock>, bench: Entity) -> Option<f32> {
    clocks.get(bench).ok().map(|clock| clock.bpm)
}

fn makeable(
    recipe: &stubby_recipes::RecipeDefinition,
    player: &PlayerInventory,
    bpm: Option<f32>,
) -> bool {
    recipe
        .inputs
        .iter()
        .all(|stack| player.items.quantity(&stack.item) >= stack.quantity)
        && recipe
            .bpm
            .is_none_or(|range| bpm.is_some_and(|bpm| range.contains(bpm)))
}

fn send_craft(
    crafting: &Crafting,
    bench: Entity,
    clocks: &Query<&AssetClock>,
    crafts: &mut MessageWriter<CraftRequest>,
) {
    if let Some(recipe) = &crafting.selected {
        crafts.write(CraftRequest {
            recipe: recipe.clone(),
            bpm: bench_bpm(clocks, bench),
        });
    }
}

/// `x N`: crafts of the selected recipe made during this hold.
fn count_crafts(mut crafting: ResMut<Crafting>, mut done: MessageReader<CraftCompleted>) {
    for craft in done.read() {
        if crafting.held.is_some() && crafting.selected.as_deref() == Some(craft.recipe.as_str()) {
            crafting.made += 1;
        }
    }
}

/// Holding the craft button crafts again after a moment, then steadily; letting go stops.
fn hold_to_craft(
    time: Res<Time<Real>>,
    buttons: Res<ButtonInput<MouseButton>>,
    screens: Res<ScreenMode>,
    clocks: Query<&AssetClock>,
    mut crafting: ResMut<Crafting>,
    mut crafts: MessageWriter<CraftRequest>,
) {
    let Some(held) = crafting.held else {
        return;
    };
    let Some(bench) = bench(&screens).filter(|_| buttons.pressed(MouseButton::Left)) else {
        crafting.held = None;
        return;
    };
    let now = held + time.delta_secs();
    let repeats = |t: f32| ((t - REPEAT_AFTER_S) / REPEAT_EVERY_S).floor().max(-1.) as i32;
    if now >= REPEAT_AFTER_S && repeats(now) > repeats(held) {
        send_craft(&crafting, bench, &clocks, &mut crafts);
    }
    crafting.held = Some(now);
}

/// The window is rebuilt when what it shows changes: the bench, its tempo, the
/// inventory, the selection or the filter.
#[allow(clippy::too_many_arguments)] // A screen reads the rules, the stock and the bench.
fn rebuild(
    mut commands: Commands,
    screens: Res<ScreenMode>,
    crafting: Res<Crafting>,
    player: Res<PlayerInventory>,
    catalog: Option<Res<RecipeCatalog>>,
    names: Option<Res<stubby_core::ItemNames>>,
    server: Option<Res<AssetServer>>,
    clocks: Query<&AssetClock>,
    placed: Query<&PlacedAsset>,
    roots: Query<Entity, With<CraftRoot>>,
    mut shown: Local<Option<String>>,
) {
    let open = bench(&screens);
    // What the window shows, in short: rebuilt only when this changes (the inventory and
    // the hold are touched every frame while crafting, which alone changes nothing shown).
    let summary = open.map(|bench| {
        let tempo = bench_bpm(&clocks, bench).map(|bpm| bpm.round() as u32);
        let held: Vec<u32> = catalog
            .iter()
            .flat_map(|c| {
                c.0.recipes()
                    .flat_map(|r| r.inputs.iter().chain(&r.outputs))
            })
            .map(|stack| player.items.quantity(&stack.item))
            .collect();
        format!(
            "{bench:?} {tempo:?} {:?} {} {} {held:?}",
            crafting.selected, crafting.only_makeable, crafting.made
        )
    });
    if summary == *shown {
        return;
    }
    *shown = summary;
    roots
        .iter()
        .for_each(|root| commands.entity(root).despawn());
    let (Some(bench), Some(catalog)) = (open, catalog) else {
        return;
    };
    let bpm = bench_bpm(&clocks, bench);
    let name = |item: &str| {
        names
            .as_ref()
            .map_or_else(|| item.replace('_', " "), |names| names.name(item))
    };
    let title = placed
        .get(bench)
        .ok()
        .and_then(|asset| catalog.0.buildable(&asset.asset))
        .map_or_else(|| "Bench".to_owned(), |b| b.name.clone());
    let recipes: Vec<_> = catalog
        .0
        .recipes()
        .filter(|r| !crafting.only_makeable || makeable(r, &player, bpm))
        .cloned()
        .collect();
    let selected = crafting
        .selected
        .as_ref()
        .and_then(|id| catalog.0.recipe(id).cloned());
    commands
        .spawn((
            CraftRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BACKDROP),
            GlobalZIndex(140),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(14),
                    padding: UiRect::all(px(20)),
                    border: UiRect::all(px(1)),
                    width: px(860),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(EDGE),
            ))
            .with_children(|window| {
                // Title bar: the bench, the tempo it hears, and its two tabs.
                window
                    .spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|bar| {
                        bar.spawn(text(title.to_uppercase(), 26., INK));
                        bar.spawn(Node {
                            column_gap: px(18),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|right| {
                            right.spawn(text(
                                bpm.map_or("No clock in reach".to_owned(), |bpm| {
                                    format!("{} BPM", bpm.round())
                                }),
                                16.,
                                if bpm.is_some() { AMBER } else { MUTED },
                            ));
                            tab(right, "CRAFT", true, None);
                            tab(right, "RACK", false, Some(RackTab(bench)));
                        });
                    });
                window
                    .spawn(Node {
                        column_gap: px(18),
                        ..default()
                    })
                    .with_children(|body| {
                        // Left: the recipes.
                        body.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            width: px(300),
                            ..default()
                        })
                        .with_children(|list| {
                            list.spawn((FilterToggle, Node::default()))
                                .with_children(|toggle| {
                                    toggle.spawn(text(
                                        if crafting.only_makeable {
                                            "Showing: makeable now"
                                        } else {
                                            "Showing: all recipes"
                                        },
                                        13.,
                                        MUTED,
                                    ));
                                })
                                .observe(toggle_filter);
                            if recipes.is_empty() {
                                list.spawn(text("Nothing to make yet.", 14., MUTED));
                            }
                            for recipe in &recipes {
                                let chosen =
                                    crafting.selected.as_deref() == Some(recipe.id.as_str());
                                let can = makeable(recipe, &player, bpm);
                                list.spawn((
                                    RecipeButton(recipe.id.clone()),
                                    Node {
                                        column_gap: px(10),
                                        align_items: AlignItems::Center,
                                        padding: UiRect::all(px(6)),
                                        border: UiRect::all(px(1)),
                                        ..default()
                                    },
                                    BackgroundColor(TILE),
                                    BorderColor::all(if chosen { AMBER } else { EDGE }),
                                ))
                                .with_children(|row| {
                                    if let (Some(server), Some(out)) =
                                        (server.as_ref(), recipe.outputs.first())
                                    {
                                        row.spawn((
                                            Node {
                                                width: px(40),
                                                height: px(40),
                                                ..default()
                                            },
                                            stubby_ui::item_icon(server, &out.item),
                                            Pickable::IGNORE,
                                        ));
                                    }
                                    row.spawn(text(
                                        recipe.name.clone(),
                                        16.,
                                        if can { INK } else { MUTED },
                                    ));
                                })
                                .observe(choose)
                                .observe(hover)
                                .observe(unhover);
                            }
                        });
                        // Right: the selected recipe.
                        body.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(14),
                            flex_grow: 1.,
                            ..default()
                        })
                        .with_children(|detail| {
                            let Some(recipe) = &selected else {
                                detail.spawn(text("Choose a recipe.", 15., MUTED));
                                return;
                            };
                            detail.spawn(text(recipe.name.clone(), 22., INK));
                            if let Some(range) = recipe.bpm {
                                let in_range = bpm.is_some_and(|bpm| range.contains(bpm));
                                detail.spawn(text(
                                    format!("Made at {} to {} BPM", range.min, range.max),
                                    15.,
                                    if in_range { AMBER } else { WARNING },
                                ));
                            }
                            detail
                                .spawn(Node {
                                    column_gap: px(12),
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|row| {
                                    for stack in &recipe.inputs {
                                        let have = player.items.quantity(&stack.item);
                                        ingredient(
                                            row,
                                            server.as_deref(),
                                            &stack.item,
                                            &name(&stack.item),
                                            format!("{have} / {}", stack.quantity),
                                            if have >= stack.quantity {
                                                AMBER
                                            } else {
                                                WARNING
                                            },
                                        );
                                    }
                                    row.spawn(text(">", 28., MUTED));
                                    for stack in &recipe.outputs {
                                        ingredient(
                                            row,
                                            server.as_deref(),
                                            &stack.item,
                                            &name(&stack.item),
                                            format!("x {}", stack.quantity),
                                            INK,
                                        );
                                    }
                                });
                            detail
                                .spawn(Node {
                                    column_gap: px(12),
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|row| {
                                    let can = makeable(recipe, &player, bpm);
                                    row.spawn((
                                        CraftButton,
                                        Node {
                                            padding: UiRect::axes(px(26), px(10)),
                                            border: UiRect::all(px(1)),
                                            ..default()
                                        },
                                        BackgroundColor(TILE),
                                        BorderColor::all(if can { AMBER } else { EDGE }),
                                    ))
                                    .with_children(|button| {
                                        button.spawn(text(
                                            "CRAFT",
                                            18.,
                                            if can { AMBER } else { MUTED },
                                        ));
                                    })
                                    .observe(press_craft);
                                    if crafting.made > 0 {
                                        row.spawn(text(format!("x {}", crafting.made), 18., AMBER));
                                    }
                                    row.spawn((
                                        PinButton,
                                        Node {
                                            padding: UiRect::axes(px(14), px(10)),
                                            border: UiRect::all(px(1)),
                                            ..default()
                                        },
                                        BackgroundColor(TILE),
                                        BorderColor::all(EDGE),
                                    ))
                                    .with_children(|button| {
                                        button.spawn(text("PIN", 14., MUTED));
                                    })
                                    .observe(pin)
                                    .observe(hover)
                                    .observe(unhover);
                                });
                        });
                    });
                window.spawn(text(
                    "Hold CRAFT to keep making     E or Esc  close",
                    13.,
                    MUTED,
                ));
            });
        });
}

fn tab(parent: &mut ChildSpawnerCommands, label: &str, active: bool, rack: Option<RackTab>) {
    let mut tab = parent.spawn((
        Node {
            padding: UiRect::axes(px(12), px(4)),
            border: UiRect::bottom(px(2)),
            ..default()
        },
        BorderColor::all(if active { AMBER } else { EDGE }),
    ));
    tab.with_children(|tab| {
        tab.spawn(text(label, 14., if active { INK } else { MUTED }));
    });
    if let Some(rack) = rack {
        tab.insert(rack)
            .observe(open_rack)
            .observe(hover)
            .observe(unhover);
    }
}

fn ingredient(
    parent: &mut ChildSpawnerCommands,
    server: Option<&AssetServer>,
    item: &str,
    name: &str,
    caption: String,
    color: Color,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        })
        .with_children(|tile| {
            let mut icon = tile.spawn((
                Node {
                    width: px(72),
                    height: px(72),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(TILE),
                BorderColor::all(EDGE),
                stubby_ui::ItemName(name.into()),
            ));
            if let Some(server) = server {
                icon.insert(stubby_ui::item_icon(server, item));
            }
            tile.spawn(text(caption, 14., color));
        });
}

fn hover(over: On<PointerOver>, mut borders: Query<&mut BorderColor, Without<RecipeButton>>) {
    if let Ok(mut border) = borders.get_mut(over.entity) {
        *border = BorderColor::all(AMBER);
    }
}

fn unhover(out: On<PointerOut>, mut borders: Query<&mut BorderColor, Without<RecipeButton>>) {
    if let Ok(mut border) = borders.get_mut(out.entity) {
        *border = BorderColor::all(EDGE);
    }
}

fn choose(click: On<PointerClick>, buttons: Query<&RecipeButton>, mut crafting: ResMut<Crafting>) {
    if let Ok(RecipeButton(id)) = buttons.get(click.entity) {
        crafting.selected = Some(id.clone());
        crafting.made = 0;
    }
}

fn toggle_filter(_: On<PointerClick>, mut crafting: ResMut<Crafting>) {
    crafting.only_makeable = !crafting.only_makeable;
}

/// Pressing CRAFT makes one now; holding it repeats (`hold_to_craft`).
fn press_craft(
    _: On<PointerPress>,
    screens: Res<ScreenMode>,
    clocks: Query<&AssetClock>,
    mut crafting: ResMut<Crafting>,
    mut crafts: MessageWriter<CraftRequest>,
) {
    let Some(bench) = bench(&screens) else {
        return;
    };
    crafting.made = 0;
    crafting.held = Some(0.);
    send_craft(&crafting, bench, &clocks, &mut crafts);
}

fn pin(_: On<PointerClick>, crafting: Res<Crafting>, mut goals: MessageWriter<GoalRequest>) {
    if let Some(recipe) = &crafting.selected {
        goals.write(GoalRequest(stubby_recipes::GoalEvent::Pin {
            recipe: recipe.clone(),
            count: 1,
        }));
    }
}

fn open_rack(click: On<PointerClick>, tabs: Query<&RackTab>, mut screens: ResMut<ScreenMode>) {
    if let Ok(RackTab(bench)) = tabs.get(click.entity) {
        screens.open = Some(Screen::MachineConfiguration(*bench));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A recipe is makeable with its inputs held and the bench's tempo in its range.
    #[test]
    fn makeable_needs_the_inputs_and_the_tempo() {
        let catalog = stubby_recipes::shipped_catalog().unwrap();
        let recipe = catalog.recipe("craft_reed_water_purifier").unwrap().clone();
        let mut player = PlayerInventory::default();
        assert!(!makeable(&recipe, &player, Some(120.)));
        player.items.grant(&recipe.inputs).unwrap();
        assert!(makeable(&recipe, &player, Some(120.)));
        assert!(!makeable(&recipe, &player, Some(72.)), "off tempo");
        assert!(!makeable(&recipe, &player, None), "no clock in reach");
    }
}
