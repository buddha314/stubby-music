//! The menus around play (add-multiplayer-and-menus 1.1–1.4), in stubby's style: the
//! loading screen while the world comes up, the main menu at launch, and the Settings
//! screen (from the main menu or the pause menu). Their logic follows the us-outpost-31
//! template's menus; their look follows the pause menu (amber on dark, a left column).
//!
//! They are `stubby_core::Screen`s, so every system already treats them as screens: the
//! cursor is free and the player neither moves nor looks. The main menu appears only
//! when the binary asks for it ([`StartAtMenu`]): review runs, exports and tests start
//! in the world as before.

use bevy::picking::events::{PointerClick, PointerOut, PointerOver};
use bevy::prelude::*;
use stubby_core::{Screen, ScreenMode};
use stubby_ui::style::{AMBER, BACKDROP, EDGE, INK, MUTED, PANEL};

use crate::settings::{self, AudioSettings, GraphicsSettings};

const BUTTON: Color = Color::srgba(0.09, 0.085, 0.075, 1.0);
/// How long loading waits for the world before going on anyway.
const LOADING_TIMEOUT_SECS: f32 = 20.0;

/// Start at the main menu (with a loading screen first). Inserted by the binaries for a
/// plain launch; absent, the game starts in the world as it always has.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct StartAtMenu;

impl StartAtMenu {
    /// A plain launch shows the menu; `--play`, a review, an export or a screenshot run
    /// starts in the world.
    pub fn wanted(args: impl IntoIterator<Item = String>) -> bool {
        !args.into_iter().skip(1).any(|arg| {
            arg == "--play"
                || arg == "--host"
                || arg == "--join"
                || arg == "--purifier-review"
                || arg == "--sampler-review"
                || arg.starts_with("--export=")
                || arg.starts_with("--screenshot=")
                || arg.starts_with("--pulse-preview=")
                || (arg.starts_with("--") && arg.contains("-review="))
        })
    }
}

/// The binary loads a scene: loading lasts until [`WorldReady`] (stubby_jackdaw inserts
/// both). Without it, the world is ready a few frames after startup.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct AwaitScene;

/// The world is up: the scene spawned (or failed to), so loading can end.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WorldReady;

/// Where Settings' Back goes.
#[derive(Resource, Clone, Copy, Debug)]
struct SettingsFrom(Screen);

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsFrom(Screen::MainMenu))
            .add_systems(Startup, (spawn_loading, spawn_main_menu, spawn_settings))
            .add_systems(
                Update,
                (
                    finish_loading,
                    menu_keys,
                    show_panels,
                    show_values.run_if(resource_exists::<GraphicsSettings>),
                ),
            );
        if app.world().contains_resource::<StartAtMenu>() {
            app.add_systems(Startup, |mut screens: ResMut<ScreenMode>| {
                info!("Starting at the main menu (loading first)");
                screens.open = Some(Screen::Loading);
            });
        }
    }
}

/// Open Settings from another screen (the pause menu uses it).
pub fn open_settings(screens: &mut ScreenMode, from: Screen, commands: &mut Commands) {
    commands.insert_resource(SettingsFrom(from));
    screens.open = Some(Screen::Settings);
}

/// Leave for the main menu by starting the game afresh with the same arguments: a fresh
/// process is the one sure way to reset the world. Under the editor's Play it just stops.
pub fn restart_to_menu(exit: &mut MessageWriter<AppExit>, commands: &mut Commands) {
    settings::save_now(commands);
    if std::env::var_os("JACKDAW_PIE").is_some() {
        info!("Leaving to the main menu: press Play in the editor to start again");
    } else if let Err(error) = std::env::current_exe().and_then(|exe| {
        std::process::Command::new(exe)
            .args(std::env::args().skip(1))
            .spawn()
    }) {
        error!("Could not restart into the main menu: {error}");
    }
    exit.write(AppExit::Success);
}

fn finish_loading(
    time: Res<Time<Real>>,
    ready: Option<Res<WorldReady>>,
    awaiting: Option<Res<AwaitScene>>,
    mut frames: Local<u32>,
    mut screens: ResMut<ScreenMode>,
) {
    if screens.open != Some(Screen::Loading) {
        return;
    }
    *frames += 1;
    let ready = ready.is_some() || (awaiting.is_none() && *frames > 5);
    let timed_out = time.elapsed_secs() > LOADING_TIMEOUT_SECS;
    if timed_out && !ready {
        warn!("No world after {LOADING_TIMEOUT_SECS} s; showing the menu anyway");
    }
    if ready || timed_out {
        info!("World ready after {:.1} s: main menu", time.elapsed_secs());
        screens.open = Some(Screen::MainMenu);
    }
}

