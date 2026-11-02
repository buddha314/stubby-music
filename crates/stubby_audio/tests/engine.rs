//! Offline tests for the `audio-engine` and `synth-modules` specs.
//!
//! Every render runs under `assert_no_alloc`, so each test also checks that the
//! audio thread's code path never allocates.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use assert_no_alloc::{AllocDisabler, assert_no_alloc, violation_count};
use stubby_audio::{
    Command, Engine, EngineConfig, EngineError, Event, ModuleId, ModuleKind, OfflineRenderer,
    PortRef,
};

#[global_allocator]
static ALLOC: AllocDisabler = AllocDisabler;

const SR: u32 = 48_000;
const BLOCK: usize = 256;

struct Rig {
    engine: Engine,
    renderer: OfflineRenderer,
    left: Vec<f32>,
    right: Vec<f32>,
    buf: Vec<f32>,
}

impl Rig {
    fn new() -> Self {
        let (engine, renderer) = Engine::offline(EngineConfig {
            sample_rate: SR,
            max_block_frames: BLOCK as u32,
        })
        .unwrap();
        Self {
            engine,
            renderer,
            left: Vec::new(),
            right: Vec::new(),
            buf: vec![0.0; BLOCK * 2],
        }
    }

    fn send(&mut self, command: Command) {
        self.engine.send(command).unwrap();
    }

    fn flush(&mut self) {
        self.engine.flush().unwrap();
    }

    fn add(&mut self, id: u64, kind: ModuleKind) -> ModuleId {
        let id = ModuleId(id);
        self.send(Command::AddModule { id, kind });
        id
    }

    fn connect(&mut self, from: PortRef, to: PortRef) {
        self.send(Command::Connect { from, to });
    }

    /// Render whole blocks covering at least `frames`, as the audio callback would.
    fn render(&mut self, frames: usize) {
        for _ in 0..frames.div_ceil(BLOCK) {
            let before = violation_count();
            let (renderer, buf) = (&mut self.renderer, &mut self.buf);
            assert_no_alloc(|| renderer.render(buf));
            assert_eq!(violation_count(), before, "audio thread allocated");
            for frame in self.buf.as_chunks::<2>().0 {
                self.left.push(frame[0]);
                self.right.push(frame[1]);
            }
        }
    }

    fn events(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        self.engine.drain_events(&mut out);
        out
    }
}

fn out(id: ModuleId) -> PortRef {
    PortRef::new(id, "out")
}

fn freq_by_zero_crossings(x: &[f32]) -> f64 {
    let mut crossings = Vec::new();
    for i in 1..x.len() {
        if x[i - 1] <= 0.0 && x[i] > 0.0 {
            crossings.push((i - 1) as f64 + (-x[i - 1] / (x[i] - x[i - 1])) as f64);
        }
    }
    assert!(crossings.len() > 2, "no oscillation");
    (crossings.len() - 1) as f64 / ((crossings[crossings.len() - 1] - crossings[0]) / SR as f64)
}

fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0, |m, &s| m.max(s.abs()))
}

/// Largest sample-to-sample step in `x`, relative to the largest step that sines
/// of the given frequencies (each at amplitude 1) can produce. Above ~1.2 is a click.
fn click_ratio(x: &[f32], freqs: &[f64]) -> f64 {
    let max_step = x
        .windows(2)
        .map(|w| (w[1] - w[0]).abs() as f64)
        .fold(0.0, f64::max);
    max_step
        / freqs
            .iter()
            .map(|f| std::f64::consts::TAU * f / SR as f64)
            .sum::<f64>()
}

// ------------------------------------------------------------- audio-engine

#[test]
fn fresh_engine_is_digital_silence() {
    let mut rig = Rig::new();
    rig.flush();
    rig.render(SR as usize);
    assert!(rig.left.iter().chain(&rig.right).all(|&s| s == 0.0));
}

