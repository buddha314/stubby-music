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

/// The master clock's tempo: one beat per minute (`lock-clocks-to-master`).
pub const MASTER_BPM: u32 = 1;
/// The fastest tempo a clock may run at, in BPM.
pub const MAX_BPM: u32 = 999;

/// A tempo clock locked to the 1 BPM master: a whole multiple of it, its beat and phase
/// derived from the engine's sample counter rather than accumulated, so it never drifts
/// and every such clock begins a beat on every master beat. Realtime-safe.
pub struct MasterLockedClock {
    /// Samples per master beat (`60 * sample_rate / MASTER_BPM`).
    master_samples: u128,
    multiple: u32,
    /// The multiple and beat index at the last sample stepped.
    last: Option<(u32, u128)>,
}
impl MasterLockedClock {
    pub fn new(sample_rate: u32, bpm: f32) -> Self {
        Self {
            master_samples: 60 * u128::from(sample_rate.max(1)) / u128::from(MASTER_BPM),
            multiple: Self::whole_bpm(bpm),
            last: None,
        }
    }
    /// A tempo as a whole multiple of the master, 1..=999.
    pub fn whole_bpm(bpm: f32) -> u32 {
        if bpm.is_finite() {
            bpm.round().clamp(1., MAX_BPM as f32) as u32
        } else {
            1
        }
    }
    pub fn bpm(&self) -> u32 {
        self.multiple * MASTER_BPM
    }
    /// Retune; the clock moves to the new multiple's place on the grid, without a tick.
    pub fn set_bpm(&mut self, bpm: f32) {
        self.multiple = Self::whole_bpm(bpm);
    }
    /// The beat index and phase (0..1) at engine sample `sample`.
    pub fn at(&self, sample: u64) -> (u128, f32) {
        let position = u128::from(sample) * u128::from(self.multiple);
        let beat = position / self.master_samples;
        let phase = (position % self.master_samples) as f64 / self.master_samples as f64;
        (beat, phase as f32)
    }
    pub fn gate_high(&self, sample: u64) -> bool {
        self.at(sample).1 < 0.5
    }
    /// Step to `sample` (called once per sample, in order). `true` on the first sample of
    /// a beat; never on the sample where the tempo changed.
    pub fn tick_at(&mut self, sample: u64) -> bool {
        let (beat, phase) = self.at(sample);
        let tick = match self.last {
            Some((multiple, last)) if multiple == self.multiple => beat != last,
            Some(_) => false,
            // A clock that starts exactly on a beat ticks there.
            None => phase == 0.,
        };
        self.last = Some((self.multiple, beat));
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

    fn ticks_between(clock: &mut MasterLockedClock, from: u64, to: u64) -> Vec<u64> {
        (from..to).filter(|&s| clock.tick_at(s)).collect()
    }

    #[test]
    fn locked_clocks_share_every_master_downbeat() {
        let rate = 100;
        let minute = 60 * rate as u64;
        // Added at different moments, at different tempos.
        let mut a = MasterLockedClock::new(rate, 90.);
        let mut b = MasterLockedClock::new(rate, 116.);
        let ticks_a = ticks_between(&mut a, 1234, 3 * minute + 1);
        let ticks_b = ticks_between(&mut b, 4321, 3 * minute + 1);
        for downbeat in [minute, 2 * minute, 3 * minute] {
            assert!(ticks_a.contains(&downbeat) && ticks_b.contains(&downbeat));
        }
    }

    #[test]
    fn locked_clock_never_drifts() {
        let rate = 48_000;
        let mut clock = MasterLockedClock::new(rate, 116.);
        let ticks = ticks_between(&mut clock, 0, 10 * 60 * rate as u64);
        assert_eq!(ticks.len(), 1160);
        for (k, tick) in ticks.iter().enumerate() {
            // Beat k starts on the first sample at or after k / 116 minutes.
            assert_eq!(*tick, (k as u64 * 60 * rate as u64).div_ceil(116));
        }
    }

    #[test]
    fn tempo_is_whole_and_retuning_does_not_tick() {
        let mut clock = MasterLockedClock::new(100, 116.4);
        assert_eq!(clock.bpm(), 116);
        assert_eq!(MasterLockedClock::whole_bpm(0.2), 1);
        assert_eq!(MasterLockedClock::whole_bpm(5000.), 999);
        clock.tick_at(10);
        clock.set_bpm(117.);
        assert!(!clock.tick_at(11));
        let next = (12..10_000).find(|&s| clock.tick_at(s)).unwrap();
        // The next beat on the 117 BPM grid: beat k starts at ceil(k * 6000 / 117).
        let beat = clock.at(11).0 as u64 + 1;
        assert_eq!(next, (beat * 6000).div_ceil(117));
    }
}
