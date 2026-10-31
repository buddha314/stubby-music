//! Connects the game to the audio engine (`stubby_audio`), which runs on its own
//! control thread. The game sends commands during the frame, they are committed as
//! one batch at the end of it, and clock ticks come back as `ClockTick` messages
//! carrying the predicted instant each tick is heard.
//!
//! The world starts silent: nothing is patched until the game adds modules. F8 (or
//! `--audio-demo`) toggles a test patch, the first discovery's square wave plus an
//! audible clock, for listening checks before discovery exists.

use bevy::prelude::*;
use std::time::Instant;
use stubby_audio::{
    AudioThread, Command, EngineConfig, Event, ModuleId, ModuleKind, Notice, PortRef,
};

/// Where the engine sends its sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioOutput {
    /// The default output device. Falls back to no audio if it cannot be opened.
    #[default]
    Device,
    /// No device: the engine renders on its own thread. For tests and headless runs.
    Offline,
    /// No engine at all.
    Off,
}

#[derive(Default)]
pub struct SynthAudioPlugin {
    pub output: AudioOutput,
    /// Start with the demo patch playing.
    pub demo: bool,
}

/// The audio engine, as the game sees it.
#[derive(Resource)]
pub struct Synth {
    thread: Option<AudioThread>,
    demo_playing: bool,
}

impl Synth {
    /// Stage a command for this frame. Ignored when audio is unavailable.
    pub fn send(&self, command: Command) {
        if let Some(thread) = &self.thread {
            thread.send(command);
        }
    }

    pub fn is_running(&self) -> bool {
        self.thread.is_some()
    }

    pub fn demo_playing(&self) -> bool {
        self.demo_playing
    }

    /// The engine thread itself, for diagnostics and tests.
    pub fn thread(&self) -> Option<&AudioThread> {
        self.thread.as_ref()
    }
}

/// A clock module crossed a beat. `at` is when that beat is heard, predicted from
/// the audio clock; schedule visuals against it rather than the frame it arrived in.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ClockTick {
    pub module: ModuleId,
    pub sample_time: u64,
    pub at: Option<Instant>,
}

impl Plugin for SynthAudioPlugin {
    fn build(&self, app: &mut App) {
        let thread = match self.output {
            AudioOutput::Device => AudioThread::start()
                .map_err(|e| warn!("audio disabled: could not open the output device: {e}"))
                .ok(),
            AudioOutput::Offline => AudioThread::start_offline(EngineConfig::default())
                .map_err(|e| warn!("audio disabled: {e}"))
                .ok(),
            AudioOutput::Off => None,
        };
        app.insert_resource(Synth {
            thread,
            demo_playing: false,
        })
        .add_message::<ClockTick>()
        .add_systems(PreUpdate, receive_from_engine)
        .add_systems(Update, toggle_demo)
        .add_systems(Last, commit_to_engine);
        if self.demo {
            app.add_systems(Startup, |mut synth: ResMut<Synth>| {
                set_demo(&mut synth, true)
            });
        }
    }
}

fn receive_from_engine(synth: Res<Synth>, mut ticks: MessageWriter<ClockTick>) {
    let Some(thread) = &synth.thread else {
        return;
    };
    let mut notices = Vec::new();
    thread.drain(&mut notices);
    for notice in notices {
        match notice {
            Notice::Event(Event::ClockTick {
                module,
                sample_time,
            }) => {
                ticks.write(ClockTick {
                    module,
                    sample_time,
                    at: thread.sample_to_instant(sample_time),
                });
            }
            Notice::Rejected { command, error } => warn!("audio rejected {command:?}: {error}"),
            Notice::Error(error) => warn!("audio: {error}"),
        }
    }
}

fn commit_to_engine(synth: Res<Synth>) {
    if let Some(thread) = &synth.thread {
        thread.commit();
    }
}

fn toggle_demo(keys: Option<Res<ButtonInput<KeyCode>>>, mut synth: ResMut<Synth>) {
    if keys.is_some_and(|k| k.just_pressed(KeyCode::F8)) {
        let on = !synth.demo_playing;
        set_demo(&mut synth, on);
    }
}

// Demo modules use high IDs so they never collide with the game's own patch.
const DEMO_SQUARE: ModuleId = ModuleId(9_001);
const DEMO_SQUARE_VCA: ModuleId = ModuleId(9_002);
const DEMO_ONE: ModuleId = ModuleId(9_003);
const DEMO_CLOCK: ModuleId = ModuleId(9_004);
const DEMO_ENV: ModuleId = ModuleId(9_005);
const DEMO_BLIP: ModuleId = ModuleId(9_006);
const DEMO_BLIP_VCA: ModuleId = ModuleId(9_007);
/// The clock's ID, so tests and visuals can pick its ticks out.
pub const DEMO_CLOCK_ID: ModuleId = DEMO_CLOCK;

fn cable(from: ModuleId, out: &'static str, to: ModuleId, input: &'static str) -> Command {
    Command::Connect {
        from: PortRef::new(from, out),
        to: PortRef::new(to, input),
    }
}

