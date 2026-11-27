//! The stubby-music audio engine.
//!
//! This crate does not depend on the Bevy engine. The game talks to it only
//! through [`Engine`]: it sends [`Command`]s, calls [`Engine::flush`] once per
//! frame, and drains [`Event`]s. Firewheel runs the audio graph on the realtime
//! thread; each module's DSP is built from fundsp units.
//!
//! See `openspec/changes/add-audio-engine` for the requirements and design.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[cfg(all(feature = "audionimbus", feature = "omg_audio"))]
compile_error!("select only one spatial audio backend: audionimbus or omg_audio");

#[cfg(feature = "audionimbus")]
pub mod audionimbus_spatial;
pub mod clock;
mod engine;
mod modules;
#[cfg(feature = "omg_audio")]
pub mod omg_spatial;
mod protocol;
mod random_clock;
mod sample_voice;
pub mod signal;
mod thread;

pub use clock::{MASTER_BPM, MAX_BPM, MasterLockedClock, PeriodicClock, PulseClock, RandomClock};
pub use engine::{Engine, EngineConfig, OfflineRenderer, WorldMeter};
pub use modules::{ParamSpec, SEMITONE_VOLTS, StepState};
pub use protocol::{
    AcousticMaterial, AcousticSceneGeometry, Bus, Command, EngineError, Event, ListenerPose,
    ModuleId, ModuleKind, PortRef, SEQUENCER_STEPS, SampleId,
};
pub use thread::{AudioThread, Notice};
