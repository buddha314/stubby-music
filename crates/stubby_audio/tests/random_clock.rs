#![allow(clippy::unwrap_used)]
use assert_no_alloc::{AllocDisabler, assert_no_alloc, violation_count};
use stubby_audio::{Command, Engine, EngineConfig, Event, ModuleId, ModuleKind, PortRef};

#[global_allocator]
static ALLOC: AllocDisabler = AllocDisabler;
const SR: u32 = 48_000;

fn render(seed: u64, block: usize, patched: bool, fixed: bool) -> (Vec<u64>, Vec<f32>) {
    let (mut engine, mut renderer) = Engine::offline(EngineConfig {
        sample_rate: SR,
        max_block_frames: 1024,
    })
    .unwrap();
    engine
        .send(Command::AddModule {
            id: ModuleId(1),
            kind: ModuleKind::RandomClock {
                seed,
                min_interval_s: if fixed { 0.125 } else { 0.05 },
                max_interval_s: if fixed { 0.125 } else { 0.12 },
            },
        })
        .unwrap();
    if patched {
        engine
            .send(Command::Connect {
                from: PortRef::new(ModuleId(1), "gate"),
                to: PortRef::output(),
            })
            .unwrap();
    }
    engine.flush().unwrap();
    let mut buffer = vec![0.; block * 2];
    let mut samples = Vec::new();
    let mut events = Vec::new();
    let frames = SR as usize * 2;
    for offset in (0..frames).step_by(block) {
        let count = (frames - offset).min(block) * 2;
        let before = violation_count();
        assert_no_alloc(|| renderer.render(&mut buffer[..count]));
        assert_eq!(
            violation_count(),
            before,
            "random clock allocated in the callback"
        );
        samples.extend(
            buffer[..count]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|frame| frame[0]),
        );
        engine.drain_events(&mut events);
    }
    (
        events
            .iter()
            .map(|Event::ClockTick { sample_time, .. }| *sample_time)
            .collect(),
        samples,
    )
}

#[test]
fn random_ticks_are_seeded_bounded_and_independent_of_callback_size() {
    let (a, silent) = render(42, 64, false, false);
    let (b, _) = render(42, 257, false, false);
    let (c, _) = render(43, 64, false, false);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert!(a.len() > 20);
    let mut intervals = vec![a[0]];
    intervals.extend(a.windows(2).map(|pair| pair[1] - pair[0]));
    let min = (0.05_f32 as f64 * SR as f64).ceil() as u64;
    let max = (0.12_f32 as f64 * SR as f64).floor() as u64;
    assert!(intervals.iter().all(|n| (min..=max).contains(n)));
    assert!(intervals.windows(2).any(|p| p[0] != p[1]));
    assert!(
        silent.iter().all(|v| *v == 0.),
        "unpatched clock must not make sound"
    );
}
#[test]
fn fixed_bounds_emit_exact_five_millisecond_triggers_after_initial_rest() {
    let (ticks, gate) = render(0, 257, true, true);
    assert_eq!(ticks[0], 6000);
    assert!(ticks.windows(2).all(|p| p[1] - p[0] == 6000));
    assert!(gate[..6000].iter().all(|v| *v == 0.));
    assert!(gate[6000..6240].iter().all(|v| *v == 1.));
    assert_eq!(gate[6240], 0.);
}
#[test]
fn removing_random_clock_stops_ticks_and_invalid_bounds_are_rejected() {
    let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
    for (min, max) in [
        (f32::NAN, 1.),
        (0., 1.),
        (2., 1.),
        (1., f32::INFINITY),
        (1., 61.),
    ] {
        assert!(
            engine
                .send(Command::AddModule {
                    id: ModuleId(9),
                    kind: ModuleKind::RandomClock {
                        seed: 42,
                        min_interval_s: min,
                        max_interval_s: max,
                    }
                })
                .is_err()
        );
    }
    engine
        .send(Command::AddModule {
            id: ModuleId(1),
            kind: ModuleKind::RandomClock {
                seed: 42,
                min_interval_s: 0.125,
                max_interval_s: 0.125,
            },
        })
        .unwrap();
    engine.flush().unwrap();
    let mut buffer = vec![0.; SR as usize];
    renderer.render(&mut buffer);
    let mut events = Vec::new();
    engine.drain_events(&mut events);
    assert!(!events.is_empty());
    engine
        .send(Command::RemoveModule { id: ModuleId(1) })
        .unwrap();
    engine.flush().unwrap();
    renderer.render(&mut buffer);
    events.clear();
    engine.drain_events(&mut events);
    assert!(events.is_empty());
}

