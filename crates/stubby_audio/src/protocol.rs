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

/// A recorded sound loaded with `Engine::load_sample` (or `AudioThread::load_sample`),
/// played as a one-shot by [`Command::PlaySample`]. Sound effects, not patch modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct SampleId(pub u32);

/// Listener pose in world coordinates. The axes are unit vectors in the game world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListenerPose {
    pub position: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub ahead: [f32; 3],
}

/// Acoustic properties of one surface material for AudioNimbus/Steam Audio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticMaterial {
    pub absorption: [f32; 3],
    pub scattering: f32,
    pub transmission: [f32; 3],
}

/// Triangle geometry in world coordinates. Material indices address `materials`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AcousticSceneGeometry {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub material_indices: Vec<usize>,
    pub materials: Vec<AcousticMaterial>,
}

impl Default for ListenerPose {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            ahead: [0.0, 0.0, 1.0],
        }
    }
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
    /// Seeded random clock. Output `"gate"` is a 5 ms trigger; each interval is
    /// chosen in [min_interval_s, max_interval_s]. Emits ClockTick after an
    /// initial resting interval. Bounds are immutable; replace the module to change them.
    RandomClock {
        seed: u64,
        min_interval_s: f32,
        max_interval_s: f32,
    },
    /// Constant CV source. Output `"out"`, parameter `"value"`.
    Constant { value: f32 },
    /// Attack/decay envelope (fundsp's stock `adsr_live`, decaying to zero). Input
    /// `"gate"`: a rise past the gate threshold starts it. Output `"out"` (0..1).
    /// Parameters `"attack"` (seconds to peak) and `"decay"` (seconds back to zero) are
    /// fixed at creation; replace the module to change them.
    Envelope { attack_s: f32, decay_s: f32 },
    /// Voltage-controlled amplifier: `out = in * max(cv, 0) * gain`. Inputs `"in"`
    /// and `"cv"`, output `"out"`, parameter `"gain"`. With no CV it is closed.
    Vca { gain: f32 },
    /// Four-step sequencer (the snail shell; sync-snail-shell-sequencer). Inputs
    /// `"clock"` and `"reset"`, outputs `"gate"` and `"cv"`; parameters `"cv_1"`..`"cv_4"`
    /// (volts on V/OCT, 0..2, snapped to semitones) and `"gate_1"`..`"gate_4"` (0 or 1).
    /// Each rising edge at CLOCK moves to the next step (1 to 4, then 1) and emits
    /// [`Event::Step`]; a rising edge at RESET makes the next clock play step 1. GATE
    /// passes the clock's level while the current step is lit; CV holds its voltage.
    StepSequencer {
        cv: [f32; SEQUENCER_STEPS],
        gate: [f32; SEQUENCER_STEPS],
    },
}

/// Steps in a snail-shell sequencer: one per whorl.
pub const SEQUENCER_STEPS: usize = 4;

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

    /// The master output's input: the world bus, then the speakers.
    pub const fn output() -> Self {
        Self::new(ModuleId::OUTPUT, "in")
    }

    /// The speakers directly, skipping the world bus: for interface sounds (a collection
    /// confirmation), which the field recorder's meter must not hear.
    pub const fn ui_output() -> Self {
        Self::new(ModuleId::OUTPUT, "ui")
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
    /// Update the listener pose used by AudioNimbus propagation simulation.
    SetListener {
        pose: ListenerPose,
    },
    /// Set the linear wet gain for AudioNimbus per-emitter reflections (0..1).
    SetReflectionWetMix {
        value: f32,
    },
    /// Place an output already cabled to the master output in the world. `offset` is
    /// listener-local for binaural rendering; `position_world` feeds acoustic scene
    /// simulation. `volume` scales it (0..1). An output no longer cabled is ignored,
    /// since a rack can rebuild its voice in the same frame.
    SetEmitter {
        output: PortRef,
        offset: [f32; 3],
        position_world: [f32; 3],
        volume: f32,
    },
    /// Play a loaded sample once, straight to the speakers at `gain`: a sound effect
    /// (a plug snapping in), not part of any patch. Overlapping plays each get a voice.
    PlaySample {
        sample: SampleId,
        gain: f32,
    },
}

/// Something that happened on the audio thread, stamped with the audio clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A clock crossed a beat. `sample_time` is the exact frame of the tick on
    /// the audio clock (see [`Engine::sample_to_instant`](crate::Engine::sample_to_instant)).
    ClockTick { module: ModuleId, sample_time: u64 },
    /// A sequencer moved to `step` (0-based) on the exact frame `sample_time`, so the
    /// game can light its playhead at the heard instant.
    Step {
        module: ModuleId,
        step: u8,
        sample_time: u64,
    },
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
    #[error("module {module:?}'s {param:?} is fixed at creation; replace the module to change it")]
    FixedParam {
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
