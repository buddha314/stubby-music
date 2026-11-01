// Compile the same generator against the editor's exact Bevy revision.
#[path = "../../crates/stubby_game/src/lib.rs"]
mod desert;
pub use desert::{
    DesertPlugin, WorldSeed, audio, calendar, controls, export, prefab_presentation, prefabs,
    pulse, resources, synth_modules,
};
