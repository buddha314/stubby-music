//! The game-side engine: validates commands, edits the Firewheel graph, and
//! collects events from the audio thread.
//!
//! Firewheel already carries graph edits and parameter changes to the audio thread
//! over its own lock-free channel, so commands are applied to the graph here and
//! published by [`Engine::flush`]. Events come back over one `rtrb` ring per clock.

#[cfg(feature = "audionimbus")]
use crate::audionimbus_spatial::{
    AudioNimbusBinauralNode, AudioNimbusDirectSimulation, AudioNimbusSpatialContext,
    DirectEffectParamsShared, DirectEmitterInput,
};
use crate::modules::{
    ClockConfig, ClockNode, ConstantNode, EnvelopeNode, EventMailbox, MixerNode, SCOPE_PARAMS,
    SequencerNode, VcaNode, VcfNode, VcoConfig, VcoNode, Waveform,
};
#[cfg(feature = "omg_audio")]
use crate::omg_spatial::{OmgSpatialNode, trace_snapshot};
use crate::protocol::{
    AcousticSceneGeometry, Bus, Command, EngineError, Event, ListenerPose, ModuleId, ModuleKind,
    PortRef, SampleId,
};
use crate::random_clock::RandomClockNode;
use crate::sample_voice::{RetireSample, SampleVoice};
use audioadapter_buffers::direct::InterleavedSlice;
use firewheel::event::NodeEventType;
use firewheel::nodes::{
    spatial_basic::SpatialBasicNode,
    triple_buffer::{TripleBufferConfig, TripleBufferNode, TripleBufferState, WindowSize},
    volume::{VolumeNode, VolumeNodeConfig},
};
use firewheel::{
    ActivateInfo, FirewheelConfig, FirewheelContext,
    backend::BackendProcessInfo,
    channel_config::{ChannelCount, NonZeroChannelCount},
    diff::{Diff, PathBuilder},
    error::{AddEdgeError, UpdateError},
    node::{NodeID, StreamStatus},
    processor::FirewheelProcessor,
};
use fundsp::wave::{Wave, WavePlayer};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::Mutex;
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
    SamplePlayer,
    Vco(VcoNode),
    Clock(ClockNode),
    RandomClock,
    Constant(ConstantNode),
    /// Timing is fixed at creation, so there is nothing to diff later.
    Envelope,
    Vca(VcaNode),
    Mixer(MixerNode),
    Vcf(VcfNode),
    Scope {
        tap: NodeID,
        capture: TripleBufferNode,
    },
    Sequencer(SequencerNode),
}

struct Module {
    kind: ModuleKind,
    node: NodeID,
    state: NodeState,
}

struct Emitter {
    node: NodeID,
    params: EmitterParams,
}

enum EmitterParams {
    Basic(SpatialBasicNode),
    #[cfg(feature = "audionimbus")]
    AudioNimbus(AudioNimbusBinauralNode),
    #[cfg(feature = "omg_audio")]
    Omg {
        position: [f32; 3],
        listener: ListenerPose,
    },
}