#[test]
fn command_applies_at_next_block() {
    let mut rig = Rig::new();
    rig.render(BLOCK * 4);
    let clock = rig.add(1, ModuleKind::Clock { bpm: 120.0 });
    rig.connect(PortRef::new(clock, "gate"), PortRef::output());
    rig.flush();
    rig.render(BLOCK);
    assert_eq!(
        rig.events(),
        vec![Event::ClockTick {
            module: clock,
            sample_time: (BLOCK * 4) as u64
        }]
    );
    assert_eq!(rig.left[BLOCK * 4], 1.0);
}

#[test]
fn adding_a_module_does_not_interrupt_a_playing_one() {
    let mut rig = Rig::new();
    let a = rig.add(1, ModuleKind::SineVco { freq_hz: 220.0 });
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 2);
    let edit = rig.left.len();

    let clock = rig.add(2, ModuleKind::Clock { bpm: 120.0 });
    let b = rig.add(3, ModuleKind::SineVco { freq_hz: 330.0 });
    rig.connect(out(b), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 2);

    let window = &rig.left[edit - 1..edit + SR as usize / 20];
    assert!(click_ratio(window, &[220.0, 330.0]) <= 1.2, "click on add");
    assert!(rig.events().contains(&Event::ClockTick {
        module: clock,
        sample_time: edit as u64
    }));
}

#[test]
fn removing_a_module_is_declicked() {
    let mut rig = Rig::new();
    let a = rig.add(1, ModuleKind::SineVco { freq_hz: 220.0 });
    let b = rig.add(2, ModuleKind::SineVco { freq_hz: 330.0 });
    rig.connect(out(a), PortRef::output());
    rig.connect(out(b), PortRef::output());
    rig.flush();
    // Remove mid-cycle, where an abrupt cut would step the most.
    rig.render(SR as usize / 2 + 37);
    let edit = rig.left.len();
    rig.send(Command::RemoveModule { id: b });
    rig.flush();
    rig.render(SR as usize / 2);

    let window = &rig.left[edit - 1..edit + SR as usize / 20];
    assert!(
        click_ratio(window, &[220.0, 330.0]) <= 1.2,
        "click on remove"
    );
    // After the fade only the 220 Hz sine remains.
    let tail = &rig.left[rig.left.len() - SR as usize / 4..];
    assert!((freq_by_zero_crossings(tail) - 220.0).abs() < 0.5);

    // Once the fade is over, the next flush deletes the node.
    let nodes = rig.engine.graph_node_count();
    rig.flush();
    assert_eq!(rig.engine.graph_node_count(), nodes - 1);
}

#[test]
fn a_new_sound_fades_in() {
    let mut rig = Rig::new();
    rig.render(BLOCK);
    let edit = rig.left.len();
    // At 5 kHz an unfaded sine reaches full scale within 3 samples.
    let a = rig.add(1, ModuleKind::SineVco { freq_hz: 5_000.0 });
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 10);
    let onset = &rig.left[edit..];
    assert!(
        peak(&onset[..20]) < 0.25,
        "no fade-in: peak {}",
        peak(&onset[..20])
    );
    assert!(peak(&onset[1_000..]) > 0.99, "fade-in never completes");
}

#[test]
fn parameter_flood_is_coalesced_and_applied() {
    let mut rig = Rig::new();
    let a = rig.add(1, ModuleKind::SineVco { freq_hz: 220.0 });
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(BLOCK);
    for i in 0..10_000 {
        rig.send(Command::SetParam {
            module: a,
            param: "freq",
            value: 100.0 + i as f32 * 0.01,
        });
    }
    rig.send(Command::SetParam {
        module: a,
        param: "freq",
        value: 440.0,
    });
    rig.flush();
    rig.render(SR as usize);
    let tail = &rig.left[rig.left.len() - SR as usize / 2..];
    assert!((freq_by_zero_crossings(tail) - 440.0).abs() < 0.1);
}

#[test]
fn retuning_does_not_click() {
    let mut rig = Rig::new();
    let a = rig.add(1, ModuleKind::SineVco { freq_hz: 220.0 });
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 4);
    let edit = rig.left.len();
    rig.send(Command::SetParam {
        module: a,
        param: "freq",
        value: 440.0,
    });
    rig.flush();
    rig.render(SR as usize / 4);
    assert!(click_ratio(&rig.left[edit - 1..edit + 2400], &[440.0]) <= 1.2);
}

