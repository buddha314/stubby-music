//! Synth modules: port and parameter metadata, plus the Firewheel nodes that run them.
//!
//! Everything in an `AudioNodeProcessor` runs on the realtime thread: no allocation,
//! no locks, no panics. `construct_processor` runs on the game thread and may allocate.

use crate::MasterLockedClock;
use crate::protocol::{Event, ModuleId, ModuleKind};
use crate::signal::{GATE_HIGH, GATE_THRESHOLD, voct_to_hz};
use firewheel::{
    channel_config::{ChannelConfig, ChannelCount},
    diff::{Diff, Patch},
    dsp::declick::{DeclickFadeCurve, Declicker},
    event::ProcEvents,
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
        NodeError, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use fundsp::prelude32::{
    AudioUnit, BufferVec, F32x, MAX_BUFFER_SIZE, Setting, Shared, U2, U3, U4, adsr_live, clip_to,
    follow, highpass, lowpass, multisplit, pass, saw, shared, sine, square, sumi, tick, triangle,
    var,
};
use std::sync::{Arc, Mutex};

const CLOCK_EVENT_CAPACITY: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    /// Values snap to multiples of this (0: continuous). 1 for a clock's BPM (a whole
    /// multiple of the 1 BPM master); 1/12 for a sequencer step's V/OCT (semitones).
    pub quantum: f32,
}
impl ParamSpec {
    /// Clamp a value to the range and snap it to the quantum.
    pub fn conform(&self, value: f32) -> f32 {
        let value = value.clamp(self.min, self.max);
        if self.quantum > 0. {
            ((value / self.quantum).round() * self.quantum).clamp(self.min, self.max)
        } else {
            value
        }
    }
}

/// A semitone on V/OCT (1 V per octave).
pub const SEMITONE_VOLTS: f32 = 1. / 12.;

const VCO_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "freq",
    min: 0.01,
    max: 20_000.0,
    default: 261.63,
    quantum: 0.0,
}];
const CLOCK_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "bpm",
    min: 1.0,
    max: 999.0,
    default: 120.0,
    quantum: 1.0,
}];
const fn step_cv(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        min: 0.0,
        max: 2.0,
        default: 0.0,
        quantum: SEMITONE_VOLTS,
    }
}
const fn step_gate(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        min: 0.0,
        max: 1.0,
        default: 0.0,
        quantum: 1.0,
    }
}
const SEQUENCER_PARAMS: &[ParamSpec] = &[
    step_cv("cv_1"),
    step_cv("cv_2"),
    step_cv("cv_3"),
    step_cv("cv_4"),
    step_gate("gate_1"),
    step_gate("gate_2"),
    step_gate("gate_3"),
    step_gate("gate_4"),
];
const CONSTANT_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "value",
    min: -10.0,
    max: 10.0,
    default: 0.0,
    quantum: 0.0,
}];
const ENVELOPE_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        name: "attack",
        min: 0.0005,
        max: 10.0,
        default: 0.002,
        quantum: 0.0,
    },
    ParamSpec {
        name: "decay",
        min: 0.001,
        max: 30.0,
        default: 0.2,
        quantum: 0.0,
    },
];
const VCA_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "gain",
    min: 0.0,
    max: 1.0,
    default: 1.0,
    quantum: 0.0,
}];

pub(crate) const SCOPE_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        name: "time",
        min: 0.002,
        max: 0.25,
        default: 0.02,
        quantum: 0.0,
    },
    ParamSpec {
        name: "scale",
        min: 0.1,
        max: 4.0,
        default: 0.5,
        quantum: 0.0,
    },
];

const VCF_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        name: "cutoff",
        min: 32.7,
        max: 8372.0,
        default: 1046.5, // C6: on the semitone grid, an octave above the purifier's C5
        quantum: 0.0,
    },
    ParamSpec {
        name: "res",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        quantum: 0.0,
    },
];

const MIXER_PARAMS: &[ParamSpec] = &[
    mixer_level("level_1"),
    mixer_level("level_2"),
    mixer_level("level_3"),
    mixer_level("level_4"),
];

const fn mixer_level(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        min: 0.0,
        max: 1.0,
        default: 0.5,
        quantum: 0.0,
    }
}

/// How long a level control takes to follow a gain change.
const GAIN_SMOOTH_SECONDS: f64 = 0.005;

