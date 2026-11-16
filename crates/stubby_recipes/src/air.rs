//! Air cleaners: foliage that cleans the air within a radius (clean-air-by-foliage).
//!
//! Each row names the placed assets it applies to (by asset ID prefix: `scrub_oak`
//! matches `scrub_oak_0`), how far they clean, how strongly at full life, and which part
//! of the asset's life its cleaning follows. Light quality at a point is computed from
//! the cleaners around it by the world; these rows are only the numbers.
use crate::rules::{RulesError, valid_id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const AIR: &str = "air.json";

/// What part of an asset's life its cleaning follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AirFollows {
    /// How grown it is (a tree from dormant to full bloom).
    Growth,
    /// How green it is: the better of the water and light around it (reeds).
    WaterOrLight,
    /// The light around it (desert brush blooms where the air is already clean).
    Light,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AirCleanerDefinition {
    pub id: String,
    /// Placed assets whose asset ID is this or starts with `<this>_` clean the air.
    pub asset: String,
    /// How far it cleans, in metres. Its cleaning falls smoothly to 0 at this distance.
    pub radius_m: f32,
    /// How strongly it cleans at its centre at full life, 0..1.
    pub strength: f32,
    pub follows: AirFollows,
    /// Harvested assets (cut reeds) stop cleaning until they regrow.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stops_when_harvested: bool,
}

/// The largest cleaning radius a row may have: the basin is ~480 m across its ring.
pub const MAX_RADIUS_M: f32 = 200.;

pub fn validate_air(rows: &[AirCleanerDefinition]) -> Vec<RulesError> {
    let mut errors = Vec::new();
    let mut assets = BTreeSet::new();
    let error = |id: &str, field: &str, message: String| RulesError {
        file: AIR.into(),
        id: id.into(),
        field: field.into(),
        message,
    };
    for row in rows {
        if !valid_id(&row.id) {
            errors.push(error(&row.id, "id", "IDs are snake_case [a-z0-9_]".into()));
        }
        if row.asset.is_empty() || !assets.insert(row.asset.clone()) {
            errors.push(error(
                &row.id,
                "asset",
                format!("`{}` is empty or already has a cleaner", row.asset),
            ));
        }
        if !(row.radius_m > 0. && row.radius_m <= MAX_RADIUS_M) {
            errors.push(error(
                &row.id,
                "radius_m",
                format!("{} m is outside 0..{MAX_RADIUS_M}", row.radius_m),
            ));
        }
        if !(0. ..=1.).contains(&row.strength) {
            errors.push(error(
                &row.id,
                "strength",
                format!("{} is outside 0..1", row.strength),
            ));
        }
    }
    errors
}

impl AirCleanerDefinition {
    /// Whether a placed asset's ID is this row's asset.
    pub fn applies_to(&self, asset: &str) -> bool {
        asset == self.asset
            || asset
                .strip_prefix(&self.asset)
                .is_some_and(|rest| rest.starts_with('_'))
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::{Rules, shipped_files};
    #[test]
    fn air_rows_load_match_assets_and_report_bad_numbers() {
        let rules = Rules::load(&shipped_files()).unwrap();
        let oak = rules.air.iter().find(|a| a.asset == "scrub_oak").unwrap();
        assert!(oak.applies_to("scrub_oak_3") && oak.applies_to("scrub_oak"));
        assert!(!oak.applies_to("scrub_oaken_0"));
        assert!(rules.ids().contains("scrub_oak_air"));
        let bad = r#"{"format":1,"air":[{"id":"x_air","asset":"x","radius_m":0.0,"strength":2.0,"follows":"growth"}]}"#;
        let mut files = shipped_files();
        files.air = bad;
        let errors = Rules::load(&files).unwrap_err().0;
        let fields: Vec<_> = errors.iter().map(|e| e.field.as_str()).collect();
        assert!(
            fields.contains(&"radius_m") && fields.contains(&"strength"),
            "{errors:?}"
        );
    }
}
