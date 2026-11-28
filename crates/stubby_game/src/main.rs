use bevy::prelude::*;

fn main() {
    let config = match stubby_game::world_generation::WorldConfig::from_args(std::env::args()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Invalid world configuration: {error}");
            std::process::exit(2);
        }
    };
    let seed = config.seed_value().expect("validated world seed");
    if let Some(path) =
        std::env::args().find_map(|s| s.strip_prefix("--export=").map(str::to_owned))
    {
        stubby_game::export::write_world_preview(std::path::Path::new(&path), &config)
            .expect("export desert preview");
        return;
    }
    let default_plugins = DefaultPlugins
        .set(AssetPlugin {
            file_path: format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Stubby • mudflat / seed {seed} • WASD • mouse look • E collect"),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        });
    let mut app = App::new();
    // A plain launch opens the main menu (add-multiplayer-and-menus).
    let start_at_menu = stubby_game::menus::StartAtMenu::wanted(std::env::args());
    if start_at_menu {
        app.insert_resource(stubby_game::menus::StartAtMenu);
    }
    // `--host [PORT]` or `--join ADDR`; the menu loads both roles so it can start either.
    use stubby_game::net::NetMode;
    let fallback = if start_at_menu {
        NetMode::Menu
    } else {
        NetMode::Offline
    };
    let mode = NetMode::from_args(std::env::args(), fallback).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    app.insert_resource(mode);
    app.insert_resource(config)
        .add_plugins(default_plugins)
        .add_plugins(stubby_game::net::physics_plugins(mode))
        .add_plugins(stubby_game::settings::SettingsPersistencePlugin)
        .add_plugins(stubby_game::DesertPlugin { seed })
        .add_plugins(stubby_game::audio::SynthAudioPlugin {
            demo: std::env::args().any(|a| a == "--audio-demo"),
            ..default()
        })
        .run();
}
