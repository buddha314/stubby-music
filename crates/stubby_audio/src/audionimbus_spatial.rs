//! AudioNimbus-backed spatial effects used by Firewheel processors.
//!
//! Create effects on the control thread, then reuse them with caller-owned buffers in
//! the audio processor. Simulation and scene updates belong on worker/control threads.

use crate::{
    PortRef,
    protocol::{AcousticMaterial, AcousticSceneGeometry, ListenerPose},
};
use arc_swap::ArcSwap;
use audionimbus::dynamic::{
    ReflectionAlgorithm, ReflectionEffect, ReflectionEffectParams as DynamicReflectionEffectParams,
};
use audionimbus::{
    AirAbsorptionModel, AmbisonicsDecodeEffect, AmbisonicsDecodeEffectParams,
    AmbisonicsDecodeEffectSettings, AudioBufferMut, AudioBufferRef, AudioSettings, BinauralEffect,
    BinauralEffectParams, BinauralEffectSettings, Context, CoordinateSystem, DirectEffect,
    DirectEffectParams, DirectEffectSettings, DirectSimulationParameters, DirectSimulationSettings,
    Direction, Directivity, DistanceAttenuationModel, Equalizer, Hrtf, HrtfInterpolation,
    HrtfSettings, Material, Occlusion, OcclusionAlgorithm, ParametricParameters,
    ParametricSettings, Point, ReflectionEffectSettings, ReflectionsSharedInputs, Rendering,
    SimulationFlags, SpeakerLayout, StaticMeshSettings, SteamAudioError, Transmission,
    TransmissionParameters, Triangle,
};
use firewheel::{
    channel_config::{ChannelConfig, ChannelCount},
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
        NodeError, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, Ordering},
    mpsc::{self, SyncSender, TrySendError},
};
use std::time::{Duration, Instant};

pub type AudioNimbusReflectionsOutput = audionimbus::wiring::SharedSimulationOutput<
    audionimbus::dynamic::ReflectionsReverbOutput<PortRef>,
>;
type ReflectionData = audionimbus::dynamic::ReflectionsReverbOutput<PortRef>;
pub type AudioNimbusReflectionsSnapshot =
    Arc<dyn std::ops::Deref<Target = ReflectionData> + Send + Sync>;
pub type AudioNimbusReflectionsFeed =
    Arc<Mutex<triple_buffer::Input<Option<AudioNimbusReflectionsSnapshot>>>>;

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
    pub reflections: AudioNimbusReflectionsFeed,
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
    latest: Arc<ArcSwap<DirectSimulationSnapshot>>,
    wake: SyncSender<bool>,
    thread: Option<std::thread::JoinHandle<()>>,
    last_wake: Option<Instant>,
    geometry: Arc<AcousticSceneGeometry>,
    geometry_revision: u64,
}

struct DirectSimulationSnapshot {
    listener: ListenerPose,
    emitters: Vec<DirectEmitterInput>,
    geometry: Arc<AcousticSceneGeometry>,
    geometry_revision: u64,
}

impl AudioNimbusDirectSimulation {
    pub fn new(sample_rate: u32, frame_size: u32) -> Result<Self, String> {
        let latest = Arc::new(ArcSwap::from_pointee(DirectSimulationSnapshot {
            listener: ListenerPose::default(),
            emitters: Vec::new(),
            geometry: Arc::new(AcousticSceneGeometry::default()),
            geometry_revision: 0,
        }));
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
            .map_err(|error| format!("spatial simulation worker did not initialize: {error}"))??;
        Ok(Self {
            latest,
            wake,
            thread: Some(thread),
            last_wake: None,
            geometry: Arc::new(AcousticSceneGeometry::default()),
            geometry_revision: 0,
        })
    }

    pub fn set_geometry(&mut self, geometry: AcousticSceneGeometry) {
        self.geometry = Arc::new(geometry);
        self.geometry_revision = self.geometry_revision.wrapping_add(1);
        let latest = self.latest.load();
        self.latest.store(Arc::new(DirectSimulationSnapshot {
            listener: latest.listener,
            emitters: latest.emitters.clone(),
            geometry: self.geometry.clone(),
            geometry_revision: self.geometry_revision,
        }));
        let _ = self.wake.try_send(true);
    }

