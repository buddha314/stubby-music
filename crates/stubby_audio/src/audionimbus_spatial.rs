//! AudioNimbus-backed spatial effects used by Firewheel processors.
//!
//! Create effects on the control thread, then reuse them with caller-owned buffers in
//! the audio processor. Simulation and scene updates belong on worker/control threads.

use audionimbus::{
    AudioBufferMut, AudioBufferRef, AudioSettings, BinauralEffect, BinauralEffectParams,
    BinauralEffectSettings, Context, Direction, Hrtf, HrtfInterpolation, HrtfSettings,
    SteamAudioError,
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
};

/// Shared AudioNimbus state for spatial processors at one sample rate and block size.
///
/// Create this once on the control thread, then use it to create one processor per
/// voice. HRTF setup is intentionally shared rather than repeated per emitter.
pub struct AudioNimbusSpatialContext {
    context: Context,
    hrtf: Hrtf,
    audio_settings: AudioSettings,
}

/// Firewheel node that runs the AudioNimbus binaural effect with fixed-size internal
/// blocks. The block adapter accepts any Firewheel callback length; it adds one
/// AudioNimbus block of latency and keeps all buffers preallocated.
#[derive(Clone)]
pub struct AudioNimbusBinauralNode {
    spatial: Arc<AudioNimbusSpatialContext>,
    direction: Arc<[AtomicU32; 3]>,
}

impl AudioNimbusBinauralNode {
    /// Creates a mono-to-stereo node using shared AudioNimbus state.
    pub fn new(spatial: Arc<AudioNimbusSpatialContext>) -> Self {
        Self {
            spatial,
            direction: Arc::new([
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(1.0_f32.to_bits()),
            ]),
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
            spatializer: self.spatial.create_binaural_spatializer()?,
            direction: self.direction.clone(),
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
        let effect = BinauralEffect::try_new(
            &self.context,
            &self.audio_settings,
            &BinauralEffectSettings {
                hrtf: self.hrtf.clone(),
            },
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

        let input_buffer = AudioBufferRef::try_from(input)?;
        let mut output_buffer = AudioBufferMut::try_from_channels([left, right])?;
        self.effect
            .apply(&self.params, &input_buffer, &mut output_buffer)?;
        Ok(())
    }

    /// Returns the HRTF handle used by this processor.
    pub fn hrtf(&self) -> &Hrtf {
        &self.hrtf
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