/// The first discovery's sound (a low square wave) plus a clock made audible
/// through an envelope and VCA. Levels keep the mix under full scale.
fn set_demo(synth: &mut Synth, on: bool) {
    if !synth.is_running() || synth.demo_playing == on {
        return;
    }
    synth.demo_playing = on;
    let commands = if on {
        vec![
            Command::AddModule {
                id: DEMO_SQUARE,
                kind: ModuleKind::SquareVco { freq_hz: 110.0 },
            },
            Command::AddModule {
                id: DEMO_ONE,
                kind: ModuleKind::Constant { value: 1.0 },
            },
            Command::AddModule {
                id: DEMO_SQUARE_VCA,
                kind: ModuleKind::Vca { gain: 0.25 },
            },
            cable(DEMO_SQUARE, "out", DEMO_SQUARE_VCA, "in"),
            cable(DEMO_ONE, "out", DEMO_SQUARE_VCA, "cv"),
            Command::Connect {
                from: PortRef::new(DEMO_SQUARE_VCA, "out"),
                to: PortRef::output(),
            },
            Command::AddModule {
                id: DEMO_CLOCK,
                kind: ModuleKind::Clock { bpm: 90.0 },
            },
            Command::AddModule {
                id: DEMO_ENV,
                kind: ModuleKind::Envelope {
                    attack_s: 0.002,
                    decay_s: 0.15,
                },
            },
            Command::AddModule {
                id: DEMO_BLIP,
                kind: ModuleKind::SineVco { freq_hz: 880.0 },
            },
            Command::AddModule {
                id: DEMO_BLIP_VCA,
                kind: ModuleKind::Vca { gain: 0.3 },
            },
            cable(DEMO_CLOCK, "gate", DEMO_ENV, "gate"),
            cable(DEMO_BLIP, "out", DEMO_BLIP_VCA, "in"),
            cable(DEMO_ENV, "out", DEMO_BLIP_VCA, "cv"),
            Command::Connect {
                from: PortRef::new(DEMO_BLIP_VCA, "out"),
                to: PortRef::output(),
            },
        ]
    } else {
        [
            DEMO_SQUARE_VCA,
            DEMO_BLIP_VCA,
            DEMO_SQUARE,
            DEMO_ONE,
            DEMO_CLOCK,
            DEMO_ENV,
            DEMO_BLIP,
        ]
        .into_iter()
        .map(|id| Command::RemoveModule { id })
        .collect()
    };
    for command in commands {
        synth.send(command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn app(demo: bool) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SynthAudioPlugin {
                output: AudioOutput::Offline,
                demo,
            });
        app
    }

    /// Run frames until `f` holds, failing after two seconds.
    fn run_until(app: &mut App, what: &str, mut f: impl FnMut(&mut App) -> bool) {
        let end = Instant::now() + Duration::from_secs(2);
        loop {
            app.update();
            if f(app) {
                return;
            }
            assert!(Instant::now() < end, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn peak(app: &App) -> f32 {
        app.world()
            .resource::<Synth>()
            .thread()
            .map_or(0.0, |t| t.offline_last_block_peak())
    }

    fn frames(app: &App) -> u64 {
        app.world()
            .resource::<Synth>()
            .thread()
            .map_or(0, |t| t.offline_frames_rendered())
    }

    #[test]
    fn a_fresh_game_is_silent() {
        let mut app = app(false);
        run_until(&mut app, "a second of audio", |app| frames(app) > 48_000);
        assert_eq!(peak(&app), 0.0);
    }

    #[derive(Resource, Default)]
    struct Heard(Vec<ClockTick>);

    #[test]
    fn demo_plays_and_clock_ticks_reach_the_game_with_instants() {
        let mut app = app(true);
        // Read ticks the way game systems will: through a MessageReader.
        app.init_resource::<Heard>().add_systems(
            Update,
            |mut ticks: MessageReader<ClockTick>, mut heard: ResMut<Heard>| {
                heard.0.extend(ticks.read().copied());
            },
        );
        run_until(&mut app, "sound", |app| peak(app) > 0.1);
        run_until(&mut app, "two ticks", |app| {
            app.world().resource::<Heard>().0.len() >= 2
        });
        let seen = &app.world().resource::<Heard>().0;
        assert!(
            seen.iter()
                .all(|t| t.module == DEMO_CLOCK_ID && t.at.is_some())
        );
        // 90 BPM at 48 kHz: 32,000 samples, two thirds of a second, apart.
        assert_eq!(seen[1].sample_time - seen[0].sample_time, 32_000);
        // Each prediction comes from a fresh audio-clock reading, which can be off by
        // about a block plus a control-thread tick.
        let gap = seen[1].at.unwrap().duration_since(seen[0].at.unwrap());
        let error_ms = (gap.as_secs_f64() - 2.0 / 3.0).abs() * 1e3;
        assert!(error_ms < 15.0, "tick instants off by {error_ms:.1} ms");
    }

    #[test]
    fn toggling_the_demo_off_returns_to_silence() {
        let mut app = app(true);
        run_until(&mut app, "sound", |app| peak(app) > 0.1);
        set_demo(&mut app.world_mut().resource_mut::<Synth>(), false);
        run_until(&mut app, "silence", |app| peak(app) == 0.0);
    }

    #[test]
    fn missing_audio_is_harmless() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SynthAudioPlugin {
                output: AudioOutput::Off,
                demo: true,
            });
        app.update();
        app.update();
        assert!(!app.world().resource::<Synth>().is_running());
    }
}
