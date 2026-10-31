use bevy::prelude::*;

fn main() {
    let seed = std::env::args()
        .skip(1)
        .find_map(|s| s.strip_prefix("--seed=").and_then(|s| s.parse().ok()))
        .unwrap_or(42);
    if let Some(path) =
        std::env::args().find_map(|s| s.strip_prefix("--export=").map(str::to_owned))
    {
        stubby_game::export::write_preview(std::path::Path::new(&path), seed)
            .expect("export desert preview");
        return;
    }
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!(
                            "Stubby • mudflat / seed {seed} • WASD • mouse look • E collect"
                        ),
                        resolution: (1440, 900).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(stubby_game::DesertPlugin { seed })
        .add_plugins(stubby_game::audio::SynthAudioPlugin {
            demo: std::env::args().any(|a| a == "--audio-demo"),
            ..default()
        })
        .run();
}