#[test]
fn removed_clock_stops_sending_events() {
    let mut rig = Rig::new();
    let clock = rig.add(1, ModuleKind::Clock { bpm: 600.0 });
    rig.flush();
    rig.render(SR as usize);
    assert!(!rig.events().is_empty());
    rig.send(Command::RemoveModule { id: clock });
    rig.flush();
    rig.render(SR as usize);
    assert!(rig.events().is_empty());
}

#[test]
fn invalid_commands_are_rejected() {
    let mut rig = Rig::new();
    let a = rig.add(1, ModuleKind::SineVco { freq_hz: 220.0 });
    let b = rig.add(2, ModuleKind::SineVco { freq_hz: 330.0 });
    let e = |r: Result<(), EngineError>| r.unwrap_err();
    let engine = &mut rig.engine;

    assert_eq!(
        e(engine.send(Command::AddModule {
            id: a,
            kind: ModuleKind::Constant { value: 0.0 }
        })),
        EngineError::DuplicateModule(a)
    );
    assert_eq!(
        e(engine.send(Command::AddModule {
            id: ModuleId::OUTPUT,
            kind: ModuleKind::Constant { value: 0.0 }
        })),
        EngineError::ReservedModule(ModuleId::OUTPUT)
    );
    assert_eq!(
        e(engine.send(Command::RemoveModule { id: ModuleId(99) })),
        EngineError::UnknownModule(ModuleId(99))
    );
    assert_eq!(
        e(engine.send(Command::SetParam {
            module: a,
            param: "bpm",
            value: 1.0
        })),
        EngineError::UnknownParam {
            module: a,
            param: "bpm"
        }
    );
    let bad_port = PortRef::new(a, "nope");
    assert_eq!(
        e(engine.send(Command::Connect {
            from: bad_port,
            to: PortRef::output()
        })),
        EngineError::NotAnOutput(bad_port)
    );
    assert_eq!(
        e(engine.send(Command::Connect {
            from: out(a),
            to: out(b)
        })),
        EngineError::NotAnInput(out(b))
    );

    engine
        .send(Command::Connect {
            from: out(a),
            to: PortRef::new(b, "voct"),
        })
        .unwrap();
    assert_eq!(
        e(engine.send(Command::Connect {
            from: out(b),
            to: PortRef::new(a, "voct")
        })),
        EngineError::Cycle
    );
    assert_eq!(
        e(engine.send(Command::Disconnect {
            from: out(b),
            to: PortRef::new(a, "voct")
        })),
        EngineError::NotConnected {
            from: out(b),
            to: PortRef::new(a, "voct")
        }
    );
    engine
        .send(Command::Disconnect {
            from: out(a),
            to: PortRef::new(b, "voct"),
        })
        .unwrap();
    engine.flush().unwrap();
}

// ------------------------------------------------------------- synth-modules

#[test]
fn sine_is_a_pure_full_scale_tone_on_both_speakers() {
    let mut rig = Rig::new();
    let a = rig.add(1, ModuleKind::SineVco { freq_hz: 440.0 });
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(SR as usize);
    let left = &rig.left[..SR as usize];
    assert!((freq_by_zero_crossings(left) - 440.0).abs() < 0.1);
    assert!((peak(left) - 1.0).abs() <= 0.01);
    assert_eq!(rig.left, rig.right, "mono module must reach both speakers");
}

#[test]
fn square_is_full_scale_with_odd_harmonics() {
    let mut rig = Rig::new();
    let a = rig.add(1, ModuleKind::SquareVco { freq_hz: 220.0 });
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(SR as usize);
    let x = &rig.left[SR as usize / 10..];
    assert!((freq_by_zero_crossings(x) - 220.0).abs() < 0.1);
    let rms = (x.iter().map(|s| s * s).sum::<f32>() / x.len() as f32).sqrt();
    // A square's RMS is close to its peak; a sine's is 0.707 of it.
    assert!(
        rms / peak(x) > 0.85,
        "not square-like: rms/peak = {}",
        rms / peak(x)
    );
    // fundsp's band-limited wavetable square peaks near 0.9; aliasing would show
    // as spikes well above full scale.
    assert!((0.85..1.25).contains(&peak(x)), "peak {}", peak(x));
    assert_eq!(rig.left, rig.right);
}

