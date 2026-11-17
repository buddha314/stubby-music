//! AudioNimbus-backed spatial effects used by Firewheel processors.
//!
//! Create effects on the control thread, then reuse them with caller-owned buffers in
//! the audio processor. Simulation and scene updates belong on worker/control threads.

use crate::PortRef;
use arc_swap::ArcSwap;
use audionimbus::{
    AirAbsorptionModel, AudioBufferMut, AudioBufferRef, AudioSettings, BinauralEffect,
    BinauralEffectParams, BinauralEffectSettings, Context, CoordinateSystem, DirectEffect,
    DirectEffectParams, DirectEffectSettings, DirectSimulationParameters, DirectSimulationSettings,
    Direction, Directivity, DistanceAttenuationModel, Equalizer, Hrtf, HrtfInterpolation,
    HrtfSettings, Occlusion, OcclusionAlgorithm, Point, SimulationFlags, SteamAudioError,
    Transmission,
};
use firewheel::{
    channel_config::{ChannelConfig, ChannelCount},
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
        NodeError, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
    mpsc::{self, SyncSender, TrySendError},
};
use std::time::{Duration, Instant};

/// Shared AudioNimbus state for spatial processors at one sample rate and block size.
///
/// Create this once on the control thread, then use it to create one processor per
/// voice. HRTF setup is intentionally shared rather than repeated per emitter.
pub struct AudioNimbusSpatialContext {
    context: Context,
    hrtf: Hrtf,
    audio_settings: AudioSettings,
}

/// Inputs submitted to the sleeping simulation worker. A one-slot wake channel and
/// an ArcSwap snapshot coalesce frame updates when simulation is slower than rendering.
#[derive(Clone)]
pub struct DirectEmitterInput {
    pub output: PortRef,
    pub position: [f32; 3],
    pub params: Arc<DirectEffectParamsShared>,
}

/// Fixed-size atomic transport for simulation output. The audio callback copies scalar
/// values into its own params object; it never loads or drops an ArcSwap snapshot.
pub struct DirectEffectParamsShared {
    values: [AtomicU32; 9],
    flags: AtomicU32,
}

impl Default for DirectEffectParamsShared {
    fn default() -> Self {
        Self {
            values: std::array::from_fn(|_| AtomicU32::new(0.0_f32.to_bits())),
            flags: AtomicU32::new(0),
        }
    }
}

impl DirectEffectParamsShared {
    pub fn store(&self, params: &DirectEffectParams) {
        let mut flags = 0;
        let mut values = [0.0; 9];
        if let Some(distance) = params.distance_attenuation {
            flags |= 1 << 0;
            values[0] = distance;
        }
        if let Some(air) = params.air_absorption {
            flags |= 1 << 1;
            values[1..4].copy_from_slice(&air.0);
        }
        if let Some(directivity) = params.directivity {
            flags |= 1 << 2;
            values[4] = directivity;
        }
        if let Some(occlusion) = params.occlusion {
            flags |= 1 << 3;
            values[5] = occlusion;
        }
        if let Some(transmission) = params.transmission {
            flags |= 1 << 4;
            let equalizer = match transmission {
                Transmission::FrequencyIndependent(eq) => eq,
                Transmission::FrequencyDependent(eq) => {
                    flags |= 1 << 5;
                    eq
                }
            };
            values[6..9].copy_from_slice(&equalizer.0);
        }
        for (slot, value) in self.values.iter().zip(values) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
        self.flags.store(flags, Ordering::Release);
    }

    fn load_into(&self, params: &mut DirectEffectParams) {
        let flags = self.flags.load(Ordering::Acquire);
        let values = self
            .values
            .each_ref()
            .map(|slot| f32::from_bits(slot.load(Ordering::Relaxed)));
        params.distance_attenuation = (flags & (1 << 0) != 0).then_some(values[0]);
        params.air_absorption =
            (flags & (1 << 1) != 0).then_some(Equalizer([values[1], values[2], values[3]]));
        params.directivity = (flags & (1 << 2) != 0).then_some(values[4]);
        params.occlusion = (flags & (1 << 3) != 0).then_some(values[5]);
        params.transmission = if flags & (1 << 4) == 0 {
            None
        } else {
            let eq = Equalizer([values[6], values[7], values[8]]);
            Some(if flags & (1 << 5) != 0 {
                Transmission::FrequencyDependent(eq)
            } else {
                Transmission::FrequencyIndependent(eq)
            })
        };
    }
}

