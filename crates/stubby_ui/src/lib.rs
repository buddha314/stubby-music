//! Shared HUD widgets (a service crate, split-game-into-systems): Satisfactory-style item
//! icons and slots, and the tooltip that names an item under the pointer. Any system's
//! screen (the inventory, a rack) uses the same ones, so items look alike everywhere.
use bevy::prelude::*;

use style::{EDGE, INK, MUTED, PANEL};

/// The HUD's one look (docs/art/style-guide.md, "HUD"): dark warm panels with a thin
/// bronze edge, bone-white ink, muted grey for secondary text, and the signal amber as the
/// only accent. Keys are bone keycaps with dark letters, as on the rack screen. Every
/// screen and HUD element takes its colours from here.
pub mod style {
    use bevy::prelude::*;

    pub const PANEL: Color = Color::srgba(0.035, 0.033, 0.03, 0.94);
    /// A see-through band over the world (the goals hub, the key hints).
    pub const BAND: Color = Color::srgba(0.035, 0.033, 0.03, 0.62);
    pub const BACKDROP: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);
    pub const TILE: Color = Color::srgba(0.09, 0.085, 0.075, 1.0);
    pub const EDGE: Color = Color::srgb(0.42, 0.29, 0.13);
    pub const INK: Color = Color::srgb(0.95, 0.94, 0.89);
    pub const MUTED: Color = Color::srgb(0.62, 0.6, 0.55);
    /// The signal amber (`#ffb350`): the one accent.
    pub const AMBER: Color = Color::srgb(1., 0.7, 0.31);
    pub const WARNING: Color = Color::srgb(0.86, 0.24, 0.18);
    pub const KEY: Color = Color::srgb(0.85, 0.84, 0.8);
    pub const KEY_INK: Color = Color::srgb(0.1, 0.1, 0.1);

    /// Text in the HUD's face. It never catches the pointer.
    pub fn label(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
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

    /// Text drawn straight over the world: a soft shadow keeps it legible on bright sky.
    pub fn world_label(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
        (
            label(value, size, color),
            TextShadow {
                offset: Vec2::splat(1.5),
                color: Color::srgba(0., 0., 0., 0.85),
            },
        )
    }

    /// A key, as a bone keycap with a dark letter (Satisfactory's prompts).
    pub fn keycap(parent: &mut ChildSpawnerCommands, key: &str, size: f32) {
        parent
            .spawn((
                Node {
                    padding: UiRect::axes(px(size * 0.5), px(size * 0.2)),
                    min_width: px(size * 1.6),
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(KEY),
                Pickable::IGNORE,
            ))
            .with_child(label(key, size, KEY_INK));
    }

    /// How a key is written on a keycap.
    pub fn key_name(key: KeyCode) -> String {
        match key {
            KeyCode::Escape => "Esc".into(),
            KeyCode::Tab => "Tab".into(),
            KeyCode::Space => "Space".into(),
            KeyCode::BracketLeft => "[".into(),
            KeyCode::BracketRight => "]".into(),
            KeyCode::Backslash => "\\".into(),
            other => {
                let name = format!("{other:?}");
                name.strip_prefix("Key")
                    .or_else(|| name.strip_prefix("Digit"))
                    .unwrap_or(&name)
                    .to_owned()
            }
        }
    }
}

/// A progress bar in the HUD style (add-crafting-durations 2.1): a dark track holding an
/// amber fill. `fill` goes on the fill, whose width [`show_progress`] sets.
pub fn progress_bar(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    width: f32,
    fill: impl Bundle,
) {
    parent
        .spawn((
            Node {
                width: px(width),
                height: px(10),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(style::TILE),
            BorderColor::all(style::EDGE),
            Pickable::IGNORE,
        ))
        .with_children(|track| {
            track.spawn((
                fill,
                Node {
                    width: percent(0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(style::AMBER),
                Pickable::IGNORE,
            ));
        });
}

/// Show `done` of `total` on a progress bar's fill, in whole `step`s when given (a hand
/// craft fills in quarter seconds).
pub fn show_progress(fill: &mut Node, done: f32, total: f32, step: Option<f32>) {
    let done = step.map_or(done, |step| (done / step).floor() * step);
    let fraction = if total > 0. {
        (done / total).clamp(0., 1.)
    } else {
        0.
    };
    let width = percent(fraction * 100.);
    if fill.width != width {
        fill.width = width;
    }
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

/// Shows an [`ItemName`] tooltip while the pointer is over a tagged icon.
pub struct ItemTooltipPlugin;
impl Plugin for ItemTooltipPlugin {
    /// Every screen that shows items adds it (the inventory, the rack, streams); the
    /// tooltip observers are installed once.
    fn is_unique(&self) -> bool {
        false
    }
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