    pub fn publish(&mut self, inputs: Vec<DirectEmitterInput>, listener: ListenerPose) {
        self.latest.store(Arc::new(DirectSimulationSnapshot {
            listener,
            emitters: inputs,
            geometry: self.geometry.clone(),
            geometry_revision: self.geometry_revision,
        }));
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
    latest: Arc<ArcSwap<DirectSimulationSnapshot>>,
    wake: mpsc::Receiver<bool>,
    sample_rate: u32,
    frame_size: u32,
    ready: SyncSender<Result<(), String>>,
) -> Result<(), String> {
    let trace = std::env::var_os("STUBBY_AUDIO_SPATIAL_TRACE").is_some();
    let context = Context::default();
    let audio_settings = AudioSettings {
        sampling_rate: sample_rate,
        frame_size,
    };
    let settings = audionimbus::dynamic::SimulationSettings::new(&audio_settings)
        .with_direct(DirectSimulationSettings {
            max_num_occlusion_samples: 16,
        })
        .with_reflections(ParametricSettings {
            max_num_rays: 512,
            num_diffuse_samples: 32,
            max_duration: 1.0,
            max_num_sources: 32,
            num_threads: 2,
            max_order: 1,
        });
    let mut simulator = audionimbus::dynamic::Simulator::try_new(&context, &settings)
        .map_err(|error| format!("create simulator: {error}"))?;
    let mut scene = audionimbus::dynamic::Scene::try_new(
        &context,
        audionimbus::dynamic::RayTracerSettings::Default,
    )
    .map_err(|error| format!("create scene: {error}"))?;
    scene.commit();
    let mut reflection_pipeline =
        audionimbus::dynamic::Simulation::new::<PortRef>(simulator.clone());
    let listener_source =
        audionimbus::dynamic::Source::try_new(&simulator, SimulationFlags::REFLECTIONS)
            .map_err(|error| format!("create reflections listener: {error}"))?;
    simulator
        .try_add_source(&listener_source)
        .map_err(|error| format!("add reflections listener: {error}"))?;
    simulator.commit();
    reflection_pipeline
        .set_listener(Some(reflections_listener(
            &listener_source,
            ListenerPose::default(),
        )))
        .map_err(|error| format!("set reflections listener: {error}"))?;
    let mut reflection_worker = reflection_pipeline
        .spawn_reflections_reverb(|error| eprintln!("AudioNimbus reflections simulation: {error}"))
        .map_err(|error| format!("start reflections simulation: {error}"))?;
    reflection_worker.pause();
    let mut reflection_running = false;
    let reflections = reflection_worker.output();
    ready
        .send(Ok(()))
        .map_err(|error| format!("publish worker readiness: {error}"))?;

    let mut source_map = std::collections::HashMap::new();
    let mut scene_revision = u64::MAX;
    let mut static_mesh = None;
    let mut traced_listener = None;
    let mut traced_reflection_counts = None;
    while let Ok(true) = wake.recv() {
        let snapshot = latest.load();
        if scene_revision != snapshot.geometry_revision {
            if let Some(handle) = static_mesh.take() {
                scene.remove_static_mesh(handle);
            }
            if !snapshot.geometry.triangles.is_empty() {
                let vertices: Vec<_> = snapshot
                    .geometry
                    .vertices
                    .iter()
                    .map(|point| Point::new(point[0], point[1], point[2]))
                    .collect();
                let mut triangles = Vec::new();
                let mut material_indices = Vec::new();
                for (index, indices) in snapshot.geometry.triangles.iter().enumerate() {
                    if indices.iter().any(|&vertex| {
                        vertex as usize >= vertices.len() || vertex > i32::MAX as u32
                    }) {
                        continue;
                    }
                    triangles.push(Triangle::new(
                        indices[0] as i32,
                        indices[1] as i32,
                        indices[2] as i32,
                    ));
                    material_indices.push(
                        snapshot
                            .geometry
                            .material_indices
                            .get(index)
                            .copied()
                            .filter(|&material| material < snapshot.geometry.materials.len())
                            .unwrap_or(0),
                    );
                }
                if !triangles.is_empty() {
                    let materials: Vec<_> = if snapshot.geometry.materials.is_empty() {
                        vec![Material::GENERIC]
                    } else {
                        snapshot
                            .geometry
                            .materials
                            .iter()
                            .copied()
                            .map(|material: AcousticMaterial| Material {
                                absorption: material.absorption,
                                scattering: material.scattering,
                                transmission: material.transmission,
                            })
                            .collect()
                    };
                    let mesh = audionimbus::dynamic::StaticMesh::try_new(
                        &scene,
                        &StaticMeshSettings {
                            vertices: &vertices,
                            triangles: &triangles,
                            material_indices: &material_indices,
                            materials: &materials,
                        },
                    )
                    .map_err(|error| format!("create acoustic mesh: {error}"))?;
                    static_mesh = Some(
                        scene
                            .try_add_static_mesh(mesh)
                            .map_err(|error| format!("add acoustic mesh: {error}"))?,
                    );
                }
            }
            scene.commit();
            simulator
                .try_set_scene(&scene)
                .map_err(|error| format!("set scene: {error}"))?;
            simulator.commit();
            if trace {
                let mut min = [f32::INFINITY; 3];
                let mut max = [f32::NEG_INFINITY; 3];
                for point in &snapshot.geometry.vertices {
                    for axis in 0..3 {
                        min[axis] = min[axis].min(point[axis]);
                        max[axis] = max[axis].max(point[axis]);
                    }
                }
                eprintln!(
                    "AudioNimbus scene rev={} vertices={} triangles={} materials={:?} bounds={min:?}..{max:?}",
                    snapshot.geometry_revision,
                    snapshot.geometry.vertices.len(),
                    snapshot.geometry.triangles.len(),
                    snapshot.geometry.materials,
                );
            }
            scene_revision = snapshot.geometry_revision;
        }
        let current: std::collections::HashSet<_> = snapshot
            .emitters
            .iter()
            .map(|emitter| emitter.output)
            .collect();
        source_map.retain(|output, source| {
            if current.contains(output) {
                true
            } else {
                let _ = simulator.try_remove_source(source);
                false
            }
        });
        for emitter in &snapshot.emitters {
            if !source_map.contains_key(&emitter.output) {
                let source = audionimbus::dynamic::Source::try_new(
                    &simulator,
                    SimulationFlags::DIRECT | SimulationFlags::REFLECTIONS,
                )
                .map_err(|error| format!("create source: {error}"))?;
                simulator
                    .try_add_source(&source)
                    .map_err(|error| format!("add source: {error}"))?;
                source_map.insert(emitter.output, source);
            }
        }
        simulator.commit();

        for emitter in &snapshot.emitters {
            let Some(source) = source_map.get(&emitter.output) else {
                continue;
            };
            let inputs = spatial_simulation_inputs(emitter.position);
            source
                .set_inputs(&inputs)
                .map_err(|error| format!("set source inputs: {error}"))?;
        }
        reflection_pipeline
            .update_sources(|active_sources| {
                for emitter in &snapshot.emitters {
                    if let Some(source) = source_map.get(&emitter.output) {
                        active_sources.push((
                            emitter.output,
                            audionimbus::dynamic::SourceWithInputs {
                                source: source.clone(),
                                simulation_inputs: spatial_simulation_inputs(emitter.position),
                            },
                        ));
                    }
                }
            })
            .map_err(|error| format!("publish reflections sources: {error}"))?;
        reflection_pipeline
            .set_listener(Some(reflections_listener(
                &listener_source,
                snapshot.listener,
            )))
            .map_err(|error| format!("update reflections listener: {error}"))?;
        let shared_inputs = audionimbus::dynamic::SimulationSharedInputs::new(CoordinateSystem {
            origin: Point::new(
                snapshot.listener.position[0],
                snapshot.listener.position[1],
                snapshot.listener.position[2],
            ),
            right: snapshot.listener.right.into(),
            up: snapshot.listener.up.into(),
            ahead: snapshot.listener.ahead.into(),
        })
        .with_reflections(ReflectionsSharedInputs {
            num_rays: 512,
            num_bounces: 4,
            duration: 1.0,
            order: 1,
            irradiance_min_distance: 1.0,
        });
        reflection_worker.set_input(shared_inputs.clone());
        if snapshot.emitters.is_empty() && reflection_running {
            reflection_worker.pause();
            reflection_running = false;
        } else if !snapshot.emitters.is_empty() && !reflection_running {
            reflection_worker.resume();
            reflection_running = true;
        }
        let reflection_snapshot = reflections.load();
        let matching_sources = snapshot
            .emitters
            .iter()
            .filter(|emitter| reflection_snapshot.sources.contains_key(&emitter.output))
            .count();
        let reflection_counts = (reflection_snapshot.sources.len(), matching_sources);
        if trace && traced_reflection_counts != Some(reflection_counts) {
            eprintln!(
                "AudioNimbus reflections outputs={} matching_emitters={matching_sources} listener={:?}",
                reflection_counts.0, snapshot.listener.position,
            );
            traced_reflection_counts = Some(reflection_counts);
        }
        let reflection_snapshot = Arc::clone(&*reflection_snapshot);
        let reflection_snapshot: AudioNimbusReflectionsSnapshot = reflection_snapshot;
        for emitter in &snapshot.emitters {
            if let Ok(mut feed) = emitter.reflections.lock() {
                feed.write(Some(Arc::clone(&reflection_snapshot)));
            }
        }
        simulator
            .set_shared_inputs(
                SimulationFlags::DIRECT | SimulationFlags::REFLECTIONS,
                &shared_inputs,
            )
            .map_err(|error| format!("set listener inputs: {error}"))?;
        simulator
            .run_direct()
            .map_err(|error| format!("run direct simulation: {error}"))?;
        let trace_listener_changed = trace && traced_listener != Some(snapshot.listener);
        for emitter in &snapshot.emitters {
            if let Some(source) = source_map.get(&emitter.output) {
                if let Ok(params) = source.get_direct_outputs() {
                    if trace_listener_changed {
                        eprintln!(
                            "AudioNimbus direct output={:?} source={:?} listener={:?} params={params:?}",
                            emitter.output, emitter.position, snapshot.listener.position,
                        );
                    }
                    emitter.params.store(&params);
                }
            }
        }
        if trace_listener_changed {
            traced_listener = Some(snapshot.listener);
        }
    }
    reflection_worker.shutdown();
    let _ = reflection_worker.join();
    Ok(())
}

fn spatial_simulation_inputs(position: [f32; 3]) -> audionimbus::dynamic::SimulationInputs {
    audionimbus::dynamic::SimulationInputs {
        source: CoordinateSystem {
            origin: Point::new(position[0], position[1], position[2]),
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
                    .with_occlusion(
                        Occlusion::new(OcclusionAlgorithm::Raycast).with_transmission(
                            TransmissionParameters {
                                num_transmission_rays: 16,
                            },
                        ),
                    ),
            ),
            reflections: Some(ParametricParameters::default().into()),
            ..Default::default()
        },
    }
}

