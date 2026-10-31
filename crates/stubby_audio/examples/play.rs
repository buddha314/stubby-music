//! Listening test on the default output device: a sine fades in, a clock starts and
//! plays a blip on every beat (each tick also printed with its audio-clock
//! timestamp), the sine is retuned by pitch CV, and finally removed. Listen for
//! clicks at each step.
//!
//! cargo run --release -p stubby_audio --example play

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, Instant};
use stubby_audio::{Command, Engine, Event, ModuleId, ModuleKind, PortRef};

const SINE: ModuleId = ModuleId(1);
const SINE_VCA: ModuleId = ModuleId(2);
const ONE: ModuleId = ModuleId(3);
const CLOCK: ModuleId = ModuleId(4);
const ENV: ModuleId = ModuleId(5);
const BLIP: ModuleId = ModuleId(6);
const BLIP_VCA: ModuleId = ModuleId(7);
const CV: ModuleId = ModuleId(8);

fn send(engine: &mut Engine, commands: &[Command]) {
    for &command in commands {
        engine.send(command).unwrap();
    }
}

fn cable(from: ModuleId, from_port: &'static str, to: ModuleId, to_port: &'static str) -> Command {
    Command::Connect {
        from: PortRef::new(from, from_port),
        to: PortRef::new(to, to_port),
    }
}

fn main() {
    let mut engine = Engine::start().expect("open audio device");
    println!("sample rate: {:?}", engine.sample_rate());

    let mut events = Vec::new();
    // Stand-in for a game loop: flush and drain once per ~16 ms frame.
    let mut run = |engine: &mut Engine, secs: f64| {
        let end = Instant::now() + Duration::from_secs_f64(secs);
        while Instant::now() < end {
            engine.flush().expect("flush");
            engine.drain_events(&mut events);
            for Event::ClockTick { sample_time, .. } in events.drain(..) {
                let late_ms = engine
                    .sample_to_instant(sample_time)
                    .map(|at| Instant::now().saturating_duration_since(at).as_secs_f64() * 1e3);
                println!("  tick at sample {sample_time} (seen {late_ms:.1?} ms after it played)");
            }
            std::thread::sleep(Duration::from_millis(16));
        }
    };

    // The tone and the blips share the output, so keep their sum under full scale.
    println!("0s  sine 220 Hz fades in (through a VCA at half gain)");
    send(
        &mut engine,
        &[
            Command::AddModule {
                id: SINE,
                kind: ModuleKind::SineVco { freq_hz: 220.0 },
            },
            Command::AddModule {
                id: ONE,
                kind: ModuleKind::Constant { value: 1.0 },
            },
            Command::AddModule {
                id: SINE_VCA,
                kind: ModuleKind::Vca { gain: 0.5 },
            },
            cable(SINE, "out", SINE_VCA, "in"),
            cable(ONE, "out", SINE_VCA, "cv"),
            Command::Connect {
                from: PortRef::new(SINE_VCA, "out"),
                to: PortRef::output(),
            },
        ],
    );
    run(&mut engine, 2.0);

    println!("2s  clock at 90 BPM starts: clock -> envelope -> VCA on an 880 Hz sine");
    send(
        &mut engine,
        &[
            Command::AddModule {
                id: CLOCK,
                kind: ModuleKind::Clock { bpm: 90.0 },
            },
            Command::AddModule {
                id: ENV,
                kind: ModuleKind::Envelope {
                    attack_s: 0.002,
                    decay_s: 0.15,
                },
            },
            Command::AddModule {
                id: BLIP,
                kind: ModuleKind::SineVco { freq_hz: 880.0 },
            },
            Command::AddModule {
                id: BLIP_VCA,
                kind: ModuleKind::Vca { gain: 0.4 },
            },
            cable(CLOCK, "gate", ENV, "gate"),
            cable(BLIP, "out", BLIP_VCA, "in"),
            cable(ENV, "out", BLIP_VCA, "cv"),
            Command::Connect {
                from: PortRef::new(BLIP_VCA, "out"),
                to: PortRef::output(),
            },
        ],
    );
    run(&mut engine, 3.0);

    println!("5s  +1 V pitch CV: sine jumps an octave");
    send(
        &mut engine,
        &[
            Command::AddModule {
                id: CV,
                kind: ModuleKind::Constant { value: 1.0 },
            },
            cable(CV, "out", SINE, "voct"),
        ],
    );
    run(&mut engine, 2.0);

    println!("7s  sine removed (fades out); blips continue");
    send(&mut engine, &[Command::RemoveModule { id: SINE_VCA }]);
    run(&mut engine, 2.0);
    println!("done");
}