#[test]
fn square_follows_pitch_cv() {
    let mut rig = Rig::new();
    let cv = rig.add(1, ModuleKind::Constant { value: -1.0 });
    let a = rig.add(2, ModuleKind::SquareVco { freq_hz: 440.0 });
    rig.connect(out(cv), PortRef::new(a, "voct"));
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(SR as usize);
    assert!((freq_by_zero_crossings(&rig.left[SR as usize / 10..]) - 220.0).abs() < 0.1);
}

#[test]
fn one_volt_of_pitch_cv_raises_one_octave() {
    let mut rig = Rig::new();
    let cv = rig.add(1, ModuleKind::Constant { value: 1.0 });
    let a = rig.add(2, ModuleKind::SineVco { freq_hz: 261.63 });
    rig.connect(out(cv), PortRef::new(a, "voct"));
    rig.connect(out(a), PortRef::output());
    rig.flush();
    rig.render(SR as usize);
    assert!((freq_by_zero_crossings(&rig.left) - 523.25).abs() < 0.1);
}

#[test]
fn an_output_fans_out_to_many_inputs() {
    let mut rig = Rig::new();
    let cv = rig.add(1, ModuleKind::Constant { value: 1.0 });
    let a = rig.add(2, ModuleKind::SineVco { freq_hz: 220.0 });
    let b = rig.add(3, ModuleKind::SineVco { freq_hz: 220.0 });
    for m in [a, b] {
        rig.connect(out(cv), PortRef::new(m, "voct"));
        rig.connect(out(m), PortRef::output());
    }
    rig.flush();
    rig.render(SR as usize);
    // Both sines received the CV: one 440 Hz tone at double amplitude.
    assert!((freq_by_zero_crossings(&rig.left) - 440.0).abs() < 0.1);
    assert!((peak(&rig.left) - 2.0).abs() < 0.02);
}

#[test]
fn an_input_takes_only_one_cable() {
    let mut rig = Rig::new();
    let cv1 = rig.add(1, ModuleKind::Constant { value: 0.0 });
    let cv2 = rig.add(2, ModuleKind::Constant { value: 0.0 });
    let a = rig.add(3, ModuleKind::SineVco { freq_hz: 220.0 });
    rig.connect(out(cv1), PortRef::new(a, "voct"));
    let err = rig
        .engine
        .send(Command::Connect {
            from: out(cv2),
            to: PortRef::new(a, "voct"),
        })
        .unwrap_err();
    assert_eq!(err, EngineError::InputOccupied(PortRef::new(a, "voct")));
}

#[test]
fn clock_at_120_bpm_ticks_every_24000_samples_on_its_gate() {
    let mut rig = Rig::new();
    let clock = rig.add(1, ModuleKind::Clock { bpm: 120.0 });
    rig.connect(PortRef::new(clock, "gate"), PortRef::output());
    rig.flush();
    rig.render(10 * SR as usize);

    let ticks: Vec<u64> = rig
        .events()
        .into_iter()
        .map(|Event::ClockTick { sample_time, .. }| sample_time)
        .filter(|&t| t < 10 * SR as u64)
        .collect();
    assert_eq!(ticks.len(), 20);
    assert!(ticks.windows(2).all(|w| w[1] - w[0] == 24_000));

    // Each tick lands exactly on a rising edge of the gate the speakers carry.
    let edges: Vec<u64> = (0..10 * SR as usize)
        .filter(|&i| rig.left[i] > 0.5 && (i == 0 || rig.left[i - 1] <= 0.5))
        .map(|i| i as u64)
        .collect();
    assert_eq!(ticks, edges);
}

