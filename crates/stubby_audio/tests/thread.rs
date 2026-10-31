//! The engine on its control thread, driven the way the game drives it.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::{Duration, Instant};
use stubby_audio::{
    AudioThread, Command, EngineConfig, EngineError, Event, ModuleId, ModuleKind, Notice, PortRef,
};

fn offline() -> AudioThread {
    AudioThread::start_offline(EngineConfig {
        sample_rate: 48_000,
        max_block_frames: 256,
    })
    .unwrap()
}

/// Poll until `f` is true, failing after two seconds.
fn wait_for(what: &str, mut f: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(2);
    while !f() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn tone(audio: &AudioThread) {
    let osc = ModuleId(1);
    audio.send(Command::AddModule {
        id: osc,
        kind: ModuleKind::SquareVco { freq_hz: 110.0 },
    });
    audio.send(Command::Connect {
        from: PortRef::new(osc, "out"),
        to: PortRef::output(),
    });
}

#[test]
fn handle_can_be_shared_across_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AudioThread>();
}

#[test]
fn silent_until_a_commit_applies_the_patch() {
    let audio = offline();
    wait_for("rendering", || audio.offline_frames_rendered() > 48_000);
    assert_eq!(
        audio.offline_last_block_peak(),
        0.0,
        "fresh engine must be silent"
    );

    // Sent but not committed: still silent.
    tone(&audio);
    let rendered = audio.offline_frames_rendered();
    wait_for("more rendering", || {
        audio.offline_frames_rendered() > rendered + 24_000
    });
    assert_eq!(
        audio.offline_last_block_peak(),
        0.0,
        "commands applied before commit"
    );

    audio.commit();
    wait_for("sound after commit", || {
        audio.offline_last_block_peak() > 0.5
    });
}

#[test]
fn rejected_commands_come_back_as_notices() {
    let audio = offline();
    let missing = ModuleId(99);
    let bad = Command::Connect {
        from: PortRef::new(missing, "out"),
        to: PortRef::output(),
    };
    audio.send(bad);
    audio.commit();
    let mut notices = Vec::new();
    wait_for("rejection", || {
        audio.drain(&mut notices);
        !notices.is_empty()
    });
    assert_eq!(
        notices[0],
        Notice::Rejected {
            command: bad,
            error: EngineError::UnknownModule(missing),
        }
    );
}

#[test]
fn clock_ticks_arrive_with_a_predicted_instant() {
    let audio = offline();
    let clock = ModuleId(1);
    audio.send(Command::AddModule {
        id: clock,
        kind: ModuleKind::Clock { bpm: 600.0 },
    });
    audio.commit();

    let mut ticks = Vec::new();
    let mut notices = Vec::new();
    wait_for("three ticks", || {
        audio.drain(&mut notices);
        ticks.extend(notices.drain(..).filter_map(|n| match n {
            Notice::Event(Event::ClockTick {
                module,
                sample_time,
            }) if module == clock => Some(sample_time),
            _ => None,
        }));
        ticks.len() >= 3
    });
    // 600 BPM at 48 kHz: one tick every 4,800 samples.
    assert!(ticks.windows(2).all(|w| w[1] - w[0] == 4_800), "{ticks:?}");
    assert_eq!(audio.sample_rate(), Some(48_000));
    let a = audio.sample_to_instant(ticks[0]).unwrap();
    let b = audio.sample_to_instant(ticks[1]).unwrap();
    let gap = b.duration_since(a).as_secs_f64();
    assert!((gap - 0.1).abs() < 1e-6, "ticks 0.1 s apart, got {gap}");
}

#[test]
fn engine_keeps_running_while_the_game_thread_stalls() {
    let audio = offline();
    tone(&audio);
    audio.commit();
    wait_for("sound", || audio.offline_last_block_peak() > 0.5);
    let before = audio.offline_frames_rendered();
    std::thread::sleep(Duration::from_millis(200)); // a 200 ms game-frame hitch
    let rendered = audio.offline_frames_rendered() - before;
    assert!(
        // 200 ms is 9,600 frames; allow for block granularity.
        rendered >= 8_640,
        "only {rendered} frames rendered during a 200 ms stall"
    );
    assert!(
        audio.offline_last_block_peak() > 0.5,
        "sound stopped during the stall"
    );
}
