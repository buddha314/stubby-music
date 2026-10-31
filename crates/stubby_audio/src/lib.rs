//! The stubby-music audio engine.
//!
//! This crate does not depend on the Bevy engine. The game talks to it only
//! through [`Engine`]: it sends [`Command`]s, calls [`Engine::flush`] once per
//! frame, and drains [`Event`]s. Firewheel runs the audio graph on the realtime
//! thread; each module's DSP is built from fundsp units.
//!
//! See `openspec/changes/add-audio-engine` for the requirements and design.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod clock;
mod engine;
mod modules;
mod protocol;
mod random_clock;
pub mod signal;

pub use clock::{PeriodicClock, PulseClock, RandomClock};
pub use engine::{Engine, EngineConfig, OfflineRenderer};
pub use modules::ParamSpec;
pub use protocol::{Command, EngineError, Event, ModuleId, ModuleKind, PortRef};