pub struct AudioNimbusDirectSimulation {
    latest: Arc<ArcSwap<Vec<DirectEmitterInput>>>,
    wake: SyncSender<bool>,
    thread: Option<std::thread::JoinHandle<()>>,
    last_wake: Option<Instant>,
}

impl AudioNimbusDirectSimulation {
    pub fn new(sample_rate: u32, frame_size: u32) -> Result<Self, String> {
        let latest = Arc::new(ArcSwap::from_pointee(Vec::new()));
        let worker_latest = latest.clone();
        let (wake, receiver) = mpsc::sync_channel(1);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("audionimbus-direct-simulation".into())
            .spawn(move || {
                let result = run_direct_simulation_worker(
                    worker_latest,
                    receiver,
                    sample_rate,
                    frame_size,
                    ready_tx.clone(),
                );
                if let Err(error) = result {
                    let _ = ready_tx.try_send(Err(error.clone()));
                    eprintln!("AudioNimbus direct simulation stopped: {error}");
                }
            })
            .map_err(|error| format!("spawn direct simulation worker: {error}"))?;
        ready_rx
            .recv()
            .map_err(|error| format!("direct simulation worker did not initialize: {error}"))??;
        Ok(Self {
            latest,
            wake,
            thread: Some(thread),
            last_wake: None,
        })
    }

    pub fn publish(&mut self, inputs: Vec<DirectEmitterInput>) {
        self.latest.store(Arc::new(inputs));
        if self
            .last_wake
            .is_some_and(|last| last.elapsed() < Duration::from_millis(50))
        {
            return;
        }
        self.last_wake = Some(Instant::now());
        match self.wake.try_send(true) {
            Ok(()) | Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => {}
        }
    }
}

