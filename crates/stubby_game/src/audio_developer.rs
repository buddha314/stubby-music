//! Session-only spatial-audio tuning exposed to the local Bevy Remote Protocol.

use bevy::prelude::*;
use bevy_stubby_audio::Synth;
use stubby_audio::Command;

/// Developer controls for the current per-emitter reflection path.
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct AudioDeveloperSettings {
    /// Linear reflection contribution. Zero is dry; 0.75 is the listening-review default.
    pub reflection_wet_mix: f32,
}

impl Default for AudioDeveloperSettings {
    fn default() -> Self {
        Self {
            reflection_wet_mix: 0.75,
        }
    }
}

pub struct AudioDeveloperPlugin;

impl Plugin for AudioDeveloperPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioDeveloperSettings>()
            .register_type::<AudioDeveloperSettings>()
            .add_systems(Update, apply_audio_developer_settings);
    }
}

fn apply_audio_developer_settings(settings: Res<AudioDeveloperSettings>, synth: Res<Synth>) {
    if settings.is_changed() {
        synth.send(Command::SetReflectionWetMix {
            value: settings.reflection_wet_mix,
        });
    }
}