fn reflections_listener(
    source: &audionimbus::dynamic::Source,
    pose: ListenerPose,
) -> audionimbus::dynamic::SourceWithInputs {
    audionimbus::dynamic::SourceWithInputs {
        source: source.clone(),
        simulation_inputs: audionimbus::dynamic::SimulationInputs {
            source: CoordinateSystem {
                origin: Point::new(pose.position[0], pose.position[1], pose.position[2]),
                right: pose.right.into(),
                up: pose.up.into(),
                ahead: pose.ahead.into(),
            },
            parameters: audionimbus::dynamic::SimulationParameters {
                reflections: Some(ParametricParameters::default().into()),
                ..Default::default()
            },
        },
    }
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
    reflection_wet_mix: Arc<AtomicU32>,
    direct_params: Arc<DirectEffectParamsShared>,
    reflections: Arc<Mutex<Option<triple_buffer::Output<Option<AudioNimbusReflectionsSnapshot>>>>>,
    reflection_feed: AudioNimbusReflectionsFeed,
    output: PortRef,
}

impl AudioNimbusBinauralNode {
    /// Creates a mono-to-stereo node using shared AudioNimbus state.
    pub fn new(
        spatial: Arc<AudioNimbusSpatialContext>,
        direct_params: Arc<DirectEffectParamsShared>,
        output: PortRef,
    ) -> Self {
        let (feed, reflections) = triple_buffer::TripleBuffer::new(&None).split();
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
            reflection_wet_mix: Arc::new(AtomicU32::new(0.75_f32.to_bits())),
            direct_params,
            reflections: Arc::new(Mutex::new(Some(reflections))),
            reflection_feed: Arc::new(Mutex::new(feed)),
            output,
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

    /// Publishes the per-emitter reflection wet gain without locking.
    pub fn set_reflection_wet_mix(&self, mix: f32) {
        self.reflection_wet_mix
            .store(mix.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
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

    pub fn reflections(&self) -> AudioNimbusReflectionsFeed {
        self.reflection_feed.clone()
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
        let reflections = self
            .reflections
            .lock()
            .map_err(|_| NodeError::from(std::io::Error::other("reflection feed poisoned")))?
            .take()
            .ok_or_else(|| {
                NodeError::from(std::io::Error::other("reflection feed already used"))
            })?;
        Ok(AudioNimbusBinauralProcessor {
            spatializer: self
                .spatial
                .create_binaural_spatializer_with_params(self.direct_params.clone())?,
            direction: self.direction.clone(),
            volume: self.volume.clone(),
            reflection_wet_mix: self.reflection_wet_mix.clone(),
            direct_params: self.direct_params.clone(),
            reflections,
            output: self.output,
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
    reflection_wet_mix: Arc<AtomicU32>,
    direct_params: Arc<DirectEffectParamsShared>,
    reflections: triple_buffer::Output<Option<AudioNimbusReflectionsSnapshot>>,
    output: PortRef,
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
                self.spatializer.set_reflection_wet_mix(f32::from_bits(
                    self.reflection_wet_mix.load(Ordering::Relaxed),
                ));
                let reflection_output = self.reflections.read();
                let reflection_params = reflection_output
                    .as_ref()
                    .and_then(|snapshot| (**snapshot).sources.get(&self.output));
                if self
                    .spatializer
                    .process(
                        &self.input,
                        &mut self.left,
                        &mut self.right,
                        reflection_params,
                    )
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
        let reflection_effect = ReflectionEffect::try_new(
            &self.context,
            &self.audio_settings,
            &ReflectionEffectSettings {
                impulse_response_size: self.audio_settings.sampling_rate,
                num_channels: 4,
            },
            ReflectionAlgorithm::Parametric,
        )?;
        let reflection_decode = AmbisonicsDecodeEffect::try_new(
            &self.context,
            &self.audio_settings,
            &AmbisonicsDecodeEffectSettings {
                speaker_layout: SpeakerLayout::Stereo,
                hrtf: self.hrtf.clone(),
                max_order: 1,
                rendering: Rendering::Binaural,
            },
        )?;
        let reflection_decode_params = AmbisonicsDecodeEffectParams {
            order: 1,
            hrtf: self.hrtf.clone(),
            orientation: CoordinateSystem::default(),
        };
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
            reflection_effect,
            reflection_decode,
            reflection_decode_params,
            direct_params: DirectEffectParams::default(),
            direct_buffer: vec![0.0; self.audio_settings.frame_size as usize],
            reflection_buffer: vec![0.0; self.audio_settings.frame_size as usize * 4],
            wet_left: vec![0.0; self.audio_settings.frame_size as usize],
            wet_right: vec![0.0; self.audio_settings.frame_size as usize],
            params,
            reflection_wet_mix: 0.75,
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
    reflection_effect: ReflectionEffect,
    reflection_decode: AmbisonicsDecodeEffect,
    reflection_decode_params: AmbisonicsDecodeEffectParams,
    direct_params: DirectEffectParams,
    direct_buffer: Vec<f32>,
    reflection_buffer: Vec<f32>,
    wet_left: Vec<f32>,
    wet_right: Vec<f32>,
    params: BinauralEffectParams,
    reflection_wet_mix: f32,
    frame_size: usize,
}

impl BinauralSpatializer {
    /// Changes the unit direction from listener to source.
    pub fn set_direction(&mut self, direction: [f32; 3]) {
        self.params.direction = Direction::new(direction[0], direction[1], direction[2]);
    }

    /// Sets the linear contribution of per-emitter reflections.
    pub fn set_reflection_wet_mix(&mut self, mix: f32) {
        self.reflection_wet_mix = mix.clamp(0.0, 1.0);
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
        reflection_params: Option<&DynamicReflectionEffectParams>,
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
        {
            let mut output_buffer = AudioBufferMut::try_from_channels([&mut *left, &mut *right])?;
            self.effect
                .apply(&self.params, &input_buffer, &mut output_buffer)?;
        }
        if let Some(reflection_params) = reflection_params
            && self.reflection_wet_mix > 0.0
        {
            self.reflection_buffer.fill(0.0);
            self.wet_left.fill(0.0);
            self.wet_right.fill(0.0);
            let input_buffer = AudioBufferRef::try_from(self.direct_buffer.as_slice())?;
            let frame_size = self.frame_size;
            let (channel_0, rest) = self.reflection_buffer.split_at_mut(frame_size);
            let (channel_1, rest) = rest.split_at_mut(frame_size);
            let (channel_2, channel_3) = rest.split_at_mut(frame_size);
            let mut reflections =
                AudioBufferMut::try_from_channels([channel_0, channel_1, channel_2, channel_3])?;
            self.reflection_effect
                .apply(reflection_params, &input_buffer, &mut reflections)
                .map_err(|_| SpatialProcessError::Reflection)?;
            let (channel_0, rest) = self.reflection_buffer.split_at(frame_size);
            let (channel_1, rest) = rest.split_at(frame_size);
            let (channel_2, channel_3) = rest.split_at(frame_size);
            let reflections =
                AudioBufferRef::try_from_channels([channel_0, channel_1, channel_2, channel_3])?;
            let mut wet = AudioBufferMut::try_from_channels([
                &mut self.wet_left[..],
                &mut self.wet_right[..],
            ])?;
            self.reflection_decode
                .apply(&self.reflection_decode_params, &reflections, &mut wet)?;
            for (dry, wet) in left.iter_mut().zip(&self.wet_left) {
                *dry += wet * self.reflection_wet_mix;
            }
            for (dry, wet) in right.iter_mut().zip(&self.wet_right) {
                *dry += wet * self.reflection_wet_mix;
            }
        }
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
    #[error("AudioNimbus reflection effect failed")]
    Reflection,
}

#[cfg(test)]
mod simulation_tests {
    use super::*;

    #[test]
    fn direct_simulation_reports_transmission_through_static_wall() {
        let mut simulation = AudioNimbusDirectSimulation::new(48_000, 256).unwrap();
        simulation.set_geometry(AcousticSceneGeometry {
            vertices: vec![
                [-10.0, -10.0, 1.0],
                [10.0, -10.0, 1.0],
                [10.0, 10.0, 1.0],
                [-10.0, 10.0, 1.0],
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            material_indices: vec![0, 0],
            materials: vec![AcousticMaterial {
                absorption: [0.8; 3],
                scattering: 0.1,
                transmission: [0.0; 3],
            }],
        });
        let params = Arc::new(DirectEffectParamsShared::default());
        let (reflection_feed, _reflection_reader) = triple_buffer::TripleBuffer::new(&None).split();
        simulation.publish(
            vec![DirectEmitterInput {
                output: PortRef::new(crate::ModuleId(42), "out"),
                position: [0.0, 0.0, 3.0],
                params: params.clone(),
                reflections: Arc::new(Mutex::new(reflection_feed)),
            }],
            ListenerPose::default(),
        );

        for _ in 0..100 {
            if params.flags.load(Ordering::Acquire) & (1 << 3) != 0 {
                let flags = params.flags.load(Ordering::Acquire);
                let transmission = params.values[6..9]
                    .iter()
                    .map(|value| f32::from_bits(value.load(Ordering::Relaxed)))
                    .collect::<Vec<_>>();
                assert!(
                    flags & (1 << 4) != 0,
                    "wall did not produce transmission output"
                );
                assert!(transmission.iter().all(|&value| value < 0.1));
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("simulation did not publish occlusion within two seconds");
    }

    #[test]
    fn rock_material_reduces_transmission_more_than_generic() {
        fn wall(transmission: [f32; 3]) -> AcousticSceneGeometry {
            AcousticSceneGeometry {
                vertices: vec![
                    [-10.0, -10.0, 1.0],
                    [10.0, -10.0, 1.0],
                    [10.0, 10.0, 1.0],
                    [-10.0, 10.0, 1.0],
                ],
                triangles: vec![[0, 1, 2], [0, 2, 3]],
                material_indices: vec![0, 0],
                materials: vec![AcousticMaterial {
                    absorption: [0.13, 0.20, 0.24],
                    scattering: 0.05,
                    transmission,
                }],
            }
        }

        fn wait_for_transmission(
            params: &DirectEffectParamsShared,
            previous: Option<[f32; 3]>,
        ) -> [f32; 3] {
            for _ in 0..100 {
                let flags = params.flags.load(Ordering::Acquire);
                if flags & (1 << 4) != 0 {
                    let transmission = params.values[6..9]
                        .iter()
                        .map(|value| f32::from_bits(value.load(Ordering::Relaxed)))
                        .collect::<Vec<_>>();
                    let transmission = [transmission[0], transmission[1], transmission[2]];
                    if previous.is_none_or(|previous| {
                        transmission
                            .iter()
                            .zip(previous)
                            .any(|(current, previous)| (current - previous).abs() > 1e-4)
                    }) {
                        return transmission;
                    }
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            panic!("simulation did not publish updated transmission within two seconds");
        }

        let output = PortRef::new(crate::ModuleId(43), "out");
        let params = Arc::new(DirectEffectParamsShared::default());
        let (reflection_feed, _reflection_reader) = triple_buffer::TripleBuffer::new(&None).split();
        let emitter = DirectEmitterInput {
            output,
            position: [0.0, 0.0, 3.0],
            params: params.clone(),
            reflections: Arc::new(Mutex::new(reflection_feed)),
        };
        let mut simulation = AudioNimbusDirectSimulation::new(48_000, 256).unwrap();
        simulation.set_geometry(wall([0.100, 0.050, 0.030]));
        simulation.publish(vec![emitter.clone()], ListenerPose::default());
        let generic = wait_for_transmission(&params, None);

        simulation.set_geometry(wall([0.015, 0.002, 0.001]));
        simulation.publish(vec![emitter], ListenerPose::default());
        let rock = wait_for_transmission(&params, Some(generic));
        eprintln!("AudioNimbus wall transmission: generic={generic:?}, rock={rock:?}");
        assert!(
            rock.iter()
                .zip(generic)
                .all(|(rock, generic)| *rock < generic),
            "rock should reduce simulated transmission in every band"
        );
    }

    #[test]
    fn parametric_reverb_renders_a_tail_in_a_synthetic_room() {
        use audionimbus::ReflectionsSharedInputs;
        use audionimbus::dynamic::{
            ReflectionAlgorithm, ReflectionEffect, SimulationSettings, Simulator, Source,
        };

        let context = Context::default();
        let audio_settings = AudioSettings {
            sampling_rate: 48_000,
            frame_size: 256,
        };
        let settings = SimulationSettings::new(&audio_settings).with_reflections(
            audionimbus::ParametricSettings {
                max_num_rays: 256,
                num_diffuse_samples: 32,
                max_duration: 1.0,
                max_num_sources: 1,
                num_threads: 1,
                max_order: 1,
            },
        );
        let mut simulator = Simulator::try_new(&context, &settings).unwrap();
        let mut scene = audionimbus::dynamic::Scene::try_new(
            &context,
            audionimbus::dynamic::RayTracerSettings::Default,
        )
        .unwrap();
        let mesh = audionimbus::dynamic::StaticMesh::try_new(
            &scene,
            &StaticMeshSettings {
                vertices: &[
                    [-4.0, -4.0, -4.0],
                    [4.0, -4.0, -4.0],
                    [4.0, 4.0, -4.0],
                    [-4.0, 4.0, -4.0],
                    [-4.0, -4.0, 4.0],
                    [4.0, -4.0, 4.0],
                    [4.0, 4.0, 4.0],
                    [-4.0, 4.0, 4.0],
                ]
                .map(|point| Point::new(point[0], point[1], point[2])),
                triangles: &[
                    Triangle::new(0, 2, 1),
                    Triangle::new(0, 3, 2),
                    Triangle::new(4, 5, 6),
                    Triangle::new(4, 6, 7),
                    Triangle::new(0, 1, 5),
                    Triangle::new(0, 5, 4),
                    Triangle::new(3, 7, 6),
                    Triangle::new(3, 6, 2),
                    Triangle::new(0, 4, 7),
                    Triangle::new(0, 7, 3),
                    Triangle::new(1, 2, 6),
                    Triangle::new(1, 6, 5),
                ],
                materials: &[Material::CONCRETE],
                material_indices: &[0; 12],
            },
        )
        .unwrap();
        scene.try_add_static_mesh(mesh).unwrap();
        scene.commit();
        simulator.try_set_scene(&scene).unwrap();
        let source = Source::try_new(&simulator, SimulationFlags::REFLECTIONS).unwrap();
        simulator.try_add_source(&source).unwrap();
        simulator.commit();
        source
            .set_inputs(&audionimbus::dynamic::SimulationInputs {
                source: CoordinateSystem {
                    origin: Point::new(1.0, 0.0, 0.0),
                    ..Default::default()
                },
                parameters: audionimbus::dynamic::SimulationParameters {
                    reflections: Some(audionimbus::ParametricParameters::default().into()),
                    ..Default::default()
                },
            })
            .unwrap();
        simulator
            .set_shared_inputs(
                SimulationFlags::REFLECTIONS,
                &audionimbus::dynamic::SimulationSharedInputs::new(CoordinateSystem::default())
                    .with_reflections(ReflectionsSharedInputs {
                        num_rays: 256,
                        num_bounces: 4,
                        duration: 1.0,
                        order: 1,
                        irradiance_min_distance: 1.0,
                    }),
            )
            .unwrap();
        simulator.run_reflections().unwrap();
        let params = source.get_reflections_outputs().unwrap();
        let effect_settings = audionimbus::ReflectionEffectSettings {
            impulse_response_size: audio_settings.sampling_rate,
            num_channels: 4,
        };
        let mut effect = ReflectionEffect::try_new(
            &context,
            &audio_settings,
            &effect_settings,
            ReflectionAlgorithm::Parametric,
        )
        .unwrap();
        let mut input = vec![0.0; audio_settings.frame_size as usize];
        let mut output = vec![0.0; 4 * audio_settings.frame_size as usize];
        input[0] = 1.0;
        let mut energy = 0.0;
        for _ in 0..16 {
            let input_buffer = AudioBufferRef::try_from(input.as_slice()).unwrap();
            let mut output_buffer = AudioBufferMut::try_new(&mut output, 4).unwrap();
            effect
                .apply(&params, &input_buffer, &mut output_buffer)
                .unwrap();
            assert!(output.iter().all(|sample| sample.is_finite()));
            energy += output.iter().map(|sample| sample * sample).sum::<f32>();
            input.fill(0.0);
        }
        assert!(energy > 0.0, "synthetic room produced no reverb tail");
    }
}
