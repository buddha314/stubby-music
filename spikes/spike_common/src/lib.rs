//! Shared harness for the audio-engine spike (`add-audio-engine` task group 2).
//!
//! Each candidate library implements [`Backend`]; [`run_suite`] then measures every
//! candidate the same way, offline, with no audio device.

use assert_no_alloc::{AllocDisabler, assert_no_alloc, reset_violation_count, violation_count};
use std::time::{Duration, Instant};

#[global_allocator]
static ALLOC: AllocDisabler = AllocDisabler;

pub const SR: u32 = 48_000;
pub const BLOCK: usize = 256;
/// Output layout, interleaved: channel 0 = audio mix, channel 1 = clock gate.
pub const CHANNELS: usize = 2;

pub type Id = u64;

/// The throwaway boundary both candidates sit behind.
///
/// Methods other than `render_block` are the *game side* and may allocate.
/// `render_block` is the *audio side* and runs under `assert_no_alloc`.
pub trait Backend {
    fn name(&self) -> &'static str;
    /// Add a sine routed to the audio mix (channel 0).
    fn add_sine(&mut self, freq: f32, amp: f32) -> Id;
    /// Add a clock whose gate drives channel 1. Gate goes high on each beat.
    fn add_clock(&mut self, bpm: f64) -> Id;
    fn remove(&mut self, id: Id);
    fn set_freq(&mut self, id: Id, freq: f32);
    /// Publish pending edits to the audio side.
    fn commit(&mut self);
    /// Render one block of `BLOCK` frames, interleaved `CHANNELS`.
    fn render_block(&mut self, out: &mut [f32]);
}

pub struct Report {
    pub name: &'static str,
    pub tone_freq_hz: f64,
    pub tone_peak: f32,
    pub clock_ticks: usize,
    pub clock_first_tick: Option<usize>,
    pub clock_max_spacing_err: usize,
    /// For each edit: max sample-to-sample step in the 50 ms after it, divided by
    /// the largest step the steady signal can produce. > ~1.2 means an audible click.
    pub edit_click_ratios: Vec<(&'static str, f64)>,
    pub alloc_violations: u32,
    pub scale_modules: usize,
    pub scale_edit_frontend: Duration,
    pub scale_render_mean: Duration,
    pub scale_render_max: Duration,
    pub scale_render_after_edit: Duration,
}

struct Recorder<B: Backend> {
    b: B,
    audio: Vec<f32>,
    gate: Vec<f32>,
    buf: Vec<f32>,
    render_times: Vec<Duration>,
}

impl<B: Backend> Recorder<B> {
    fn new(b: B) -> Self {
        Self {
            b,
            audio: Vec::new(),
            gate: Vec::new(),
            buf: vec![0.0; BLOCK * CHANNELS],
            render_times: Vec::new(),
        }
    }