impl ModuleKind {
    pub fn inputs(&self) -> &'static [&'static str] {
        match self {
            ModuleKind::SamplePlayer => &["trigger"],
            ModuleKind::SineVco { .. }
            | ModuleKind::SquareVco { .. }
            | ModuleKind::SawVco { .. }
            | ModuleKind::TriangleVco { .. } => &["voct"],
            ModuleKind::Clock { .. }
            | ModuleKind::RandomClock { .. }
            | ModuleKind::Constant { .. } => &[],
            ModuleKind::Envelope { .. } => &["gate"],
            ModuleKind::Vca { .. } => &["in", "cv"],
            ModuleKind::Mixer { .. } => &["in_1", "in_2", "in_3", "in_4"],
            ModuleKind::Vcf { .. } | ModuleKind::Scope { .. } => &["in"],
            ModuleKind::StepSequencer { .. } => &["clock", "reset"],
        }
    }

    pub fn outputs(&self) -> &'static [&'static str] {
        match self {
            ModuleKind::SineVco { .. }
            | ModuleKind::SquareVco { .. }
            | ModuleKind::SawVco { .. }
            | ModuleKind::TriangleVco { .. }
            | ModuleKind::Constant { .. }
            | ModuleKind::Envelope { .. }
            | ModuleKind::Vca { .. }
            | ModuleKind::Scope { .. }
            | ModuleKind::SamplePlayer => &["out"],
            ModuleKind::Mixer { .. } => &["mix"],
            ModuleKind::Vcf { .. } => &["lpf", "hpf"],
            ModuleKind::Clock { .. } => &["gate", "phase"],
            ModuleKind::RandomClock { .. } => &["gate"],
            ModuleKind::StepSequencer { .. } => &["gate", "cv"],
        }
    }

    pub fn params(&self) -> &'static [ParamSpec] {
        match self {
            ModuleKind::SineVco { .. }
            | ModuleKind::SquareVco { .. }
            | ModuleKind::SawVco { .. }
            | ModuleKind::TriangleVco { .. } => VCO_PARAMS,
            ModuleKind::Clock { .. } => CLOCK_PARAMS,
            ModuleKind::RandomClock { .. } | ModuleKind::SamplePlayer => &[],
            ModuleKind::Constant { .. } => CONSTANT_PARAMS,
            ModuleKind::Envelope { .. } => ENVELOPE_PARAMS,
            ModuleKind::Vca { .. } => VCA_PARAMS,
            ModuleKind::Mixer { .. } => MIXER_PARAMS,
            ModuleKind::Vcf { .. } => VCF_PARAMS,
            ModuleKind::Scope { .. } => SCOPE_PARAMS,
            ModuleKind::StepSequencer { .. } => SEQUENCER_PARAMS,
        }
    }

    pub fn param(&self, name: &str) -> Option<&'static ParamSpec> {
        self.params().iter().find(|p| p.name == name)
    }

    /// The variant's name, as data files name it (`"SquareVco"`).
    pub fn name(&self) -> &'static str {
        match self {
            ModuleKind::SineVco { .. } => "SineVco",
            ModuleKind::SquareVco { .. } => "SquareVco",
            ModuleKind::SawVco { .. } => "SawVco",
            ModuleKind::TriangleVco { .. } => "TriangleVco",
            ModuleKind::Clock { .. } => "Clock",
            ModuleKind::RandomClock { .. } => "RandomClock",
            ModuleKind::Constant { .. } => "Constant",
            ModuleKind::Envelope { .. } => "Envelope",
            ModuleKind::Vca { .. } => "Vca",
            ModuleKind::Mixer { .. } => "Mixer",
            ModuleKind::Vcf { .. } => "Vcf",
            ModuleKind::Scope { .. } => "Scope",
            ModuleKind::SamplePlayer => "SamplePlayer",
            ModuleKind::StepSequencer { .. } => "StepSequencer",
        }
    }

    /// A module of the named kind at its parameters' defaults. `None` for an unknown
    /// name, and for `RandomClock`, whose seed and bounds are not parameters.
    pub fn default_of(name: &str) -> Option<ModuleKind> {
        Self::from_params(name, |_| None)
    }

    /// A module of the named kind, each parameter taken from `value` or its default.
    /// Values are not clamped here; callers validate against [`ParamSpec`] ranges.
    pub fn from_params(name: &str, value: impl Fn(&str) -> Option<f32>) -> Option<ModuleKind> {
        let get = |specs: &[ParamSpec], param: &str| {
            let default = specs
                .iter()
                .find(|p| p.name == param)
                .map_or(0.0, |p| p.default);
            value(param).unwrap_or(default)
        };
        Some(match name {
            "SineVco" => ModuleKind::SineVco {
                freq_hz: get(VCO_PARAMS, "freq"),
            },
            "SquareVco" => ModuleKind::SquareVco {
                freq_hz: get(VCO_PARAMS, "freq"),
            },
            "SawVco" => ModuleKind::SawVco {
                freq_hz: get(VCO_PARAMS, "freq"),
            },
            "TriangleVco" => ModuleKind::TriangleVco {
                freq_hz: get(VCO_PARAMS, "freq"),
            },
            "Clock" => ModuleKind::Clock {
                bpm: get(CLOCK_PARAMS, "bpm"),
            },
            "Constant" => ModuleKind::Constant {
                value: get(CONSTANT_PARAMS, "value"),
            },
            "Envelope" => ModuleKind::Envelope {
                attack_s: get(ENVELOPE_PARAMS, "attack"),
                decay_s: get(ENVELOPE_PARAMS, "decay"),
            },
            "Vca" => ModuleKind::Vca {
                gain: get(VCA_PARAMS, "gain"),
            },
            "Mixer" => ModuleKind::Mixer {
                levels: std::array::from_fn(|i| get(MIXER_PARAMS, MIXER_PARAMS[i].name)),
            },
            "Vcf" => ModuleKind::Vcf {
                cutoff_hz: get(VCF_PARAMS, "cutoff"),
                resonance: get(VCF_PARAMS, "res"),
            },
            "SamplePlayer" => ModuleKind::SamplePlayer,
            "Scope" => ModuleKind::Scope {
                time: get(SCOPE_PARAMS, "time"),
                scale: get(SCOPE_PARAMS, "scale"),
            },
            "StepSequencer" => ModuleKind::StepSequencer {
                cv: ["cv_1", "cv_2", "cv_3", "cv_4"].map(|p| get(SEQUENCER_PARAMS, p)),
                gate: ["gate_1", "gate_2", "gate_3", "gate_4"].map(|p| get(SEQUENCER_PARAMS, p)),
            },
            _ => return None,
        })
    }
}

