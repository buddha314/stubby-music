//! The goals hub, upper right (add-inventory-and-goals-hud 2.5, goals-hub spec).
//!
//! Laid out as Satisfactory's HUD, in our colours: the goal tracker is a translucent
//! band per goal (Satisfactory's `Objective:` phase above its `Milestone:`), a centred
//! `Label: Name` header over a thin rule; under it, each item the goal needs as a
//! To-Do List row: name and `paid + held / required` over a thin bar, the item's tile at
//! the right end, turned into a tick when the item is covered. The pinned recipes are
//! the To-Do List below it: a header strip, each recipe with its count, a rule, and the
//! same item rows for their totals.
//!
//! The hub reads `Goals` and `PlayerInventory` from the crafting plugin and names items
//! from `ItemNames`; it never changes either. It is rebuilt only when what it shows
//! changes, never every frame, and hidden while a screen is open.
use bevy::prelude::*;
use stubby_core::{ItemNames, ScreenMode};
use stubby_recipes::{GoalProgress, Horizon, Inventory, ItemProgress, ProgressState, Rules};

use super::recipes_plugin::{Goals, PlayerInventory};

/// Satisfactory's tracker is a see-through band, not a framed window.
const BAND: Color = Color::srgba(0.035, 0.033, 0.03, 0.55);
const STRIP: Color = Color::srgba(0.035, 0.033, 0.03, 0.85);
const TILE: Color = Color::srgba(0.09, 0.085, 0.075, 0.9);
const INK: Color = Color::srgb(0.95, 0.94, 0.89);
const MUTED: Color = Color::srgb(0.7, 0.68, 0.63);
const TRACK: Color = Color::srgba(1., 1., 1., 0.14);
/// The signal amber (style guide), the one accent: labels, rules and progress.
const AMBER: Color = Color::srgb(1., 0.7, 0.31);
const WIDTH: f32 = 320.;
const TILE_SIZE: f32 = 34.;

pub struct GoalsHubPlugin;
impl Plugin for GoalsHubPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (show, fill).chain());
    }
}

#[derive(Component)]
struct HubRoot;

/// What the hub shows. The hub is rebuilt when this changes.
#[derive(Debug, Default, PartialEq)]
struct HubView {
    goals: Vec<GoalView>,
    /// Pinned recipes by name, with their counts.
    pinned: Vec<(String, u32)>,
    /// The pins' ingredient totals.
    pins: Vec<ItemRow>,
}

#[derive(Debug, PartialEq)]
struct GoalView {
    horizon: Horizon,
    name: String,
    description: String,
    /// Empty for a goal met by a condition (a chore).
    items: Vec<ItemRow>,
}

#[derive(Debug, PartialEq)]
struct ItemRow {
    item: String,
    name: String,
    progress: ItemProgress,
}

fn hub_view(
    rules: &Rules,
    progress: &GoalProgress,
    inventory: &Inventory,
    names: &ItemNames,
) -> HubView {
    let row = |item: &str, progress: ItemProgress| ItemRow {
        item: item.into(),
        name: names.name(item),
        progress,
    };
    let goals = [Horizon::Long, Horizon::Short]
        .into_iter()
        .filter_map(|horizon| progress.active_goal(rules, horizon))
        .map(|goal| {
            let mut items: Vec<&str> = Vec::new();
            for stack in &goal.requires {
                if !items.contains(&stack.item.as_str()) {
                    items.push(&stack.item);
                }
            }
            GoalView {
                horizon: goal.horizon,
                name: goal.name.clone(),
                description: goal.description.clone(),
                items: items
                    .into_iter()
                    .filter_map(|item| {
                        Some(row(item, progress.item_progress(goal, item, inventory)?))
                    })
                    .collect(),
            }
        })
        .collect();
    let pinned = progress
        .pins()
        .iter()
        .map(|(recipe, count)| {
            let name = rules
                .definition
                .recipes
                .iter()
                .find(|candidate| candidate.id == recipe)
                .map_or_else(|| recipe.to_owned(), |recipe| recipe.name.clone());
            (name, count)
        })
        .collect();
    let pins = progress
        .pins()
        .totals(rules)
        .unwrap_or_default()
        .into_iter()
        .map(|stack| {
            row(
                &stack.item,
                ItemProgress::new(0, inventory.quantity(&stack.item), stack.quantity),
            )
        })
        .collect();
    HubView {
        goals,
        pinned,
        pins,
    }
}

