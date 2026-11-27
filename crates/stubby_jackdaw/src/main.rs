use bevy::prelude::*;
mod lod_review;
mod prefab_bridge;
mod seeded_layout;

fn main() -> AppExit {
    let plugins = DefaultPlugins.set(AssetPlugin {
        file_path: format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")),
        ..default()
    });
    let plugins = jackdaw_runtime::maybe_windowless(plugins);
    let config = match stubby_jackdaw::world_generation::WorldConfig::from_args(std::env::args()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Invalid world configuration: {error}");
            return AppExit::error();
        }
    };
    // Phase and asset review scenes are seen at noon, as reviews are (AGENTS.md).
    let phase_review = config.scene.starts_with("world/phase_reviews/")
        || config.scene.starts_with("world/asset_reviews/");
    let seed = config.seed_value().expect("validated world seed");
    // Dev builds bring up the rules designer beside the game (add-machine-rules D4).
    stubby_jackdaw::designer::launch();
    let legacy_review = std::env::args().any(|arg| arg.starts_with("--replenishment-review="));
    let mut app = App::new();
    if !legacy_review {
        app.insert_resource(stubby_jackdaw::prefabs::AuthoredLevel);
    }
    if phase_review {
        app.add_systems(Startup, phase_review_daylight);
    }
    // A plain launch opens the main menu; loading lasts until the scene is in
    // (add-multiplayer-and-menus).
    if !phase_review && stubby_jackdaw::menus::StartAtMenu::wanted(std::env::args()) {
        app.insert_resource(stubby_jackdaw::menus::StartAtMenu);
    }
    if !legacy_review {
        app.insert_resource(stubby_jackdaw::menus::AwaitScene)
            .add_observer(
                |_: On<jackdaw_runtime::JackdawSceneSpawned>, mut commands: Commands| {
                    commands.insert_resource(stubby_jackdaw::menus::WorldReady);
                },
            )
            .add_observer(
                |_: On<jackdaw_runtime::JackdawSceneSpawnFailed>, mut commands: Commands| {
                    commands.insert_resource(stubby_jackdaw::menus::WorldReady);
                },
            );
    }
    app.insert_resource(config)
        .add_plugins(plugins)
        .add_plugins(stubby_game::settings::SettingsPersistencePlugin)
        .add_plugins(jackdaw_runtime::JackdawPlugin)
        .add_plugins(stubby_jackdaw::DesertPlugin { seed })
        .add_plugins(prefab_bridge::PrefabBridgePlugin)
        .add_plugins(seeded_layout::SeededLayoutPlugin)
        .add_plugins(lod_review::LodReviewPlugin)
        .add_plugins(stubby_jackdaw::audio::SynthAudioPlugin {
            demo: std::env::args().any(|a| a == "--audio-demo"),
            ..default()
        })
        .add_systems(Startup, load_authored_scene)
        .run()
}

fn load_authored_scene(
    mut commands: Commands,
    assets: Res<AssetServer>,
    config: Res<stubby_jackdaw::world_generation::WorldConfig>,
) {
    if std::env::args().any(|arg| arg.starts_with("--replenishment-review=")) {
        return;
    }
    commands.spawn(jackdaw_runtime::JackdawSceneRoot(
        assets.load(config.scene.clone()),
    ));
}

fn phase_review_daylight(mut calendar: ResMut<stubby_jackdaw::calendar::Calendar>) {
    let seconds = stubby_jackdaw::calendar::seconds_at_solar_hour(12.0);
    calendar.elapsed = std::time::Duration::from_secs_f32(seconds);
}