#[cfg(test)]
mod kind_tests {
    use super::*;

    #[test]
    fn every_buildable_kind_takes_exactly_its_params() {
        for name in [
            "SineVco",
            "SquareVco",
            "SawVco",
            "TriangleVco",
            "Clock",
            "Constant",
            "Envelope",
            "Vca",
            "Mixer",
            "Vcf",
            "Scope",
            "StepSequencer",
        ] {
            let kind = ModuleKind::default_of(name).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(kind.name(), name);
            // Each listed parameter reaches the variant: set it to a marker and read it back.
            for spec in kind.params() {
                let marked = ModuleKind::from_params(name, |p| (p == spec.name).then_some(0.123))
                    .unwrap_or_else(|| panic!("{name}"));
                assert_ne!(
                    marked, kind,
                    "{name}.{} does not reach the module",
                    spec.name
                );
            }
        }
        assert_eq!(ModuleKind::default_of("RandomClock"), None);
    }
}

fn node_info(
    name: &'static str,
    num_inputs: ChannelCount,
    num_outputs: ChannelCount,
) -> AudioNodeInfo {
    AudioNodeInfo::new()
        .debug_name(name)
        .channel_config(ChannelConfig {
            num_inputs,
            num_outputs,
        })
}

/// Fades a sound source in when it is created and out before it is removed, so
/// patch edits never click. Firewheel only declicks bypass toggles, not node
/// add/remove. Gate and CV modules do not use this: fading a gate moves its edge.
struct SourceFade {
    declick: Declicker,
    started: bool,
}

impl SourceFade {
    fn new() -> Self {
        Self {
            declick: Declicker::SettledAt0,
            started: false,
        }
    }

    fn set_enabled(&mut self, enabled: bool, extra: &ProcExtra) {
        self.started = true;
        self.declick.fade_to_enabled(enabled, &extra.declick_values);
    }

    fn apply(&mut self, outputs: &mut [&mut [f32]], frames: usize, extra: &ProcExtra) {
        if !self.started {
            self.set_enabled(true, extra);
        }
        self.declick.process(
            outputs,
            0..frames,
            &extra.declick_values,
            1.0,
            DeclickFadeCurve::Linear,
        );
    }
}

// ---------------------------------------------------------------- VCOs

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Waveform {
    #[default]
    Sine,
    /// Band-limited (fundsp wavetable), so it does not alias.
    Square,
    Saw,
    Triangle,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct VcoConfig {
    pub waveform: Waveform,
}

#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct VcoNode {
    pub freq: f32,
    /// Set false to fade out before removal.
    pub enabled: bool,
}

