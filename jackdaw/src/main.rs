use bevy::prelude::*;
mod prefab_bridge;

fn main() -> AppExit {
    let plugins = DefaultPlugins.set(AssetPlugin {
        file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
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
    let seed = config.seed_value().expect("validated world seed");
    let legacy_review = std::env::args().any(|arg| arg.starts_with("--replenishment-review="));
    let mut app = App::new();
    if !legacy_review {
        app.insert_resource(stubby_jackdaw::prefabs::AuthoredLevel);
    }
    app.insert_resource(config)
        .add_plugins(plugins)
        .add_plugins(jackdaw_runtime::JackdawPlugin)
        .add_plugins(stubby_jackdaw::DesertPlugin { seed })
        .add_plugins(prefab_bridge::PrefabBridgePlugin)
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
