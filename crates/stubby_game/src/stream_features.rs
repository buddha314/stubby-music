//! Things that sit on a laid stream (ripple pools, `add-ripple-pool`) keep its time: a
//! placement marked [`OnStream`] carries its stream's clock while the stream sounds, so
//! its one-beat clips play on that clock and its beat-only parts show
//! (`stubby_world::prefab_presentation`); a silent stream leaves it still. The stream and
//! the placement are two systems, so the rule that joins them lives here, in the game.
use bevy::prelude::*;
use stubby_core::{AssetClock, MachineVoice};
use stubby_streams::LaidStream;

/// Placed on a laid stream: the stream entity it belongs to.
#[derive(Component, Clone, Copy, Debug)]
pub struct OnStream(pub Entity);

pub struct StreamFeaturesPlugin;
impl Plugin for StreamFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, keep_stream_time.after(stubby_core::PrefabBinding));
    }
}

#[allow(clippy::type_complexity)] // Features, their streams, and the machines voicing them.
fn keep_stream_time(
    mut commands: Commands,
    features: Query<(Entity, &OnStream, Option<&AssetClock>)>,
    streams: Query<&LaidStream>,
    machines: Query<(&AssetClock, &MachineVoice)>,
) {
    for (entity, on, held) in &features {
        let clock = streams
            .get(on.0)
            .ok()
            .and_then(|stream| stream.source)
            .and_then(|source| machines.get(source).ok())
            .filter(|(_, voice)| voice.sounding)
            .map(|(clock, _)| *clock);
        match (clock, held) {
            (Some(clock), Some(held)) if held.clock == clock.clock && held.bpm == clock.bpm => {}
            (Some(clock), _) => {
                commands.entity(entity).insert(clock);
            }
            (None, Some(_)) => {
                commands.entity(entity).remove::<AssetClock>();
            }
            (None, None) => {}
        }
    }
}
