//! A plucked voice: an oscillator through an envelope and VCA, opened by a gate. The one
//! patch every short synthesized sound uses (bug clicks, collection cues), built from the
//! engine's existing modules; no DSP here.
use serde::Deserialize;
use stubby_audio::{Command, ModuleId, ModuleKind, PortRef};

#[derive(Clone, Copy, Deserialize)]
/// A voice's pitch, envelope, level and how far it carries.
pub struct WaveformSound {
    pub freq_hz: f32,
    pub attack_seconds: f32,
    pub decay_seconds: f32,
    pub gain: f32,
    pub hearing_radius_metres: f32,
}
/// One oscillator, envelope and VCA, triggered by an existing gate source, sent `to` an
/// output: `PortRef::output()` for a sound in the world (a bug's click),
/// `PortRef::ui_output()` for an interface sound (a collection confirmation).
pub fn voice_commands(
    oscillator_kind: ModuleKind,
    ids: [ModuleId; 3],
    gate: PortRef,
    gain: f32,
    sound: WaveformSound,
    to: PortRef,
) -> [Command; 7] {
    let [oscillator, envelope, vca] = ids;
    [
        Command::AddModule {
            id: oscillator,
            kind: oscillator_kind,
        },
        Command::AddModule {
            id: envelope,
            kind: ModuleKind::Envelope {
                attack_s: sound.attack_seconds,
                decay_s: sound.decay_seconds,
            },
        },
        Command::AddModule {
            id: vca,
            kind: ModuleKind::Vca { gain },
        },
        Command::Connect {
            from: gate,
            to: PortRef::new(envelope, "gate"),
        },
        Command::Connect {
            from: PortRef::new(envelope, "out"),
            to: PortRef::new(vca, "cv"),
        },
        Command::Connect {
            from: PortRef::new(oscillator, "out"),
            to: PortRef::new(vca, "in"),
        },
        Command::Connect {
            from: PortRef::new(vca, "out"),
            to,
        },
    ]
}