impl Drop for AudioNimbusDirectSimulation {
    fn drop(&mut self) {
        let _ = self.wake.send(false);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run_direct_simulation_worker(
    latest: Arc<ArcSwap<Vec<DirectEmitterInput>>>,
    wake: mpsc::Receiver<bool>,
    sample_rate: u32,
    frame_size: u32,
    ready: SyncSender<Result<(), String>>,
) -> Result<(), String> {
    let context = Context::default();
    let audio_settings = AudioSettings {
        sampling_rate: sample_rate,
        frame_size,
    };
    let settings = audionimbus::dynamic::SimulationSettings::new(&audio_settings).with_direct(
        DirectSimulationSettings {
            max_num_occlusion_samples: 16,
        },
    );
    let mut simulator = audionimbus::dynamic::Simulator::try_new(&context, &settings)
        .map_err(|error| format!("create simulator: {error}"))?;
    let scene = audionimbus::dynamic::Scene::try_new(
        &context,
        audionimbus::dynamic::RayTracerSettings::Default,
    )
    .map_err(|error| format!("create scene: {error}"))?;
    simulator
        .try_set_scene(&scene)
        .map_err(|error| format!("set scene: {error}"))?;
    simulator.commit();
    ready
        .send(Ok(()))
        .map_err(|error| format!("publish worker readiness: {error}"))?;

    let mut sources = std::collections::HashMap::new();
    while let Ok(true) = wake.recv() {
        let snapshot = latest.load();
        let current: std::collections::HashSet<_> =
            snapshot.iter().map(|emitter| emitter.output).collect();
        sources.retain(|output, source| {
            if current.contains(output) {
                true
            } else {
                let _ = simulator.try_remove_source(source);
                false
            }
        });
        for emitter in snapshot.iter() {
            if !sources.contains_key(&emitter.output) {
                let source =
                    audionimbus::dynamic::Source::try_new(&simulator, SimulationFlags::DIRECT)
                        .map_err(|error| format!("create source: {error}"))?;
                simulator
                    .try_add_source(&source)
                    .map_err(|error| format!("add source: {error}"))?;
                sources.insert(emitter.output, source);
            }
        }
        simulator.commit();

        for emitter in snapshot.iter() {
            let Some(source) = sources.get(&emitter.output) else {
                continue;
            };
            let inputs = audionimbus::dynamic::SimulationInputs {
                source: CoordinateSystem {
                    origin: Point::new(
                        emitter.position[0],
                        emitter.position[1],
                        emitter.position[2],
                    ),
                    ..CoordinateSystem::default()
                },
                parameters: audionimbus::dynamic::SimulationParameters {
                    direct: Some(
                        DirectSimulationParameters::new()
                            .with_distance_attenuation(DistanceAttenuationModel::default())
                            .with_air_absorption(AirAbsorptionModel::default())
                            .with_directivity(Directivity::WeightedDipole {
                                weight: 0.0,
                                power: 1.0,
                            })
                            .with_occlusion(Occlusion::new(OcclusionAlgorithm::Raycast)),
                    ),
                    ..Default::default()
                },
            };
            source
                .set_inputs(&inputs)
                .map_err(|error| format!("set source inputs: {error}"))?;
        }
        simulator
            .set_shared_inputs(
                SimulationFlags::DIRECT,
                &audionimbus::dynamic::SimulationSharedInputs::new(CoordinateSystem::default()),
            )
            .map_err(|error| format!("set listener inputs: {error}"))?;
        simulator
            .run_direct()
            .map_err(|error| format!("run direct simulation: {error}"))?;
        for emitter in snapshot.iter() {
            if let Some(source) = sources.get(&emitter.output) {
                if let Ok(params) = source.get_direct_outputs() {
                    emitter.params.store(&params);
                }
            }
        }
    }
    Ok(())
}

/// Firewheel node that runs the AudioNimbus binaural effect with fixed-size internal
/// blocks. The block adapter accepts any Firewheel callback length; it adds one
/// AudioNimbus block of latency and keeps all buffers preallocated.
#[derive(Clone)]
pub struct AudioNimbusBinauralNode {
    spatial: Arc<AudioNimbusSpatialContext>,
    direction: Arc<[AtomicU32; 3]>,
    position: Arc<[AtomicU32; 3]>,
    volume: Arc<AtomicU32>,
    direct_params: Arc<DirectEffectParamsShared>,
}

impl AudioNimbusBinauralNode {
    /// Creates a mono-to-stereo node using shared AudioNimbus state.
    pub fn new(
        spatial: Arc<AudioNimbusSpatialContext>,
        direct_params: Arc<DirectEffectParamsShared>,
    ) -> Self {
        Self {
            spatial,
            direction: Arc::new([
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(1.0_f32.to_bits()),
            ]),
            position: Arc::new([
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
            ]),
            volume: Arc::new(AtomicU32::new(1.0_f32.to_bits())),
            direct_params,
        }
    }

    /// Publishes a listener-relative unit direction without locking.
    pub fn set_direction(&self, direction: [f32; 3]) {
        let length = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
            .sqrt();
        let normalized = if length > f32::EPSILON {
            [
                direction[0] / length,
                direction[1] / length,
                direction[2] / length,
            ]
        } else {
            [0.0, 0.0, 1.0]
        };
        for (slot, value) in self.direction.iter().zip(normalized) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    /// Publishes the game-side hearing-radius gain without locking.
    pub fn set_volume(&self, volume: f32) {
        self.volume
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn set_position(&self, position: [f32; 3]) {
        for (slot, value) in self.position.iter().zip(position) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
            .each_ref()
            .map(|slot| f32::from_bits(slot.load(Ordering::Relaxed)))
    }

    pub fn direct_params(&self) -> Arc<DirectEffectParamsShared> {
        self.direct_params.clone()
    }
}

impl AudioNode for AudioNimbusBinauralNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &Self::Configuration) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("audionimbus-binaural")
            .channel_config(ChannelConfig::new(ChannelCount::MONO, ChannelCount::STEREO)))
    }

