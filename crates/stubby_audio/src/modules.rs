//! Synth modules: port and parameter metadata, plus the Firewheel nodes that run them.
//!
//! Everything in an `AudioNodeProcessor` runs on the realtime thread: no allocation,
//! no locks, no panics. `construct_processor` runs on the game thread and may allocate.

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
use fundsp::prelude32::{AudioUnit, BufferVec, F32x, MAX_BUFFER_SIZE, Setting, sine, square};
use std::sync::{Arc, Mutex};

const CLOCK_EVENT_CAPACITY: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

const VCO_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "freq",
    min: 1.0,
    max: 20_000.0,
    default: 261.63,
}];
const CLOCK_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "bpm",
    min: 1.0,
    max: 999.0,
    default: 120.0,
}];
const CONSTANT_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "value",
    min: -10.0,
    max: 10.0,
    default: 0.0,
}];
const ENVELOPE_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        name: "attack",
        min: 0.0005,
        max: 10.0,
        default: 0.002,
    },
    ParamSpec {
        name: "decay",
        min: 0.001,
        max: 30.0,
        default: 0.2,
    },
];
const VCA_PARAMS: &[ParamSpec] = &[ParamSpec {
    name: "gain",
    min: 0.0,
    max: 1.0,
    default: 1.0,
}];

/// How long a VCA takes to follow a change of its gain parameter.
const GAIN_SMOOTH_SECONDS: f64 = 0.005;

impl ModuleKind {
    pub fn inputs(&self) -> &'static [&'static str] {
        match self {
            ModuleKind::SineVco { .. } | ModuleKind::SquareVco { .. } => &["voct"],
            ModuleKind::Clock { .. } | ModuleKind::Constant { .. } => &[],
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
        }
    }

    pub fn params(&self) -> &'static [ParamSpec] {
        match self {
            ModuleKind::SineVco { .. } | ModuleKind::SquareVco { .. } => VCO_PARAMS,
            ModuleKind::Clock { .. } => CLOCK_PARAMS,
            ModuleKind::Constant { .. } => CONSTANT_PARAMS,
            ModuleKind::Envelope { .. } => ENVELOPE_PARAMS,
            ModuleKind::Vca { .. } => VCA_PARAMS,
        }
    }

    pub fn param(&self, name: &str) -> Option<&'static ParamSpec> {
        self.params().iter().find(|p| p.name == name)
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
        let sample_rate = cx.stream_info.sample_rate.get() as f64;
        Ok(ClockProc {
            module: config.module,
            events: producer,
            sample_rate,
            samples_per_beat: samples_per_beat(sample_rate, self.bpm),
            pos: 0.0,
            tick_pending: true,
        })
    }
}

fn samples_per_beat(sample_rate: f64, bpm: f32) -> f64 {
    sample_rate * 60.0 / (bpm.max(1.0) as f64)
}

struct ClockProc {
    module: ModuleId,
    events: rtrb::Producer<Event>,
    sample_rate: f64,
    samples_per_beat: f64,
    /// Frames since the current beat began. Counting frames, not accumulating a
    /// phase increment, keeps the clock drift-free.
    pos: f64,
    tick_pending: bool,
}

impl AudioNodeProcessor for ClockProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for patch in events.drain_patches::<ClockNode>() {
            match patch {
                ClockNodePatch::Bpm(bpm) => {
                    // Keep the position within the beat across a tempo change.
                    let phase = self.pos / self.samples_per_beat;
                    self.samples_per_beat = samples_per_beat(self.sample_rate, bpm);
                    self.pos = phase * self.samples_per_beat;
                }
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
            if self.tick_pending {
                self.tick_pending = false;
                // A full ring drops the tick rather than block or allocate.
                let _ = self.events.push(Event::ClockTick {
                    module: self.module,
                    sample_time: block_start + i as u64,
                });
            }
            *g = if self.pos < self.samples_per_beat * 0.5 {
                GATE_HIGH
            } else {
                0.0
            };
            *p = (self.pos / self.samples_per_beat) as f32;
            self.pos += 1.0;
            if self.pos >= self.samples_per_beat {
                self.pos -= self.samples_per_beat;
                self.tick_pending = true;
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
        let mut proc = EnvelopeProc {
            sample_rate: cx.stream_info.sample_rate.get() as f64,
            attack_step: 0.0,
            decay_coeff: 0.0,
            level: 0.0,
            attacking: false,
            gate_high: false,
        };
        proc.set_attack(self.attack);
        proc.set_decay(self.decay);
        Ok(proc)
    }
}

struct EnvelopeProc {
    sample_rate: f64,
    /// Level added per sample while attacking.
    attack_step: f32,
    /// Per-sample multiplier while decaying; reaches -60 dB after `decay` seconds.
    decay_coeff: f32,
    level: f32,
    attacking: bool,
    gate_high: bool,
}

impl EnvelopeProc {
    fn set_attack(&mut self, seconds: f32) {
        self.attack_step = (1.0 / (seconds.max(1e-4) as f64 * self.sample_rate)) as f32;
    }

    fn set_decay(&mut self, seconds: f32) {
        let samples = seconds.max(1e-4) as f64 * self.sample_rate;
        self.decay_coeff = 0.001f64.powf(1.0 / samples) as f32;
    }
}

impl AudioNodeProcessor for EnvelopeProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for patch in events.drain_patches::<EnvelopeNode>() {
            match patch {
                EnvelopeNodePatch::Attack(s) => self.set_attack(s),
                EnvelopeNodePatch::Decay(s) => self.set_decay(s),
            }
        }
    }

    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        let (Some(gate), Some(out)) = (buffers.inputs.first(), buffers.outputs.first_mut()) else {
            return ProcessStatus::ClearAllOutputs;
        };
        for (o, &g) in out.iter_mut().zip(gate.iter()) {
            let high = g > GATE_THRESHOLD;
            if high && !self.gate_high {
                // Retrigger from the current level, so the output never jumps.
                self.attacking = true;
            }
            self.gate_high = high;
            if self.attacking {
                self.level += self.attack_step;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.attacking = false;
                }
            } else {
                self.level *= self.decay_coeff;
                if self.level < 1e-6 {
                    self.level = 0.0;
                }
            }
            *o = self.level;
        }
        ProcessStatus::OutputsModified
    }
}

// ---------------------------------------------------------------- VCA

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
        let sample_rate = cx.stream_info.sample_rate.get() as f64;
        Ok(VcaProc {
            target: self.gain,
            gain: self.gain,
            smooth: (1.0 - (-1.0 / (GAIN_SMOOTH_SECONDS * sample_rate)).exp()) as f32,
            fade: SourceFade::new(),
        })
    }
}

struct VcaProc {
    target: f32,
    /// The gain actually applied, easing toward `target` so changes never zipper.
    gain: f32,
    smooth: f32,
    fade: SourceFade,
}

impl AudioNodeProcessor for VcaProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, extra: &mut ProcExtra) {
        for patch in events.drain_patches::<VcaNode>() {
            match patch {
                VcaNodePatch::Gain(g) => self.target = g,
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
        for ((o, &x), &c) in out.iter_mut().zip(input.iter()).zip(cv.iter()) {
            self.gain += (self.target - self.gain) * self.smooth;
            *o = x * c.max(0.0) * self.gain;
        }
        self.fade.apply(buffers.outputs, info.frames, extra);
        ProcessStatus::OutputsModified
    }
}
