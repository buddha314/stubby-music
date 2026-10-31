//! Firewheel behind the spike boundary.
//!
//! Game side: `FirewheelContext`; edits are published with `update()`.
//! Audio side: `FirewheelProcessor`, driven here by hand so the suite runs offline.

use audioadapter_buffers::direct::InterleavedSlice;
use firewheel::{
    ActivateInfo, FirewheelConfig, FirewheelContext,
    backend::BackendProcessInfo,
    channel_config::{ChannelConfig, ChannelCount},
    diff::{Diff, Patch, PathBuilder},
    event::ProcEvents,
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
        NodeError, NodeID, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus, StreamStatus,
    },
    processor::FirewheelProcessor,
};
use spike_common::{BLOCK, Backend, CHANNELS, Id, SR};
use std::{collections::HashMap, num::NonZeroU32, time::Duration};

/// Phase-continuous sine with a patchable frequency.
#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
struct SineNode {
    freq: f32,
    amp: f32,
}

impl AudioNode for SineNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("sine")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::ZERO,
                num_outputs: ChannelCount::MONO,
            }))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let sr_recip = cx.stream_info.sample_rate_recip;
        Ok(SineProc {
            phase: 0.0,
            inc: self.freq as f64 * sr_recip,
            amp: self.amp,
            sr_recip,
        })
    }
}

struct SineProc {
    phase: f64,
    inc: f64,
    amp: f32,
    sr_recip: f64,
}

impl AudioNodeProcessor for SineProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for patch in events.drain_patches::<SineNode>() {
            match patch {
                SineNodePatch::Freq(f) => self.inc = f as f64 * self.sr_recip,
                SineNodePatch::Amp(a) => self.amp = a,
            }
        }
    }

    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        for s in buffers.outputs[0].iter_mut() {
            *s = (self.phase * std::f64::consts::TAU).sin() as f32 * self.amp;
            self.phase = (self.phase + self.inc).fract();
        }
        ProcessStatus::OutputsModified
    }
}

/// Same sine, but the DSP is fundsp's SIMD `sine()` hosted inside a firewheel node.
/// Measures the hybrid option: firewheel's graph, fundsp's DSP library.
#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
struct FunSineNode {
    freq: f32,
    amp: f32,
}

impl AudioNode for FunSineNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("fundsp_sine")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::ZERO,
                num_outputs: ChannelCount::MONO,
            }))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        // Runs on the game thread, so allocating the unit and buffers here is fine.
        let mut unit: Box<dyn fundsp::prelude32::AudioUnit> = Box::new(fundsp::prelude32::sine());
        unit.set_sample_rate(cx.stream_info.sample_rate.get() as f64);
        unit.allocate();
        Ok(FunSineProc {
            unit,
            freq: self.freq,
            amp: self.amp,
            inbuf: fundsp::prelude32::BufferVec::new(1),
            outbuf: fundsp::prelude32::BufferVec::new(1),
        })
    }
}

struct FunSineProc {
    unit: Box<dyn fundsp::prelude32::AudioUnit>,
    freq: f32,
    amp: f32,
    inbuf: fundsp::prelude32::BufferVec,
    outbuf: fundsp::prelude32::BufferVec,
}

impl AudioNodeProcessor for FunSineProc {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for patch in events.drain_patches::<FunSineNode>() {
            match patch {
                FunSineNodePatch::Freq(f) => self.freq = f,
                FunSineNodePatch::Amp(a) => self.amp = a,
            }
        }
    }

    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        use fundsp::prelude32::{F32x, MAX_BUFFER_SIZE};
        self.inbuf.channel_mut(0).fill(F32x::splat(self.freq));
        for chunk in buffers.outputs[0].chunks_mut(MAX_BUFFER_SIZE) {
            self.unit.process(
                chunk.len(),
                &self.inbuf.buffer_ref(),
                &mut self.outbuf.buffer_mut(),
            );
            for (o, s) in chunk.iter_mut().zip(self.outbuf.channel_f32(0)) {
                *o = *s * self.amp;
            }
        }
        ProcessStatus::OutputsModified
    }
}

