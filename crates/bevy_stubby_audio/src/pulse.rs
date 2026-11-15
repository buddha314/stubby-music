//! The pulse bus: engine clock ticks, carried at their sample time and predicted
//! display instant, for anything that flashes or moves in time with the music. The audio
//! clock is the musical source of truth (AGENTS.md); visuals follow it, not frame time.
use crate::ClockTick;
use bevy::prelude::*;
use std::{collections::HashMap, time::Instant};

/// The clock a [`PulseFollower`] follows until it is told otherwise: the first engine
/// clock ID.
pub const DEFAULT_CLOCK: u64 = 1;

/// Every follower gets the same sample timestamp and predicted display instant.
#[derive(Message, Debug, Clone, Copy)]
pub struct SamplePulse {
    pub clock: u64,
    pub sample_time: u64,
    pub at: Instant,
}

/// Attach to future pulse-driven assets as well as the lightning bug.
#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component, Default)]
pub struct PulseFollower {
    pub clock: u64,
}
impl Default for PulseFollower {
    fn default() -> Self {
        Self {
            clock: DEFAULT_CLOCK,
        }
    }
}

/// Receives engine clock ticks into the shared [`PulseBus`]. Added by every plugin that
/// follows pulses (lightning bugs, synth module lights), guarded by `is_plugin_added`.
pub struct PulseBusPlugin;
impl Plugin for PulseBusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PulseBus>()
            .add_message::<SamplePulse>()
            .init_resource::<PulseAliases>()
            .add_systems(Update, poll_audio.in_set(PulseReceive));
    }
}

/// Systems that read pulse levels run after this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PulseReceive;

#[derive(Resource, Default)]
pub struct PulseBus {
    first: HashMap<u64, SamplePulse>,
    pending: Vec<SamplePulse>,
    latest: HashMap<u64, SamplePulse>,
}
impl PulseBus {
    pub fn forget(&mut self, clock: u64) {
        self.pending.retain(|pulse| pulse.clock != clock);
        self.latest.remove(&clock);
        self.first.remove(&clock);
    }
    pub fn elapsed(&self, clock: u64, now: Instant, sample_rate: u32) -> Option<f64> {
        let first = self.first.get(&clock)?;
        let last = self.latest.get(&clock)?;
        let age = now.checked_duration_since(last.at)?.as_secs_f64();
        Some(
            last.sample_time.saturating_sub(first.sample_time) as f64 / f64::from(sample_rate)
                + age,
        )
    }
    pub fn queue(&mut self, pulse: SamplePulse) {
        self.pending.push(pulse);
    }
    pub fn advance(&mut self, now: Instant) {
        let latest = &mut self.latest;
        let first = &mut self.first;
        self.pending.retain(|pulse| {
            if pulse.at > now {
                return true;
            }
            first.entry(pulse.clock).or_insert(*pulse);
            let entry = latest.entry(pulse.clock).or_insert(*pulse);
            if pulse.sample_time >= entry.sample_time {
                *entry = *pulse;
            }
            false
        });
    }
    /// The most recent pulse from `clock` that has reached its display instant.
    pub fn last(&self, clock: u64) -> Option<SamplePulse> {
        self.latest.get(&clock).copied()
    }
    pub fn level(&self, clock: u64, now: Instant, attack: f32, duration: f32) -> f32 {
        self.latest
            .get(&clock)
            .and_then(|pulse| now.checked_duration_since(pulse.at))
            .map_or(0., |age| {
                flash_envelope(age.as_secs_f32(), attack, duration)
            })
    }
}

fn flash_envelope(age: f32, attack: f32, duration: f32) -> f32 {
    if age < 0. || age >= duration {
        0.
    } else if age < attack {
        age / attack
    } else {
        ((duration - age) / (duration - attack)).powi(2)
    }
}

/// Logical clock aliases: pulses from the key's clock are also delivered as the value's.
#[derive(Resource, Default)]
pub struct PulseAliases(pub HashMap<u64, u64>);

/// Moves engine clock ticks onto the bus. Public so tests can drive the bus alone.
pub fn poll_audio(
    mut ticks: MessageReader<ClockTick>,
    aliases: Res<PulseAliases>,
    mut bus: ResMut<PulseBus>,
    mut pulses: MessageWriter<SamplePulse>,
) {
    for tick in ticks.read() {
        if let Some(at) = tick.at {
            let pulse = SamplePulse {
                clock: tick.module.0,
                sample_time: tick.sample_time,
                at,
            };
            bus.queue(pulse);
            pulses.write(pulse);
            if let Some(clock) = aliases.0.get(&pulse.clock) {
                let alias = SamplePulse {
                    clock: *clock,
                    ..pulse
                };
                bus.queue(alias);
                pulses.write(alias);
            }
        }
    }
    bus.advance(Instant::now());
}
