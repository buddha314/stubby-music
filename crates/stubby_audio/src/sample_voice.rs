//! A patch-cable trigger for fundsp's one-shot wave player.

use crate::signal::is_gate_high;
use firewheel::{
    channel_config::{ChannelConfig, ChannelCount},
    event::ProcEvents,
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
        NodeError, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use fundsp::{audionode::AudioNode as _, wave::WavePlayer};

pub(crate) struct SampleVoice;

impl AudioNode for SampleVoice {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("triggered sample")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::MONO,
                num_outputs: ChannelCount::MONO,
            }))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(Voice {
            fade_frames: (cx.stream_info.sample_rate.get() / 1000).max(1),
            ..Voice::default()
        })
    }
}

#[derive(Default)]
struct Voice {
    selected: Option<WavePlayer>,
    pending: Option<WavePlayer>,
    selection_changed: bool,
    gate_high: bool,
    playing: bool,
    retiring: bool,
    fade_frames: u32,
    fade_left: u32,
    previous: f32,
    last: f32,
}

impl Voice {
    fn fade(&mut self) {
        self.previous = self.last;
        self.fade_left = self.fade_frames;
    }
}

impl AudioNodeProcessor for Voice {
    fn events(&mut self, _: &ProcInfo, events: &mut ProcEvents, _: &mut ProcExtra) {
        for mut event in events.drain() {
            // Firewheel disposes of the swapped-out player on its control thread.
            if event.downcast_swap(&mut self.pending) {
                self.selection_changed = true;
            } else if event.downcast_ref::<RetireSample>().is_some() {
                self.retiring = true;
                self.playing = false;
                self.fade();
            }
        }
    }

    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        for (&gate, output) in buffers.inputs[0]
            .iter()
            .zip(buffers.outputs[0].iter_mut())
            .take(info.frames)
        {
            let high = is_gate_high(gate);
            if high && !self.gate_high && !self.retiring {
                if self.selection_changed {
                    std::mem::swap(&mut self.selected, &mut self.pending);
                    self.selection_changed = false;
                }
                if let Some(player) = &mut self.selected {
                    player.reset();
                }
                self.playing = self.selected.is_some();
                self.fade();
            }
            self.gate_high = high;
            let sample = if self.playing {
                self.selected
                    .as_mut()
                    .map_or(0., |player| player.tick(&Default::default())[0])
            } else {
                0.
            };
            if self.fade_left > 0 {
                let blend = self.fade_left as f32 / self.fade_frames as f32;
                self.last = self.previous * blend + sample * (1. - blend);
                self.fade_left -= 1;
            } else {
                self.last = sample;
            }
            *output = self.last;
        }
        ProcessStatus::OutputsModified
    }
}

pub(crate) struct RetireSample;
