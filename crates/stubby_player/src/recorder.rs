//! The field recorder's record light and level meter (add-field-recorder 2.2,
//! sync-recorder-feedback, after the Godot game's design D4 and D5).
//!
//! The record light answers a capture: each successful sound harvest sends
//! [`RecorderCaptured`] (the game's glue does, from the harvest), and `Light_rec` rises in
//! 0.05 s and fades over 0.4 s. It is an envelope, not a blink on frame time.
//!
//! The meter answers real sound: its eight `Level_n` bars show the engine's world bus
//! (`bevy_stubby_audio::WorldLevel`: the world and the music, never UI sounds), bar `n`
//! lit at or above -48 + 6n dBFS, holding peaks and falling at 24 dB/s. The silent
//! opening leaves them dark. Bars swap between the model's own lit and dim screen
//! materials (`Level_0` and `Level_7` as authored); the light scales its own material
//! between the profile's rest and peak strengths, as rack lights do.
use bevy::prelude::*;
use bevy_stubby_audio::WorldLevel;

use crate::hand::{HandItem, Held};

/// Seconds for the record light to rise.
pub const ATTACK_SECONDS: f32 = 0.05;
/// Seconds for it to fade back to rest.
pub const DECAY_SECONDS: f32 = 0.4;
/// Bar 0 lights at this level; each bar after it 6 dB higher.
pub const FLOOR_DB: f32 = -48.;
pub const STEP_DB: f32 = 6.;
/// How fast a held peak falls.
pub const FALL_DB_PER_SECOND: f32 = 24.;

#[derive(serde::Deserialize)]
struct Profile {
    lights: Vec<LightProfile>,
    screen: ScreenProfile,
}
#[derive(serde::Deserialize)]
struct LightProfile {
    node: String,
    rest_strength: f32,
    peak_strength: f32,
}
#[derive(serde::Deserialize)]
struct ScreenProfile {
    bars: usize,
}
fn profile() -> Profile {
    serde_json::from_str(include_str!(
        "../../../assets/equipment/field_recorder/asset.json"
    ))
    .expect("the field recorder's profile")
}

/// A sound was captured (a successful sound harvest). The record light answers it.
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct RecorderCaptured;

/// The record light's envelope at `t` seconds since a capture: 0 → 1 → 0.
pub fn envelope(t: f32) -> f32 {
    if t < ATTACK_SECONDS {
        t / ATTACK_SECONDS
    } else {
        (1. - (t - ATTACK_SECONDS) / DECAY_SECONDS).max(0.)
    }
}

/// The meter's shown level after `seconds`: the input when it is louder, else the held
/// peak falling at 24 dB/s.
pub fn meter_step(shown_db: f32, input_db: f32, seconds: f32) -> f32 {
    input_db.max(shown_db - FALL_DB_PER_SECOND * seconds)
}

/// How many of `bars` bars a level lights.
pub fn lit_bars(db: f32, bars: usize) -> usize {
    (0..bars)
        .filter(|&n| db >= FLOOR_DB + STEP_DB * n as f32)
        .count()
}

#[derive(Resource)]
struct RecorderState {
    /// Seconds since the last capture, while the light is still fading.
    since_capture: Option<f32>,
    shown_db: f32,
    rest: f32,
    peak: f32,
    bars: usize,
}

/// The recorder model's parts, found once its scene has spawned.
#[derive(Component)]
struct RecorderParts {
    /// Every mesh under `Light_rec`, with its own material and the emissive it was
    /// exported with (at peak strength).
    light: Vec<(Handle<StandardMaterial>, LinearRgba)>,
    /// Each bar's mesh entities, bar 0 first.
    bars: Vec<Vec<Entity>>,
    lit: Handle<StandardMaterial>,
    dim: Handle<StandardMaterial>,
    shown_bars: Option<usize>,
    shown_light: Option<f32>,
}

pub(crate) fn add(app: &mut App) {
    let profile = profile();
    let light = profile
        .lights
        .iter()
        .find(|light| light.node == "Light_rec")
        .expect("the recorder's profile names Light_rec");
    app.add_message::<RecorderCaptured>()
        .init_resource::<WorldLevel>()
        .insert_resource(RecorderState {
            since_capture: None,
            shown_db: f32::NEG_INFINITY,
            rest: light.rest_strength,
            peak: light.peak_strength,
            bars: profile.screen.bars,
        })
        .add_systems(Update, (find_parts, (capture, show).chain()));
}