    fn blocks(&mut self, n: usize) {
        for _ in 0..n {
            let t = Instant::now();
            let (b, buf) = (&mut self.b, &mut self.buf);
            assert_no_alloc(|| b.render_block(buf));
            self.render_times.push(t.elapsed());
            for f in self.buf.chunks_exact(CHANNELS) {
                self.audio.push(f[0]);
                self.gate.push(f[1]);
            }
        }
    }
}

fn freq_by_zero_crossings(x: &[f32]) -> f64 {
    let mut crossings = Vec::new();
    for i in 1..x.len() {
        if x[i - 1] <= 0.0 && x[i] > 0.0 {
            let frac = -x[i - 1] as f64 / (x[i] - x[i - 1]) as f64;
            crossings.push((i - 1) as f64 + frac);
        }
    }
    if crossings.len() < 2 {
        return 0.0;
    }
    (crossings.len() - 1) as f64 / ((crossings[crossings.len() - 1] - crossings[0]) / SR as f64)
}

fn rising_edges(gate: &[f32]) -> Vec<usize> {
    let mut prev = 0.0;
    let mut edges = Vec::new();
    for (i, &g) in gate.iter().enumerate() {
        if prev <= 0.5 && g > 0.5 {
            edges.push(i);
        }
        prev = g;
    }
    edges
}

fn max_step(x: &[f32]) -> f64 {
    x.windows(2)
        .map(|w| (w[1] - w[0]).abs() as f64)
        .fold(0.0, f64::max)
}

fn step_bound(sines: &[(f32, f32)]) -> f64 {
    sines
        .iter()
        .map(|&(f, a)| std::f64::consts::TAU * f as f64 * a as f64 / SR as f64)
        .sum()
}

pub fn run_suite<B: Backend>(make: impl Fn() -> B) -> Report {
    reset_violation_count();

    // 1. Tone accuracy.
    let mut r = Recorder::new(make());
    r.b.add_sine(440.0, 1.0);
    r.b.commit();
    r.blocks(SR as usize / BLOCK + 1);
    let tone = &r.audio[..SR as usize];
    let tone_freq_hz = freq_by_zero_crossings(tone);
    let tone_peak = tone.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
    let name = r.b.name();

    // 2. Clock timing: commit after 10 silent blocks, then render 10 s.
    let mut r = Recorder::new(make());
    r.blocks(10);
    r.b.add_clock(120.0);
    r.b.commit();
    r.blocks(10 * SR as usize / BLOCK);
    let edges = rising_edges(&r.gate);
    let clock_max_spacing_err = edges
        .windows(2)
        .map(|w| (w[1] - w[0]).abs_diff(SR as usize / 2))
        .max()
        .unwrap_or(usize::MAX);
    let start = 10 * BLOCK;
    let clock_ticks = edges
        .iter()
        .filter(|&&e| e >= start && e < start + 10 * SR as usize)
        .count();

    // 3. Clicks on live edits.
    let mut r = Recorder::new(make());
    let a = r.b.add_sine(220.0, 0.5);
    r.b.commit();
    r.blocks(50);
    let bb = r.b.add_sine(330.0, 0.5);
    r.b.commit();
    r.blocks(50);
    r.b.set_freq(a, 440.0);
    r.b.commit();
    r.blocks(50);
    r.b.remove(bb);
    r.b.commit();
    r.blocks(50);
    let window = SR as usize / 20;
    let ratio = |block: usize, sines: &[(f32, f32)]| {
        let s = block * BLOCK;
        max_step(&r.audio[s.saturating_sub(1)..s + window]) / step_bound(sines)
    };
    let edit_click_ratios = vec![
        ("steady", ratio(25, &[(220.0, 0.5)])),
        ("add", ratio(50, &[(220.0, 0.5), (330.0, 0.5)])),
        ("set freq", ratio(100, &[(440.0, 0.5), (330.0, 0.5)])),
        ("remove", ratio(150, &[(440.0, 0.5), (330.0, 0.5)])),
    ];

    // 4. Scale: 200 sines + clock.
    let n = 200;
    let mut r = Recorder::new(make());
    r.b.add_clock(120.0);
    let mut ids = Vec::new();
    let mut edit_times = Vec::new();
    for i in 0..n {
        let t = Instant::now();
        ids.push(r.b.add_sine(100.0 + i as f32 * 7.0, 0.004));
        r.b.commit();
        if i >= n - 10 {
            edit_times.push(t.elapsed());
        }
        r.blocks(1);
    }
    r.render_times.clear();
    r.blocks(400);
    let scale_render_mean = r.render_times.iter().sum::<Duration>() / r.render_times.len() as u32;
    let scale_render_max = *r.render_times.iter().max().unwrap();
    r.render_times.clear();
    let mut after_edit = Duration::ZERO;
    for &id in ids.iter().rev().take(10) {
        r.b.remove(id);
        r.b.commit();
        r.blocks(1);
        after_edit = after_edit.max(r.render_times[r.render_times.len() - 1]);
    }

    Report {
        name,
        tone_freq_hz,
        tone_peak,
        clock_ticks,
        clock_first_tick: edges.first().copied(),
        clock_max_spacing_err,
        edit_click_ratios,
        alloc_violations: violation_count(),
        scale_modules: n + 1,
        scale_edit_frontend: edit_times.iter().sum::<Duration>() / edit_times.len() as u32,
        scale_render_mean,
        scale_render_max,
        scale_render_after_edit: after_edit,
    }
}

impl Report {
    pub fn print(&self) {
        let budget = Duration::from_secs_f64(BLOCK as f64 / SR as f64);
        println!("## {}\n", self.name);
        println!("| Check | Result |\n| --- | --- |");
        println!(
            "| Tone (target 440 Hz, peak 1.0) | {:.3} Hz, peak {:.4} |",
            self.tone_freq_hz, self.tone_peak
        );
        println!(
            "| Clock 120 BPM, 10 s (target 20 ticks, 24000 apart) | {} ticks, max spacing error {} samples |",
            self.clock_ticks, self.clock_max_spacing_err
        );
        println!(
            "| Edit latency (clock committed before sample 2560) | first tick at sample {:?} |",
            self.clock_first_tick
        );
        for (what, r) in &self.edit_click_ratios {
            println!("| Click ratio: {what} (≤1.2 is clean) | {r:.2} |");
        }
        println!(
            "| Audio-thread allocations, whole suite | {} |",
            self.alloc_violations
        );
        println!(
            "| Game-side add+commit at ~{} modules | {:?} |",
            self.scale_modules, self.scale_edit_frontend
        );
        println!(
            "| Render block at {} modules: mean / max (budget {:?}) | {:?} / {:?} |",
            self.scale_modules, budget, self.scale_render_mean, self.scale_render_max
        );
        println!(
            "| Worst render block right after a remove | {:?} |",
            self.scale_render_after_edit
        );
        println!();
    }
}
