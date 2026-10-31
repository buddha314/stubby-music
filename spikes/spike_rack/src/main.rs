//! Spike for `add-rack-and-spatial-audio`:
//!
//! A. A VCV-style rack: every module once per sample, cables read the previous
//!    sample's outputs (1-sample delay), so feedback patches are legal. Measures cost
//!    per 256-frame block as the module count grows.
//! B. Firewheel spatialization: N moving emitters through `firewheel-ircam-hrtf`
//!    versus `SpatialBasicNode`. Measures cost per block as N grows.
//!
//! cargo run --release -p spike_rack

use assert_no_alloc::{assert_no_alloc, reset_violation_count, violation_count};
use audioadapter_buffers::direct::InterleavedSlice;
use firewheel::{
    ActivateInfo, FirewheelConfig, FirewheelContext,
    backend::BackendProcessInfo,
    channel_config::{ChannelConfig, ChannelCount, NonZeroChannelCount},
    diff::{Diff, Patch, PathBuilder},
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig, NodeError, NodeID,
        ProcBuffers, ProcExtra, ProcInfo, ProcessStatus, StreamStatus,
    },
    nodes::spatial_basic::SpatialBasicNode,
    processor::FirewheelProcessor,
};
mod hrtf_node;
use hrtf_node::{FftSize, HrtfConfig, HrtfNode};
use fundsp::prelude32::{AudioUnit, sine, square};
use spike_common::{BLOCK, SR};
use std::time::{Duration, Instant};

// ======================================================================== A. rack

const NONE: u32 = u32::MAX;
const MAX_INPUTS: usize = 4;

trait Module {
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    /// One sample. `inp` holds this module's inputs as of the previous sample.
    fn process(&mut self, inp: &[f32], out: &mut [f32]);
}

/// Oscillator. Inputs: V/oct, FM (linear, scaled by `fm_index`).
struct Vco {
    unit: Box<dyn AudioUnit>,
    base_hz: f32,
    fm_index: f32,
}

impl Module for Vco {
    fn num_inputs(&self) -> usize {
        2
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn process(&mut self, inp: &[f32], out: &mut [f32]) {
        let hz = self.base_hz * inp[0].exp2() * (1.0 + self.fm_index * inp[1]);
        let mut o = [0.0];
        self.unit.tick(&[hz.max(0.0)], &mut o);
        out[0] = if o[0].is_finite() { o[0] * 5.0 } else { 0.0 };
    }
}

struct Clock {
    spb: f64,
    pos: f64,
}

impl Module for Clock {
    fn num_inputs(&self) -> usize {
        0
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn process(&mut self, _: &[f32], out: &mut [f32]) {
        out[0] = if self.pos < self.spb * 0.5 { 10.0 } else { 0.0 };
        self.pos += 1.0;
        if self.pos >= self.spb {
            self.pos -= self.spb;
        }
    }
}

/// Attack/decay envelope, retriggered on a rising gate (Schmitt 0.1 / 1.0 V).
struct Env {
    level: f32,
    attack: f32,
    decay: f32,
    rising: bool,
    gate_high: bool,
}

impl Module for Env {
    fn num_inputs(&self) -> usize {
        1
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn process(&mut self, inp: &[f32], out: &mut [f32]) {
        let g = inp[0];
        if !self.gate_high && g >= 1.0 {
            self.gate_high = true;
            self.rising = true;
        } else if self.gate_high && g <= 0.1 {
            self.gate_high = false;
        }
        if self.rising {
            self.level += self.attack;
            if self.level >= 1.0 {
                self.level = 1.0;
                self.rising = false;
            }
        } else {
            self.level = (self.level - self.decay).max(0.0);
        }
        out[0] = self.level * 10.0;
    }
}

/// out = in0 * in1 / 10 V.
struct Vca;

impl Module for Vca {
    fn num_inputs(&self) -> usize {
        2
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn process(&mut self, inp: &[f32], out: &mut [f32]) {
        out[0] = inp[0] * inp[1] * 0.1;
    }
}

struct Mix4;

impl Module for Mix4 {
    fn num_inputs(&self) -> usize {
        4
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn process(&mut self, inp: &[f32], out: &mut [f32]) {
        out[0] = (inp[0] + inp[1] + inp[2] + inp[3]) * 0.25;
    }
}

struct Rack {
    modules: Vec<Box<dyn Module>>,
    in_base: Vec<usize>,
    out_base: Vec<usize>,
    /// For each input port, the output slot it reads, or NONE.
    in_src: Vec<u32>,
    /// Output slots, double-buffered: one is read (previous sample), one written.
    slots: [Vec<f32>; 2],
    parity: usize,
    /// Output slots tapped as emission stems.
    stems: Vec<usize>,
}

impl Rack {
    fn new() -> Self {
        Self {
            modules: Vec::new(),
            in_base: Vec::new(),
            out_base: Vec::new(),
            in_src: Vec::new(),
            slots: [Vec::new(), Vec::new()],
            parity: 0,
            stems: Vec::new(),
        }
    }

