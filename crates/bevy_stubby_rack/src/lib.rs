//! The rack (split-game-into-systems): every machine is configured on the same Eurorack
//! screen, with modules as data (`stubby_rack`), patch cables, knobs and the module assets
//! behind them. It takes items from whatever implements `stubby_core::Stock` and never
//! names crafting, the player or the world.
pub mod machine_config;
pub mod rack;
pub mod record;
pub mod samples;
pub mod synth_modules;