/// Gate high for the first half of each beat. Counts samples, so it never drifts.
#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
struct ClockNode {
    bpm: f64,
}

impl AudioNode for ClockNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("clock")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::ZERO,
                num_outputs: ChannelCount::MONO,
            }))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(ClockProc {
            samples_per_beat: cx.stream_info.sample_rate.get() as f64 * 60.0 / self.bpm,
            pos: 0.0,
        })
    }
}

struct ClockProc {
    samples_per_beat: f64,
    pos: f64,
}

impl AudioNodeProcessor for ClockProc {
    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        for s in buffers.outputs[0].iter_mut() {
            *s = if self.pos < self.samples_per_beat * 0.5 {
                1.0
            } else {
                0.0
            };
            self.pos += 1.0;
            if self.pos >= self.samples_per_beat {
                self.pos -= self.samples_per_beat;
            }
        }
        ProcessStatus::OutputsModified
    }
}

/// Writes zeros to every graph output and reports them as modified.
///
/// Works around a firewheel 0.14 bug: when a graph output channel is silent,
/// `process.rs` calls `fill_frames_with`, which zeros *all* channels for that block,
/// erasing channels already written. Keeping every output non-silent avoids it.
#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
struct KeepAlive {}

impl AudioNode for KeepAlive {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("keepalive")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::ZERO,
                num_outputs: ChannelCount::STEREO,
            }))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        _: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(KeepAliveProc)
    }
}

struct KeepAliveProc;

impl AudioNodeProcessor for KeepAliveProc {
    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        for ch in buffers.outputs.iter_mut() {
            ch.fill(0.0);
        }
        ProcessStatus::OutputsModified
    }
}

fn keep_alive(cx: &mut FirewheelContext) {
    let node = cx.add_node(KeepAlive {}, None).expect("add keepalive");
    let out = cx.graph_out_node_id();
    cx.connect(node, out, &[(0, 0), (1, 1)], false)
        .expect("connect keepalive");
}

/// The audio-thread half.
pub struct Audio {
    processor: FirewheelProcessor,
    frames_rendered: u64,
}

impl Audio {
    pub fn render(&mut self, out: &mut [f32]) {
        let frames = out.len() / CHANNELS;
        let empty: [f32; 0] = [];
        let (Ok(input), Ok(mut output)) = (
            InterleavedSlice::new(&empty[..], 0, frames),
            InterleavedSlice::new_mut(out, CHANNELS, frames),
        ) else {
            return;
        };
        self.processor.process(
            &input,
            &mut output,
            BackendProcessInfo {
                frames,
                process_timestamp: None,
                duration_since_stream_start: Duration::from_secs_f64(
                    self.frames_rendered as f64 / SR as f64,
                ),
                input_stream_status: StreamStatus::empty(),
                output_stream_status: StreamStatus::empty(),
                dropped_frames: 0,
                process_to_playback_delay: None,
            },
        );
        self.frames_rendered += frames as u64;
    }
}

pub struct FirewheelBackend {
    cx: FirewheelContext,
    audio: Option<Audio>,
    nodes: HashMap<Id, (NodeID, SineNode)>,
    fundsp_dsp: bool,
    clocks: HashMap<Id, NodeID>,
    next_id: Id,
}

fn config() -> FirewheelConfig {
    FirewheelConfig {
        num_graph_inputs: ChannelCount::ZERO,
        num_graph_outputs: ChannelCount::STEREO,
        initial_node_capacity: 512,
        initial_edge_capacity: 1024,
        ..Default::default()
    }
}

impl FirewheelBackend {
    /// Offline: activate the context and keep the processor to drive by hand.
    pub fn new(fundsp_dsp: bool) -> Self {
        let mut cx = FirewheelContext::new(config());
        let processor = cx
            .activate(ActivateInfo {
                sample_rate: NonZeroU32::new(SR).unwrap(),
                max_block_frames: NonZeroU32::new(BLOCK as u32).unwrap(),
                num_stream_in_channels: 0,
                num_stream_out_channels: CHANNELS as u32,
                input_to_output_latency_seconds: 0.0,
            })
            .expect("activate");
        keep_alive(&mut cx);
        cx.update().expect("update");
        Self {
            cx,
            audio: Some(Audio {
                processor,
                frames_rendered: 0,
            }),
            nodes: HashMap::new(),
            clocks: HashMap::new(),
            next_id: 0,
            fundsp_dsp,
        }
    }

