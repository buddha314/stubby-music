//! The game-side engine: validates commands, edits the Firewheel graph, and
//! collects events from the audio thread.
//!
//! Firewheel already carries graph edits and parameter changes to the audio thread
//! over its own lock-free channel, so commands are applied to the graph here and
//! published by [`Engine::flush`]. Events come back over one `rtrb` ring per clock.

use crate::modules::{
    ClockConfig, ClockNode, ConstantNode, EnvelopeNode, EventMailbox, VcaNode, VcoConfig, VcoNode,
    Waveform,
};
use crate::protocol::{Command, EngineError, Event, ModuleId, ModuleKind, PortRef};
use audioadapter_buffers::direct::InterleavedSlice;
use firewheel::{
    ActivateInfo, FirewheelConfig, FirewheelContext,
    backend::BackendProcessInfo,
    channel_config::ChannelCount,
    diff::{Diff, PathBuilder},
    error::{AddEdgeError, UpdateError},
    node::{NodeID, StreamStatus},
    processor::FirewheelProcessor,
};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::time::{Duration, Instant};

/// Output is always stereo; mono module outputs feed both channels.
pub const OUTPUT_CHANNELS: usize = 2;

/// How long a removed sound source keeps its node while it fades out. Comfortably
/// longer than Firewheel's declick fade plus one audio block.
const RETIRE_SECONDS: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineConfig {
    /// Sample rate for offline rendering. A live stream uses the device's rate.
    pub sample_rate: u32,
    /// Largest block the offline renderer processes at once.
    pub max_block_frames: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            max_block_frames: 1024,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum NodeState {
    Vco(VcoNode),
    Clock(ClockNode),
    Constant(ConstantNode),
    Envelope(EnvelopeNode),
    Vca(VcaNode),
}

struct Module {
    kind: ModuleKind,
    node: NodeID,
    state: NodeState,
}

pub struct Engine {
    cx: FirewheelContext,
    modules: HashMap<ModuleId, Module>,
    /// Single-cable inputs: input → the output feeding it.
    cables: HashMap<PortRef, PortRef>,
    /// Outputs cabled to the master output.
    to_master: HashSet<PortRef>,
    pending_params: HashMap<(ModuleId, &'static str), f32>,
    mailbox: EventMailbox,
    event_rings: Vec<(ModuleId, rtrb::Consumer<Event>)>,
    /// Removed modules still fading out: node, and the audio-clock frame after
    /// which it can be deleted.
    retiring: Vec<(NodeID, u64)>,
    stream: Option<firewheel::cpal::CpalStream>,
}

fn firewheel_config() -> FirewheelConfig {
    FirewheelConfig {
        num_graph_inputs: ChannelCount::ZERO,
        num_graph_outputs: ChannelCount::STEREO,
        initial_node_capacity: 512,
        initial_edge_capacity: 1024,
        ..Default::default()
    }
}

impl Engine {
    fn new() -> Self {
        Self {
            cx: FirewheelContext::new(firewheel_config()),
            modules: HashMap::new(),
            cables: HashMap::new(),
            to_master: HashSet::new(),
            pending_params: HashMap::new(),
            mailbox: EventMailbox::default(),
            event_rings: Vec::new(),
            retiring: Vec::new(),
            stream: None,
        }
    }

    /// Start the engine on the default output device.
    pub fn start() -> Result<Self, EngineError> {
        let mut engine = Self::new();
        let stream = firewheel::cpal::CpalStream::new(&mut engine.cx, Default::default())
            .map_err(|e| EngineError::Device(e.to_string()))?;
        engine.stream = Some(stream);
        Ok(engine)
    }

    /// An engine with no audio device, plus the renderer that plays the audio
    /// thread's part. Used by tests and anything that renders to a buffer.
    pub fn offline(config: EngineConfig) -> Result<(Self, OfflineRenderer), EngineError> {
        let sample_rate =
            NonZeroU32::new(config.sample_rate).ok_or(EngineError::Config("sample_rate is 0"))?;
        let max_block = NonZeroU32::new(config.max_block_frames)
            .ok_or(EngineError::Config("max_block_frames is 0"))?;
        let mut engine = Self::new();
        let processor = engine
            .cx
            .activate(ActivateInfo {
                sample_rate,
                max_block_frames: max_block,
                num_stream_in_channels: 0,
                num_stream_out_channels: OUTPUT_CHANNELS as u32,
                input_to_output_latency_seconds: 0.0,
            })
            .map_err(|e| EngineError::Device(e.to_string()))?;
        let renderer = OfflineRenderer {
            processor,
            sample_rate: config.sample_rate,
            max_block: config.max_block_frames as usize,
            frames_rendered: 0,
        };
        Ok((engine, renderer))
    }

    /// Validate a command and apply it to the graph. Takes effect on the audio
    /// thread at the next [`flush`](Self::flush). Never blocks.
    pub fn send(&mut self, command: Command) -> Result<(), EngineError> {
        match command {
            Command::AddModule { id, kind } => self.add_module(id, kind),
            Command::RemoveModule { id } => self.remove_module(id),
            Command::Connect { from, to } => self.connect(from, to),
            Command::Disconnect { from, to } => self.disconnect(from, to),
            Command::SetParam {
                module,
                param,
                value,
            } => {
                let spec = self
                    .module(module)?
                    .kind
                    .param(param)
                    .ok_or(EngineError::UnknownParam { module, param })?;
                let name = spec.name;
                self.pending_params
                    .insert((module, name), value.clamp(spec.min, spec.max));
                Ok(())
            }
        }
    }

    /// Publish everything sent since the last flush to the audio thread. Call once
    /// per frame; Firewheel also uses this call to free nodes removed earlier.
    ///
    /// On [`EngineError::QueueFull`] the edits stay queued; call again next frame.
    pub fn flush(&mut self) -> Result<(), EngineError> {
        for ((id, param), value) in std::mem::take(&mut self.pending_params) {
            let Some(module) = self.modules.get_mut(&id) else {
                continue;
            };
            let mut queue = self.cx.event_queue(module.node);
            match (&mut module.state, param) {
                (NodeState::Vco(old), "freq") => {
                    let new = VcoNode {
                        freq: value,
                        ..*old
                    };
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                (NodeState::Clock(old), "bpm") => {
                    let new = ClockNode { bpm: value };
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                (NodeState::Constant(old), "value") => {
                    let new = ConstantNode { value };
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                (NodeState::Envelope(old), "attack" | "decay") => {
                    let new = if param == "attack" {
                        EnvelopeNode {
                            attack: value,
                            ..*old
                        }
                    } else {
                        EnvelopeNode {
                            decay: value,
                            ..*old
                        }
                    };
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                (NodeState::Vca(old), "gain") => {
                    let new = VcaNode {
                        gain: value,
                        ..*old
                    };
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                _ => {}
            }
        }
        let now = self.cx.audio_clock().samples.0.max(0) as u64;
        let cx = &mut self.cx;
        self.retiring.retain(|&(node, after)| {
            if now < after {
                return true;
            }
            let _ = cx.remove_node(node);
            false
        });
        if let Some(stream) = &mut self.stream {
            stream.log_status();
        }
        self.cx.update().map_err(|e| match e {
            UpdateError::MsgChannelFull => EngineError::QueueFull,
            other => EngineError::Graph(other.to_string()),
        })
    }

    /// Move every event the audio thread has produced into `out`, oldest first per clock.
    pub fn drain_events(&mut self, out: &mut Vec<Event>) {
        if let Ok(mut mailbox) = self.mailbox.lock() {
            self.event_rings.append(&mut mailbox);
        }
        for (_, ring) in &mut self.event_rings {
            while let Ok(event) = ring.pop() {
                out.push(event);
            }
        }
        // A processor that was rebuilt or removed abandons its ring.
        self.event_rings
            .retain(|(_, ring)| !ring.is_abandoned() || !ring.is_empty());
    }

    pub fn sample_rate(&self) -> Option<u32> {
        self.cx.stream_info().map(|info| info.sample_rate.get())
    }

    /// Frames processed by the audio thread so far, corrected for time since the
    /// last block. This is the clock [`Event`] timestamps are on.
    pub fn audio_clock(&self) -> u64 {
        self.cx.audio_clock_corrected().samples.0.max(0) as u64
    }

    /// Predict the wall-clock instant at which frame `sample_time` is processed,
    /// so visuals can be scheduled against audio time (design D6).
    pub fn sample_to_instant(&self, sample_time: u64) -> Option<Instant> {
        let sample_rate = self.sample_rate()? as f64;
        let now = Instant::now();
        let clock = self.cx.audio_clock_corrected().samples.0;
        let delta = (sample_time as i64 - clock) as f64 / sample_rate;
        if delta >= 0.0 {
            now.checked_add(Duration::from_secs_f64(delta))
        } else {
            now.checked_sub(Duration::from_secs_f64(-delta))
        }
    }

    /// Nodes in the Firewheel graph, including the graph's own input/output nodes
    /// and removed modules still fading out. For diagnostics and tests.
    pub fn graph_node_count(&self) -> usize {
        self.cx.nodes().count()
    }

    fn module(&self, id: ModuleId) -> Result<&Module, EngineError> {
        self.modules.get(&id).ok_or(EngineError::UnknownModule(id))
    }

    fn add_module(&mut self, id: ModuleId, kind: ModuleKind) -> Result<(), EngineError> {
        if id == ModuleId::OUTPUT {
            return Err(EngineError::ReservedModule(id));
        }
        if self.modules.contains_key(&id) {
            return Err(EngineError::DuplicateModule(id));
        }
        let (node, state) = match kind {
            ModuleKind::SineVco { freq_hz } | ModuleKind::SquareVco { freq_hz } => {
                let waveform = match kind {
                    ModuleKind::SquareVco { .. } => Waveform::Square,
                    _ => Waveform::Sine,
                };
                let state = VcoNode {
                    freq: freq_hz,
                    enabled: true,
                };
                let config = VcoConfig { waveform };
                (self.cx.add_node(state, Some(config)), NodeState::Vco(state))
            }
            ModuleKind::Clock { bpm } => {
                let state = ClockNode { bpm };
                let config = ClockConfig {
                    module: id,
                    mailbox: self.mailbox.clone(),
                };
                (
                    self.cx.add_node(state, Some(config)),
                    NodeState::Clock(state),
                )
            }
            ModuleKind::Constant { value } => {
                let state = ConstantNode { value };
                (self.cx.add_node(state, None), NodeState::Constant(state))
            }
            ModuleKind::Envelope { attack_s, decay_s } => {
                let state = EnvelopeNode {
                    attack: attack_s,
                    decay: decay_s,
                };
                (self.cx.add_node(state, None), NodeState::Envelope(state))
            }
            ModuleKind::Vca { gain } => {
                let state = VcaNode {
                    gain,
                    enabled: true,
                };
                (self.cx.add_node(state, None), NodeState::Vca(state))
            }
        };
        let node = node.map_err(|e| EngineError::Graph(e.to_string()))?;
        self.modules.insert(id, Module { kind, node, state });
        Ok(())
    }

    fn remove_module(&mut self, id: ModuleId) -> Result<(), EngineError> {
        if id == ModuleId::OUTPUT {
            return Err(EngineError::ReservedModule(id));
        }
        let module = self
            .modules
            .remove(&id)
            .ok_or(EngineError::UnknownModule(id))?;
        // Sound sources fade out now and are deleted once the fade has finished.
        // Gates and CV switch instantly (see `SourceFade`).
        let mut queue = self.cx.event_queue(module.node);
        let fading = match module.state {
            NodeState::Vco(old) => {
                VcoNode {
                    enabled: false,
                    ..old
                }
                .diff(&old, PathBuilder::default(), &mut queue);
                true
            }
            NodeState::Vca(old) => {
                VcaNode {
                    enabled: false,
                    ..old
                }
                .diff(&old, PathBuilder::default(), &mut queue);
                true
            }
            NodeState::Clock(_) | NodeState::Constant(_) | NodeState::Envelope(_) => false,
        };
        if fading {
            let sample_rate = self.sample_rate().unwrap_or(48_000) as f64;
            let after = self.cx.audio_clock().samples.0.max(0) as u64
                + (RETIRE_SECONDS * sample_rate) as u64;
            self.retiring.push((module.node, after));
        } else {
            let _ = self.cx.remove_node(module.node);
        }
        // Firewheel removes a node's edges with it; our tables forget them now, since
        // the module no longer exists as far as the game is concerned.
        self.cables
            .retain(|to, from| to.module != id && from.module != id);
        self.to_master.retain(|from| from.module != id);
        self.pending_params.retain(|(m, _), _| *m != id);
        self.event_rings.retain(|(m, _)| *m != id);
        Ok(())
    }

    fn output_port(&self, port: PortRef) -> Result<(NodeID, u32), EngineError> {
        let module = self.module(port.module).map_err(|_| match port.module {
            ModuleId::OUTPUT => EngineError::NotAnOutput(port),
            id => EngineError::UnknownModule(id),
        })?;
        let index = module
            .kind
            .outputs()
            .iter()
            .position(|&p| p == port.port)
            .ok_or(EngineError::NotAnOutput(port))?;
        Ok((module.node, index as u32))
    }

    fn input_port(&self, port: PortRef) -> Result<(NodeID, u32), EngineError> {
        let module = self.module(port.module)?;
        let index = module
            .kind
            .inputs()
            .iter()
            .position(|&p| p == port.port)
            .ok_or(EngineError::NotAnInput(port))?;
        Ok((module.node, index as u32))
    }

    fn connect(&mut self, from: PortRef, to: PortRef) -> Result<(), EngineError> {
        let (src, src_port) = self.output_port(from)?;
        let map_edge_error = |e: AddEdgeError| match e {
            AddEdgeError::CycleDetected => EngineError::Cycle,
            other => EngineError::Graph(other.to_string()),
        };
        if to == PortRef::output() {
            if self.to_master.contains(&from) {
                return Ok(());
            }
            let out = self.cx.graph_out_node_id();
            // Mono to both speakers. Feeding both channels identically also keeps
            // clear of Firewheel 0.14's silent-channel bug (stubby-music#4).
            self.cx
                .connect(src, out, &[(src_port, 0), (src_port, 1)], false)
                .map_err(map_edge_error)?;
            self.to_master.insert(from);
            return Ok(());
        }
        if to.module == ModuleId::OUTPUT {
            return Err(EngineError::NotAnInput(to));
        }
        let (dst, dst_port) = self.input_port(to)?;
        if self.cables.contains_key(&to) {
            return Err(EngineError::InputOccupied(to));
        }
        self.cx
            .connect(src, dst, &[(src_port, dst_port)], true)
            .map_err(map_edge_error)?;
        self.cables.insert(to, from);
        Ok(())
    }

    fn disconnect(&mut self, from: PortRef, to: PortRef) -> Result<(), EngineError> {
        let (src, src_port) = self.output_port(from)?;
        if to == PortRef::output() {
            if !self.to_master.remove(&from) {
                return Err(EngineError::NotConnected { from, to });
            }
            let out = self.cx.graph_out_node_id();
            self.cx
                .disconnect(src, out, &[(src_port, 0), (src_port, 1)]);
            return Ok(());
        }
        let (dst, dst_port) = self.input_port(to)?;
        if self.cables.get(&to) != Some(&from) {
            return Err(EngineError::NotConnected { from, to });
        }
        self.cx.disconnect(src, dst, &[(src_port, dst_port)]);
        self.cables.remove(&to);
        Ok(())
    }
}

/// Plays the audio thread's part when there is no device: each `render` call is
/// one or more audio callbacks.
pub struct OfflineRenderer {
    processor: FirewheelProcessor,
    sample_rate: u32,
    max_block: usize,
    frames_rendered: u64,
}

impl OfflineRenderer {
    /// Render interleaved stereo into `out`. Realtime-safe: no allocation.
    pub fn render(&mut self, out: &mut [f32]) {
        let empty: [f32; 0] = [];
        for chunk in out.chunks_mut(self.max_block * OUTPUT_CHANNELS) {
            let frames = chunk.len() / OUTPUT_CHANNELS;
            let (Ok(input), Ok(mut output)) = (
                InterleavedSlice::new(&empty[..], 0, frames),
                InterleavedSlice::new_mut(chunk, OUTPUT_CHANNELS, frames),
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
                        self.frames_rendered as f64 / self.sample_rate as f64,
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

    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }
}
