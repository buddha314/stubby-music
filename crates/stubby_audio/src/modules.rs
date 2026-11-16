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
    AudioUnit, BufferVec, F32x, MAX_BUFFER_SIZE, Setting, Shared, adsr_live, clip_to, follow, pass,
    shared, sine, square, tick, var,
};
use std::sync::{Arc, Mutex};

const CLOCK_EVENT_CAPACITY: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    /// Only whole values (a clock's BPM, a whole multiple of the 1 BPM master).
    pub whole: bool,
}

const VCO_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "freq",
    min: 0.01,
    max: 20_000.0,
    default: 261.63,
    whole: false,
}];
const CLOCK_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "bpm",
    min: 1.0,
    max: 999.0,
    default: 120.0,
    whole: true,
}];
const CONSTANT_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "value",
    min: -10.0,
    max: 10.0,
    default: 0.0,
    whole: false,
}];
const ENVELOPE_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        name: "attack",
        min: 0.0005,
        max: 10.0,
        default: 0.002,
        whole: false,
    },
    ParamSpec {
        name: "decay",
        min: 0.001,
        max: 30.0,
        default: 0.2,
        whole: false,
    },
];
const VCA_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "gain",
    min: 0.0,
    max: 1.0,
    default: 1.0,
    whole: false,
}];

/// How long a VCA takes to follow a change of its gain parameter.
const GAIN_SMOOTH_SECONDS: f64 = 0.005;

impl ModuleKind {
    pub fn inputs(&self) -> &'static [&'static str] {
        match self {
            ModuleKind::SineVco { .. } | ModuleKind::SquareVco { .. } => &["voct"],
            ModuleKind::Clock { .. }
            | ModuleKind::RandomClock { .. }
            | ModuleKind::Constant { .. } => &[],
            ModuleKind::Envelope { .. } => &["gate"],
            ModuleKind::Vca { .. } => &["in", "cv"],
        }
    }

    pub fn outputs(&self) -> &'static [&'static str] {
        match self {
            ModuleKind::SineVco { .. }
            | ModuleKind::SquareVco { .. }
            | ModuleKind::Constant { .. }
            | ModuleKind::Envelope { .. }
            | ModuleKind::Vca { .. } => &["out"],
            ModuleKind::Clock { .. } => &["gate", "phase"],
            ModuleKind::RandomClock { .. } => &["gate"],
        }
    }

    pub fn params(&self) -> &'static [ParamSpec] {
        match self {
            ModuleKind::SineVco { .. } | ModuleKind::SquareVco { .. } => VCO_PARAMS,
            ModuleKind::Clock { .. } => CLOCK_PARAMS,
            ModuleKind::RandomClock { .. } => &[],
            ModuleKind::Constant { .. } => CONSTANT_PARAMS,
            ModuleKind::Envelope { .. } => ENVELOPE_PARAMS,
            ModuleKind::Vca { .. } => VCA_PARAMS,
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
            ModuleKind::Clock { .. } => "Clock",
            ModuleKind::RandomClock { .. } => "RandomClock",
            ModuleKind::Constant { .. } => "Constant",
            ModuleKind::Envelope { .. } => "Envelope",
            ModuleKind::Vca { .. } => "Vca",
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
            "Clock",
            "Constant",
            "Envelope",
            "Vca",
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
        assert_eq!(ModuleKind::default_of("SawVco"), None);
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
            for (o, s) in chunk.iter_mut().zip(self.out.channel_f32(0)) {
                *o = *s;
            }
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
        Ok(FundspProc::new(Box::new(unit), 1, cx))
    }
}

/// A fundsp unit run as a node: inputs and output copied through fundsp buffers.
struct FundspProc {
    unit: Box<dyn AudioUnit>,
    inputs: BufferVec,
    out: BufferVec,
}

impl FundspProc {
    fn new(mut unit: Box<dyn AudioUnit>, inputs: usize, cx: ConstructProcessorContext) -> Self {
        unit.set_sample_rate(cx.stream_info.sample_rate.get() as f64);
        unit.reset();
        unit.allocate();
        Self {
            unit,
            inputs: BufferVec::new(inputs),
            out: BufferVec::new(1),
        }
    }

    fn run(&mut self, inputs: &[&[f32]], out: &mut [f32]) {
        for (chunk_index, chunk) in out.chunks_mut(MAX_BUFFER_SIZE).enumerate() {
            let start = chunk_index * MAX_BUFFER_SIZE;
            for (channel, input) in inputs.iter().enumerate() {
                for (i, &v) in input.iter().skip(start).take(chunk.len()).enumerate() {
                    self.inputs.set_f32(channel, i, v);
                }
            }
            self.unit.process(
                chunk.len(),
                &self.inputs.buffer_ref(),
                &mut self.out.buffer_mut(),
            );
            for (o, s) in chunk.iter_mut().zip(self.out.channel_f32(0)) {
                *o = *s;
            }
        }
    }
}

impl AudioNodeProcessor for FundspProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        // Timing is fixed at creation; the engine refuses changes before they get here.
        for _ in events.drain_patches::<EnvelopeNode>() {}
    }

    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        let (Some(gate), Some(out)) = (buffers.inputs.first(), buffers.outputs.first_mut()) else {
            return ProcessStatus::ClearAllOutputs;
        };
        self.run(&[gate], out);
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
            fundsp: FundspProc::new(Box::new(unit), 2, cx),
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
        let [input, cv, ..] = buffers.inputs else {
            return ProcessStatus::ClearAllOutputs;
        };
        let Some(out) = buffers.outputs.first_mut() else {
            return ProcessStatus::ClearAllOutputs;
        };
        self.fundsp.run(&[input, cv], out);
        self.fade.apply(buffers.outputs, info.frames, extra);
        ProcessStatus::OutputsModified
    }
}
