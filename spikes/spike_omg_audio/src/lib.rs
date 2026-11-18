//! Minimal Firewheel host for OMG's low-level live-signal renderer.
//!
//! This is an isolated compatibility spike. It intentionally leaves source
//! geometry and per-frame parameter replacement to the next prototype step.

use firewheel::{
    channel_config::{ChannelConfig, ChannelCount},
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
        NodeError, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use omg_audio::Frame;
use omg_dsp::{
    Renderer,
    ambi::NCH,
    output::OutputStage,
    renderer::{BLOCK, BusBlock},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct OmgLiveNode {
    params: Arc<omg_core::params::ParamBlock>,
}

impl OmgLiveNode {
    /// Build one node from a simulation result. `set_params` runs while
    /// Firewheel constructs the processor, outside its audio callback.
    pub fn from_frame(frame: &Frame, source: omg_audio::SourceId) -> Option<Self> {
        Some(Self {
            params: Arc::new(frame.block(source)?.clone()),
        })
    }
}

impl AudioNode for OmgLiveNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("omg-live-renderer-spike")
            .channel_config(ChannelConfig::new(ChannelCount::MONO, ChannelCount::STEREO)))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let sample_rate = cx.stream_info.sample_rate.get() as f32;
        let mut renderer = Renderer::new(sample_rate);
        renderer.set_params(&self.params);
        Ok(OmgLiveProcessor {
            renderer,
            output: OutputStage::from_speaker_bytes(None, sample_rate),
            bus: [[0.0; BLOCK]; NCH],
            point_left: [0.0; BLOCK],
            point_right: [0.0; BLOCK],
        })
    }
}

pub struct OmgLiveProcessor {
    renderer: Renderer,
    output: OutputStage,
    bus: BusBlock,
    point_left: [f32; BLOCK],
    point_right: [f32; BLOCK],
}

impl OmgLiveProcessor {
    /// Process caller-owned mono blocks without allocating. Firewheel invokes
    /// this same method from `AudioNodeProcessor::process`.
    pub fn process_samples(&mut self, input: &[f32], left: &mut [f32], right: &mut [f32]) {
        let frames = input.len().min(left.len()).min(right.len());
        for start in (0..frames).step_by(BLOCK) {
            let count = (frames - start).min(BLOCK);
            self.bus
                .iter_mut()
                .for_each(|channel| channel[..count].fill(0.0));
            self.point_left[..count].fill(0.0);
            self.point_right[..count].fill(0.0);
            self.renderer.process_block(
                &input[start..start + count],
                &mut self.bus,
                &mut self.point_left[..count],
                &mut self.point_right[..count],
            );
            for index in 0..count {
                let sh = std::array::from_fn(|channel| self.bus[channel][index]);
                let (l, r) =
                    self.output
                        .process(&sh, self.point_left[index], self.point_right[index]);
                left[start + index] = l;
                right[start + index] = r;
            }
        }
    }
}

impl AudioNodeProcessor for OmgLiveProcessor {
    fn process(
        &mut self,
        info: &ProcInfo,
        ProcBuffers { inputs, outputs }: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        let Some(input) = inputs.first() else {
            return ProcessStatus::ClearAllOutputs;
        };
        let (left_output, right_outputs) = outputs.split_at_mut(1);
        let (Some(left), Some(right)) = (left_output.first_mut(), right_outputs.first_mut()) else {
            return ProcessStatus::ClearAllOutputs;
        };
        let frames = info
            .frames
            .min(input.len())
            .min(left.len())
            .min(right.len());
        self.process_samples(&input[..frames], &mut left[..frames], &mut right[..frames]);
        ProcessStatus::OutputsModified
    }
}

/// Tiny game-side signal stand-in, evaluated by Firewheel immediately before
/// the OMG node so the test covers live modular audio rather than a clip.
#[cfg(test)]
struct SquareSource;

#[cfg(test)]
impl AudioNode for SquareSource {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("procedural-square-source")
            .channel_config(ChannelConfig::new(ChannelCount::ZERO, ChannelCount::MONO)))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(SquareProcessor {
            phase: 0.0,
            increment: 220.0 * cx.stream_info.sample_rate_recip,
        })
    }
}

