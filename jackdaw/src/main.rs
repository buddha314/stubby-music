use bevy::prelude::*;
mod prefab_bridge;

fn main() -> AppExit {
    let plugins = DefaultPlugins.set(AssetPlugin {
        file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
        ..default()
    });
    let plugins = jackdaw_runtime::maybe_windowless(plugins);
    let seed = std::env::args()
        .find_map(|arg| arg.strip_prefix("--seed=").and_then(|s| s.parse().ok()))
        .unwrap_or(42);
    let legacy_review = std::env::args().any(|arg| arg.starts_with("--replenishment-review="));
    let mut app = App::new();
    if !legacy_review {
        app.insert_resource(stubby_jackdaw::prefabs::AuthoredLevel);
    }
    app.add_plugins(plugins)
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

fn load_authored_scene(mut commands: Commands, assets: Res<AssetServer>) {
    if std::env::args().any(|arg| arg.starts_with("--replenishment-review=")) {
        return;
    }
    let scene = std::env::args()
        .find_map(|arg| arg.strip_prefix("--scene=").map(str::to_owned))
        .unwrap_or_else(|| "scene.bsn".into());
    commands.spawn(jackdaw_runtime::JackdawSceneRoot(assets.load(scene)));
}
