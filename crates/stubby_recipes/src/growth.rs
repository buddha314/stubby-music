//! How a growing mine (a scrub oak) grows with water (sync-stream-growth-and-dismantling,
//! the Godot game's `growth.json`): a few bloom levels, each reached by water of at least
//! some clarity. Mud (clarity 0) reaches only the lowest; cleaner water reaches higher.
//! While watered, growth rises toward the cap its water allows; without water it eases
//! back toward dormant, more slowly. Growth is simulation time, not the music's clock.
use crate::rules::RulesError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A growing plant's bloom levels, lowest first (the growth stages of
/// `assets/flora/scrub_oak/asset.json`). A plant at a level is at every level below it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum GrowthLevel {
    Budding,
    Leafing,
    Leafed,
    FullBloom,
}

/// One bloom level: the growth value it starts at and the water that can reach it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GrowthStage {
    pub level: GrowthLevel,
    /// Growth (0 dormant .. 1 full bloom) at which the plant is at this level.
    pub growth: f32,
    /// The least clear water (0 mud .. 1 clear) that grows the plant this far.
    pub min_clarity: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GrowthDefinition {
    /// Lowest first.
    pub levels: Vec<GrowthStage>,
    /// A flowing stream this close (horizontally, metres) waters the plant.
    pub water_reach_m: f32,
    /// Growth gained per minute of simulation time while watered.
    pub rise_per_minute: f32,
    /// Growth lost per minute of simulation time without water.
    pub ease_per_minute: f32,
}

impl GrowthDefinition {
    /// The most growth water of `clarity` allows; no water (`None`) allows none.
    pub fn cap(&self, clarity: Option<f32>) -> f32 {
        let Some(clarity) = clarity else { return 0. };
        self.levels
            .iter()
            .filter(|stage| clarity + 1e-6 >= stage.min_clarity)
            .map(|stage| stage.growth)
            .fold(0., f32::max)
    }

    /// The highest level `growth` has reached; `None` while dormant.
    pub fn level(&self, growth: f32) -> Option<GrowthLevel> {
        self.levels
            .iter()
            .filter(|stage| growth + 1e-6 >= stage.growth)
            .map(|stage| stage.level)
            .max()
    }

    /// Growth after `seconds` of simulation time with water of `clarity` (or none).
    pub fn step(&self, growth: f32, clarity: Option<f32>, seconds: f32) -> f32 {
        let target = self.cap(clarity);
        if growth < target {
            (growth + self.rise_per_minute / 60. * seconds).min(target)
        } else {
            (growth - self.ease_per_minute / 60. * seconds).max(target)
        }
    }

    pub(crate) fn validate(&self, file: &str, id: &str) -> Vec<RulesError> {
        let err = |message: String| RulesError {
            file: file.into(),
            id: id.into(),
            field: "growth".into(),
            message,
        };
        let mut errors = Vec::new();
        if self.levels.is_empty() {
            errors.push(err("needs at least one level".into()));
        }
        for pair in self.levels.windows(2) {
            if pair[1].level <= pair[0].level || pair[1].growth <= pair[0].growth {
                errors.push(err(format!(
                    "levels go lowest first: {:?} after {:?}",
                    pair[1].level, pair[0].level
                )));
            }
            if pair[1].min_clarity < pair[0].min_clarity {
                errors.push(err(format!(
                    "{:?} needs less clear water than the level below it",
                    pair[1].level
                )));
            }
        }
        for stage in &self.levels {
            if !(stage.growth > 0. && stage.growth <= 1.) {
                errors.push(err(format!("{:?}: growth is in (0, 1]", stage.level)));
            }
            if !(0. ..=1.).contains(&stage.min_clarity) {
                errors.push(err(format!("{:?}: min_clarity is in [0, 1]", stage.level)));
            }
        }
        for (name, value) in [
            ("water_reach_m", self.water_reach_m),
            ("rise_per_minute", self.rise_per_minute),
            ("ease_per_minute", self.ease_per_minute),
        ] {
            if !(value > 0. && value.is_finite()) {
                errors.push(err(format!("{name} must be positive")));
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Godot game's scrub oak (`growth.json`, 2026-11-13).
    fn oak() -> GrowthDefinition {
        let stage = |level, growth, min_clarity| GrowthStage {
            level,
            growth,
            min_clarity,
        };
        GrowthDefinition {
            levels: vec![
                stage(GrowthLevel::Budding, 0.3, 0.),
                stage(GrowthLevel::Leafed, 0.7, 0.5),
                stage(GrowthLevel::FullBloom, 1., 1.),
            ],
            water_reach_m: 3.,
            rise_per_minute: 0.3,
            ease_per_minute: 0.05,
        }
    }

    #[test]
    fn mud_grows_an_oak_to_budding_and_no_further() {
        let oak = oak();
        let mut growth = 0.;
        for _ in 0..600 {
            growth = oak.step(growth, Some(0.), 1.);
        }
        assert_eq!(growth, 0.3);
        assert_eq!(oak.level(growth), Some(GrowthLevel::Budding));
        assert_eq!(oak.cap(Some(0.5)), 0.7);
        assert_eq!(oak.cap(Some(1.)), 1.);
    }

    #[test]
    fn without_water_it_eases_back_slower_than_it_grew() {
        let oak = oak();
        // A minute of mud: 0.3. A minute dry: 0.25, still above dormant.
        let grown = oak.step(0., Some(0.), 60.);
        let dried = oak.step(grown, None, 60.);
        assert!((dried - 0.25).abs() < 1e-5, "{dried}");
        assert_eq!(oak.level(dried), None);
        // Many small steps land where one big one does (frame-rate independent).
        let mut small = 0.;
        for _ in 0..3600 {
            small = oak.step(small, Some(0.), 1. / 60.);
        }
        assert!((small - grown).abs() < 1e-4);
    }

    #[test]
    fn levels_must_rise() {
        let mut bad = oak();
        bad.levels.swap(0, 1);
        bad.rise_per_minute = 0.;
        let errors = bad.validate("mines.json", "scrub_oak");
        assert!(errors.iter().any(|e| e.message.contains("lowest first")));
        assert!(errors.iter().any(|e| e.message.contains("rise_per_minute")));
    }
}