fn font(size: f32) -> TextFont {
    TextFont {
        font_size: bevy::text::FontSize::Px(size),
        ..default()
    }
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        font(size),
        TextColor(color),
        Pickable::IGNORE,
    )
}

fn column(gap: f32) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(gap),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// A thin full-width rule under a header.
fn rule(parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands, color: Color) {
    parent.spawn((
        Node {
            height: px(1),
            ..default()
        },
        BackgroundColor(color),
        Pickable::IGNORE,
    ));
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        HubRoot,
        Node {
            position_type: PositionType::Absolute,
            // Under the calendar line.
            right: px(20),
            top: px(52),
            width: px(WIDTH),
            flex_direction: FlexDirection::Column,
            row_gap: px(16),
            display: Display::None,
            ..default()
        },
        Pickable::IGNORE,
    ));
}

fn show(screens: Option<Res<ScreenMode>>, mut roots: Query<&mut Visibility, With<HubRoot>>) {
    let hidden = screens.is_some_and(|screens| screens.open.is_some());
    for mut visibility in &mut roots {
        visibility.set_if_neq(if hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
}

fn fill(
    mut commands: Commands,
    goals: Option<Res<Goals>>,
    player: Option<Res<PlayerInventory>>,
    names: Option<Res<ItemNames>>,
    server: Option<Res<AssetServer>>,
    mut shown: Local<Option<HubView>>,
    mut roots: Query<(Entity, &mut Node), With<HubRoot>>,
) {
    let (Some(goals), Some(player)) = (goals, player) else {
        return;
    };
    let names_changed = names.as_ref().is_some_and(|names| names.is_changed());
    if shown.is_some() && !goals.is_changed() && !player.is_changed() && !names_changed {
        return;
    }
    let empty = ItemNames::default();
    let names = names.as_deref().unwrap_or(&empty);
    let view = hub_view(&goals.rules, &goals.progress, &player.items, names);
    if shown.as_ref() == Some(&view) {
        return;
    }
    let Ok((root, mut node)) = roots.single_mut() else {
        return;
    };
    // After the last authored goal and with nothing pinned the hub goes away; play goes on.
    node.display = if view.goals.is_empty() && view.pins.is_empty() {
        Display::None
    } else {
        Display::Flex
    };
    let server = server.as_deref();
    commands
        .entity(root)
        .despawn_related::<Children>()
        .with_children(|hub| {
            for goal in &view.goals {
                goal_band(hub, server, goal);
            }
            if !view.pins.is_empty() {
                to_do_list(hub, server, &view);
            }
        });
    *shown = Some(view);
}

/// One goal: `Label: Name` centred over a rule, then its item rows. A chore (a goal met
/// by a condition) has no items, so its one line of description says what to do.
fn goal_band(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    server: Option<&AssetServer>,
    goal: &GoalView,
) {
    let label = match goal.horizon {
        Horizon::Long => "Long-term goal: ",
        Horizon::Short => "Goal: ",
    };
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::axes(px(10), px(8)),
                ..default()
            },
            BackgroundColor(BAND),
            Pickable::IGNORE,
        ))
        .with_children(|band| {
            band.spawn((
                Text::new(label),
                font(14.),
                TextColor(AMBER),
                TextLayout::justify(Justify::Center),
                Node {
                    align_self: AlignSelf::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_child((TextSpan::new(goal.name.clone()), font(15.), TextColor(INK)));
            rule(band, AMBER.with_alpha(0.8));
            if goal.items.is_empty() && !goal.description.is_empty() {
                band.spawn((
                    text(goal.description.clone(), 13., MUTED),
                    TextLayout::justify(Justify::Center),
                ));
            }
            for row in &goal.items {
                item_row(band, server, row);
            }
        });
}

/// The pins, as Satisfactory's To-Do List: a header strip, each pinned recipe and its
/// count, a rule, then the totals as item rows.
fn to_do_list(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    server: Option<&AssetServer>,
    view: &HubView,
) {
    parent
        .spawn((column(0.), BackgroundColor(BAND)))
        .with_children(|list| {
            list.spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    padding: UiRect::vertical(px(5)),
                    ..default()
                },
                BackgroundColor(STRIP),
                Pickable::IGNORE,
            ))
            .with_child(text("TO-DO LIST", 13., INK));
            list.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    padding: UiRect::axes(px(10), px(8)),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                for (name, count) in &view.pinned {
                    body.spawn((
                        Node {
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|line| {
                        line.spawn(text(name.clone(), 15., INK));
                        line.spawn(text(count.to_string(), 15., INK));
                    });
                }
                rule(body, INK.with_alpha(0.6));
                for row in &view.pins {
                    item_row(body, server, row);
                }
            });
        });
}

