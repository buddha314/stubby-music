//! The game-facing vocabulary: module IDs, ports, commands, events and errors.

/// A module in the patch. The game assigns IDs, so it never waits on the audio
/// side to learn one (design D5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ModuleId(pub u64);

impl ModuleId {
    /// The reserved master output. Its one input, `"in"`, accepts any number of
    /// cables and sends their sum to both speakers.
    pub const OUTPUT: ModuleId = ModuleId(0);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModuleKind {
    /// Sine oscillator. Input `"voct"`, output `"out"`, parameter `"freq"` (Hz).
    SineVco { freq_hz: f32 },
    /// Band-limited square oscillator; same ports and parameter as `SineVco`. Its
    /// odd harmonics make it far easier to locate by ear than a sine.
    SquareVco { freq_hz: f32 },
    /// Clock. Outputs `"gate"` (high for the first half of each beat) and `"phase"`
    /// (0..1 ramp per beat), parameter `"bpm"`. Emits [`Event::ClockTick`] per beat.
    Clock { bpm: f32 },
    /// Constant CV source. Output `"out"`, parameter `"value"`.
    Constant { value: f32 },
    /// Attack/decay envelope. Input `"gate"`: each rising edge starts a linear attack
    /// to 1.0, then an exponential decay. Output `"out"` (0..1). Parameters `"attack"`
    /// (seconds to peak) and `"decay"` (seconds from peak to -60 dB).
    Envelope { attack_s: f32, decay_s: f32 },
    /// Voltage-controlled amplifier: `out = in * max(cv, 0) * gain`. Inputs `"in"`
    /// and `"cv"`, output `"out"`, parameter `"gain"`. With no CV it is closed.
    Vca { gain: f32 },
}

/// One named port on one module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortRef {
    pub module: ModuleId,
    pub port: &'static str,
}

impl PortRef {
    pub const fn new(module: ModuleId, port: &'static str) -> Self {
        Self { module, port }
    }

    /// The master output's input.
    pub const fn output() -> Self {
        Self::new(ModuleId::OUTPUT, "in")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    AddModule {
        id: ModuleId,
        kind: ModuleKind,
    },
    RemoveModule {
        id: ModuleId,
    },
    /// Connect an output port to an input port.
    Connect {
        from: PortRef,
        to: PortRef,
    },
    Disconnect {
        from: PortRef,
        to: PortRef,
    },
    /// Set a parameter. Repeated sets of one parameter before the next
    /// [`Engine::flush`](crate::Engine::flush) coalesce to the last value.
    SetParam {
        module: ModuleId,
        param: &'static str,
        value: f32,
    },
}

/// Something that happened on the audio thread, stamped with the audio clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A clock crossed a beat. `sample_time` is the exact frame of the tick on
    /// the audio clock (see [`Engine::sample_to_instant`](crate::Engine::sample_to_instant)).
    ClockTick { module: ModuleId, sample_time: u64 },
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EngineError {
    #[error("module {0:?} already exists")]
    DuplicateModule(ModuleId),
    #[error("module {0:?} does not exist")]
    UnknownModule(ModuleId),
    #[error("module {0:?} is reserved")]
    ReservedModule(ModuleId),
    #[error("{0:?} is not an output port")]
    NotAnOutput(PortRef),
    #[error("{0:?} is not an input port")]
    NotAnInput(PortRef),
    #[error("module {module:?} has no parameter {param:?}")]
    UnknownParam {
        module: ModuleId,
        param: &'static str,
    },
    #[error("input {0:?} already has a cable")]
    InputOccupied(PortRef),
    #[error("no cable from {from:?} to {to:?}")]
    NotConnected { from: PortRef, to: PortRef },
    #[error("that cable would create a feedback loop")]
    Cycle,
    #[error("the audio thread's queue is full; retry next frame")]
    QueueFull,
    #[error("audio graph error: {0}")]
    Graph(String),
    #[error("audio device error: {0}")]
    Device(String),
    #[error("invalid engine config: {0}")]
    Config(&'static str),
}
