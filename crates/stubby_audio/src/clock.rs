//! Clock sources share a sample-stepping interface, independent of Bevy and devices.

/// The clock abstraction. The audio processor calls this once per sample; `true`
/// marks a pulse at that exact sample. Implementations must be realtime-safe.
/// Each instance owns its phase/random state; sharing a pulse is an explicit
/// decision made by followers selecting the same module ID.
pub trait PulseClock {
    fn advance(&mut self) -> bool;
}

/// A drift-free periodic clock. The first sample starts a beat.
pub struct PeriodicClock {
    sample_rate: f64,
    samples_per_beat: f64,
    position: f64,
    tick_pending: bool,
}
impl PeriodicClock {
    pub fn new(sample_rate: u32, bpm: f32) -> Self {
        let sample_rate = sample_rate.max(1) as f64;
        Self {
            sample_rate,
            samples_per_beat: sample_rate * 60. / bpm.max(1.) as f64,
            position: 0.,
            tick_pending: true,
        }
    }
    pub fn gate_high(&self) -> bool {
        self.position < self.samples_per_beat * 0.5
    }
    pub fn phase(&self) -> f32 {
        (self.position / self.samples_per_beat) as f32
    }
    pub fn set_bpm(&mut self, bpm: f32) {
        let phase = self.position / self.samples_per_beat;
        self.samples_per_beat = self.sample_rate * 60. / bpm.max(1.) as f64;
        self.position = phase * self.samples_per_beat;
    }
}
impl PulseClock for PeriodicClock {
    fn advance(&mut self) -> bool {
        let tick = self.tick_pending;
        self.tick_pending = false;
        self.position += 1.;
        if self.position >= self.samples_per_beat {
            self.position -= self.samples_per_beat;
            self.tick_pending = true;
        }
        tick
    }
}

/// Seeded irregular clock with inclusive sample interval bounds. Its first tick
/// follows a random resting interval. Zero/descending bounds clamp to at least
/// one sample; the engine validates user-facing seconds before constructing it.
pub struct RandomClock {
    rng: u64,
    min_frames: u64,
    max_frames: u64,
    until_tick: u64,
}
impl RandomClock {
    pub fn new(seed: u64, min_interval_samples: u64, max_interval_samples: u64) -> Self {
        let min_frames = min_interval_samples.max(1);
        let mut clock = Self {
            rng: seed,
            min_frames,
            max_frames: max_interval_samples.max(min_frames),
            until_tick: 0,
        };
        clock.until_tick = clock.interval();
        clock
    }
    fn interval(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9e3779b97f4a7c15);
        let mut n = self.rng;
        n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
        n ^= n >> 31;
        self.min_frames + n % (self.max_frames - self.min_frames + 1)
    }
}
impl PulseClock for RandomClock {
    fn advance(&mut self) -> bool {
        let tick = self.until_tick == 0;
        if tick {
            self.until_tick = self.interval();
        }
        self.until_tick -= 1;
        tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_clock_types_use_the_same_interface_without_sharing_state() {
        fn ticks(clock: &mut dyn PulseClock, count: usize) -> Vec<usize> {
            (0..count).filter(|_| clock.advance()).collect()
        }
        let mut regular = PeriodicClock::new(100, 60.);
        let mut a = RandomClock::new(42, 50, 150);
        let mut b = RandomClock::new(43, 50, 150);
        assert_eq!(ticks(&mut regular, 301), vec![0, 100, 200, 300]);
        let first = ticks(&mut a, 1000);
        let second = ticks(&mut b, 1000);
        assert_ne!(first, second);
        let mut again = RandomClock::new(42, 50, 150);
        assert_eq!(first, ticks(&mut again, 1000));
    }
}
