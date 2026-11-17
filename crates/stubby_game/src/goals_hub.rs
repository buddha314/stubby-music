//! The goals hub, upper right, as Satisfactory's milestone tracker
//! (add-inventory-and-goals-hud 2.5, goals-hub spec): the active long-term and
//! short-term goals, each item they need as `paid + held / required` with a progress
//! bar, and the pinned recipes' totals (the To-Do List).
//!
//! The hub reads `Goals` and `PlayerInventory` from the crafting plugin and names items
//! from `ItemNames`; it never changes either. It is rebuilt only when what it shows
//! changes, never every frame, and hidden while a screen is open.
use bevy::prelude::*;
use stubby_core::{ItemNames, ScreenMode};
use stubby_recipes::{GoalProgress, Horizon, Inventory, ItemProgress, ProgressState, Rules};

use super::recipes_plugin::{Goals, PlayerInventory};

const PANEL: Color = Color::srgba(0.035, 0.033, 0.03, 0.82);
const EDGE: Color = Color::srgb(0.42, 0.29, 0.13);
const INK: Color = Color::srgb(0.95, 0.94, 0.89);
const MUTED: Color = Color::srgb(0.55, 0.53, 0.49);
const TRACK: Color = Color::srgba(1., 1., 1., 0.08);
/// The signal amber (style guide), the one colour for progress.
const AMBER: Color = Color::srgb(1., 0.7, 0.31);
const ICON: f32 = 28.;

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
    HubView { goals, pins }
}

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

fn spawn(mut commands: Commands) {
    commands.spawn((
        HubRoot,
        Node {
            position_type: PositionType::Absolute,
            // Under the calendar line.
            right: px(20),
            top: px(52),
            width: px(300),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(1)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(EDGE),
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
                hub.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|section| {
                    let label = match goal.horizon {
                        Horizon::Long => "LONG-TERM GOAL",
                        Horizon::Short => "GOAL",
                    };
                    section.spawn((text(label, 11., AMBER), Pickable::IGNORE));
                    section.spawn((text(goal.name.clone(), 17., INK), Pickable::IGNORE));
                    if !goal.description.is_empty() {
                        section
                            .spawn((text(goal.description.clone(), 13., MUTED), Pickable::IGNORE));
                    }
                    for row in &goal.items {
                        item_row(section, server, row);
                    }
                });
            }
            if !view.pins.is_empty() {
                hub.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|section| {
                    section.spawn((text("TO-DO", 11., AMBER), Pickable::IGNORE));
                    for row in &view.pins {
                        item_row(section, server, row);
                    }
                });
            }
        });
    *shown = Some(view);
}

/// One item: its icon, its name and `paid + held / required`, and the bar.
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
                margin: UiRect::top(px(2)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|line| {
            let mut icon = line.spawn((
                Node {
                    width: px(ICON),
                    height: px(ICON),
                    flex_shrink: 0.,
                    ..default()
                },
                Pickable::IGNORE,
            ));
            if let Some(server) = server {
                icon.insert(stubby_ui::item_icon(server, &row.item));
            }
            line.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.,
                    row_gap: px(3),
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
                    label.spawn((text(row.name.clone(), 14., INK), Pickable::IGNORE));
                    label.spawn((text(counts, 14., INK), Pickable::IGNORE));
                });
                body.spawn((
                    Node {
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            flex_grow: 1.,
                            height: px(5),
                            ..default()
                        },
                        BackgroundColor(TRACK),
                        Pickable::IGNORE,
                    ))
                    .with_children(|track| {
                        // Low reads as a dimmer amber, so a bar under half looks unready.
                        let alpha = match state {
                            ProgressState::Empty => 0.,
                            ProgressState::Low => 0.5,
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
                    tick(bar, state == ProgressState::Full);
                });
            });
        });
}

/// The tick beside a full bar: two strokes, drawn rather than a glyph, so no font needs
/// one. Its space is kept when hidden so the bars stay aligned.
fn tick(parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands, shown: bool) {
    let color = if shown { AMBER } else { Color::NONE };
    parent
        .spawn((
            Node {
                width: px(12),
                height: px(10),
                flex_shrink: 0.,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|tick| {
            for (left, top, width, angle) in [(0., 5., 5., 45.), (3., 3., 9., -50.)] {
                tick.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(left),
                        top: px(top),
                        width: px(width),
                        height: px(2),
                        ..default()
                    },
                    UiTransform {
                        rotation: Rot2::degrees(angle),
                        ..default()
                    },
                    BackgroundColor(color),
                    Pickable::IGNORE,
                ));
            }
        });
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