/// Escape steps back from Settings; the main menu and loading ignore it.
fn menu_keys(
    keys: Res<ButtonInput<KeyCode>>,
    from: Res<SettingsFrom>,
    mut screens: ResMut<ScreenMode>,
) {
    if keys.just_pressed(KeyCode::Escape) && screens.open == Some(Screen::Settings) {
        screens.open = Some(from.0);
    }
}

// --- Look --- //

/// Which screen a panel belongs to.
#[derive(Component, Clone, Copy)]
struct Panel(Screen);

/// What a button does.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
enum Action {
    Play,
    Host,
    Settings,
    Quit,
    Back,
    Display,
    Vsync,
    Shadows,
    UiScale(i8),
    Volume(Volume, i8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Volume {
    Master,
    World,
    Interface,
}

/// A label showing one setting's value.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
enum Value {
    Display,
    Vsync,
    Shadows,
    UiScale,
    Volume(Volume),
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

/// A full-window backdrop with a left column, like the pause menu.
fn panel(commands: &mut Commands, screen: Screen, title: &str, subtitle: &str) -> Entity {
    let mut column = Entity::PLACEHOLDER;
    commands
        .spawn((
            Panel(screen),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                padding: UiRect::left(px(96)),
                ..default()
            },
            BackgroundColor(BACKDROP),
            GlobalZIndex(160),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            column = root
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10),
                        width: px(380),
                        padding: UiRect::all(px(20)),
                        border: UiRect::left(px(3)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderColor::all(AMBER),
                ))
                .with_children(|column| {
                    column.spawn(text(title, 30., INK));
                    if !subtitle.is_empty() {
                        column.spawn(text(subtitle, 14., MUTED));
                    }
                    column.spawn((
                        Node {
                            height: px(1),
                            margin: UiRect::vertical(px(8)),
                            ..default()
                        },
                        BackgroundColor(EDGE),
                    ));
                })
                .id();
        });
    column
}