impl AudioNode for VcoNode {
    type Configuration = VcoConfig;

    fn info(&self, _: &VcoConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info("vco", ChannelCount::MONO, ChannelCount::MONO))
    }

    fn construct_processor(
        &self,
        config: &VcoConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let mut unit: Box<dyn AudioUnit> = match config.waveform {
            Waveform::Sine => Box::new(sine()),
            Waveform::Square => Box::new(square()),
            Waveform::Saw => Box::new(saw()),
            Waveform::Triangle => Box::new(triangle()),
        };
        unit.set_sample_rate(cx.stream_info.sample_rate.get() as f64);
        // Start at phase 0 so offline renders are deterministic.
        unit.set(Setting::phase(0.0));
        unit.reset();
        unit.allocate();
        Ok(VcoProc {
            unit,
            freq: self.freq,
            hz: BufferVec::new(1),
            out: BufferVec::new(1),
            fade: SourceFade::new(),
        })
    }
}

struct VcoProc {
    /// A fundsp oscillator: input 0 is frequency in Hz.
    unit: Box<dyn AudioUnit>,
    freq: f32,
    hz: BufferVec,
    out: BufferVec,
    fade: SourceFade,
}

impl AudioNodeProcessor for VcoProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, extra: &mut ProcExtra) {
        for patch in events.drain_patches::<VcoNode>() {
            match patch {
                VcoNodePatch::Freq(f) => self.freq = f,
                VcoNodePatch::Enabled(on) => self.fade.set_enabled(on, extra),
            }
        }
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        extra: &mut ProcExtra,
    ) -> ProcessStatus {
        let (Some(cv), Some(out)) = (buffers.inputs.first(), buffers.outputs.first_mut()) else {
            return ProcessStatus::ClearAllOutputs;
        };
        let cv_silent = info.in_silence_mask.is_channel_silent(0);
        let cv_constant = info.in_constant_mask.is_channel_constant(0);
        let constant_hz = if cv_silent {
            Some(self.freq)
        } else if cv_constant {
            cv.first().map(|&v| voct_to_hz(self.freq, v))
        } else {
            None
        };

        for (chunk_index, chunk) in out.chunks_mut(MAX_BUFFER_SIZE).enumerate() {
            let start = chunk_index * MAX_BUFFER_SIZE;
            match constant_hz {
                Some(hz) => self.hz.channel_mut(0).fill(F32x::splat(hz)),
                None => {
                    for (i, &v) in cv.iter().skip(start).take(chunk.len()).enumerate() {
                        self.hz.set_f32(0, i, voct_to_hz(self.freq, v));
                    }
                }
            }
            self.unit.process(
                chunk.len(),
                &self.hz.buffer_ref(),
                &mut self.out.buffer_mut(),
            );
            chunk.copy_from_slice(&self.out.channel_f32(0)[..chunk.len()]);
        }
        self.fade.apply(buffers.outputs, info.frames, extra);
        ProcessStatus::OutputsModified
    }
}

// ---------------------------------------------------------------- Clock

#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct ClockNode {
    pub bpm: f32,
}

/// Where a clock's processor hands the game the receiving end of its event ring.
///
/// Firewheel can rebuild processors (for example when the audio stream restarts),
/// so each construction makes a fresh ring and posts its consumer here. The mutex is
/// only ever locked on the game thread.
pub(crate) type EventMailbox = Arc<Mutex<Vec<(ModuleId, rtrb::Consumer<Event>)>>>;

#[derive(Clone, Default)]
pub(crate) struct ClockConfig {
    pub module: ModuleId,
    pub mailbox: EventMailbox,
}

impl AudioNode for ClockNode {
    type Configuration = ClockConfig;

    fn info(&self, _: &ClockConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info("clock", ChannelCount::ZERO, ChannelCount::STEREO))
    }

    fn construct_processor(
        &self,
        config: &ClockConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let (producer, consumer) = rtrb::RingBuffer::new(CLOCK_EVENT_CAPACITY);
        if let Ok(mut mailbox) = config.mailbox.lock() {
            mailbox.push((config.module, consumer));
        }
        Ok(ClockProc {
            module: config.module,
            events: producer,
            clock: MasterLockedClock::new(cx.stream_info.sample_rate.get(), self.bpm),
        })
    }
}

struct ClockProc {
    module: ModuleId,
    events: rtrb::Producer<Event>,
    /// Locked to the 1 BPM master on the engine's sample counter (`lock-clocks-to-master`).
    clock: MasterLockedClock,
}

