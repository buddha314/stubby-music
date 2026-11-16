//! The whole game assembles: every plugin is added once and builds. A plugin conflict
//! (the same plugin added by two systems) panics here instead of on the player's screen.
use bevy::prelude::*;

#[test]
fn the_whole_game_assembles() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .add_plugins(stubby_game::DesertPlugin { seed: 42 });
    app.finish();
    app.cleanup();
}