/// One item, as a To-Do List row: name and counts over a thin bar, the item's tile at
/// the right end, a tick tile once paid plus held covers it.
fn item_row(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    server: Option<&AssetServer>,
    row: &ItemRow,
) {
    let ItemProgress {
        paid,
        held,
        required,
        state,
    } = row.progress;
    let counts = if paid > 0 {
        format!("{paid} paid + {held} / {required}")
    } else {
        format!("{held} / {required}")
    };
    let filled = ((u64::from(paid) + u64::from(held)) as f32 / required.max(1) as f32).min(1.);
    parent
        .spawn((
            Node {
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|line| {
            line.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.,
                    row_gap: px(4),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                body.spawn((
                    Node {
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|label| {
                    label.spawn(text(row.name.clone(), 14., INK));
                    label.spawn(text(counts, 14., INK));
                });
                body.spawn((
                    Node {
                        height: px(4),
                        ..default()
                    },
                    BackgroundColor(TRACK),
                    Pickable::IGNORE,
                ))
                .with_children(|track| {
                    // Under half reads as a paler amber, as Satisfactory's white-then-orange.
                    let alpha = match state {
                        ProgressState::Empty => 0.,
                        ProgressState::Low => 0.45,
                        ProgressState::High | ProgressState::Full => 1.,
                    };
                    track.spawn((
                        Node {
                            width: percent(filled * 100.),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(AMBER.with_alpha(alpha)),
                        Pickable::IGNORE,
                    ));
                });
            });
            item_tile(line, server, &row.item, state == ProgressState::Full);
        });
}

/// The item's tile at a row's right end; a tick on amber once the item is covered.
fn item_tile(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    server: Option<&AssetServer>,
    item: &str,
    done: bool,
) {
    let mut tile = parent.spawn((
        Node {
            width: px(TILE_SIZE),
            height: px(TILE_SIZE),
            flex_shrink: 0.,
            padding: UiRect::all(px(3)),
            ..default()
        },
        BackgroundColor(if done { AMBER } else { TILE }),
        Pickable::IGNORE,
    ));
    if done {
        // Two strokes, drawn rather than a glyph, so no font needs one.
        tile.with_children(|tick| {
            for (left, top, width, angle) in [(7., 18., 10., 45.), (12., 14., 18., -50.)] {
                tick.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(left),
                        top: px(top),
                        width: px(width),
                        height: px(3),
                        ..default()
                    },
                    UiTransform {
                        rotation: Rot2::degrees(angle),
                        ..default()
                    },
                    BackgroundColor(INK),
                    Pickable::IGNORE,
                ));
            }
        });
    } else if let Some(server) = server {
        tile.with_child((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            stubby_ui::item_icon(server, item),
            Pickable::IGNORE,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stubby_recipes::{GoalDefinition, Stack};

    fn rules() -> Rules {
        let mut rules = stubby_recipes::shipped_rules().unwrap();
        rules.goals.insert(
            0,
            GoalDefinition {
                id: "hand_in".into(),
                name: "Hand in".into(),
                description: String::new(),
                horizon: Horizon::Long,
                requires: vec![Stack {
                    item: "square_wave".into(),
                    quantity: 4,
                }],
                condition: None,
                after: vec![],
                unlocks: vec![],
            },
        );
        rules
    }

    #[test]
    fn the_hub_shows_the_active_goals_and_their_progress() {
        let rules = rules();
        let mut inventory = Inventory::default();
        inventory
            .grant(&[Stack {
                item: "square_wave".into(),
                quantity: 1,
            }])
            .unwrap();
        let view = hub_view(
            &rules,
            &GoalProgress::default(),
            &inventory,
            &ItemNames::default(),
        );
        let names: Vec<_> = view.goals.iter().map(|g| g.name.as_str()).collect();
        // Long first; "Water the tree" is not open yet, so it is not shown.
        assert_eq!(names, ["Hand in", "Build a heartbeat"]);
        let row = &view.goals[0].items[0];
        assert_eq!((row.progress.held, row.progress.required), (1, 4));
        assert_eq!(row.progress.state, ProgressState::Low);
        assert!(view.goals[1].items.is_empty());
    }
}