impl AudioNodeProcessor for ClockProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for patch in events.drain_patches::<ClockNode>() {
            match patch {
                ClockNodePatch::Bpm(bpm) => self.clock.set_bpm(bpm),
            }
        }
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        let [gate, phase, ..] = buffers.outputs else {
            return ProcessStatus::ClearAllOutputs;
        };
        let block_start = info.clock_samples.0.max(0) as u64;
        for (i, (g, p)) in gate.iter_mut().zip(phase.iter_mut()).enumerate() {
            let sample = block_start + i as u64;
            let (_, beat_phase) = self.clock.at(sample);
            *p = beat_phase;
            *g = if beat_phase < 0.5 { GATE_HIGH } else { 0. };
            if self.clock.tick_at(sample) {
                let _ = self.events.push(Event::ClockTick {
                    module: self.module,
                    sample_time: sample,
                });
            }
        }
        ProcessStatus::OutputsModified
    }
}

// ---------------------------------------------------------------- Step sequencer

/// The snail-shell sequencer's settings: each step's voltage and whether it is lit.
#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SequencerNode {
    pub cv_1: f32,
    pub cv_2: f32,
    pub cv_3: f32,
    pub cv_4: f32,
    pub gate_1: f32,
    pub gate_2: f32,
    pub gate_3: f32,
    pub gate_4: f32,
}
impl SequencerNode {
    pub fn new(cv: [f32; crate::SEQUENCER_STEPS], gate: [f32; crate::SEQUENCER_STEPS]) -> Self {
        let [cv_1, cv_2, cv_3, cv_4] = cv;
        let [gate_1, gate_2, gate_3, gate_4] = gate;
        Self {
            cv_1,
            cv_2,
            cv_3,
            cv_4,
            gate_1,
            gate_2,
            gate_3,
            gate_4,
        }
    }
    /// The node with one parameter changed (`cv_<n>` or `gate_<n>`).
    pub fn with(self, param: &str, value: f32) -> Self {
        let mut node = self;
        match param {
            "cv_1" => node.cv_1 = value,
            "cv_2" => node.cv_2 = value,
            "cv_3" => node.cv_3 = value,
            "cv_4" => node.cv_4 = value,
            "gate_1" => node.gate_1 = value,
            "gate_2" => node.gate_2 = value,
            "gate_3" => node.gate_3 = value,
            "gate_4" => node.gate_4 = value,
            _ => {}
        }
        node
    }
}

/// The sequencer's stepping, apart from Firewheel so it is tested sample by sample.
#[derive(Debug, Clone)]
pub struct StepState {
    pub cv: [f32; crate::SEQUENCER_STEPS],
    pub gate: [f32; crate::SEQUENCER_STEPS],
    /// The current step, or `None` before the first clock (and after a reset).
    step: Option<usize>,
    clock_high: bool,
    reset_high: bool,
}
impl StepState {
    pub fn new(cv: [f32; crate::SEQUENCER_STEPS], gate: [f32; crate::SEQUENCER_STEPS]) -> Self {
        Self {
            cv,
            gate,
            step: None,
            clock_high: false,
            reset_high: false,
        }
    }
    /// One sample: the clock and reset inputs in, `(gate, cv, the step moved to)` out.
    pub fn tick(&mut self, clock: f32, reset: f32) -> (f32, f32, Option<usize>) {
        let reset_now = reset > GATE_HIGH * 0.5;
        if reset_now && !self.reset_high {
            self.step = None;
        }
        self.reset_high = reset_now;
        let clock_now = clock > GATE_HIGH * 0.5;
        let mut moved = None;
        if clock_now && !self.clock_high {
            let next = self.step.map_or(0, |s| (s + 1) % crate::SEQUENCER_STEPS);
            self.step = Some(next);
            moved = Some(next);
        }
        self.clock_high = clock_now;
        let lit = self.step.is_some_and(|s| self.gate[s] >= 0.5);
        let gate = if lit && clock_now { clock } else { 0. };
        let cv = self.cv[self.step.unwrap_or(0)];
        (gate, cv, moved)
    }
}

impl AudioNode for SequencerNode {
    type Configuration = ClockConfig;

    fn info(&self, _: &ClockConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info(
            "step_sequencer",
            ChannelCount::STEREO,
            ChannelCount::STEREO,
        ))
    }

    fn construct_processor(
        &self,
        config: &ClockConfig,
        _: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let (producer, consumer) = rtrb::RingBuffer::new(CLOCK_EVENT_CAPACITY);
        if let Ok(mut mailbox) = config.mailbox.lock() {
            mailbox.push((config.module, consumer));
        }
        Ok(SequencerProc {
            module: config.module,
            events: producer,
            state: StepState::new(
                [self.cv_1, self.cv_2, self.cv_3, self.cv_4],
                [self.gate_1, self.gate_2, self.gate_3, self.gate_4],
            ),
        })
    }
}