    fn construct_processor(
        &self,
        _: &Self::Configuration,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        if cx.stream_info.sample_rate.get() != self.spatial.audio_settings.sampling_rate {
            return Err(NodeError::from(std::io::Error::other(
                "AudioNimbus HRTF sample rate differs from Firewheel stream",
            )));
        }
        let block_size = self.spatial.audio_settings.frame_size as usize;
        Ok(AudioNimbusBinauralProcessor {
            spatializer: self
                .spatial
                .create_binaural_spatializer_with_params(self.direct_params.clone())?,
            direction: self.direction.clone(),
            volume: self.volume.clone(),
            direct_params: self.direct_params.clone(),
            block_size,
            frame_index: 0,
            input: vec![0.0; block_size],
            left: vec![0.0; block_size],
            right: vec![0.0; block_size],
            pending_left: vec![0.0; block_size],
            pending_right: vec![0.0; block_size],
        })
    }
}

struct AudioNimbusBinauralProcessor {
    spatializer: BinauralSpatializer,
    direction: Arc<[AtomicU32; 3]>,
    volume: Arc<AtomicU32>,
    direct_params: Arc<DirectEffectParamsShared>,
    block_size: usize,
    frame_index: usize,
    input: Vec<f32>,
    left: Vec<f32>,
    right: Vec<f32>,
    pending_left: Vec<f32>,
    pending_right: Vec<f32>,
}

impl AudioNodeProcessor for AudioNimbusBinauralProcessor {
    fn process(
        &mut self,
        info: &ProcInfo,
        ProcBuffers { inputs, outputs }: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        let Some(input) = inputs.first() else {
            return ProcessStatus::ClearAllOutputs;
        };
        let (left_outputs, right_outputs) = outputs.split_at_mut(1);
        let (Some(left_output), Some(right_output)) =
            (left_outputs.first_mut(), right_outputs.first_mut())
        else {
            return ProcessStatus::ClearAllOutputs;
        };
        if input.len() < info.frames
            || left_output.len() < info.frames
            || right_output.len() < info.frames
        {
            return ProcessStatus::ClearAllOutputs;
        }

        for frame in 0..info.frames {
            let slot = self.frame_index;
            left_output[frame] = self.pending_left[slot];
            right_output[frame] = self.pending_right[slot];
            self.input[slot] = input[frame];
            self.frame_index += 1;

            if self.frame_index == self.block_size {
                let direction = [
                    f32::from_bits(self.direction[0].load(Ordering::Relaxed)),
                    f32::from_bits(self.direction[1].load(Ordering::Relaxed)),
                    f32::from_bits(self.direction[2].load(Ordering::Relaxed)),
                ];
                self.spatializer.set_direction(direction);
                self.direct_params
                    .load_into(self.spatializer.direct_params_mut());
                if self
                    .spatializer
                    .process(&self.input, &mut self.left, &mut self.right)
                    .is_ok()
                {
                    std::mem::swap(&mut self.pending_left, &mut self.left);
                    std::mem::swap(&mut self.pending_right, &mut self.right);
                } else {
                    self.pending_left.fill(0.0);
                    self.pending_right.fill(0.0);
                }
                self.frame_index = 0;
            }
        }

        let volume = f32::from_bits(self.volume.load(Ordering::Relaxed));
        for frame in 0..info.frames {
            left_output[frame] *= volume;
            right_output[frame] *= volume;
        }

        ProcessStatus::OutputsModified
    }
}

impl AudioNimbusSpatialContext {
    /// Creates shared HRTF state. HRTF creation may block, so call on a control thread.
    pub fn new(sample_rate: u32, frame_size: u32) -> Result<Self, SteamAudioError> {
        let context = Context::default();
        let audio_settings = AudioSettings {
            sampling_rate: sample_rate,
            frame_size,
        };
        let hrtf = Hrtf::try_new(&context, &audio_settings, &HrtfSettings::default())?;
        Ok(Self {
            context,
            hrtf,
            audio_settings,
        })
    }

