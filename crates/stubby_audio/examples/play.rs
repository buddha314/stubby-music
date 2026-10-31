//! Listening test on the default output device: a sine fades in, a clock starts
//! ticking (printed with its audio-clock timestamp), the sine is retuned by pitch
//! CV, and finally removed. Listen for clicks at each step.
//!
//! cargo run --release -p stubby_audio --example play

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, Instant};
use stubby_audio::{Command, Engine, Event, ModuleId, ModuleKind, PortRef};

const SINE: ModuleId = ModuleId(1);
const CLOCK: ModuleId = ModuleId(2);
const CV: ModuleId = ModuleId(3);

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

    println!("0s  sine 220 Hz fades in");
    engine
        .send(Command::AddModule {
            id: SINE,
            kind: ModuleKind::SineVco { freq_hz: 220.0 },
        })
        .unwrap();
    engine
        .send(Command::SetParam {
            module: SINE,
            param: "freq",
            value: 220.0,
        })
        .unwrap();
    engine
        .send(Command::Connect {
            from: PortRef::new(SINE, "out"),
            to: PortRef::output(),
        })
        .unwrap();
    run(&mut engine, 2.0);

    println!("2s  clock at 90 BPM starts");
    engine
        .send(Command::AddModule {
            id: CLOCK,
            kind: ModuleKind::Clock { bpm: 90.0 },
        })
        .unwrap();
    run(&mut engine, 3.0);

    println!("5s  +1 V pitch CV: sine jumps an octave");
    engine
        .send(Command::AddModule {
            id: CV,
            kind: ModuleKind::Constant { value: 1.0 },
        })
        .unwrap();
    engine
        .send(Command::Connect {
            from: PortRef::new(CV, "out"),
            to: PortRef::new(SINE, "voct"),
        })
        .unwrap();
    run(&mut engine, 2.0);

    println!("7s  sine removed (fades out)");
    engine.send(Command::RemoveModule { id: SINE }).unwrap();
    run(&mut engine, 1.5);
    println!("done");
}
