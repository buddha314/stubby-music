#[cfg(all(feature = "dev", not(debug_assertions)))]
compile_error!("The dev feature enables dynamic linking; omit it for release builds.");

// The crafting system, as a Bevy plugin.
pub use bevy_stubby_recipes as recipes_plugin;

// The audio engine as a Bevy service (bevy_stubby_audio), under its old module names.
pub use bevy_stubby_audio as audio;
pub use bevy_stubby_audio::module_ids;
pub use stubby_world::{WorldSeed, calendar, export, prefab_presentation, world_generation};
// The player system (stubby_player), under its old module names.
pub use stubby_player::{controls, hand, interaction};
mod build;
mod inventory_screen;
mod pause_menu;
// The rack system (bevy_stubby_rack), under its old module names.
pub use bevy_stubby_rack::{machine_config, rack, synth_modules};
pub mod glue;
mod goals_hub;
mod grotto_audio_review;
mod prefab_review;
/// The prefab lifecycle (stubby_world), plus the game's crafting view of placed sources.
pub mod prefabs {
    pub use crate::glue::{PrefabGluePlugin, WorldSources};
    pub use stubby_world::prefabs::*;
}
// The lightning bug (stubby_life), under its old module name.
pub use stubby_life::bug as pulse;
pub mod resources;
mod review;
// Stream laying (stubby_streams), under its old module name.
pub use stubby_streams as streams;
mod sky_review;
mod stream_review;
mod watering;
mod waveform_audio;

use bevy::{camera::Exposure, prelude::*};

pub struct DesertPlugin {
    pub seed: u64,
}
impl Plugin for DesertPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            pulse::LightningBugPlugin { seed: self.seed },
            resources::ResourceRecipesPlugin,
            inventory_screen::InventoryScreenPlugin,
            hand::HandPlugin,
            rack::RackPlugin::<resources::PlayerInventory>::default(),
            glue::GluePlugin,
            interaction::InteractionPlugin,
            stubby_world::DesertWorldPlugin { seed: self.seed },
            prefabs::PrefabGluePlugin,
            review::ReplenishmentReviewPlugin,
            prefab_review::PrefabReviewPlugin,
            controls::PlayerControlsPlugin,
            machine_config::MachineConfigurationPlugin,
            waveform_audio::WaveformAudioPlugin,
            synth_modules::SynthModulesPlugin::default(),
        ))
        .add_plugins(grotto_audio_review::GrottoAudioReviewPlugin)
        .add_plugins((
            streams::StreamBuildPlugin::<resources::PlayerInventory>::default(),
            stream_review::StreamReviewPlugin,
            sky_review::SkyReviewPlugin,
            goals_hub::GoalsHubPlugin,
            pause_menu::PauseMenuPlugin,
            build::BuildPlugin,
            watering::WateringPlugin,
        ))
        .add_systems(Startup, spawn_camera.after(stubby_world::WorldSetup));
        // Dev builds serve Bevy's remote protocol on 127.0.0.1 (CORS open for the rules
        // designer's page), so tools can read the running game (add-crafting-designer 3.8).
        #[cfg(debug_assertions)]
        app.add_plugins((
            bevy::remote::RemotePlugin::default(),
            bevy::remote::http::RemoteHttpPlugin::default().with_headers(
                bevy::remote::http::Headers::new()
                    .insert("Access-Control-Allow-Origin", "*")
                    .insert("Access-Control-Allow-Headers", "Content-Type"),
            ),
        ));
        app.add_systems(Update, capture_preview);
    }
}

fn spawn_camera(mut commands: Commands, ground: Res<stubby_core::GroundHeight>) {
    commands.spawn((
        stubby_world::sky::camera_sky(),
        Camera3d::default(),
        bevy::post_process::bloom::Bloom::NATURAL,
        Exposure { ev100: 10.5 },
        bevy::ui::IsDefaultUiCamera,
        controls::Player::on(&ground),
        controls::spawn_transform(&ground),
        DistanceFog {
            color: Color::srgb(0.56, 0.43, 0.33),
            falloff: FogFalloff::Exponential { density: 0.004 },
            ..default()
        },
    ));
}

// Capture after pipeline warmup; useful for reviewing a seed without desktop tools.
fn capture_preview(mut commands: Commands, time: Res<Time>, mut captured: Local<bool>) {
    if !*captured
        && time.elapsed_secs() > 15.
        && let Some(path) =
            std::env::args().find_map(|s| s.strip_prefix("--screenshot=").map(str::to_owned))
    {
        *captured = true;
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path));
    }
}