    /// Creates a per-voice binaural effect sharing this context and HRTF.
    pub fn create_binaural_spatializer(&self) -> Result<BinauralSpatializer, SteamAudioError> {
        self.create_binaural_spatializer_with_params(Arc::new(DirectEffectParamsShared::default()))
    }

    fn create_binaural_spatializer_with_params(
        &self,
        _direct_params: Arc<DirectEffectParamsShared>,
    ) -> Result<BinauralSpatializer, SteamAudioError> {
        let effect = BinauralEffect::try_new(
            &self.context,
            &self.audio_settings,
            &BinauralEffectSettings {
                hrtf: self.hrtf.clone(),
            },
        )?;
        let direct_effect = DirectEffect::try_new(
            &self.context,
            &self.audio_settings,
            &DirectEffectSettings { num_channels: 1 },
        )?;
        let params = BinauralEffectParams {
            direction: Direction::new(0.0, 0.0, 1.0),
            interpolation: HrtfInterpolation::Bilinear,
            spatial_blend: 1.0,
            hrtf: self.hrtf.clone(),
            peak_delays: None,
        };
        Ok(BinauralSpatializer {
            _context: self.context.clone(),
            hrtf: self.hrtf.clone(),
            effect,
            direct_effect,
            direct_params: DirectEffectParams::default(),
            direct_buffer: vec![0.0; self.audio_settings.frame_size as usize],
            params,
            frame_size: self.audio_settings.frame_size as usize,
        })
    }
}

/// A reusable mono-to-binaural processor.
///
/// The effect owns its Steam Audio state and keeps all sample storage caller-owned.
/// Build it before installing it into the audio graph. This type does not establish
/// native callback allocation or lock guarantees; those must be verified against the
/// exact SDK build and Firewheel host before enabling it in a live callback.
pub struct BinauralSpatializer {
    _context: Context,
    hrtf: Hrtf,
    effect: BinauralEffect,
    direct_effect: DirectEffect,
    direct_params: DirectEffectParams,
    direct_buffer: Vec<f32>,
    params: BinauralEffectParams,
    frame_size: usize,
}

impl BinauralSpatializer {
    /// Changes the unit direction from listener to source.
    pub fn set_direction(&mut self, direction: [f32; 3]) {
        self.params.direction = Direction::new(direction[0], direction[1], direction[2]);
    }

    /// Processes one mono block into separate caller-owned left/right slices.
    ///
    /// AudioNimbus receives views over the caller-owned buffers, so this method does
    /// not allocate sample storage.
    pub fn process(
        &mut self,
        input: &[f32],
        left: &mut [f32],
        right: &mut [f32],
    ) -> Result<(), SpatialProcessError> {
        if input.len() != self.frame_size
            || left.len() != self.frame_size
            || right.len() != self.frame_size
        {
            return Err(SpatialProcessError::FrameSize {
                expected: self.frame_size,
                input: input.len(),
                left: left.len(),
                right: right.len(),
            });
        }

        self.direct_buffer.copy_from_slice(input);
        {
            let mut buffer = AudioBufferMut::try_from(&mut self.direct_buffer[..])?;
            self.direct_effect
                .apply_in_place(&self.direct_params, &mut buffer)?;
        }
        let input_buffer = AudioBufferRef::try_from(&self.direct_buffer[..])?;
        let mut output_buffer = AudioBufferMut::try_from_channels([left, right])?;
        self.effect
            .apply(&self.params, &input_buffer, &mut output_buffer)?;
        Ok(())
    }

    /// Returns the HRTF handle used by this processor.
    pub fn hrtf(&self) -> &Hrtf {
        &self.hrtf
    }

    fn direct_params_mut(&mut self) -> &mut DirectEffectParams {
        &mut self.direct_params
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpatialProcessError {
    #[error(
        "AudioNimbus block size mismatch: expected {expected}, input {input}, left {left}, right {right}"
    )]
    FrameSize {
        expected: usize,
        input: usize,
        left: usize,
        right: usize,
    },
    #[error(transparent)]
    Buffer(#[from] audionimbus::audio_buffer::AudioBufferError),
    #[error(transparent)]
    Effect(#[from] audionimbus::effect::EffectError),
}