pub struct Engine {
    cx: FirewheelContext,
    modules: HashMap<ModuleId, Module>,
    /// Single-cable inputs: input → the output feeding it.
    cables: HashMap<PortRef, PortRef>,
    /// Outputs cabled to the master output.
    to_master: HashSet<PortRef>,
    /// Outputs cabled straight to the speakers (`PortRef::ui_output`).
    to_ui: HashSet<PortRef>,
    /// Master outputs placed in the world, each through its own spatial node.
    emitters: HashMap<PortRef, Emitter>,
    listener_pose: ListenerPose,
    #[cfg(feature = "audionimbus")]
    audionimbus_spatial: Option<Arc<AudioNimbusSpatialContext>>,
    #[cfg(feature = "audionimbus")]
    audionimbus_simulation: Option<AudioNimbusDirectSimulation>,
    #[cfg(feature = "audionimbus")]
    reflection_wet_mix: f32,
    #[cfg(feature = "omg_audio")]
    omg_geometry: AcousticSceneGeometry,
    pending_params: HashMap<(ModuleId, &'static str), f32>,
    mailbox: EventMailbox,
    event_rings: Vec<(ModuleId, rtrb::Consumer<Event>)>,
    /// Removed modules still fading out: node, and the audio-clock frame after
    /// which it can be deleted.
    retiring: Vec<(NodeID, u64)>,
    /// Loaded one-shot samples, one buffer per channel, at the engine's sample rate.
    samples: HashMap<SampleId, Vec<Vec<f32>>>,
    sample_waves: HashMap<SampleId, Arc<Wave>>,
    stream: Option<firewheel::cpal::CpalStream>,
    /// The world bus: every module output and emitter is heard through it, then the
    /// speakers. One-shot UI samples skip it, so its meter hears only the world.
    world: NodeID,
    /// The player's volume stages: world bus -> `world_gain` -> `master`, interface
    /// sounds -> `interface` -> `master` -> speakers.
    volumes: Volumes,
    world_meter: WorldMeter,
    scopes: ScopeCaptures,
}

/// The world bus's level, readable from any thread (lock-free atomics). The field
/// recorder's meter reads it (sync-recorder-feedback); UI sounds never reach it.
#[derive(Clone)]
pub struct WorldMeter(firewheel::nodes::peak_meter::PeakMeterState<2>);
impl WorldMeter {
    /// The loudest channel's peak over the latest audio block, in dBFS
    /// (`f32::NEG_INFINITY` for silence).
    pub fn peak_db(&self) -> f32 {
        let [left, right] = self.0.peak_gain_db(-120.);
        left.max(right)
    }
}

/// Recent scope samples. Firewheel owns the realtime writer. Readers run off the
/// callback and may skip a frame while the control thread edits the registry.
#[derive(Clone, Default)]
pub(crate) struct ScopeCaptures(Arc<Mutex<HashMap<ModuleId, TripleBufferState>>>);

impl ScopeCaptures {
    pub fn read(&self, id: ModuleId, trace: &mut [f32]) -> bool {
        let mut state = {
            let Ok(scopes) = self.0.try_lock() else {
                return false;
            };
            let Some(state) = scopes.get(&id) else {
                return false;
            };
            state.clone()
        };
        let mut guard = state.output();
        let Some(data) = guard.data() else {
            return false;
        };
        let Some(channel) = data.buffer.iter_channels().next() else {
            return false;
        };
        let Some(samples) = channel
            .get(..data.frames)
            .filter(|samples| !samples.is_empty())
        else {
            return false;
        };
        let last = trace.len().saturating_sub(1).max(1);
        for (i, position) in trace.iter_mut().enumerate() {
            *position = samples[i * (samples.len() - 1) / last];
        }
        true
    }
}

fn scope_window(seconds: f32) -> WindowSize {
    let spec = &SCOPE_PARAMS[0];
    let seconds = if seconds.is_nan() {
        spec.default
    } else {
        spec.conform(seconds)
    };
    WindowSize::Seconds(f64::from(seconds))
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

/// The player's volume stages, one per [`Bus`].
struct Volumes {
    nodes: [(Bus, NodeID); 3],
    levels: [VolumeNode; 3],
}

impl Volumes {
    fn index(&self, bus: Bus) -> usize {
        self.nodes.iter().position(|(b, _)| *b == bus).unwrap_or(0)
    }

    fn node(&self, bus: Bus) -> NodeID {
        self.nodes[self.index(bus)].1
    }
}

impl Engine {
    fn new() -> Result<Self, EngineError> {
        let graph = |e: String| EngineError::Graph(format!("world bus: {e}"));
        let mut cx = FirewheelContext::new(firewheel_config());
        let world = cx
            .add_node(
                firewheel::nodes::peak_meter::PeakMeterNode::<2> { enabled: true },
                None,
            )
            .map_err(|e| graph(e.to_string()))?;
        let out = cx.graph_out_node_id();
        let stage = |cx: &mut FirewheelContext| {
            cx.add_node(
                VolumeNode::from_linear(1.0),
                Some(VolumeNodeConfig {
                    channels: NonZeroChannelCount::STEREO,
                }),
            )
            .map_err(|e| graph(e.to_string()))
        };
        let master = stage(&mut cx)?;
        let world_gain = stage(&mut cx)?;
        let interface = stage(&mut cx)?;
        for (from, to) in [
            (world, world_gain),
            (world_gain, master),
            (interface, master),
            (master, out),
        ] {
            cx.connect(from, to, &[(0, 0), (1, 1)], false)
                .map_err(|e| graph(e.to_string()))?;
        }
        let volumes = Volumes {
            nodes: [
                (Bus::Master, master),
                (Bus::World, world_gain),
                (Bus::Interface, interface),
            ],
            levels: [VolumeNode::from_linear(1.0); 3],
        };
        let world_meter = WorldMeter(
            cx.node_state::<firewheel::nodes::peak_meter::PeakMeterState<2>>(world)
                .ok_or_else(|| graph("no meter state".into()))?
                .clone(),
        );
        Ok(Self {
            world,
            volumes,
            world_meter,
            scopes: ScopeCaptures::default(),
            cx,
            modules: HashMap::new(),
            cables: HashMap::new(),
            to_master: HashSet::new(),
            to_ui: HashSet::new(),
            emitters: HashMap::new(),
            listener_pose: ListenerPose::default(),
            #[cfg(feature = "audionimbus")]
            audionimbus_spatial: None,
            #[cfg(feature = "audionimbus")]
            audionimbus_simulation: None,
            #[cfg(feature = "audionimbus")]
            reflection_wet_mix: 0.75,
            #[cfg(feature = "omg_audio")]
            omg_geometry: AcousticSceneGeometry::default(),
            pending_params: HashMap::new(),
            mailbox: EventMailbox::default(),
            event_rings: Vec::new(),
            retiring: Vec::new(),
            samples: HashMap::new(),
            sample_waves: HashMap::new(),
            stream: None,
        })
    }

    /// The world bus's meter.
    pub fn world_meter(&self) -> WorldMeter {
        self.world_meter.clone()
    }

    pub(crate) fn scope_captures(&self) -> ScopeCaptures {
        self.scopes.clone()
    }

    /// Copy a scope's latest capture into evenly spaced display samples.
    pub fn read_scope(&self, id: ModuleId, trace: &mut [f32]) -> bool {
        self.scopes.read(id, trace)
    }

    /// Start the engine on the default output device.
    pub fn start() -> Result<Self, EngineError> {
        let mut engine = Self::new()?;
        let stream = firewheel::cpal::CpalStream::new(&mut engine.cx, Default::default())
            .map_err(|e| EngineError::Device(e.to_string()))?;
        engine.stream = Some(stream);
        #[cfg(feature = "audionimbus")]
        engine.initialize_audionimbus()?;
        Ok(engine)
    }

    /// Replace the acoustic geometry snapshot used by the selected spatial backend.
    pub fn set_acoustic_geometry(&mut self, geometry: AcousticSceneGeometry) {
        #[cfg(feature = "audionimbus")]
        if let Some(simulation) = &mut self.audionimbus_simulation {
            simulation.set_geometry(geometry.clone());
        }
        #[cfg(feature = "omg_audio")]
        {
            self.omg_geometry = geometry.clone();
        }
        #[cfg(not(feature = "audionimbus"))]
        let _ = geometry;
    }

    /// An engine with no audio device, plus the renderer that plays the audio
    /// thread's part. Used by tests and anything that renders to a buffer.
    pub fn offline(config: EngineConfig) -> Result<(Self, OfflineRenderer), EngineError> {
        let sample_rate =
            NonZeroU32::new(config.sample_rate).ok_or(EngineError::Config("sample_rate is 0"))?;
        let max_block = NonZeroU32::new(config.max_block_frames)
            .ok_or(EngineError::Config("max_block_frames is 0"))?;
        let mut engine = Self::new()?;
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
        #[cfg(feature = "audionimbus")]
        engine.initialize_audionimbus()?;
        Ok((engine, renderer))
    }

    #[cfg(feature = "audionimbus")]
    fn initialize_audionimbus(&mut self) -> Result<(), EngineError> {
        let stream = self
            .cx
            .stream_info()
            .ok_or(EngineError::Config("audio stream is not active"))?;
        let spatial = AudioNimbusSpatialContext::new(stream.sample_rate.get(), 256)
            .map_err(|error| EngineError::Device(format!("AudioNimbus initialization: {error}")))?;
        let simulation = AudioNimbusDirectSimulation::new(stream.sample_rate.get(), 256)
            .map_err(|error| EngineError::Device(format!("AudioNimbus simulation: {error}")))?;
        self.audionimbus_spatial = Some(Arc::new(spatial));
        self.audionimbus_simulation = Some(simulation);
        Ok(())
    }

    fn set_bus_volume(&mut self, bus: Bus, volume: f32) -> Result<(), EngineError> {
        if !volume.is_finite() {
            return Err(EngineError::Config("bus volume must be finite"));
        }
        let index = self.volumes.index(bus);
        let old = self.volumes.levels[index];
        let new = VolumeNode::from_linear(volume.clamp(0.0, 2.0));
        let node = self.volumes.nodes[index].1;
        new.diff(&old, PathBuilder::default(), &mut self.cx.event_queue(node));
        self.volumes.levels[index] = new;
        Ok(())
    }

    /// Validate a command and apply it to the graph. Takes effect on the audio
    /// thread at the next [`flush`](Self::flush). Never blocks.
    pub fn send(&mut self, command: Command) -> Result<(), EngineError> {
        match command {
            Command::SelectSample { module, sample } => self.select_sample(module, sample),
            Command::AddModule { id, kind } => self.add_module(id, kind),
            Command::PlaySample { sample, gain } => self.play_sample(sample, gain),
            Command::SetBusVolume { bus, volume } => self.set_bus_volume(bus, volume),
            Command::RemoveModule { id } => self.remove_module(id),
            Command::SetEmitter {
                output,
                offset,
                position_world,
                volume,
            } => self.set_emitter(output, offset, position_world, volume),
            Command::SetListener { pose } => {
                self.listener_pose = pose;
                Ok(())
            }
            Command::SetReflectionWetMix { value } => {
                if !value.is_finite() {
                    return Err(EngineError::Config("reflection wet mix must be finite"));
                }
                #[cfg(feature = "audionimbus")]
                {
                    self.reflection_wet_mix = value.clamp(0.0, 1.0);
                    for emitter in self.emitters.values() {
                        if let EmitterParams::AudioNimbus(node) = &emitter.params {
                            node.set_reflection_wet_mix(self.reflection_wet_mix);
                        }
                    }
                }
                #[cfg(not(feature = "audionimbus"))]
                return Err(EngineError::Config("AudioNimbus is disabled"));
                #[cfg(feature = "audionimbus")]
                Ok(())
            }
            Command::Connect { from, to } => self.connect(from, to),
            Command::Disconnect { from, to } => self.disconnect(from, to),
            Command::SetParam {
                module,
                param,
                value,
            } => {
                let kind = self.module(module)?.kind;
                let spec = kind
                    .param(param)
                    .ok_or(EngineError::UnknownParam { module, param })?;
                if matches!(kind, ModuleKind::Envelope { .. }) {
                    return Err(EngineError::FixedParam {
                        module,
                        param: spec.name,
                    });
                }
                let name = spec.name;
                self.pending_params
                    .insert((module, name), spec.conform(value));
                Ok(())
            }
        }
    }

    /// Publish everything sent since the last flush to the audio thread. Call once
    /// per frame; Firewheel also uses this call to free nodes removed earlier.
    ///
    /// On [`EngineError::QueueFull`] the edits stay queued; call again next frame.
    pub fn flush(&mut self) -> Result<(), EngineError> {
        #[cfg(feature = "audionimbus")]
        if let Some(simulation) = &mut self.audionimbus_simulation {
            let inputs = self
                .emitters
                .iter()
                .filter_map(|(&output, emitter)| match &emitter.params {
                    EmitterParams::AudioNimbus(node) => Some(DirectEmitterInput {
                        output,
                        position: node.position(),
                        params: node.direct_params(),
                        reflections: node.reflections(),
                    }),
                    EmitterParams::Basic(_) => None,
                    #[cfg(feature = "omg_audio")]
                    EmitterParams::Omg { .. } => None,
                })
                .collect();
            simulation.publish(inputs, self.listener_pose);
        }
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
                (NodeState::Sequencer(old), param) => {
                    let new = old.with(param, value);
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                (NodeState::Scope { tap, capture }, "time") => {
                    let new = TripleBufferNode {
                        window_size: scope_window(value),
                    };
                    new.diff(
                        capture,
                        PathBuilder::default(),
                        &mut self.cx.event_queue(*tap),
                    );
                    *capture = new;
                }
                (NodeState::Vcf(old), param) => {
                    let mut new = *old;
                    match param {
                        "cutoff" => new.cutoff = value,
                        "res" => new.resonance = value,
                        _ => continue,
                    }
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                (NodeState::Mixer(old), param) => {
                    let new = old.with(param, value);
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
            ModuleKind::SamplePlayer => {
                (self.cx.add_node(SampleVoice, None), NodeState::SamplePlayer)
            }
            ModuleKind::SineVco { freq_hz }
            | ModuleKind::SquareVco { freq_hz }
            | ModuleKind::SawVco { freq_hz }
            | ModuleKind::TriangleVco { freq_hz } => {
                let waveform = match kind {
                    ModuleKind::SquareVco { .. } => Waveform::Square,
                    ModuleKind::SawVco { .. } => Waveform::Saw,
                    ModuleKind::TriangleVco { .. } => Waveform::Triangle,
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
            ModuleKind::RandomClock {
                seed,
                min_interval_s,
                max_interval_s,
            } => {
                if !min_interval_s.is_finite()
                    || !max_interval_s.is_finite()
                    || min_interval_s < 0.01
                    || max_interval_s < min_interval_s
                    || max_interval_s > 60.
                {
                    return Err(EngineError::Config(
                        "random clock bounds must be finite, ordered, and within 0.01..60 seconds",
                    ));
                }
                let config = ClockConfig {
                    module: id,
                    mailbox: self.mailbox.clone(),
                };
                let node = RandomClockNode {
                    seed,
                    min_interval_s,
                    max_interval_s,
                };
                (self.cx.add_node(node, Some(config)), NodeState::RandomClock)
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
                (self.cx.add_node(state, None), NodeState::Envelope)
            }
            ModuleKind::Vcf {
                cutoff_hz,
                resonance,
            } => {
                let state = VcfNode {
                    cutoff: cutoff_hz,
                    resonance,
                    enabled: true,
                };
                (self.cx.add_node(state, None), NodeState::Vcf(state))
            }
            ModuleKind::Scope { time, .. } => {
                let (node, state) = self.add_scope(id, time)?;
                (Ok(node), state)
            }
            ModuleKind::Mixer { levels } => {
                let state = MixerNode {
                    levels,
                    enabled: true,
                };
                (self.cx.add_node(state, None), NodeState::Mixer(state))
            }
            ModuleKind::Vca { gain } => {
                let state = VcaNode {
                    gain,
                    enabled: true,
                };
                (self.cx.add_node(state, None), NodeState::Vca(state))
            }
            ModuleKind::StepSequencer { cv, gate } => {
                let specs = kind.params();
                let conform = |values: [f32; crate::SEQUENCER_STEPS], offset: usize| {
                    std::array::from_fn(|i| specs[offset + i].conform(values[i]))
                };
                let state =
                    SequencerNode::new(conform(cv, 0), conform(gate, crate::SEQUENCER_STEPS));
                let config = ClockConfig {
                    module: id,
                    mailbox: self.mailbox.clone(),
                };
                (
                    self.cx.add_node(state, Some(config)),
                    NodeState::Sequencer(state),
                )
            }
        };
        let node = node.map_err(|e| EngineError::Graph(e.to_string()))?;
        self.modules.insert(id, Module { kind, node, state });
        Ok(())
    }

    fn add_scope(&mut self, id: ModuleId, time: f32) -> Result<(NodeID, NodeState), EngineError> {
        let graph = |error: &dyn std::fmt::Display| EngineError::Graph(error.to_string());
        let channels = NonZeroChannelCount::MONO;
        let capture = TripleBufferNode {
            window_size: scope_window(time),
        };
        let tap = self
            .cx
            .add_node(
                capture,
                Some(TripleBufferConfig {
                    channels,
                    max_window_size: WindowSize::Seconds(f64::from(SCOPE_PARAMS[0].max)),
                }),
            )
            .map_err(|error| graph(&error))?;
        let result = (|| {
            let state = self
                .cx
                .node_state::<TripleBufferState>(tap)
                .cloned()
                .ok_or_else(|| EngineError::Graph("scope capture has no state".into()))?;
            let pass = self
                .cx
                .add_node(
                    VolumeNode::from_linear(1.0),
                    Some(VolumeNodeConfig { channels }),
                )
                .map_err(|error| graph(&error))?;
            if let Err(error) = self.cx.connect(pass, tap, &[(0, 0)], false) {
                let _ = self.cx.remove_node(pass);
                return Err(graph(&error));
            }
            if let Ok(mut scopes) = self.scopes.0.lock() {
                scopes.insert(id, state);
            }
            Ok((pass, NodeState::Scope { tap, capture }))
        })();
        if result.is_err() {
            let _ = self.cx.remove_node(tap);
        }
        result
    }

    /// Load (or replace) a one-shot sample: one buffer per channel (mono or stereo),
    /// recorded at `sample_rate`; it is resampled to the engine's rate here, off the
    /// audio thread.
    pub fn load_sample(&mut self, id: SampleId, sample_rate: u32, channels: Vec<Vec<f32>>) {
        let target = self.sample_rate().unwrap_or(48_000);
        let channels = if sample_rate == target || sample_rate == 0 {
            channels
        } else {
            let ratio = sample_rate as f64 / target as f64;
            channels
                .iter()
                .map(|c| {
                    let n = ((c.len() as f64) / ratio) as usize;
                    (0..n)
                        .map(|i| {
                            let x = i as f64 * ratio;
                            let k = x as usize;
                            let t = (x - k as f64) as f32;
                            let a = c.get(k).copied().unwrap_or(0.);
                            let b = c.get(k + 1).copied().unwrap_or(a);
                            a + (b - a) * t
                        })
                        .collect()
                })
                .collect()
        };
        self.sample_waves.remove(&id);
        self.samples.insert(id, channels);
    }

    fn select_sample(
        &mut self,
        module: ModuleId,
        sample: Option<SampleId>,
    ) -> Result<(), EngineError> {
        let target = self
            .modules
            .get(&module)
            .ok_or(EngineError::UnknownModule(module))?;
        if !matches!(target.state, NodeState::SamplePlayer) {
            return Err(EngineError::Config("module is not a sample player"));
        }
        let node = target.node;
        let player = sample
            .map(|id| {
                self.sample_wave(id)
                    .map(|wave| WavePlayer::new(wave, 0, 0, wave.length(), None))
            })
            .transpose()?;
        self.cx.queue_event_for(node, NodeEventType::custom(player));
        Ok(())
    }

    fn sample_wave(&mut self, id: SampleId) -> Result<&Arc<Wave>, EngineError> {
        use std::collections::hash_map::Entry;
        match self.sample_waves.entry(id) {
            Entry::Occupied(wave) => Ok(wave.into_mut()),
            Entry::Vacant(slot) => {
                let channels = self
                    .samples
                    .get(&id)
                    .ok_or(EngineError::Config("unknown sample"))?;
                let first = channels
                    .first()
                    .filter(|channel| !channel.is_empty())
                    .ok_or(EngineError::Config("empty sample"))?;
                let mut mono = first.clone();
                if channels.len() > 1 {
                    if channels.iter().any(|channel| channel.len() != mono.len()) {
                        return Err(EngineError::Config("sample channel lengths differ"));
                    }
                    let scale = 1. / channels.len() as f32;
                    for (index, output) in mono.iter_mut().enumerate() {
                        *output = channels.iter().map(|channel| channel[index] * scale).sum();
                    }
                }
                let rate = f64::from(
                    self.cx
                        .stream_info()
                        .map_or(48_000, |info| info.sample_rate.get()),
                );
                Ok(slot.insert(Arc::new(Wave::from_samples(rate, &mono))))
            }
        }
    }

    /// One voice per play: a sampler node to the speakers, retired once it has played.
    fn play_sample(&mut self, id: SampleId, gain: f32) -> Result<(), EngineError> {
        use firewheel::nodes::sampler::{SamplerConfig, SamplerNode};
        let channels = self
            .samples
            .get(&id)
            .cloned()
            .ok_or(EngineError::Config("unknown sample"))?;
        let frames = channels.first().map_or(0, Vec::len);
        let mut node = SamplerNode {
            volume: firewheel::Volume::Linear(gain.clamp(0., 4.)),
            ..SamplerNode::default()
        };
        node.start_or_restart();
        let id = self
            .cx
            .add_node(node, Some(SamplerConfig::default()))
            .map_err(|e| EngineError::Graph(e.to_string()))?;
        self.cx
            .queue_event_for(id, SamplerNode::set_sample_event(channels));
        self.cx.queue_event_for(id, node.sync_play_event());
        let out = self.volumes.node(Bus::Interface);
        self.cx
            .connect(id, out, &[(0, 0), (1, 1)], false)
            .map_err(|e| EngineError::Graph(e.to_string()))?;
        let now = self.cx.audio_clock().samples.0.max(0) as u64;
        let rate = self.sample_rate().unwrap_or(48_000) as u64;
        self.retiring.push((id, now + frames as u64 + rate / 4));
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
            NodeState::Vcf(old) => {
                VcfNode {
                    enabled: false,
                    ..old
                }
                .diff(&old, PathBuilder::default(), &mut queue);
                true
            }
            NodeState::Mixer(old) => {
                MixerNode {
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
            NodeState::Scope { tap, .. } => {
                let _ = self.cx.remove_node(tap);
                if let Ok(mut scopes) = self.scopes.0.lock() {
                    scopes.remove(&id);
                }
                // The scope is often a voice's last stage (the purifier's outlet): its
                // pass-through and outputs stay until the modules before it have faded
                // out, or their fades never reach the speakers and the voice clicks off.
                true
            }
            NodeState::SamplePlayer => {
                self.cx
                    .queue_event_for(module.node, NodeEventType::custom(RetireSample));
                true
            }
            NodeState::Clock(_)
            | NodeState::RandomClock
            | NodeState::Constant(_)
            | NodeState::Envelope
            | NodeState::Sequencer(_) => false,
        };
        let spatial: Vec<NodeID> = self
            .emitters
            .iter()
            .filter(|(from, _)| from.module == id)
            .map(|(_, emitter)| emitter.node)
            .collect();
        self.emitters.retain(|from, _| from.module != id);
        if fading {
            let sample_rate = self.sample_rate().unwrap_or(48_000) as f64;
            let after = self.cx.audio_clock().samples.0.max(0) as u64
                + (RETIRE_SECONDS * sample_rate) as u64;
            self.retiring.push((module.node, after));
            // Its fade out still has to reach the speakers.
            self.retiring
                .extend(spatial.into_iter().map(|node| (node, after)));
        } else {
            let _ = self.cx.remove_node(module.node);
            for node in spatial {
                let _ = self.cx.remove_node(node);
            }
        }
        // Firewheel removes a node's edges with it; our tables forget them now, since
        // the module no longer exists as far as the game is concerned.
        self.cables
            .retain(|to, from| to.module != id && from.module != id);
        self.to_master.retain(|from| from.module != id);
        self.to_ui.retain(|from| from.module != id);
        self.pending_params.retain(|(m, _), _| *m != id);
        self.event_rings.retain(|(m, _)| *m != id);
        Ok(())
    }

    /// Route a master output through its spatial node (added on first use) and update
    /// where it is. The hearing range is the game's `volume`, so the node's own distance
    /// loss is off; it still pans and muffles.
    fn set_emitter(
        &mut self,
        output: PortRef,
        offset: [f32; 3],
        position_world: [f32; 3],
        volume: f32,
    ) -> Result<(), EngineError> {
        #[cfg(not(feature = "audionimbus"))]
        let _ = position_world;
        if !self.to_master.contains(&output) {
            return Ok(());
        }
        #[cfg(feature = "omg_audio")]
        if std::env::var_os("STUBBY_OMG_AUDIO_REVIEW").is_some() {
            return self.set_omg_review_emitter(output, position_world, volume);
        }
        let gain = volume.clamp(0., 1.);
        let volume = firewheel::Volume::Linear(gain);
        if let Some(emitter) = self.emitters.get_mut(&output) {
            match &mut emitter.params {
                EmitterParams::Basic(old) => {
                    let new = SpatialBasicNode {
                        volume,
                        offset: offset.into(),
                        ..*old
                    };
                    let mut queue = self.cx.event_queue(emitter.node);
                    new.diff(old, PathBuilder::default(), &mut queue);
                    *old = new;
                }
                #[cfg(feature = "audionimbus")]
                EmitterParams::AudioNimbus(node) => {
                    node.set_position(position_world);
                    node.set_direction(offset);
                    node.set_volume(gain);
                }
                #[cfg(feature = "omg_audio")]
                EmitterParams::Omg { .. } => {}
            }
            return Ok(());
        }
        let graph = |e: AddEdgeError| EngineError::Graph(e.to_string());
        let (src, src_port) = self.output_port(output)?;
        #[cfg(feature = "audionimbus")]
        let (node, params) = if let (Some(spatial), Some(_simulation)) =
            (&self.audionimbus_spatial, &self.audionimbus_simulation)
        {
            let params = AudioNimbusBinauralNode::new(
                spatial.clone(),
                Arc::new(DirectEffectParamsShared::default()),
                output,
            );
            params.set_reflection_wet_mix(self.reflection_wet_mix);
            params.set_direction(offset);
            params.set_position(position_world);
            params.set_volume(gain);
            let node = self
                .cx
                .add_node(params.clone(), None)
                .map_err(|e| EngineError::Graph(e.to_string()))?;
            (node, EmitterParams::AudioNimbus(params))
        } else {
            let mut params = SpatialBasicNode {
                volume,
                offset: offset.into(),
                ..SpatialBasicNode::default()
            };
            params.distance_attenuation.distance_gain_factor = 0.;
            let node = self
                .cx
                .add_node(params, None)
                .map_err(|e| EngineError::Graph(e.to_string()))?;
            (node, EmitterParams::Basic(params))
        };
        #[cfg(not(feature = "audionimbus"))]
        let (node, params) = {
            let mut params = SpatialBasicNode {
                volume,
                offset: offset.into(),
                ..SpatialBasicNode::default()
            };
            params.distance_attenuation.distance_gain_factor = 0.;
            let node = self
                .cx
                .add_node(params, None)
                .map_err(|e| EngineError::Graph(e.to_string()))?;
            (node, EmitterParams::Basic(params))
        };
        let out = self.world;
        self.cx
            .disconnect(src, out, &[(src_port, 0), (src_port, 1)]);
        self.cx
            .connect(src, node, &[(src_port, 0)], false)
            .map_err(graph)?;
        self.cx
            .connect(node, out, &[(0, 0), (1, 1)], false)
            .map_err(graph)?;
        self.emitters.insert(output, Emitter { node, params });
        Ok(())
    }

    #[cfg(feature = "omg_audio")]
    fn set_omg_review_emitter(
        &mut self,
        output: PortRef,
        position: [f32; 3],
        volume: f32,
    ) -> Result<(), EngineError> {
        // The grotto review holds each camera for five seconds. Rebuild only when
        // moving to another authored view; this experiment does not support smooth
        // moving-source/listener updates.
        if let Some(existing) = self.emitters.get(&output)
            && let EmitterParams::Omg {
                position: old_position,
                listener: old_listener,
            } = &existing.params
        {
            let moved_source = position
                .iter()
                .zip(old_position)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                > 0.25;
            let moved_listener = self
                .listener_pose
                .position
                .iter()
                .zip(old_listener.position)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                > 0.25;
            if !moved_source && !moved_listener {
                return Ok(());
            }
        }
        let (frame, source) = trace_snapshot(&self.omg_geometry, self.listener_pose, position)
            .map_err(EngineError::Config)?;
        if let Some(block) = frame.block(source) {
            eprintln!(
                "OMG review trace: {} triangles, {} direct taps {:?}, reverb levels {:?}",
                self.omg_geometry.triangles.len(),
                block.taps.len(),
                block.taps.iter().map(|tap| tap.gains).collect::<Vec<_>>(),
                block.reverb.level
            );
        }
        let params = OmgSpatialNode::from_frame(&frame, source, volume)
            .ok_or(EngineError::Config("OMG frame has no emitter parameters"))?;
        let node = self
            .cx
            .add_node(params, None)
            .map_err(|error| EngineError::Graph(error.to_string()))?;
        let (src, src_port) = self.output_port(output)?;
        if let Some(old) = self.emitters.remove(&output) {
            let _ = self.cx.remove_node(old.node);
        }
        let out = self.world;
        self.cx
            .disconnect(src, out, &[(src_port, 0), (src_port, 1)]);
        self.cx
            .connect(src, node, &[(src_port, 0)], false)
            .map_err(|error| EngineError::Graph(error.to_string()))?;
        self.cx
            .connect(node, out, &[(0, 0), (1, 1)], false)
            .map_err(|error| EngineError::Graph(error.to_string()))?;
        self.emitters.insert(
            output,
            Emitter {
                node,
                params: EmitterParams::Omg {
                    position,
                    listener: self.listener_pose,
                },
            },
        );
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
            let out = self.world;
            // Mono to both speakers. Feeding both channels identically also keeps
            // clear of Firewheel 0.14's silent-channel bug (stubby-music#4).
            self.cx
                .connect(src, out, &[(src_port, 0), (src_port, 1)], false)
                .map_err(map_edge_error)?;
            self.to_master.insert(from);
            return Ok(());
        }
        if to == PortRef::ui_output() {
            if self.to_ui.insert(from) {
                let out = self.volumes.node(Bus::Interface);
                self.cx
                    .connect(src, out, &[(src_port, 0), (src_port, 1)], false)
                    .map_err(map_edge_error)?;
            }
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
            if let Some(emitter) = self.emitters.remove(&from) {
                let _ = self.cx.remove_node(emitter.node);
                return Ok(());
            }
            let out = self.world;
            self.cx
                .disconnect(src, out, &[(src_port, 0), (src_port, 1)]);
            return Ok(());
        }
        if to == PortRef::ui_output() {
            if !self.to_ui.remove(&from) {
                return Err(EngineError::NotConnected { from, to });
            }
            let out = self.volumes.node(Bus::Interface);
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

#[cfg(all(test, feature = "audionimbus"))]
mod audionimbus_control_tests {
    use super::*;

    #[test]
    fn reflection_wet_mix_rejects_non_finite_and_clamps_finite_values() {
        let (mut engine, _renderer) = Engine::offline(EngineConfig::default()).unwrap();
        assert!(
            engine
                .send(Command::SetReflectionWetMix { value: f32::NAN })
                .is_err()
        );
        assert_eq!(engine.reflection_wet_mix, 0.75);

        engine
            .send(Command::SetReflectionWetMix { value: -2.0 })
            .unwrap();
        assert_eq!(engine.reflection_wet_mix, 0.0);
        engine
            .send(Command::SetReflectionWetMix { value: 1.5 })
            .unwrap();
        assert_eq!(engine.reflection_wet_mix, 1.0);
    }
}

#[cfg(all(test, feature = "omg_audio"))]
mod omg_review_tests {
    use super::*;

    #[test]
    fn static_review_emitter_preserves_audible_signal() {
        let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
        let source = PortRef::new(ModuleId(99_001), "out");
        engine
            .send(Command::AddModule {
                id: source.module,
                kind: ModuleKind::SquareVco { freq_hz: 220.0 },
            })
            .unwrap();
        engine
            .send(Command::Connect {
                from: source,
                to: PortRef::output(),
            })
            .unwrap();
        engine
            .send(Command::SetListener {
                pose: ListenerPose {
                    position: [0.0, 1.6, 0.0],
                    right: [1.0, 0.0, 0.0],
                    up: [0.0, 1.0, 0.0],
                    ahead: [0.0, 0.0, -1.0],
                },
            })
            .unwrap();
        engine
            .set_omg_review_emitter(source, [0.0, 0.65, -5.0], 0.65)
            .unwrap();
        engine.flush().unwrap();

        let mut output = vec![0.0; 48_000 * 2];
        renderer.render(&mut output);
        let peak = output
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        assert!(
            peak > 1e-4,
            "OMG review route returned silence (peak={peak})"
        );
    }
}