#[allow(clippy::type_complexity)] // The held recorder, its descendants, names and materials.
fn find_parts(
    mut commands: Commands,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    held: Query<(Entity, &Held), Without<RecorderParts>>,
    children: Query<&Children>,
    names: Query<&Name>,
    meshes: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    for (root, item) in &held {
        if item.0 != HandItem::FieldRecorder {
            continue;
        }
        let named = |name: &str| {
            children
                .iter_descendants(root)
                .find(|&e| names.get(e).is_ok_and(|n| n.as_str() == name))
        };
        let meshes_under = |entity: Entity| -> Vec<Entity> {
            std::iter::once(entity)
                .chain(children.iter_descendants(entity))
                .filter(|&e| meshes.contains(e))
                .collect()
        };
        let Some(light) = named("Light_rec") else {
            continue; // the scene has not spawned yet
        };
        let bars: Vec<Vec<Entity>> = (0..8)
            .map_while(|n| named(&format!("Level_{n}")).map(meshes_under))
            .collect();
        let material = |bar: Option<&Vec<Entity>>| {
            bar.and_then(|b| b.first())
                .and_then(|&e| meshes.get(e).ok())
                .map(|m| m.0.clone())
        };
        // As authored, the first bars are lit and the last dim.
        let (Some(lit), Some(dim)) = (material(bars.first()), material(bars.last())) else {
            continue;
        };
        let mut own = Vec::new();
        for entity in meshes_under(light) {
            let Some(copy) = meshes
                .get(entity)
                .ok()
                .and_then(|m| materials.get(&m.0))
                .cloned()
            else {
                continue;
            };
            let base = copy.emissive;
            let handle = materials.add(copy);
            commands
                .entity(entity)
                .insert(MeshMaterial3d(handle.clone()));
            own.push((handle, base));
        }
        commands.entity(root).insert(RecorderParts {
            light: own,
            bars,
            lit,
            dim,
            shown_bars: None,
            shown_light: None,
        });
    }
}

fn capture(
    mut captured: MessageReader<RecorderCaptured>,
    time: Res<Time<Real>>,
    level: Res<WorldLevel>,
    mut state: ResMut<RecorderState>,
) {
    let dt = time.delta_secs();
    if captured.read().last().is_some() {
        // Rise from wherever the light is now, so repeated captures never flicker.
        let now = state.since_capture.map_or(0., envelope);
        state.since_capture = Some(now * ATTACK_SECONDS);
    } else if let Some(t) = state.since_capture.as_mut() {
        *t += dt;
        if *t > ATTACK_SECONDS + DECAY_SECONDS {
            state.since_capture = None;
        }
    }
    state.shown_db = meter_step(state.shown_db, level.peak_db, dt);
}

fn show(
    state: Res<RecorderState>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut recorders: Query<&mut RecorderParts>,
    mut bars: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    let light = state.since_capture.map_or(0., envelope);
    let strength = state.rest + (state.peak - state.rest) * light;
    let lit = lit_bars(state.shown_db, state.bars);
    for mut parts in &mut recorders {
        if parts.shown_light != Some(strength) {
            for (handle, base) in &parts.light {
                if let Some(mut m) = materials.get_mut(handle) {
                    m.emissive = *base * (strength / state.peak.max(f32::EPSILON));
                }
            }
            parts.shown_light = Some(strength);
        }
        if parts.shown_bars != Some(lit) {
            for (n, meshes) in parts.bars.iter().enumerate() {
                let want = if n < lit { &parts.lit } else { &parts.dim };
                for &entity in meshes {
                    if let Ok(mut material) = bars.get_mut(entity)
                        && material.0 != *want
                    {
                        material.0 = want.clone();
                    }
                }
            }
            parts.shown_bars = Some(lit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_record_light_rises_fast_and_fades_slowly() {
        assert_eq!(envelope(0.), 0.);
        assert_eq!(envelope(ATTACK_SECONDS), 1.);
        assert!((envelope(ATTACK_SECONDS + 0.2) - 0.5).abs() < 1e-5);
        assert_eq!(envelope(ATTACK_SECONDS + DECAY_SECONDS), 0.);
    }

    #[test]
    fn the_meter_lights_a_bar_per_6_db_and_falls_at_24_db_a_second() {
        assert_eq!(
            lit_bars(f32::NEG_INFINITY, 8),
            0,
            "the silent opening is dark"
        );
        assert_eq!(lit_bars(-48., 8), 1);
        assert_eq!(lit_bars(-45., 8), 1, "a bug's click 2 m away: one bar");
        assert_eq!(lit_bars(-6., 8), 8);
        // A peak holds, then falls: half a second later it is 12 dB lower.
        let after = meter_step(-6., f32::NEG_INFINITY, 0.5);
        assert_eq!(after, -18.);
        assert_eq!(
            meter_step(after, -3., 0.5),
            -3.,
            "a louder sound jumps straight up"
        );
    }

    #[test]
    fn the_profile_names_the_light_and_eight_bars() {
        let profile = profile();
        assert_eq!(profile.screen.bars, 8);
        assert!(profile.lights.iter().any(|l| l.node == "Light_rec"));
    }
}
