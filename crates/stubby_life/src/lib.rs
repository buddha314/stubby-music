//! Plants and animals (split-game-into-systems): the life music grows. Today the lightning
//! bug; the flora's presentation joins it. The game's glue makes them harvestable sources,
//! places them and waters them; this crate only knows how they look, sound and live.
pub mod bug;

use bevy_stubby_audio::voice::WaveformSound;
use serde::Deserialize;

/// The lightning bug's click: the square wave's pitch and shape, shared by every
/// voice that plays a square wave from a bug (the bugs, collection cues, the burbler's VCO).
pub fn bug_click() -> WaveformSound {
    #[derive(Deserialize)]
    struct Preset {
        click: WaveformSound,
    }
    let preset: Preset = serde_json::from_str(include_str!(
        "../../../assets/creatures/lightning_bug/asset.json"
    ))
    .expect("shipped waveform preset is valid");
    preset.click
}