struct SequencerProc {
    module: ModuleId,
    events: rtrb::Producer<Event>,
    state: StepState,
}

impl AudioNodeProcessor for SequencerProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for patch in events.drain_patches::<SequencerNode>() {
            let s = &mut self.state;
            match patch {
                SequencerNodePatch::Cv1(v) => s.cv[0] = v,
                SequencerNodePatch::Cv2(v) => s.cv[1] = v,
                SequencerNodePatch::Cv3(v) => s.cv[2] = v,
                SequencerNodePatch::Cv4(v) => s.cv[3] = v,
                SequencerNodePatch::Gate1(v) => s.gate[0] = v,
                SequencerNodePatch::Gate2(v) => s.gate[1] = v,
                SequencerNodePatch::Gate3(v) => s.gate[2] = v,
                SequencerNodePatch::Gate4(v) => s.gate[3] = v,
            }
        }
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        let [clock, reset, ..] = buffers.inputs else {
            return ProcessStatus::ClearAllOutputs;
        };
        let [gate, cv, ..] = buffers.outputs else {
            return ProcessStatus::ClearAllOutputs;
        };
        let block_start = info.clock_samples.0.max(0) as u64;
        for i in 0..info.frames {
            let (g, c, moved) = self.state.tick(clock[i], reset[i]);
            gate[i] = g;
            cv[i] = c;
            if let Some(step) = moved {
                let _ = self.events.push(Event::Step {
                    module: self.module,
                    step: step as u8,
                    sample_time: block_start + i as u64,
                });
            }
        }
        ProcessStatus::OutputsModified
    }
}

#[cfg(test)]
mod sequencer_tests {
    use super::*;
    /// Run a clock of `period` samples (high for the first half) for `samples`.
    fn run(
        state: &mut StepState,
        period: usize,
        samples: usize,
        reset_at: Option<usize>,
    ) -> Vec<(f32, f32, Option<usize>)> {
        (0..samples)
            .map(|i| {
                let clock = if i % period < period / 2 {
                    GATE_HIGH
                } else {
                    0.
                };
                let reset = if reset_at.is_some_and(|r| i == r) {
                    GATE_HIGH
                } else {
                    0.
                };
                state.tick(clock, reset)
            })
            .collect()
    }

    #[test]
    fn steps_one_to_four_on_rising_edges_and_wrap() {
        let mut state = StepState::new([0.; 4], [0.; 4]);
        let out = run(&mut state, 10, 50, None);
        let moves: Vec<_> = out
            .iter()
            .enumerate()
            .filter_map(|(i, o)| o.2.map(|s| (i, s)))
            .collect();
        assert_eq!(moves, [(0, 0), (10, 1), (20, 2), (30, 3), (40, 0)]);
    }

    #[test]
    fn gate_passes_the_clock_pulse_on_lit_steps_only() {
        // Steps 2 and 4 lit: a backbeat.
        let mut state = StepState::new([0.; 4], [0., 1., 0., 1.]);
        let out = run(&mut state, 10, 40, None);
        let gated: Vec<bool> = (0..4).map(|beat| out[beat * 10].0 > 0.5).collect();
        assert_eq!(gated, [false, true, false, true]);
        // The gate lasts as long as the clock's pulse, not the whole step.
        assert!(out[12].0 > 0.5 && out[16].0 == 0.);
    }

    #[test]
    fn cv_holds_each_steps_voltage() {
        let mut state = StepState::new([0., 0.25, 7. / 12., 1.], [1.; 4]);
        let out = run(&mut state, 10, 40, None);
        let held: Vec<f32> = (0..4).map(|beat| out[beat * 10 + 7].1).collect();
        assert_eq!(held, [0., 0.25, 7. / 12., 1.]);
    }

    #[test]
    fn reset_makes_the_next_clock_play_step_one() {
        let mut state = StepState::new([0.; 4], [0.; 4]);
        let out = run(&mut state, 10, 40, Some(25));
        let moves: Vec<_> = out.iter().filter_map(|o| o.2).collect();
        assert_eq!(moves, [0, 1, 2, 0]);
    }