    fn id(&mut self) -> Id {
        self.next_id += 1;
        self.next_id
    }
}

impl Backend for FirewheelBackend {
    fn name(&self) -> &'static str {
        if self.fundsp_dsp {
            "firewheel + fundsp DSP"
        } else {
            "firewheel (scalar f64 sine)"
        }
    }

    fn add_sine(&mut self, freq: f32, amp: f32) -> Id {
        let params = SineNode { freq, amp };
        let node = if self.fundsp_dsp {
            self.cx.add_node(FunSineNode { freq, amp }, None)
        } else {
            self.cx.add_node(params, None)
        }
        .expect("add sine");
        let out = self.cx.graph_out_node_id();
        self.cx
            .connect(node, out, &[(0, 0)], false)
            .expect("connect sine");
        let id = self.id();
        self.nodes.insert(id, (node, params));
        id
    }

    fn add_clock(&mut self, bpm: f64) -> Id {
        let node = self
            .cx
            .add_node(ClockNode { bpm }, None)
            .expect("add clock");
        let out = self.cx.graph_out_node_id();
        self.cx
            .connect(node, out, &[(0, 1)], false)
            .expect("connect clock");
        let id = self.id();
        self.clocks.insert(id, node);
        id
    }

    fn remove(&mut self, id: Id) {
        if let Some((node, _)) = self.nodes.remove(&id) {
            let _ = self.cx.remove_node(node);
        } else if let Some(node) = self.clocks.remove(&id) {
            let _ = self.cx.remove_node(node);
        }
    }

    fn set_freq(&mut self, id: Id, freq: f32) {
        if let Some((node, params)) = self.nodes.get_mut(&id) {
            let new = SineNode { freq, ..*params };
            new.diff(
                params,
                PathBuilder::default(),
                &mut self.cx.event_queue(*node),
            );
            *params = new;
        }
    }

    fn commit(&mut self) {
        self.cx.update().expect("update");
    }

    fn render_block(&mut self, out: &mut [f32]) {
        if let Some(audio) = &mut self.audio {
            audio.render(out);
        }
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("live") => live(),
        _ => {
            spike_common::run_suite(|| FirewheelBackend::new(false)).print();
            spike_common::run_suite(|| FirewheelBackend::new(true)).print();
        }
    }
}

/// Play through firewheel's own cpal backend so a human can listen for clicks.
fn live() {
    use firewheel::cpal::{CpalConfig, CpalStream};
    use std::{thread::sleep, time::Instant};

    let mut b = FirewheelBackend {
        cx: FirewheelContext::new(config()),
        audio: None,
        nodes: HashMap::new(),
        clocks: HashMap::new(),
        next_id: 0,
        fundsp_dsp: true,
    };
    keep_alive(&mut b.cx);
    let mut stream = CpalStream::new(&mut b.cx, CpalConfig::default()).expect("start stream");

    // The context must be updated regularly, like a game loop would.
    let mut run = |b: &mut FirewheelBackend, secs: u64| {
        let end = Instant::now() + Duration::from_secs(secs);
        while Instant::now() < end {
            b.commit();
            stream.log_status();
            sleep(Duration::from_millis(16));
        }
    };
    println!("0s  sine 220 Hz");
    let a = b.add_sine(220.0, 0.3);
    run(&mut b, 2);
    println!("2s  add sine 330 Hz");
    let c = b.add_sine(330.0, 0.3);
    run(&mut b, 2);
    println!("4s  first sine jumps to 440 Hz");
    b.set_freq(a, 440.0);
    run(&mut b, 2);
    println!("6s  remove 330 Hz");
    b.remove(c);
    run(&mut b, 2);
    println!("8s  done");
}
