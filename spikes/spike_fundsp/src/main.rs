//! fundsp `Net` behind the spike boundary.
//!
//! Game side: `Net` frontend; edits are published with `commit()`.
//! Audio side: `NetBackend`, which swaps in committed versions and sends old ones back.

use fundsp::prelude32::*;
use spike_common::{BLOCK, Backend, CHANNELS, Id, SR};
use std::collections::HashMap;

const MIX_INPUTS: usize = 256;

/// Sums 256 inputs. Unconnected inputs read zero.
#[derive(Clone)]
struct Mixer;

impl AudioNode for Mixer {
    const ID: u64 = 0x5717_0001;
    type Inputs = numeric_array::typenum::U256;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, numeric_array::typenum::U256>) -> Frame<f32, U1> {
        [input.iter().sum::<f32>()].into()
    }

    /// Block version: SIMD-add each input channel into the output.
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let n = simd_items(size);
        let out = &mut output.channel_mut(0)[..n];
        out.fill(F32x::ZERO);
        for ch in 0..MIX_INPUTS {
            for (o, i) in out.iter_mut().zip(&input.channel(ch)[..n]) {
                *o += *i;
            }
        }
    }
}

/// Gate high for the first half of each beat. Counts samples, so it never drifts.
#[derive(Clone)]
struct Clock {
    samples_per_beat: f64,
    bpm: f64,
    pos: f64,
}

impl AudioNode for Clock {
    const ID: u64 = 0x5717_0002;
    type Inputs = U0;
    type Outputs = U1;

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.samples_per_beat = sample_rate * 60.0 / self.bpm;
    }

    fn reset(&mut self) {
        self.pos = 0.0;
    }

    fn tick(&mut self, _input: &Frame<f32, U0>) -> Frame<f32, U1> {
        let gate = if self.pos < self.samples_per_beat * 0.5 {
            1.0
        } else {
            0.0
        };
        self.pos += 1.0;
        if self.pos >= self.samples_per_beat {
            self.pos -= self.samples_per_beat;
        }
        [gate].into()
    }
}

enum Module {
    Sine {
        node: NodeId,
        port: usize,
        freq: Shared,
    },
    Clock {
        node: NodeId,
    },
}

/// The audio-thread half: owns the `NetBackend` and a preallocated scratch buffer.
pub struct Audio {
    backend: NetBackend,
    scratch: BufferVec,
}

impl Audio {
    /// Render any number of interleaved `[audio, gate]` frames, in fundsp-sized chunks.
    pub fn render(&mut self, out: &mut [f32]) {
        for frames in out.chunks_mut(MAX_BUFFER_SIZE * CHANNELS) {
            let n = frames.len() / CHANNELS;
            self.backend
                .process(n, &BufferRef::empty(), &mut self.scratch.buffer_mut());
            for i in 0..n {
                for ch in 0..CHANNELS {
                    frames[i * CHANNELS + ch] = self.scratch.at_f32(ch, i);
                }
            }
        }
    }
}

pub struct FundspBackend {
    net: Net,
    audio: Option<Audio>,
    mixer: NodeId,
    free_ports: Vec<usize>,
    modules: HashMap<Id, Module>,
    next_id: Id,
    /// Declick edits with fades (fundsp's `fade_in`/`crossfade`) instead of raw push/remove.
    declick: bool,
}

impl FundspBackend {
    pub fn new(declick: bool) -> Self {
        let mut net = Net::new(0, CHANNELS);
        net.set_sample_rate(SR as f64);
        let mixer = net.push(Box::new(An(Mixer)));
        net.connect_output(mixer, 0, 0);
        let backend = net.backend();
        Self {
            net,
            audio: Some(Audio {
                backend,
                scratch: BufferVec::new(CHANNELS),
            }),
            mixer,
            free_ports: (0..MIX_INPUTS).rev().collect(),
            modules: HashMap::new(),
            next_id: 0,
            declick,
        }
    }

    fn id(&mut self) -> Id {
        self.next_id += 1;
        self.next_id
    }
}