    #[test]
    fn step_voltages_snap_to_semitones() {
        let spec = SEQUENCER_PARAMS[0];
        assert!((spec.conform(0.3) - 4. / 12.).abs() < 1e-6);
        assert_eq!(spec.conform(5.), 2.);
        assert_eq!(SEQUENCER_PARAMS[4].conform(0.7), 1.);
    }
}

// ---------------------------------------------------------------- Constant

#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct ConstantNode {
    pub value: f32,
}

impl AudioNode for ConstantNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info(
            "constant",
            ChannelCount::ZERO,
            ChannelCount::MONO,
        ))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        _: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(ConstantProc { value: self.value })
    }
}

struct ConstantProc {
    value: f32,
}

impl AudioNodeProcessor for ConstantProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for patch in events.drain_patches::<ConstantNode>() {
            match patch {
                ConstantNodePatch::Value(v) => self.value = v,
            }
        }
    }

    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        for ch in buffers.outputs.iter_mut() {
            ch.fill(self.value);
        }
        ProcessStatus::OutputsModified
    }
}

// ---------------------------------------------------------------- Envelope

/// fundsp's stock envelope (`adsr_live`), as attack then decay to silence, started by a
/// gate rising past the gate threshold. Its timing is fixed when the module is created, as
/// a random clock's bounds are; re-add the module to change it. (Stock components until
/// the game runs; sound design comes after.)
#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct EnvelopeNode {
    pub attack: f32,
    pub decay: f32,
}

impl AudioNode for EnvelopeNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info(
            "envelope",
            ChannelCount::MONO,
            ChannelCount::MONO,
        ))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let (attack, decay) = (self.attack.max(1e-4), self.decay.max(1e-4));
        // A gate is high above GATE_THRESHOLD; the envelope starts when its input turns positive.
        // `adsr_live` starts only on a rise it sees after a low input, so a gate already high
        // on the first sample (a clock's first beat) would be missed: delay it one sample.
        let unit = (tick() - GATE_THRESHOLD) >> adsr_live(attack, decay, 0.0, decay);
        Ok(FundspProc::new(Box::new(unit), cx))
    }
}

/// A fundsp unit run as a node: inputs and outputs copied through fundsp buffers.
struct FundspProc {
    unit: Box<dyn AudioUnit>,
    inputs: BufferVec,
    out: BufferVec,
}

impl FundspProc {
    fn new(mut unit: Box<dyn AudioUnit>, cx: ConstructProcessorContext) -> Self {
        unit.set_sample_rate(cx.stream_info.sample_rate.get() as f64);
        unit.reset();
        unit.allocate();
        let inputs = BufferVec::new(unit.inputs());
        let out = BufferVec::new(unit.outputs());
        Self { unit, inputs, out }
    }

    fn run(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], frames: usize) {
        for start in (0..frames).step_by(MAX_BUFFER_SIZE) {
            let len = (frames - start).min(MAX_BUFFER_SIZE);
            for (channel, input) in inputs.iter().enumerate() {
                for (i, &v) in input[start..start + len].iter().enumerate() {
                    self.inputs.set_f32(channel, i, v);
                }
            }
            self.unit
                .process(len, &self.inputs.buffer_ref(), &mut self.out.buffer_mut());
            for (channel, output) in outputs.iter_mut().enumerate() {
                output[start..start + len].copy_from_slice(&self.out.channel_f32(channel)[..len]);
            }
        }
    }
}

impl AudioNodeProcessor for FundspProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        // Timing is fixed at creation; the engine refuses changes before they get here.
        for _ in events.drain_patches::<EnvelopeNode>() {}
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        self.run(buffers.inputs, buffers.outputs, info.frames);
        ProcessStatus::OutputsModified
    }
}

// ---------------------------------------------------------------- VCA

/// A VCA from fundsp's stock parts: `in * max(cv, 0) * gain`, with the gain smoothed by
/// `follow` so changes never zipper. With no CV it is closed.
#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct VcaNode {
    pub gain: f32,
    /// Set false to fade out before removal. A VCA's output is audio.
    pub enabled: bool,
}

impl AudioNode for VcaNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info("vca", ChannelCount::STEREO, ChannelCount::MONO))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let gain = shared(self.gain);
        let unit = (pass() * (pass() >> clip_to(0.0, f32::MAX)))
            * (var(&gain) >> follow(GAIN_SMOOTH_SECONDS as f32));
        Ok(VcaProc {
            fundsp: FundspProc::new(Box::new(unit), cx),
            gain,
            fade: SourceFade::new(),
        })
    }
}

struct VcaProc {
    fundsp: FundspProc,
    /// The gain target, read by the unit's `var`; setting it does not allocate.
    gain: Shared,
    fade: SourceFade,
}

