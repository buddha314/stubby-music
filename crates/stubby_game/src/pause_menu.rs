//! The pause menu, Satisfactory's escape menu (user, 2026-11-17): in the world, the
//! first Escape frees the cursor and a second opens it (`controls::toggle_screens`). The
//! world dims behind it, and a column on the left offers Resume Game and Exit to Desktop
//! Settings and Exit to Main Menu (add-multiplayer-and-menus). Escape or Resume Game
//! closes it.
//! The world keeps running behind it, as Satisfactory's does in multiplayer.
use bevy::picking::events::{PointerClick, PointerOut, PointerOver};
use bevy::prelude::*;
use stubby_core::{Screen, ScreenMode};
use stubby_ui::style::{AMBER, BACKDROP, EDGE, INK, MUTED, PANEL};

const BUTTON: Color = Color::srgba(0.09, 0.085, 0.075, 1.0);

pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn).add_systems(Update, show);
    }
}

#[derive(Component)]
struct PauseRoot;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Choice {
    Resume,
    Save,
    Settings,
    ExitToMainMenu,
    ExitToDesktop,
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

fn spawn(mut commands: Commands) {
    commands
        .spawn((
            PauseRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                padding: UiRect::left(px(96)),
                ..default()
            },
            BackgroundColor(BACKDROP),
            GlobalZIndex(150),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    width: px(340),
                    padding: UiRect::all(px(20)),
                    border: UiRect::left(px(3)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(AMBER),
            ))
            .with_children(|panel| {
                panel.spawn(text("PAUSED", 30., INK));
                panel.spawn(text("The world keeps time while you are away.", 14., MUTED));
                panel.spawn((
                    Node {
                        height: px(1),
                        margin: UiRect::vertical(px(8)),
                        ..default()
                    },
                    BackgroundColor(EDGE),
                ));
                for (choice, label) in [
                    (Choice::Resume, "Resume Game"),
                    (Choice::Save, "Save Game"),
                    (Choice::Settings, "Settings"),
                    (Choice::ExitToMainMenu, "Exit to Main Menu"),
                    (Choice::ExitToDesktop, "Exit to Desktop"),
                ] {
                    panel
                        .spawn((
                            choice,
                            Node {
                                padding: UiRect::axes(px(16), px(12)),
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BackgroundColor(BUTTON),
                            BorderColor::all(EDGE),
                        ))
                        .with_child(text(label, 20., INK))
                        .observe(hover)
                        .observe(unhover)
                        .observe(choose);
                }
            });
        });
}

fn show(screens: Res<ScreenMode>, mut roots: Query<&mut Visibility, With<PauseRoot>>) {
    let open = screens.open == Some(Screen::Pause);
    for mut visibility in &mut roots {
        visibility.set_if_neq(if open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

fn hover(over: On<PointerOver>, mut buttons: Query<&mut BorderColor, With<Choice>>) {
    if let Ok(mut border) = buttons.get_mut(over.entity) {
        *border = BorderColor::all(AMBER);
    }
}

fn unhover(out: On<PointerOut>, mut buttons: Query<&mut BorderColor, With<Choice>>) {
    if let Ok(mut border) = buttons.get_mut(out.entity) {
        *border = BorderColor::all(EDGE);
    }
}

fn choose(
    click: On<PointerClick>,
    choices: Query<&Choice>,
    mut screens: ResMut<ScreenMode>,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
) {
    if screens.open != Some(Screen::Pause) {
        return;
    }
    match choices.get(click.entity) {
        Ok(Choice::Resume) => screens.close(),
        Ok(Choice::Save) => commands.queue(
            |world: &mut World| match crate::saves::write_quicksave(world) {
                Ok(path) => info!("Saved the game to {}", path.display()),
                Err(error) => error!("Could not save the game: {error}"),
            },
        ),
        Ok(Choice::Settings) => {
            crate::menus::open_settings(&mut screens, Screen::Pause, &mut commands);
        }
        Ok(Choice::ExitToMainMenu) => {
            info!("Exit to Main Menu chosen from the pause menu");
            crate::menus::restart_to_menu(&mut exit, &mut commands);
        }
        Ok(Choice::ExitToDesktop) => {
            info!("Exit to Desktop chosen from the pause menu");
            crate::settings::save_now(&mut commands);
            exit.write(AppExit::Success);
        }
        Err(_) => {}
    }
}