const FADE: f32 = 0.005;

impl Backend for FundspBackend {
    fn name(&self) -> &'static str {
        if self.declick {
            "fundsp Net (with fades)"
        } else {
            "fundsp Net (raw)"
        }
    }

    fn add_sine(&mut self, freq: f32, amp: f32) -> Id {
        let f = shared(freq);
        let unit = Box::new((var(&f) >> sine()) * amp);
        let node = if self.declick {
            self.net.fade_in(Fade::Smooth, FADE, unit)
        } else {
            self.net.push(unit)
        };
        let port = self.free_ports.pop().expect("mixer full");
        self.net.connect(node, 0, self.mixer, port);
        let id = self.id();
        self.modules.insert(
            id,
            Module::Sine {
                node,
                port,
                freq: f,
            },
        );
        id
    }

    fn add_clock(&mut self, bpm: f64) -> Id {
        let node = self.net.push(Box::new(An(Clock {
            samples_per_beat: 0.0,
            bpm,
            pos: 0.0,
        })));
        self.net.connect_output(node, 0, 1);
        let id = self.id();
        self.modules.insert(id, Module::Clock { node });
        id
    }

    fn remove(&mut self, id: Id) {
        match self.modules.remove(&id) {
            Some(Module::Sine { node, port, .. }) => {
                if self.declick {
                    // Fade to silence; a real engine would remove the node once the fade ends.
                    self.net
                        .crossfade(node, Fade::Smooth, FADE, Box::new(zero()));
                } else {
                    self.net.remove(node);
                }
                self.free_ports.push(port);
            }
            Some(Module::Clock { node }) => {
                self.net.remove(node);
            }
            None => {}
        }
    }

    fn set_freq(&mut self, id: Id, freq: f32) {
        if let Some(Module::Sine { freq: f, .. }) = self.modules.get(&id) {
            f.set_value(freq);
        }
    }

    fn commit(&mut self) {
        self.net.commit();
    }

    fn render_block(&mut self, out: &mut [f32]) {
        debug_assert_eq!(out.len(), BLOCK * CHANNELS);
        if let Some(audio) = &mut self.audio {
            audio.render(out);
        }
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("live") => live(),
        _ => {
            spike_common::run_suite(|| FundspBackend::new(false)).print();
            spike_common::run_suite(|| FundspBackend::new(true)).print();
        }
    }
}

/// Play through the default device so a human can listen for clicks.
fn live() {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::{thread::sleep, time::Duration};

    let mut b = FundspBackend::new(true);
    let mut audio = b.audio.take().expect("audio half");
    let device = cpal::default_host()
        .default_output_device()
        .expect("no output device");
    let config = cpal::StreamConfig {
        channels: CHANNELS as u16,
        sample_rate: SR,
        buffer_size: cpal::BufferSize::Fixed(BLOCK as u32),
    };
    let stream = device
        .build_output_stream(
            config,
            move |out: &mut [f32], _| {
                assert_no_alloc::assert_no_alloc(|| audio.render(out));
                // Listen to the mix in both ears; the gate channel is for measurement only.
                for f in out.chunks_exact_mut(CHANNELS) {
                    f[1] = f[0];
                }
            },
            |e| eprintln!("stream error: {e}"),
            None,
        )
        .expect("build stream");
    stream.play().expect("play");

    let step = |msg: &str| println!("{msg}");
    step("0s  sine 220 Hz fades in");
    let a = b.add_sine(220.0, 0.3);
    b.commit();
    sleep(Duration::from_secs(2));
    step("2s  add sine 330 Hz");
    let c = b.add_sine(330.0, 0.3);
    b.commit();
    sleep(Duration::from_secs(2));
    step("4s  first sine jumps to 440 Hz");
    b.set_freq(a, 440.0);
    sleep(Duration::from_secs(2));
    step("6s  remove 330 Hz");
    b.remove(c);
    b.commit();
    sleep(Duration::from_secs(2));
    step("8s  done");
}