    fn add(&mut self, m: Box<dyn Module>) -> usize {
        self.in_base.push(self.in_src.len());
        self.out_base.push(self.slots[0].len());
        self.in_src.extend(std::iter::repeat_n(NONE, m.num_inputs()));
        for s in &mut self.slots {
            s.extend(std::iter::repeat_n(0.0, m.num_outputs()));
        }
        self.modules.push(m);
        self.modules.len() - 1
    }

    /// Cable from (module, output) to (module, input). Any topology, cycles included.
    fn cable(&mut self, from: (usize, usize), to: (usize, usize)) {
        self.in_src[self.in_base[to.0] + to.1] = (self.out_base[from.0] + from.1) as u32;
    }

    /// Render `frames` samples. `stems_out` is stem-major: stem s, frame i at `s * frames + i`.
    fn render(&mut self, frames: usize, stems_out: &mut [f32]) {
        let mut inp = [0.0f32; MAX_INPUTS];
        for i in 0..frames {
            let (a, b) = self.slots.split_at_mut(1);
            let (prev, cur) = if self.parity == 0 { (&b[0], &mut a[0]) } else { (&a[0], &mut b[0]) };
            for (m, module) in self.modules.iter_mut().enumerate() {
                let n_in = module.num_inputs();
                let base = self.in_base[m];
                for (k, v) in inp.iter_mut().enumerate().take(n_in) {
                    let src = self.in_src[base + k];
                    *v = if src == NONE { 0.0 } else { prev[src as usize] };
                }
                let ob = self.out_base[m];
                module.process(&inp[..n_in], &mut cur[ob..ob + module.num_outputs()]);
            }
            for (s, &slot) in self.stems.iter().enumerate() {
                stems_out[s * frames + i] = cur[slot];
            }
            self.parity ^= 1;
        }
    }
}

/// A plausible patch of roughly `n` modules: voices of
/// clock → env, lfo → vco.voct, vco → vca ← env, vca → vco.fm (feedback),
/// with every 4 voices summed by a mixer. Each VCA output is an emission stem.
fn build_patch(n: usize) -> Rack {
    let mut rack = Rack::new();
    let mut vcas = Vec::new();
    let mut voice = 0;
    while rack.modules.len() + 6 <= n {
        let base = 110.0 * (1.0 + (voice % 12) as f32 / 12.0);
        let clock = rack.add(Box::new(Clock { spb: SR as f64 * 60.0 / (90.0 + voice as f64), pos: 0.0 }));
        let env = rack.add(Box::new(Env { level: 0.0, attack: 0.01, decay: 0.0002, rising: false, gate_high: false }));
        let lfo = rack.add(Box::new(Vco { unit: Box::new(sine()), base_hz: 0.3, fm_index: 0.0 }));
        let osc: Box<dyn AudioUnit> = if voice % 2 == 0 { Box::new(square()) } else { Box::new(sine()) };
        let vco = rack.add(Box::new(Vco { unit: osc, base_hz: base, fm_index: 0.03 }));
        let vca = rack.add(Box::new(Vca));
        rack.cable((clock, 0), (env, 0));
        rack.cable((lfo, 0), (vco, 0));
        rack.cable((vco, 0), (vca, 0));
        rack.cable((env, 0), (vca, 1));
        rack.cable((vca, 0), (vco, 1)); // feedback loop: legal thanks to the 1-sample cable
        vcas.push(vca);
        voice += 1;
    }
    for chunk in vcas.chunks(4) {
        if rack.modules.len() >= n {
            break;
        }
        let mix = rack.add(Box::new(Mix4));
        for (k, &v) in chunk.iter().enumerate() {
            rack.cable((v, 0), (mix, k));
        }
    }
    rack.stems = vcas.iter().take(32).map(|&v| rack.out_base[v]).collect();
    rack
}

struct Timing {
    mean: Duration,
    p99: Duration,
    max: Duration,
}

fn summarize(mut t: Vec<Duration>) -> Timing {
    t.sort();
    let n = t.len();
    Timing { mean: t.iter().sum::<Duration>() / n as u32, p99: t[n * 99 / 100], max: t[n - 1] }
}

fn pct(d: Duration) -> f64 {
    100.0 * d.as_secs_f64() / (BLOCK as f64 / SR as f64)
}

fn rack_part() {
    println!("## A. Rack: per-sample, 1-sample cables, feedback in every voice\n");
    println!("| Modules | Cables | Mean / block | p99 | Max | Mean % of budget | ns per module-sample |");
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for n in [50, 100, 200, 400] {
        let mut rack = build_patch(n);
        let cables = rack.in_src.iter().filter(|&&s| s != NONE).count();
        let mut stems = vec![0.0f32; rack.stems.len() * BLOCK];
        for _ in 0..200 {
            rack.render(BLOCK, &mut stems); // warm up
        }
        reset_violation_count();
        let mut times = Vec::with_capacity(1000);
        for _ in 0..1000 {
            let start = Instant::now();
            assert_no_alloc(|| rack.render(BLOCK, &mut stems));
            times.push(start.elapsed());
        }
        assert_eq!(violation_count(), 0, "rack allocated on the audio path");
        let t = summarize(times);
        let per = t.mean.as_secs_f64() * 1e9 / (rack.modules.len() * BLOCK) as f64;
        println!(
            "| {} | {} | {:?} | {:?} | {:?} | {:.1}% | {:.1} |",
            rack.modules.len(),
            cables,
            t.mean,
            t.p99,
            t.max,
            pct(t.mean),
            per
        );
    }

    // Feedback sanity: self-modulating voices stay finite and bounded, and the rack
    // is deterministic.
    let render = || {
        let mut rack = build_patch(200);
        let mut stems = vec![0.0f32; rack.stems.len() * SR as usize];
        rack.render(SR as usize, &mut stems);
        stems
    };
    let (a, b) = (render(), render());
    let peak = a.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    println!(
        "\nFeedback check (200 modules, 1 s): all finite = {}, peak = {peak:.2} V, deterministic = {}\n",
        a.iter().all(|s| s.is_finite()),
        a == b
    );
}

// ===================================================================== B. spatial

#[derive(Diff, Patch, Debug, Clone, Copy, PartialEq)]
struct ToneNode {
    freq: f32,
}

impl AudioNode for ToneNode {
    type Configuration = EmptyConfig;
    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("tone")
            .channel_config(ChannelConfig { num_inputs: ChannelCount::ZERO, num_outputs: ChannelCount::MONO }))
    }
    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(ToneProc { phase: 0.0, inc: self.freq / cx.stream_info.sample_rate.get() as f32 })
    }
}

struct ToneProc {
    phase: f32,
    inc: f32,
}

impl AudioNodeProcessor for ToneProc {
    fn process(&mut self, _: &ProcInfo, buffers: ProcBuffers, _: &mut ProcExtra) -> ProcessStatus {
        // Square-ish tone: broadband, like what players will locate.
        for s in buffers.outputs[0].iter_mut() {
            *s = if self.phase < 0.5 { 0.2 } else { -0.2 };
            self.phase = (self.phase + self.inc).fract();
        }
        ProcessStatus::OutputsModified
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Spatializer {
    Hrtf,
    Basic,
}

fn render_block(processor: &mut FirewheelProcessor, buf: &mut [f32], frames_done: u64) {
    let empty: [f32; 0] = [];
    let (Ok(input), Ok(mut output)) =
        (InterleavedSlice::new(&empty[..], 0, BLOCK), InterleavedSlice::new_mut(buf, 2, BLOCK))
    else {
        return;
    };
    processor.process(
        &input,
        &mut output,
        BackendProcessInfo {
            frames: BLOCK,
            process_timestamp: None,
            duration_since_stream_start: Duration::from_secs_f64(frames_done as f64 / SR as f64),
            input_stream_status: StreamStatus::empty(),
            output_stream_status: StreamStatus::empty(),
            dropped_frames: 0,
            process_to_playback_delay: None,
        },
    );
}

fn emitter_pos(i: usize, t: f32) -> (f32, f32, f32) {
    let angle = i as f32 * 2.399 + t * 0.5;
    let dist = 5.0 + (i % 7) as f32 * 4.0;
    (angle.cos() * dist, 0.5, angle.sin() * dist)
}

fn spatial_run(kind: Spatializer, n: usize, fft: FftSize) -> (Timing, u32) {
    let mut cx = FirewheelContext::new(FirewheelConfig {
        num_graph_inputs: ChannelCount::ZERO,
        num_graph_outputs: ChannelCount::STEREO,
        initial_node_capacity: 256,
        initial_edge_capacity: 512,
        ..Default::default()
    });
    let mut processor = cx
        .activate(ActivateInfo {
            sample_rate: SR.try_into().unwrap(),
            max_block_frames: (BLOCK as u32).try_into().unwrap(),
            num_stream_in_channels: 0,
            num_stream_out_channels: 2,
            input_to_output_latency_seconds: 0.0,
        })
        .unwrap();
    let out = cx.graph_out_node_id();
    let mut spat: Vec<(NodeID, HrtfNode, SpatialBasicNode)> = Vec::new();
    for i in 0..n {
        let tone = cx.add_node(ToneNode { freq: 150.0 + i as f32 * 13.0 }, None).unwrap();
        let (x, y, z) = emitter_pos(i, 0.0);
        let hrtf = HrtfNode { offset: glam::Vec3::new(x, y, z), ..Default::default() };
        let basic = SpatialBasicNode { offset: firewheel::vector::Vec3::new(x, y, z), ..Default::default() };
        let (node, ports): (NodeID, &[(u32, u32)]) = match kind {
            Spatializer::Hrtf => (
                cx.add_node(
                    hrtf.clone(),
                    Some(HrtfConfig { input_channels: NonZeroChannelCount::MONO, fft_size: fft.clone(), ..Default::default() }),
                )
                .unwrap(),
                &[(0, 0)],
            ),
            Spatializer::Basic => (cx.add_node(basic, None).unwrap(), &[(0, 0), (0, 1)]),
        };
        cx.connect(tone, node, ports, false).unwrap();
        cx.connect(node, out, &[(0, 0), (1, 1)], false).unwrap();
        spat.push((node, hrtf, basic));
    }
    cx.update().unwrap();

    let mut buf = vec![0.0f32; BLOCK * 2];
    let mut frames = 0u64;
    for _ in 0..100 {
        render_block(&mut processor, &mut buf, frames);
        frames += BLOCK as u64;
    }
    reset_violation_count();
    let mut times = Vec::new();
    // About 3 audio blocks per 60 fps frame: move every emitter once per "frame".
    for block in 0..1500 {
        if block % 3 == 0 {
            let t = block as f32 * BLOCK as f32 / SR as f32;
            for (i, (node, hrtf, basic)) in spat.iter_mut().enumerate() {
                let (x, y, z) = emitter_pos(i, t);
                match kind {
                    Spatializer::Hrtf => {
                        let new = HrtfNode { offset: glam::Vec3::new(x, y, z), ..hrtf.clone() };
                        new.diff(hrtf, PathBuilder::default(), &mut cx.event_queue(*node));
                        *hrtf = new;
                    }
                    Spatializer::Basic => {
                        let new = SpatialBasicNode { offset: firewheel::vector::Vec3::new(x, y, z), ..*basic };
                        new.diff(basic, PathBuilder::default(), &mut cx.event_queue(*node));
                        *basic = new;
                    }
                }
            }
            cx.update().unwrap();
        }
        let start = Instant::now();
        assert_no_alloc(|| render_block(&mut processor, &mut buf, frames));
        times.push(start.elapsed());
        frames += BLOCK as u64;
    }
    (summarize(times), violation_count())
}

fn spatial_part() {
    println!("## B. Spatialization: N moving emitters, positions updated every 3 blocks\n");
    println!("| Emitters | Spatializer | Mean / block | p99 | Max | Mean % of budget | Audio-thread allocs |");
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for n in [1, 8, 16, 32, 64] {
        for (kind, name) in [(Spatializer::Hrtf, "HRTF (IRCAM)"), (Spatializer::Basic, "basic pan")] {
            let (t, allocs) = spatial_run(kind, n, FftSize::default());
            println!("| {n} | {name} | {:?} | {:?} | {:?} | {:.1}% | {allocs} |", t.mean, t.p99, t.max, pct(t.mean));
        }
    }
    println!();
}

fn fft_part() {
    println!("## C. HRTF FFT size: default 4x128 (512) vs 2x128 (256, matches the audio block)\n");
    println!("| Emitters | FFT | Mean / block | p99 | Max | Mean % of budget |");
    println!("| --- | --- | --- | --- | --- | --- |");
    for n in [16, 32] {
        for (fft, name) in [
            (FftSize { slice_count: 4, slice_len: 128 }, "4x128"),
            (FftSize { slice_count: 2, slice_len: 128 }, "2x128"),
        ] {
            let (t, _) = spatial_run(Spatializer::Hrtf, n, fft);
            println!("| {n} | {name} | {:?} | {:?} | {:?} | {:.1}% |", t.mean, t.p99, t.max, pct(t.mean));
        }
    }
    println!();
}

fn main() {
    println!("Budget per {BLOCK}-frame block at {SR} Hz: {:.3} ms\n", BLOCK as f64 / SR as f64 * 1e3);
    match std::env::args().nth(1).as_deref() {
        Some("rack") => rack_part(),
        Some("spatial") => spatial_part(),
        Some("fft") => fft_part(),
        _ => {
            rack_part();
            spatial_part();
        }
    }
}
