//! Seeded, sample-counted random triggers. No allocation or locks in `process`.
use crate::{Event, ModuleId, PulseClock, RandomClock, modules::ClockConfig, signal::GATE_HIGH};
use firewheel::{
    channel_config::{ChannelConfig, ChannelCount},
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, NodeError,
        ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RandomClockNode {
    pub seed: u64,
    pub min_interval_s: f32,
    pub max_interval_s: f32,
}

impl AudioNode for RandomClockNode {
    type Configuration = ClockConfig;
    fn info(&self, _: &ClockConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("random-clock")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::ZERO,
                num_outputs: ChannelCount::MONO,
            }))
    }
    fn construct_processor(
        &self,
        config: &ClockConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let (events, consumer) = rtrb::RingBuffer::new(1024);
        // Construction runs off the realtime thread, just like the regular clock.
        if let Ok(mut mailbox) = config.mailbox.lock() {
            mailbox.push((config.module, consumer));
        }
        let rate = cx.stream_info.sample_rate.get() as f64;
        let min_frames = (self.min_interval_s as f64 * rate).ceil().max(1.) as u64;
        let max_frames = (self.max_interval_s as f64 * rate).floor().max(1.) as u64;
        Ok(RandomClockProc {
            module: config.module,
            events,
            clock: RandomClock::new(self.seed, min_frames, max_frames),
            trigger_left: 0,
            trigger_frames: (rate * 0.005).round().max(1.) as u32,
        })
    }
}

struct RandomClockProc {
    module: ModuleId,
    events: rtrb::Producer<Event>,
    clock: RandomClock,
    trigger_left: u32,
    trigger_frames: u32,
}
impl AudioNodeProcessor for RandomClockProc {
    fn process(
        &mut self,
        info: &ProcInfo,
        buffers: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        let Some(gate) = buffers.outputs.first_mut() else {
            return ProcessStatus::ClearAllOutputs;
        };
        let start = info.clock_samples.0.max(0) as u64;
        for (i, value) in gate.iter_mut().enumerate() {
            if self.clock.advance() {
                let _ = self.events.push(Event::ClockTick {
                    module: self.module,
                    sample_time: start + i as u64,
                });
                self.trigger_left = self.trigger_frames;
            }
            *value = if self.trigger_left > 0 { GATE_HIGH } else { 0. };
            self.trigger_left = self.trigger_left.saturating_sub(1);
        }
        ProcessStatus::OutputsModified
    }
}