#[cfg(test)]
struct SquareProcessor {
    phase: f64,
    increment: f64,
}

#[cfg(test)]
impl AudioNodeProcessor for SquareProcessor {
    fn process(
        &mut self,
        _: &ProcInfo,
        ProcBuffers { outputs, .. }: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        let Some(output) = outputs.first_mut() else {
            return ProcessStatus::ClearAllOutputs;
        };
        for sample in output.iter_mut() {
            *sample = if self.phase < 0.5 { 0.2 } else { -0.2 };
            self.phase = (self.phase + self.increment).fract();
        }
        ProcessStatus::OutputsModified
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_no_alloc::{AllocDisabler, assert_no_alloc};
    use audioadapter_buffers::direct::InterleavedSlice;
    use firewheel::{
        ActivateInfo, FirewheelConfig, FirewheelContext, backend::BackendProcessInfo,
        channel_config::ChannelCount, node::StreamStatus,
    };
    use omg_audio::{Config, Simulation, SourceDesc};
    use std::{num::NonZeroU32, time::Duration};

    #[global_allocator]
    static ALLOC: AllocDisabler = AllocDisabler;

    #[test]
    fn live_signal_render_is_finite_and_callback_allocation_free() {
        let mut simulation = Simulation::new(Config {
            v4: false,
            v5: false,
            v6: true,
            v61: true,
            v62: true,
            ..Config::default()
        });
        let source = simulation.add_source(SourceDesc::at([0.0, 0.0, 0.0]));
        simulation.set_listener([1.0, 0.0, 0.0], 0.0, 0.0, 0.0);
        let frame = simulation.tick();
        let node = OmgLiveNode::from_frame(&frame, source).expect("source frame");
        let mut cx = FirewheelContext::new(FirewheelConfig {
            num_graph_inputs: ChannelCount::ZERO,
            num_graph_outputs: ChannelCount::STEREO,
            ..Default::default()
        });
        let source_id = cx.add_node(SquareSource, None).unwrap();
        let node_id = cx.add_node(node, None).unwrap();
        let graph_out = cx.graph_out_node_id();
        cx.connect(source_id, node_id, &[(0, 0)], false).unwrap();
        cx.connect(node_id, graph_out, &[(0, 0), (1, 1)], false)
            .unwrap();
        let mut processor = cx
            .activate(ActivateInfo {
                sample_rate: NonZeroU32::new(48_000).unwrap(),
                max_block_frames: NonZeroU32::new(BLOCK as u32).unwrap(),
                num_stream_in_channels: 0,
                num_stream_out_channels: 2,
                input_to_output_latency_seconds: 0.0,
            })
            .unwrap();
        cx.update().unwrap();

        // A one-metre direct path takes about 140 samples to arrive.
        let mut output = [0.0; BLOCK * 2];
        let mut heard = false;
        for block in 0..16 {
            let empty: [f32; 0] = [];
            let input = InterleavedSlice::new(&empty, 0, BLOCK).unwrap();
            let mut output_view = InterleavedSlice::new_mut(&mut output, 2, BLOCK).unwrap();
            let before = assert_no_alloc::violation_count();
            assert_no_alloc(|| {
                processor.process(
                    &input,
                    &mut output_view,
                    BackendProcessInfo {
                        frames: BLOCK,
                        process_timestamp: None,
                        duration_since_stream_start: Duration::from_secs_f64(
                            block as f64 * BLOCK as f64 / 48_000.0,
                        ),
                        input_stream_status: StreamStatus::empty(),
                        output_stream_status: StreamStatus::empty(),
                        dropped_frames: 0,
                        process_to_playback_delay: None,
                    },
                );
            });
            assert_eq!(assert_no_alloc::violation_count(), before);
            assert!(output.iter().all(|sample| sample.is_finite()));
            heard |= output.iter().any(|sample| sample.abs() > 1.0e-6);
        }
        assert!(heard, "the generated live source never reached the output");
    }
}