#[test]
fn clock_phase_ramps_once_per_beat() {
    let mut rig = Rig::new();
    let clock = rig.add(1, ModuleKind::Clock { bpm: 120.0 });
    rig.connect(PortRef::new(clock, "phase"), PortRef::output());
    rig.flush();
    rig.render(SR as usize);
    assert_eq!(rig.left[0], 0.0);
    assert!((rig.left[12_000] - 0.5).abs() < 1e-4);
    assert!(rig.left[23_999] > 0.999);
    assert_eq!(rig.left[24_000], 0.0);
}

#[test]
fn tempo_change_keeps_the_beat_position() {
    let mut rig = Rig::new();
    let clock = rig.add(1, ModuleKind::Clock { bpm: 120.0 });
    rig.flush();
    rig.render(12_032); // just past half a beat at 120 BPM
    rig.send(Command::SetParam {
        module: clock,
        param: "bpm",
        value: 60.0,
    });
    rig.flush();
    rig.render(SR as usize * 2);
    let ticks: Vec<u64> = rig
        .events()
        .into_iter()
        .map(|Event::ClockTick { sample_time, .. }| sample_time)
        .collect();
    // Half a beat had elapsed, so the next tick is half a 60 BPM beat later.
    assert_eq!(ticks[0], 0);
    assert!(
        ticks[1].abs_diff(12_032 + 23_936) <= 1,
        "next tick at {}",
        ticks[1]
    );
}

// ------------------------------------------------------------- envelope and VCA

fn envelope(rig: &mut Rig, id: u64, attack_s: f32, decay_s: f32) -> ModuleId {
    rig.add(id, ModuleKind::Envelope { attack_s, decay_s })
}

#[test]
fn envelope_attacks_then_decays_to_minus_60_db() {
    let mut rig = Rig::new();
    let clock = rig.add(1, ModuleKind::Clock { bpm: 120.0 });
    let env = envelope(&mut rig, 2, 0.002, 0.2);
    rig.connect(PortRef::new(clock, "gate"), PortRef::new(env, "gate"));
    rig.connect(out(env), PortRef::output());
    rig.flush();
    rig.render(SR as usize);

    let attack = (0.002 * SR as f32) as usize; // 96 samples
    let decay = (0.2 * SR as f32) as usize; // 9,600 samples
    let x = &rig.left;
    assert!(
        x[attack / 2] > 0.4 && x[attack / 2] < 0.6,
        "mid-attack {}",
        x[attack / 2]
    );
    assert!((x[attack] - 1.0).abs() < 0.02, "peak {}", x[attack]);
    let at_decay = x[attack + decay];
    assert!(
        (0.0008..0.0012).contains(&at_decay),
        "-60 dB point {at_decay}"
    );
    assert!(x[23_999] < 1e-4, "silent before the next beat");
    // The second beat (sample 24,000) retriggers it.
    assert!((x[24_000 + attack] - 1.0).abs() < 0.02);
}

#[test]
fn envelope_retrigger_never_jumps() {
    let mut rig = Rig::new();
    // 600 BPM: a new beat every 4,800 samples, long before a 0.5 s decay ends.
    let clock = rig.add(1, ModuleKind::Clock { bpm: 600.0 });
    let env = envelope(&mut rig, 2, 0.01, 0.5);
    rig.connect(PortRef::new(clock, "gate"), PortRef::new(env, "gate"));
    rig.connect(out(env), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 2);
    let max_step = rig
        .left
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max);
    let attack_step = 1.0 / (0.01 * SR as f32);
    assert!(max_step <= attack_step * 1.01, "jump of {max_step}");
}