#[test]
fn lightning_bug_click_is_a_short_existing_square_envelope_vca_patch() {
    let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
    for (id, kind) in [
        (
            1,
            ModuleKind::RandomClock {
                seed: 42,
                min_interval_s: 0.125,
                max_interval_s: 0.125,
            },
        ),
        (2, ModuleKind::SquareVco { freq_hz: 2200. }),
        (
            3,
            ModuleKind::Envelope {
                attack_s: 0.0005,
                decay_s: 0.004,
            },
        ),
        (4, ModuleKind::Vca { gain: 0.12 }),
    ] {
        engine
            .send(Command::AddModule {
                id: ModuleId(id),
                kind,
            })
            .unwrap();
    }
    for (from, output, to, input) in [
        (1, "gate", 3, "gate"),
        (3, "out", 4, "cv"),
        (2, "out", 4, "in"),
    ] {
        engine
            .send(Command::Connect {
                from: PortRef::new(ModuleId(from), output),
                to: PortRef::new(ModuleId(to), input),
            })
            .unwrap();
    }
    engine
        .send(Command::Connect {
            from: PortRef::new(ModuleId(4), "out"),
            to: PortRef::output(),
        })
        .unwrap();
    engine.flush().unwrap();
    let mut output = vec![0.; SR as usize * 2];
    let before = violation_count();
    assert_no_alloc(|| renderer.render(&mut output));
    assert_eq!(violation_count(), before);
    let samples: Vec<_> = output
        .as_chunks::<2>()
        .0
        .iter()
        .map(|frame| frame[0])
        .collect();
    assert!(samples[..6000].iter().all(|sample| *sample == 0.));
    assert!(samples[6000..6400].iter().any(|sample| sample.abs() > 0.01));
    assert!(
        samples[7000..12000]
            .iter()
            .all(|sample| sample.abs() < 1e-5)
    );
    let mut events = Vec::new();
    engine.drain_events(&mut events);
    assert_eq!(
        events[0],
        Event::ClockTick {
            module: ModuleId(1),
            sample_time: 6000
        }
    );
}

#[test]
fn square_wave_can_run_at_lfo_rate_and_follow_gate_conventions() {
    let (mut engine, mut renderer) = Engine::offline(EngineConfig::default()).unwrap();
    engine
        .send(Command::AddModule {
            id: ModuleId(1),
            kind: ModuleKind::SquareVco { freq_hz: 220. },
        })
        .unwrap();
    engine
        .send(Command::SetParam {
            module: ModuleId(1),
            param: "freq",
            value: 0.5,
        })
        .unwrap();
    engine
        .send(Command::Connect {
            from: PortRef::new(ModuleId(1), "out"),
            to: PortRef::output(),
        })
        .unwrap();
    engine.flush().unwrap();
    let mut output = vec![0.; SR as usize * 4];
    let before = violation_count();
    assert_no_alloc(|| renderer.render(&mut output));
    assert_eq!(violation_count(), before);
    let a = output[(SR as usize / 2) * 2];
    let b = output[(SR as usize * 3 / 2) * 2];
    assert!(
        a * b < -0.5,
        "half-cycle samples must have opposite polarity: {a}, {b}"
    );
    assert_ne!(
        stubby_audio::signal::is_gate_high(a),
        stubby_audio::signal::is_gate_high(b)
    );
}
