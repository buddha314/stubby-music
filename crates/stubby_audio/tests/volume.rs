//! The player's volume stages (`Command::SetBusVolume`): the world is heard through
//! World and Master, interface sounds through Interface and Master.
#![allow(clippy::unwrap_used)]
use stubby_audio::{Bus, Command, Engine, EngineConfig, ModuleId, ModuleKind, PortRef};

const SR: u32 = 48_000;

/// The loudest sample in the second half of one second of a gate patched to the world,
/// after `volumes` are set (the first half lets any volume ramp settle).
fn world_peak(volumes: &[(Bus, f32)]) -> f32 {
    let (mut engine, mut renderer) = Engine::offline(EngineConfig {
        sample_rate: SR,
        max_block_frames: 1024,
    })
    .unwrap();
    engine
        .send(Command::AddModule {
            id: ModuleId(1),
            kind: ModuleKind::RandomClock {
                seed: 7,
                min_interval_s: 0.05,
                max_interval_s: 0.05,
            },
        })
        .unwrap();
    engine
        .send(Command::Connect {
            from: PortRef::new(ModuleId(1), "gate"),
            to: PortRef::output(),
        })
        .unwrap();
    for &(bus, volume) in volumes {
        engine.send(Command::SetBusVolume { bus, volume }).unwrap();
    }
    engine.flush().unwrap();
    let mut buffer = vec![0.; 1024 * 2];
    let mut peak = 0f32;
    let frames = SR as usize;
    for offset in (0..frames).step_by(1024) {
        let count = (frames - offset).min(1024) * 2;
        renderer.render(&mut buffer[..count]);
        if offset >= frames / 2 {
            peak = buffer[..count].iter().fold(peak, |p, s| p.max(s.abs()));
        }
    }
    peak
}

#[test]
fn the_world_is_heard_at_full_volume_by_default() {
    assert!(world_peak(&[]) > 0.1);
}

#[test]
fn master_and_world_silence_the_world_and_interface_does_not() {
    assert_eq!(world_peak(&[(Bus::Master, 0.0)]), 0.0);
    assert_eq!(world_peak(&[(Bus::World, 0.0)]), 0.0);
    assert!(world_peak(&[(Bus::Interface, 0.0)]) > 0.1);
}

#[test]
fn half_volume_is_quieter() {
    let full = world_peak(&[]);
    let half = world_peak(&[(Bus::Master, 0.5)]);
    assert!(half > 0.0 && half < full * 0.75, "{half} vs {full}");
}

#[test]
fn a_volume_must_be_finite() {
    let (mut engine, _renderer) = Engine::offline(EngineConfig::default()).unwrap();
    assert!(
        engine
            .send(Command::SetBusVolume {
                bus: Bus::Master,
                volume: f32::NAN
            })
            .is_err()
    );
}