#[test]
fn vca_scales_by_cv_and_gain_and_is_closed_without_cv() {
    let mut rig = Rig::new();
    let osc = rig.add(1, ModuleKind::SineVco { freq_hz: 440.0 });
    let cv = rig.add(2, ModuleKind::Constant { value: 0.5 });
    let vca = rig.add(3, ModuleKind::Vca { gain: 0.5 });
    rig.connect(out(osc), PortRef::new(vca, "in"));
    rig.connect(out(cv), PortRef::new(vca, "cv"));
    rig.connect(out(vca), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 2);
    let tail = &rig.left[SR as usize / 4..];
    assert!((peak(tail) - 0.25).abs() < 0.005, "peak {}", peak(tail));

    rig.send(Command::Disconnect {
        from: out(cv),
        to: PortRef::new(vca, "cv"),
    });
    rig.flush();
    rig.render(SR as usize / 4);
    let end = &rig.left[rig.left.len() - BLOCK..];
    assert!(peak(end) < 1e-6, "VCA leaks {} with no CV", peak(end));
}

#[test]
fn vca_gain_change_does_not_zipper() {
    let mut rig = Rig::new();
    let osc = rig.add(1, ModuleKind::SineVco { freq_hz: 440.0 });
    let cv = rig.add(2, ModuleKind::Constant { value: 1.0 });
    let vca = rig.add(3, ModuleKind::Vca { gain: 0.1 });
    rig.connect(out(osc), PortRef::new(vca, "in"));
    rig.connect(out(cv), PortRef::new(vca, "cv"));
    rig.connect(out(vca), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 4);
    let edit = rig.left.len();
    rig.send(Command::SetParam {
        module: vca,
        param: "gain",
        value: 1.0,
    });
    rig.flush();
    rig.render(SR as usize / 4);
    assert!(click_ratio(&rig.left[edit - 1..edit + 2400], &[440.0]) <= 1.2);
    let tail = &rig.left[rig.left.len() - SR as usize / 10..];
    assert!((peak(tail) - 1.0).abs() < 0.01);
}

#[test]
fn clock_blip_patch_sounds_on_every_tick() {
    let mut rig = Rig::new();
    let clock = rig.add(1, ModuleKind::Clock { bpm: 120.0 });
    let env = envelope(&mut rig, 2, 0.002, 0.15);
    let osc = rig.add(3, ModuleKind::SineVco { freq_hz: 880.0 });
    let vca = rig.add(4, ModuleKind::Vca { gain: 1.0 });
    rig.connect(PortRef::new(clock, "gate"), PortRef::new(env, "gate"));
    rig.connect(out(osc), PortRef::new(vca, "in"));
    rig.connect(out(env), PortRef::new(vca, "cv"));
    rig.connect(out(vca), PortRef::output());
    rig.flush();
    rig.render(2 * SR as usize);

    let ticks: Vec<usize> = rig
        .events()
        .into_iter()
        .map(|Event::ClockTick { sample_time, .. }| sample_time as usize)
        .filter(|&t| t + 24_000 <= rig.left.len())
        .collect();
    assert_eq!(ticks, vec![0, 24_000, 48_000, 72_000]);
    for &t in &ticks {
        assert!(
            peak(&rig.left[t..t + 2_000]) > 0.5,
            "no blip after tick {t}"
        );
        assert!(
            peak(&rig.left[t + 20_000..t + 24_000]) < 0.01,
            "blip rings past tick {t}"
        );
        // A 2 ms attack: the onset is smooth, not a click.
        let onset = if t == 0 {
            &rig.left[..400]
        } else {
            &rig.left[t - 1..t + 400]
        };
        assert!(click_ratio(onset, &[880.0]) <= 1.2, "click at tick {t}");
    }
}

#[test]
fn a_loaded_sample_plays_once_to_the_speakers() {
    let mut rig = Rig::new();
    // A 10 ms click: a decaying burst.
    let click: Vec<f32> = (0..480).map(|i| 0.8 * (1. - i as f32 / 480.) * if i % 2 == 0 { 1. } else { -1. }).collect();
    rig.engine.load_sample(stubby_audio::SampleId(1), SR, vec![click]);
    rig.send(Command::PlaySample {
        sample: stubby_audio::SampleId(1),
        gain: 1.,
    });
    rig.flush();
    rig.render(SR as usize / 10);
    assert!(peak(&rig.left) > 0.1, "the click sounds");
    let after = rig.left.len();
    rig.render(SR as usize / 10);
    assert!(peak(&rig.left[after..]) < 1e-4, "and is over");
}