fn button(parent: &mut ChildSpawnerCommands, action: Action, label: &str, size: f32) {
    parent
        .spawn((
            action,
            Node {
                padding: UiRect::axes(px(16), px(10)),
                border: UiRect::all(px(1)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BUTTON),
            BorderColor::all(EDGE),
        ))
        .with_child(text(label, size, INK))
        .observe(hover)
        .observe(unhover)
        .observe(choose);
}

fn spawn_loading(mut commands: Commands) {
    let column = panel(&mut commands, Screen::Loading, "STUBBY", "");
    commands
        .entity(column)
        .with_child(text("Waking the desert…", 18., MUTED));
}

fn spawn_main_menu(mut commands: Commands) {
    let column = panel(
        &mut commands,
        Screen::MainMenu,
        "STUBBY",
        "The music of nature brings light to the world.",
    );
    commands.entity(column).with_children(|column| {
        button(column, Action::Play, "Play", 20.);
        button(column, Action::Host, "Host a Game", 20.);
        column.spawn(text(
            "Join: start with --join ADDRESS for now. Steam: coming.",
            14.,
            MUTED,
        ));
        button(column, Action::Settings, "Settings", 20.);
        button(column, Action::Quit, "Exit to Desktop", 20.);
    });
}

fn spawn_settings(mut commands: Commands) {
    let column = panel(
        &mut commands,
        Screen::Settings,
        "SETTINGS",
        "Saved for next time.",
    );
    commands.entity(column).with_children(|column| {
        column.spawn(text("Display", 14., AMBER));
        setting_row(
            column,
            "Window",
            Value::Display,
            &[(Action::Display, "Change")],
        );
        setting_row(
            column,
            "Vertical sync",
            Value::Vsync,
            &[(Action::Vsync, "Toggle")],
        );
        setting_row(
            column,
            "Shadows",
            Value::Shadows,
            &[(Action::Shadows, "Toggle")],
        );
        setting_row(
            column,
            "Interface size",
            Value::UiScale,
            &[(Action::UiScale(-1), "-"), (Action::UiScale(1), "+")],
        );
        column.spawn(text("Sound", 14., AMBER));
        for (name, volume) in [
            ("Master", Volume::Master),
            ("World", Volume::World),
            ("Interface", Volume::Interface),
        ] {
            setting_row(
                column,
                name,
                Value::Volume(volume),
                &[
                    (Action::Volume(volume, -1), "-"),
                    (Action::Volume(volume, 1), "+"),
                ],
            );
        }
        button(column, Action::Back, "Back", 20.);
    });
}

/// A row: the setting's name, its value, and its buttons.
fn setting_row(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    value: Value,
    actions: &[(Action, &str)],
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    width: px(130),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_child(text(name, 16., INK));
            row.spawn((
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_child((text("", 16., MUTED), value));
            for &(action, label) in actions {
                button(row, action, label, 16.);
            }
        });
}

fn show_panels(screens: Res<ScreenMode>, mut panels: Query<(&Panel, &mut Visibility)>) {
    for (Panel(screen), mut visibility) in &mut panels {
        visibility.set_if_neq(if screens.open == Some(*screen) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

fn percent_text(value: f32) -> String {
    format!("{:.0}%", value * 100.0)
}

fn show_values(
    graphics: Res<GraphicsSettings>,
    audio: Res<AudioSettings>,
    mut labels: Query<(&Value, &mut Text)>,
) {
    let on = |b: bool| if b { "On" } else { "Off" }.to_string();
    for (value, mut text) in &mut labels {
        let shown = match value {
            Value::Display => graphics.display.name().to_string(),
            Value::Vsync => on(graphics.vsync),
            Value::Shadows => on(graphics.shadows),
            Value::UiScale => percent_text(graphics.ui_scale),
            Value::Volume(Volume::Master) => percent_text(audio.master),
            Value::Volume(Volume::World) => percent_text(audio.world),
            Value::Volume(Volume::Interface) => percent_text(audio.interface),
        };
        if text.0 != shown {
            text.0 = shown;
        }
    }
}

fn hover(over: On<PointerOver>, mut buttons: Query<&mut BorderColor, With<Action>>) {
    if let Ok(mut border) = buttons.get_mut(over.entity) {
        *border = BorderColor::all(AMBER);
    }
}

fn unhover(out: On<PointerOut>, mut buttons: Query<&mut BorderColor, With<Action>>) {
    if let Ok(mut border) = buttons.get_mut(out.entity) {
        *border = BorderColor::all(EDGE);
    }
}

/// Step a fraction by tenths (or twentieths for small values), within `low..=high`.
fn step(value: f32, by: i8, low: f32, high: f32) -> f32 {
    let next = ((value * 20.0).round() + f32::from(by) * 2.0) / 20.0;
    next.clamp(low, high)
}

#[allow(clippy::too_many_arguments)]
fn choose(
    click: On<PointerClick>,
    actions: Query<&Action>,
    from: Res<SettingsFrom>,
    mut screens: ResMut<ScreenMode>,
    mut graphics: ResMut<GraphicsSettings>,
    mut audio: ResMut<AudioSettings>,
    mut exit: MessageWriter<AppExit>,
    mode: Option<Res<stubby_net::NetMode>>,
    mut commands: Commands,
) {
    let Ok(&action) = actions.get(click.entity) else {
        return;
    };
    let on = |screen| screens.open == Some(screen);
    match action {
        Action::Play if on(Screen::MainMenu) => {
            // A networked build plays on a host only this machine can reach.
            if mode.is_some_and(|m| m.is_networked()) {
                stubby_net::play_solo(&mut commands);
            }
            screens.close();
        }
        Action::Host if on(Screen::MainMenu) => {
            if mode.is_some_and(|m| m.is_networked()) {
                stubby_net::host(&mut commands, stubby_core::multiplayer::DEFAULT_PORT);
                info!(
                    "Hosting on port {}: others join with --join <this machine's address>",
                    stubby_core::multiplayer::DEFAULT_PORT
                );
            }
            screens.close();
        }
        Action::Settings if on(Screen::MainMenu) => {
            open_settings(&mut screens, Screen::MainMenu, &mut commands);
        }
        Action::Quit if on(Screen::MainMenu) => {
            settings::save_now(&mut commands);
            exit.write(AppExit::Success);
        }
        Action::Back if on(Screen::Settings) => screens.open = Some(from.0),
        _ if !on(Screen::Settings) => {}
        Action::Display => graphics.display = graphics.display.next(),
        Action::Vsync => graphics.vsync = !graphics.vsync,
        Action::Shadows => graphics.shadows = !graphics.shadows,
        Action::UiScale(by) => {
            let (low, high) = GraphicsSettings::UI_SCALE;
            graphics.ui_scale = step(graphics.ui_scale, by, low, high);
        }
        Action::Volume(volume, by) => {
            let level = match volume {
                Volume::Master => &mut audio.master,
                Volume::World => &mut audio.world,
                Volume::Interface => &mut audio.interface,
            };
            *level = step(*level, by, 0.0, 1.0);
        }
        _ => return,
    }
    if matches!(
        action,
        Action::Display | Action::Vsync | Action::Shadows | Action::UiScale(_) | Action::Volume(..)
    ) {
        settings::save(&mut commands);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        std::iter::once("stubby")
            .chain(list.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn a_plain_launch_shows_the_menu_and_reviews_do_not() {
        assert!(StartAtMenu::wanted(args(&["--seed=42"])));
        assert!(StartAtMenu::wanted(args(&["--seed=42", "--audio-demo"])));
        assert!(!StartAtMenu::wanted(args(&["--seed=42", "--play"])));
        assert!(!StartAtMenu::wanted(args(&["--sky-review=/tmp/x"])));
        assert!(!StartAtMenu::wanted(args(&[
            "--replenishment-review=/tmp/x"
        ])));
        assert!(!StartAtMenu::wanted(args(&["--purifier-review"])));
        assert!(!StartAtMenu::wanted(args(&["--screenshot=/tmp/x.png"])));
        assert!(!StartAtMenu::wanted(args(&["--export=/tmp/x"])));
    }

    /// The menus with the game's screen state, headless.
    fn app(start_at_menu: bool) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::input::InputPlugin))
            .init_resource::<ScreenMode>()
            .init_resource::<GraphicsSettings>()
            .init_resource::<AudioSettings>();
        if start_at_menu {
            app.insert_resource(StartAtMenu);
        }
        app.add_plugins(MenusPlugin);
        app
    }

    /// Escape as the window delivers it (pressing `ButtonInput` directly is undone by
    /// the input system at the start of the frame).
    fn press_escape(app: &mut App) {
        app.world_mut()
            .write_message(bevy::input::keyboard::KeyboardInput {
                key_code: KeyCode::Escape,
                logical_key: bevy::input::keyboard::Key::Escape,
                state: bevy::input::ButtonState::Pressed,
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
    }

    fn open(app: &App) -> Option<Screen> {
        app.world().resource::<ScreenMode>().open
    }

    #[test]
    fn loading_leads_to_the_main_menu_which_escape_does_not_close() {
        let mut app = app(true);
        app.update();
        assert_eq!(open(&app), Some(Screen::Loading));
        for _ in 0..8 {
            app.update();
        }
        assert_eq!(open(&app), Some(Screen::MainMenu));
        press_escape(&mut app);
        app.update();
        assert_eq!(open(&app), Some(Screen::MainMenu));
    }

    #[test]
    fn a_scene_holds_loading_until_the_world_is_ready() {
        let mut app = app(true);
        app.insert_resource(AwaitScene);
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(open(&app), Some(Screen::Loading));
        app.insert_resource(WorldReady);
        app.update();
        assert_eq!(open(&app), Some(Screen::MainMenu));
    }

    #[test]
    fn escape_steps_back_from_settings_to_where_it_was_opened() {
        let mut app = app(false);
        app.update();
        assert_eq!(open(&app), None, "no menu unless the binary asks");
        app.world_mut()
            .run_system_cached(|mut screens: ResMut<ScreenMode>, mut commands: Commands| {
                open_settings(&mut screens, Screen::Pause, &mut commands);
            })
            .unwrap();
        app.update();
        assert_eq!(open(&app), Some(Screen::Settings));
        press_escape(&mut app);
        app.update();
        assert_eq!(open(&app), Some(Screen::Pause));
    }

    #[test]
    fn steps_stay_in_range() {
        assert_eq!(step(1.0, 1, 0.0, 1.0), 1.0);
        assert_eq!(step(0.0, -1, 0.0, 1.0), 0.0);
        assert!((step(0.5, 1, 0.0, 1.0) - 0.6).abs() < 1e-6);
        assert!((step(1.0, -1, 0.75, 2.0) - 0.9).abs() < 1e-6);
    }
}
