//! The art-authored lighting keys (`assets/lighting/lighting_keys.json`) drive the fog
//! and the lights' colours through the day and from bleak to bloomed. Brightness (lux,
//! exposure) stays physical and is configured in `assets/lighting/sky.json`
//! (`brightness`), so the night can be lightened there (user direction, 2026-11-16).
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub sun_color: [f32; 3],
    pub sun_elevation_deg: f32,
    pub ambient_color: [f32; 3],
    pub fog_color: [f32; 3],
    pub fog_density: f32,
}
#[derive(Deserialize)]
struct Mood {
    keys: HashMap<String, Key>,
}
#[derive(Deserialize)]
struct KeysFile {
    moods: HashMap<String, Mood>,
}
/// The keys, by mood (`bleak`, `bloomed`) and time (`dawn`, `day`, `dusk`, `night`).
#[derive(Resource)]
pub struct LightingKeys {
    bleak: [Key; 4],
    bloomed: [Key; 4],
}
const TIMES: [&str; 4] = ["dawn", "day", "dusk", "night"];
impl LightingKeys {
    pub fn shipped() -> Self {
        let file: KeysFile =
            serde_json::from_str(include_str!("../../../assets/lighting/lighting_keys.json"))
                .expect("lighting keys");
        let mood = |name: &str| TIMES.map(|time| file.moods[name].keys[time]);
        Self {
            bleak: mood("bleak"),
            bloomed: mood("bloomed"),
        }
    }
    /// The key for a sun direction and light quality (0 bleak, 1 bloomed).
    pub fn sample(&self, sun: Vec3, quality: f32) -> Key {
        let afternoon = sun.x < 0.;
        let elevation = sun.y.clamp(-1., 1.).asin().to_degrees();
        let bleak = along_the_day(&self.bleak, elevation, afternoon);
        let bloomed = along_the_day(&self.bloomed, elevation, afternoon);
        mix(&bleak, &bloomed, quality.clamp(0., 1.))
    }
}

/// Where night ends: the sun this far below the horizon is full night (the stars are out).
const NIGHT_ELEVATION_DEG: f32 = -8.;

/// Night -> dawn -> day in the morning, day -> dusk -> night in the afternoon, placed by
/// each key's own sun elevation.
fn along_the_day(keys: &[Key; 4], elevation: f32, afternoon: bool) -> Key {
    let [dawn, day, dusk, night] = keys;
    let edge = if afternoon { dusk } else { dawn };
    let between = |a: f32, b: f32| ((elevation - a) / (b - a)).clamp(0., 1.);
    if elevation >= edge.sun_elevation_deg {
        mix(
            edge,
            day,
            smooth(between(edge.sun_elevation_deg, day.sun_elevation_deg)),
        )
    } else {
        mix(
            night,
            edge,
            smooth(between(NIGHT_ELEVATION_DEG, edge.sun_elevation_deg)),
        )
    }
}
fn smooth(t: f32) -> f32 {
    t * t * (3. - 2. * t)
}
fn mix(a: &Key, b: &Key, t: f32) -> Key {
    let rgb = |a: [f32; 3], b: [f32; 3]| (Vec3::from(a).lerp(Vec3::from(b), t)).to_array();
    Key {
        sun_color: rgb(a.sun_color, b.sun_color),
        sun_elevation_deg: a.sun_elevation_deg + (b.sun_elevation_deg - a.sun_elevation_deg) * t,
        ambient_color: rgb(a.ambient_color, b.ambient_color),
        fog_color: rgb(a.fog_color, b.fog_color),
        fog_density: a.fog_density + (b.fog_density - a.fog_density) * t,
    }
}

/// Light quality where the camera stands: 0 bleak, 1 bloomed. The light-quality field
/// (10 m cells, driven by plant density) will set it; until then the world is bleak.
#[derive(Resource, Default, Clone, Copy, Debug, Reflect)]
#[reflect(Resource)]
pub struct CameraLightQuality(pub f32);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{seconds_at_solar_hour, sun_direction};
    #[test]
    fn keys_follow_the_day_and_the_mood() {
        let keys = LightingKeys::shipped();
        let at = |hour: f32, quality: f32| {
            keys.sample(sun_direction(seconds_at_solar_hour(hour)), quality)
        };
        // Noon and midnight are their keys; dawn and dusk use their own.
        assert_eq!(at(12., 0.).fog_color, keys.bleak[1].fog_color);
        assert_eq!(at(0., 0.).fog_color, keys.bleak[3].fog_color);
        assert_eq!(at(0., 1.).ambient_color, keys.bloomed[3].ambient_color);
        let morning = at(6.4, 0.);
        let evening = at(17.6, 0.);
        assert!(
            morning.sun_color != evening.sun_color,
            "dawn and dusk differ"
        );
        // Continuous through the whole day: no step of a few seconds jumps.
        let fog = |hour: f32| Vec3::from(at(hour, 0.5).fog_color);
        let steps = 24_000;
        for i in 0..steps {
            let hour = i as f32 * 24. / steps as f32;
            assert!(
                fog(hour).distance(fog(hour + 24. / steps as f32)) < 0.01,
                "jump at {hour}"
            );
        }
    }
}
