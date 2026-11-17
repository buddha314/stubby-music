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

#[cfg(feature = "audionimbus")]
#[test]
fn audionimbus_live_emitter_path_is_allocation_free_at_supported_rates() {
    for sample_rate in [44_100, 48_000] {
        let (mut engine, mut renderer) = Engine::offline(EngineConfig {
            sample_rate,
            max_block_frames: BLOCK as u32,
        })
        .unwrap();
        let oscillator = ModuleId(90_001);
        engine
            .send(Command::AddModule {
                id: oscillator,
                kind: ModuleKind::SquareVco { freq_hz: 220.0 },
            })
            .unwrap();
        engine
            .send(Command::Connect {
                from: out(oscillator),
                to: PortRef::output(),
            })
            .unwrap();
        engine
            .send(Command::SetEmitter {
                output: out(oscillator),
                offset: [1.0, 0.0, 2.0],
                position_world: [1.0, 0.0, 2.0],
                volume: 1.0,
            })
            .unwrap();
        engine.flush().unwrap();

        let mut output = vec![0.0; BLOCK * 2];
        let mut block_times = Vec::with_capacity(32);
        let mut first_active_block = None;
        for block in 0..32 {
            let before = violation_count();
            let started = std::time::Instant::now();
            assert_no_alloc(|| renderer.render(&mut output));
            block_times.push(started.elapsed());
            assert_eq!(violation_count(), before, "AudioNimbus callback allocated");
            assert!(output.iter().all(|sample| sample.is_finite()));
            if first_active_block.is_none() && output.iter().any(|sample| sample.abs() > 1e-5) {
                first_active_block = Some(block);
            }
        }
        assert_eq!(
            first_active_block,
            Some(1),
            "expected one 256-frame block of latency"
        );
        block_times.sort_unstable();
        eprintln!(
            "AudioNimbus 1-emitter offline at {sample_rate} Hz: first active block={first_active_block:?}, p50={:?}, p95={:?}, max={:?}",
            block_times[16], block_times[30], block_times[31],
        );
    }
}