impl AudioNodeProcessor for VcaProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, extra: &mut ProcExtra) {
        for patch in events.drain_patches::<VcaNode>() {
            match patch {
                VcaNodePatch::Gain(g) => self.gain.set_value(g),
                VcaNodePatch::Enabled(on) => self.fade.set_enabled(on, extra),
            }
        }
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        extra: &mut ProcExtra,
    ) -> ProcessStatus {
        self.fundsp
            .run(buffers.inputs, buffers.outputs, info.frames);
        self.fade.apply(buffers.outputs, info.frames, extra);
        ProcessStatus::OutputsModified
    }
}

// ---------------------------------------------------------------- Mixer

#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct MixerNode {
    pub levels: [f32; 4],
    pub enabled: bool,
}

impl MixerNode {
    pub fn with(mut self, param: &str, value: f32) -> Self {
        if let Some(index) = MIXER_PARAMS.iter().position(|spec| spec.name == param) {
            self.levels[index] = value;
        }
        self
    }
}

impl AudioNode for MixerNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info(
            "mixer",
            ChannelCount::from(4usize),
            ChannelCount::MONO,
        ))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let gains = self.levels.map(shared);
        let unit = sumi::<U4, _, _>(|i| {
            pass() * (var(&gains[i as usize]) >> follow(GAIN_SMOOTH_SECONDS as f32))
        });
        Ok(MixerProc {
            fundsp: FundspProc::new(Box::new(unit), cx),
            gains,
            fade: SourceFade::new(),
        })
    }
}

struct MixerProc {
    fundsp: FundspProc,
    gains: [Shared; 4],
    fade: SourceFade,
}

impl AudioNodeProcessor for MixerProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, extra: &mut ProcExtra) {
        for patch in events.drain_patches::<MixerNode>() {
            match patch {
                MixerNodePatch::Levels((index, value)) => {
                    if let Some(gain) = self.gains.get(index) {
                        gain.set_value(value);
                    }
                }
                MixerNodePatch::Enabled(enabled) => self.fade.set_enabled(enabled, extra),
            }
        }
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        extra: &mut ProcExtra,
    ) -> ProcessStatus {
        self.fundsp
            .run(buffers.inputs, buffers.outputs, info.frames);
        self.fade.apply(buffers.outputs, info.frames, extra);
        ProcessStatus::OutputsModified
    }
}

// ---------------------------------------------------------------- VCF

#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
pub(crate) struct VcfNode {
    pub cutoff: f32,
    pub resonance: f32,
    pub enabled: bool,
}

impl AudioNode for VcfNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(node_info("vcf", ChannelCount::MONO, ChannelCount::STEREO))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let max_cutoff = cx.stream_info.sample_rate.get() as f32 * 0.45;
        let cutoff = shared(self.cutoff);
        let resonance = shared(self.resonance);
        // Zero resonance gives the Butterworth response. Keep the highest Q finite.
        let hz = (var(&cutoff) >> clip_to(0.0, max_cutoff)) >> follow(GAIN_SMOOTH_SECONDS as f32);
        let q = (var(&resonance) >> clip_to(0.0, 1.0) >> follow(GAIN_SMOOTH_SECONDS as f32))
            * (8.0 - std::f32::consts::FRAC_1_SQRT_2)
            + std::f32::consts::FRAC_1_SQRT_2;
        let unit = (pass() | hz | q) >> multisplit::<U3, U2>() >> (lowpass() | highpass());
        Ok(VcfProc {
            fundsp: FundspProc::new(Box::new(unit), cx),
            cutoff,
            resonance,
            fade: SourceFade::new(),
        })
    }
}

struct VcfProc {
    fundsp: FundspProc,
    cutoff: Shared,
    resonance: Shared,
    fade: SourceFade,
}

impl AudioNodeProcessor for VcfProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, extra: &mut ProcExtra) {
        for patch in events.drain_patches::<VcfNode>() {
            match patch {
                VcfNodePatch::Cutoff(hz) => self.cutoff.set_value(hz),
                VcfNodePatch::Resonance(resonance) => self.resonance.set_value(resonance),
                VcfNodePatch::Enabled(enabled) => self.fade.set_enabled(enabled, extra),
            }
        }
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        extra: &mut ProcExtra,
    ) -> ProcessStatus {
        self.fundsp
            .run(buffers.inputs, buffers.outputs, info.frames);
        self.fade.apply(buffers.outputs, info.frames, extra);
        ProcessStatus::OutputsModified
    }
}
