//! The game as Jackdaw builds it. Jackdaw's project names this package; the game is
//! `stubby_game`, re-exported here so the editor bridge keeps one path.
use bevy::prelude::*;

pub use stubby_game::*;

/// The game plugin `jackdaw.toml` names. Jackdaw finds a project's game plugin by reading
/// this package's own source for `impl Plugin for`, so a re-export from `stubby_game` is
/// not enough: the plugin is declared here and adds the game's.
pub struct DesertPlugin {
    pub seed: u64,
}
impl Plugin for DesertPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(stubby_game::DesertPlugin { seed: self.seed });
    }
}
