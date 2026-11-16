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
        app.add_plugins(stubby_game::DesertPlugin { seed: self.seed })
            .add_systems(
                PostUpdate,
                switch_off_preview_lights
                    .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
            );
    }
}

/// The level's lights tagged `HiddenInGame` light the editor's view only: the game's
/// calendar owns the sun and moon. Jackdaw's Play leaves `HiddenInGame` entities drawn
/// (so volumes can be placed against the running game), and the scene re-applies their
/// stored visibility, which left a second, fixed 9500 lux sun lighting the scene and the
/// night atmosphere. Kept hidden every frame; a hidden light is not extracted, as in the
/// standalone game.
#[allow(clippy::type_complexity)] // Any light kind.
fn switch_off_preview_lights(
    mut lights: Query<
        &mut Visibility,
        (
            With<jackdaw_scene_types::HiddenInGame>,
            Or<(With<DirectionalLight>, With<PointLight>, With<SpotLight>)>,
        ),
    >,
) {
    for mut visibility in &mut lights {
        visibility.set_if_neq(Visibility::Hidden);
    }
}