#[cfg(feature = "audionimbus")]
#[test]
fn audionimbus_static_wall_transmission_attenuates_live_emitter() {
    fn measure(through_wall: bool) -> f32 {
        let (mut engine, mut renderer) = Engine::offline(EngineConfig {
            sample_rate: SR,
            max_block_frames: BLOCK as u32,
        })
        .unwrap();
        if through_wall {
            engine.set_acoustic_geometry(stubby_audio::AcousticSceneGeometry {
                vertices: vec![
                    [-10.0, -10.0, 1.0],
                    [10.0, -10.0, 1.0],
                    [10.0, 10.0, 1.0],
                    [-10.0, 10.0, 1.0],
                ],
                triangles: vec![[0, 1, 2], [0, 2, 3]],
                material_indices: vec![0, 0],
                materials: vec![stubby_audio::AcousticMaterial {
                    absorption: [0.8; 3],
                    scattering: 0.1,
                    transmission: [0.0; 3],
                }],
            });
        }
        let oscillator = ModuleId(90_002);
        engine
            .send(Command::AddModule {
                id: oscillator,
                kind: ModuleKind::SquareVco { freq_hz: 220.0 },
            })
            .unwrap();
        engine
            .send(Command::Connect {
                from: out(oscillator),
                to: PortRef::output(),
            })
            .unwrap();
        engine
            .send(Command::SetEmitter {
                output: out(oscillator),
                offset: [0.0, 0.0, 3.0],
                position_world: [0.0, 0.0, 3.0],
                volume: 1.0,
            })
            .unwrap();
        engine.flush().unwrap();

        let mut output = vec![0.0; BLOCK * 2];
        let mut energy = 0.0;
        let mut samples = 0usize;
        for block in 0..24 {
            engine.flush().unwrap();
            let before = violation_count();
            assert_no_alloc(|| renderer.render(&mut output));
            assert_eq!(violation_count(), before, "AudioNimbus callback allocated");
            if block >= 16 {
                energy += output.iter().map(|sample| sample * sample).sum::<f32>();
                samples += output.len();
            }
            std::thread::sleep(std::time::Duration::from_millis(55));
        }
        (energy / samples as f32).sqrt()
    }

    let clear_rms = measure(false);
    let blocked_rms = measure(true);
    eprintln!("AudioNimbus synthetic wall RMS: clear={clear_rms}, blocked={blocked_rms}");
    assert!(clear_rms > 0.001, "clear path was silent: {clear_rms}");
    assert!(
        blocked_rms < clear_rms * 0.8,
        "wall did not attenuate enough: clear={clear_rms}, blocked={blocked_rms}"
    );
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
    // The clock sounds from the next block, mid-beat on the master grid (lock-clocks-to-
    // master): its gate is high there, and no beat starts inside this block.
    assert_eq!(rig.left[BLOCK * 4 - 1], 0.0);
    assert_eq!(rig.left[BLOCK * 4], 1.0);
    assert!(rig.events().is_empty());
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
    // The new clock's first beat is the next 120 BPM beat on the master grid.
    let beat = SR as u64 / 2;
    assert!(rig.events().contains(&Event::ClockTick {
        module: clock,
        sample_time: (edit as u64).div_ceil(beat) * beat
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
        .filter_map(|event| match event {
            Event::ClockTick { sample_time, .. } => Some(sample_time),
            _ => None,
        })
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
fn tempo_change_keeps_the_master_grid() {
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
        .filter_map(|event| match event {
            Event::ClockTick { sample_time, .. } => Some(sample_time),
            _ => None,
        })
        .collect();
    // No tick at the change; the next falls on the 60 BPM grid, one master-locked
    // beat (48,000 samples) from the start.
    assert_eq!(ticks[..3], [0, SR as u64, 2 * SR as u64]);
}

// ------------------------------------------------------------- envelope and VCA

fn envelope(rig: &mut Rig, id: u64, attack_s: f32, decay_s: f32) -> ModuleId {
    rig.add(id, ModuleKind::Envelope { attack_s, decay_s })
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
        .filter_map(|event| match event {
            Event::ClockTick { sample_time, .. } => Some(sample_time as usize),
            _ => None,
        })
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

// ------------------------------------------------------------- step sequencer

#[test]
fn a_clocked_sequencer_steps_and_gates_its_lit_steps() {
    let mut rig = Rig::new();
    let clock = rig.add(1, ModuleKind::Clock { bpm: 120.0 });
    let seq = rig.add(
        2,
        ModuleKind::StepSequencer {
            cv: [0.0; 4],
            gate: [0.0; 4],
        },
    );
    rig.connect(PortRef::new(clock, "gate"), PortRef::new(seq, "clock"));
    rig.connect(PortRef::new(seq, "gate"), PortRef::output());
    // Light steps 2 and 4 (a backbeat); a fractional CV snaps to a semitone.
    for (param, value) in [("gate_2", 1.0), ("gate_4", 1.0), ("cv_3", 0.3)] {
        rig.send(Command::SetParam {
            module: seq,
            param,
            value,
        });
    }
    rig.flush();
    rig.render(SR as usize * 2); // four beats at 120 BPM
    let steps: Vec<(u8, u64)> = rig
        .events()
        .into_iter()
        .filter_map(|event| match event {
            Event::Step {
                step, sample_time, ..
            } => Some((step, sample_time)),
            _ => None,
        })
        .collect();
    let beat = SR as u64 / 2;
    // The sequencer steps on each of the clock's beats (one sample later, the cable).
    assert_eq!(steps.iter().map(|s| s.0).collect::<Vec<_>>(), [0, 1, 2, 3]);
    for (k, (_, at)) in steps.iter().enumerate() {
        assert!(at.abs_diff(k as u64 * beat) <= 1, "step {k} at {at}");
    }
    // Only beats 2 and 4 reach the output.
    let loud = |k: u64| rig.left[(k * beat + 100) as usize] > 0.5;
    assert_eq!(
        (0..4).map(loud).collect::<Vec<_>>(),
        [false, true, false, true]
    );
}

// ------------------------------------------------------------ the world bus

/// The field recorder's meter (sync-recorder-feedback) hears what is placed in the world
/// and the music, never one-shot UI sounds such as the patch snap.
#[test]
fn the_world_meter_hears_the_world_but_not_ui_sounds() {
    let mut rig = Rig::new();
    let meter = rig.engine.world_meter();
    rig.flush();
    rig.render(BLOCK * 4);
    assert_eq!(meter.peak_db(), f32::NEG_INFINITY, "a silent opening");

    let snap = stubby_audio::SampleId(1);
    rig.engine
        .load_sample(snap, SR, vec![vec![0.5; SR as usize]; 2]);
    rig.send(Command::PlaySample {
        sample: snap,
        gain: 1.0,
    });
    rig.flush();
    rig.render(BLOCK * 8);
    assert!(peak(&rig.left[BLOCK * 4..]) > 0.1, "the UI sound plays");
    assert_eq!(
        meter.peak_db(),
        f32::NEG_INFINITY,
        "but not on the world bus"
    );

    // A synthesized confirmation patched to the UI output: heard, not metered.
    let cue = rig.add(2, ModuleKind::SineVco { freq_hz: 880.0 });
    rig.connect(out(cue), PortRef::ui_output());
    rig.flush();
    let from = rig.left.len();
    rig.render(SR as usize / 10);
    assert!(peak(&rig.left[from..]) > 0.1, "the cue plays");
    assert_eq!(
        meter.peak_db(),
        f32::NEG_INFINITY,
        "the cue is not on the world bus"
    );

    let vco = rig.add(1, ModuleKind::SineVco { freq_hz: 220.0 });
    rig.connect(out(vco), PortRef::output());
    rig.flush();
    rig.render(SR as usize / 10);
    assert!(meter.peak_db() > -12., "{}", meter.peak_db());
}
