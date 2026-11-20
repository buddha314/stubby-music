//! Game metadata around native scatter: eligibility, stable keys and saved edits.
use super::*;
use bevy::shape::Aabb2d;
use jackdaw_terrain::{
    ScatterMask,
    channel::{ChannelData, ChannelElement},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScatterRule {
    pub id: String,
    /// Increment when sampling inputs or the asset change. Mask edits retain keys.
    pub revision: u32,
    pub region: String,
    pub asset: String,
    pub resolution: u32,
    pub origin: [f32; 2],
    pub size: [f32; 2],
    pub density: f32,
    /// Conservative XZ half extent of this rule's unit-scale asset.
    pub clearance: f32,
    /// Native row-major cell IDs. Zero-valued cells reject placement.
    #[serde(default)]
    pub blocked_cells: Vec<u32>,
    /// The palette native scatter picks from (model paths under `assets/`); empty means
    /// `asset` alone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
    /// Closest two of this rule's placements may sit (native scatter), metres.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub min_spacing: f32,
    /// Uniform scale range (native scatter); default no variance.
    #[serde(default = "unit_scale", skip_serializing_if = "is_unit_scale")]
    pub scale: [f32; 2],
    /// Where this rule may place, relative to what is already there: authored objects
    /// and the placements of the rules before it (add-hybrid-world-placement 2b).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub distances: Vec<DistanceRule>,
}

fn is_zero(v: &f32) -> bool {
    *v == 0.
}
fn unit_scale() -> [f32; 2] {
    [1., 1.]
}
fn is_unit_scale(v: &[f32; 2]) -> bool {
    *v == [1., 1.]
}

/// A distance a rule's placements keep from the nearest anchor: an authored object or an
/// earlier placement whose model's file stem starts with one of `stems`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistanceRule {
    pub stems: Vec<String>,
    /// At least this far from the nearest anchor (no anchor: always met).
    #[serde(default)]
    pub min_m: f32,
    /// At most this far from the nearest anchor (no anchor: never met).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_m: Option<f32>,
    /// Denser nearer the anchor, falling to none at `max_m` (native weight channel): reeds
    /// thick at a burbler's pool, thinning out as the ground dries.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub denser_nearer: bool,
}

impl DistanceRule {
    fn nearest(&self, anchors: &[(String, Vec2)], at: Vec2) -> Option<f32> {
        anchors
            .iter()
            .filter(|(stem, _)| self.stems.iter().any(|s| stem.starts_with(s.as_str())))
            .map(|(_, p)| p.distance(at))
            .min_by(f32::total_cmp)
    }
    /// Whether `at` meets this rule, and its density weight there (0..=1).
    fn weigh(&self, anchors: &[(String, Vec2)], at: Vec2) -> Option<f32> {
        let Some(d) = self.nearest(anchors, at) else {
            return self.max_m.is_none().then_some(1.);
        };
        if d < self.min_m || self.max_m.is_some_and(|max| d > max) {
            return None;
        }
        Some(match (self.denser_nearer, self.max_m) {
            (true, Some(max)) if max > self.min_m => 1. - (d - self.min_m) / (max - self.min_m),
            _ => 1.,
        })
    }
    fn valid(&self) -> bool {
        !self.stems.is_empty()
            && self.stems.iter().all(|s| !s.trim().is_empty())
            && self.min_m.is_finite()
            && self.min_m >= 0.
            && self
                .max_m
                .is_none_or(|max| max.is_finite() && max >= self.min_m)
            && (!self.denser_nearer || self.max_m.is_some())
    }
}

/// A model path's file stem, the name rules and authored objects are matched by.
pub fn model_stem(path: &str) -> &str {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectedArea {
    pub min: [f32; 2],
    pub max: [f32; 2],
}
impl ProtectedArea {
    fn bounds(&self, clearance: f32) -> Aabb2d {
        let min = Vec2::from_array(self.min);
        let max = Vec2::from_array(self.max);
        Aabb2d::new((min + max) * 0.5, (max - min) * 0.5 + clearance)
    }
    fn valid(&self) -> bool {
        let min = Vec2::from_array(self.min);
        let max = Vec2::from_array(self.max);
        min.is_finite() && max.is_finite() && min.cmple(max).all()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementKey {
    pub generator_revision: String,
    pub seed: String,
    pub content_revision: String,
    pub scene: String,
    pub rule: String,
    pub rule_revision: u32,
    pub region: String,
    pub cell: u32,
}

/// Native stored-scatter convention: position, yaw and uniform scale. Convert to
/// Bevy Transform at the runtime boundary; no new transform or math implementation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementPose {
    pub position: [f32; 3],
    pub yaw: f32,
    pub scale: f32,
}
impl PlacementPose {
    pub fn transform(&self) -> Transform {
        Transform::from_translation(Vec3::from_array(self.position))
            .with_rotation(Quat::from_rotation_y(self.yaw))
            .with_scale(Vec3::splat(self.scale))
    }
    fn valid(&self) -> bool {
        Vec3::from_array(self.position).is_finite()
            && self.yaw.is_finite()
            && self.scale.is_finite()
            && self.scale > 0.
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlacementEdit {
    Suppress {
        key: PlacementKey,
    },
    Move {
        key: PlacementKey,
        pose: PlacementPose,
    },
}
impl PlacementEdit {
    fn key(&self) -> &PlacementKey {
        match self {
            Self::Suppress { key } | Self::Move { key, .. } => key,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlacementRecord {
    pub key: PlacementKey,
    pub asset: String,
    pub pose: PlacementPose,
    /// The rule's clearance at this placement's scale: what later rules keep clear of.
    #[serde(default)]
    pub clearance: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResolvedLayout {
    pub placements: Vec<PlacementRecord>,
    /// Conflicting moves are omitted; the original edit stays in WorldConfig.
    pub diagnostics: Vec<String>,
}

impl WorldConfig {
    pub(super) fn validate_placement_inputs(&self) -> Result<()> {
        if self.protected_areas.iter().any(|a| !a.valid()) {
            return Err("protected area needs finite ordered min/max bounds".into());
        }
        let mut rules = BTreeSet::new();
        for rule in &self.scatter {
            if rule.id.trim().is_empty()
                || rule.region.trim().is_empty()
                || rule.asset.trim().is_empty()
                || rule.revision == 0
                || !rules.insert((&rule.id, &rule.region))
                || !rule.clearance.is_finite()
                || rule.clearance < 0.
                || !rule.min_spacing.is_finite()
                || rule.min_spacing < 0.
                || !(rule.scale[0].is_finite()
                    && rule.scale[0] > 0.
                    && rule.scale[1] >= rule.scale[0])
                || rule.variants.iter().any(|v| v.trim().is_empty())
                || !rule.distances.iter().all(DistanceRule::valid)
            {
                return Err("invalid or duplicate scatter rule identity/clearance".into());
            }
            // Bound input size before allocating any native channel/heightmap.
            if !(2..=512).contains(&rule.resolution) {
                return Err("scatter resolution must be 2..=512".into());
            }
            let size = Vec2::from_array(rule.size);
            let cell = size / (rule.resolution - 1) as f32;
            let area = cell.element_product();
            if !size.is_finite()
                || size.min_element() <= 0.
                || !Vec2::from_array(rule.origin).is_finite()
                || !area.is_finite()
                || area <= 0.
                || !rule.density.is_finite()
                || rule.density < 0.
                || rule.density * area > 1.
                || rule
                    .blocked_cells
                    .iter()
                    .any(|i| *i >= rule.resolution.pow(2))
            {
                return Err("invalid scatter domain, density or blocked cell".into());
            }
        }
        if self
            .authored_footprints
            .values()
            .any(|metres| !metres.is_finite() || *metres < 0.)
        {
            return Err("authored footprints need finite, non-negative metres".into());
        }
        let mut edits = BTreeSet::new();
        for edit in &self.edits {
            if !edits.insert(edit.key()) {
                return Err("duplicate edits for the same placement key".into());
            }
            if let PlacementEdit::Move { pose, .. } = edit
                && !pose.valid()
            {
                return Err("placement override needs finite pose and positive scale".into());
            }
        }
        Ok(())
    }

    /// Clearance around the authored objects of a saved scene, from their world positions
    /// and each asset's configured footprint, for [`Self::resolve_layout`] (which grows it
    /// again by each candidate's own clearance). It reads positions only: authored objects
    /// keep their IDs and transforms, and generated content never moves them.
    pub fn authored_clearance<'a>(
        &self,
        authored: impl IntoIterator<Item = (&'a str, Vec3)>,
    ) -> Vec<ProtectedArea> {
        authored
            .into_iter()
            .filter(|(_, at)| at.is_finite())
            .map(|(asset, at)| {
                let reach = self.authored_footprints.get(asset).copied().unwrap_or(0.);
                let center = at.xz();
                ProtectedArea {
                    min: (center - reach).to_array(),
                    max: (center + reach).to_array(),
                }
            })
            .collect()
    }

    /// Resolve data only. `authored_clearance` comes from the caller's saved scene
    /// world bounds; this function never edits its entities or resource ledger.
    pub fn resolve_layout(&self, authored_clearance: &[ProtectedArea]) -> Result<ResolvedLayout> {
        self.resolve(authored_clearance, &[])
    }

    /// The seeded opening around a saved scene: `authored` is each authored object's
    /// model stem and world position. They are kept clear by their footprints, and rules'
    /// `distances` measure from them as from earlier placements. Never moves them.
    pub fn resolve_opening(&self, authored: &[(String, Vec3)]) -> Result<ResolvedLayout> {
        let clearance = self.authored_clearance(authored.iter().map(|(s, p)| (s.as_str(), *p)));
        self.resolve(&clearance, authored)
    }

    /// A rule's own sampling seed: the world seed mixed with the rule's ID (FNV-1a), so
    /// rules sharing a grid do not draw the same numbers in the same cells.
    pub fn rule_seed(&self, rule: &ScatterRule) -> Result<u64> {
        Ok(rule
            .id
            .bytes()
            .fold(self.seed_value()? ^ 0xcbf29ce484222325, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
            }))
    }

    fn resolve(
        &self,
        authored_clearance: &[ProtectedArea],
        authored: &[(String, Vec3)],
    ) -> Result<ResolvedLayout> {
        self.validate()?;
        if authored_clearance.iter().any(|a| !a.valid()) {
            return Err("invalid authored clearance bounds".into());
        }
        let edits: BTreeMap<_, _> = self.edits.iter().map(|e| (e.key(), e)).collect();
        let mut seen = BTreeSet::new();
        let mut output = ResolvedLayout::default();
        // Anchors for `distances`: authored objects, then each rule's placements in turn.
        let mut anchors: Vec<(String, Vec2)> = authored
            .iter()
            .filter(|(_, p)| p.is_finite())
            .map(|(s, p)| (s.clone(), p.xz()))
            .chain(
                self.landmarks
                    .iter()
                    .map(|(name, at)| (name.clone(), Vec2::from_array(*at))),
            )
            .collect();
        // In configuration order: a rule reads the placements of the rules before it.
        for rule in &self.scatter {
            let map = ground_heightmap(rule.resolution, rule.size.into(), rule.origin.into())?;
            let cells = (rule.resolution * rule.resolution) as usize;
            let mut eligible = ChannelData::new("eligible", ChannelElement::U8, rule.resolution);
            let mut weight = ChannelData::new("weight", ChannelElement::U8, rule.resolution);
            let step = map.cell_size();
            for cell in 0..cells {
                let grid = Vec2::new(
                    (cell as u32 % rule.resolution) as f32,
                    (cell as u32 / rule.resolution) as f32,
                );
                let at = map.origin + grid * step;
                let w = rule
                    .distances
                    .iter()
                    .try_fold(1f32, |w, d| d.weigh(&anchors, at).map(|x| w * x));
                if let Some(w) = w.filter(|w| *w > 0.) {
                    eligible.values[cell] = 1;
                    weight.values[cell] = (w * 1000.).round().clamp(1., 1000.) as u16;
                }
            }
            for cell in &rule.blocked_cells {
                eligible.values[*cell as usize] = 0;
            }
            let palette: Vec<&str> = if rule.variants.is_empty() {
                vec![rule.asset.as_str()]
            } else {
                rule.variants.iter().map(String::as_str).collect()
            };
            let weighted = rule.distances.iter().any(|d| d.denser_nearer);
            let candidates = jackdaw_terrain::scatter(
                &map,
                &[eligible, weight],
                &ScatterParams {
                    seed: self.rule_seed(rule)?,
                    density: rule.density,
                    mask: Some(ScatterMask {
                        channel: 0,
                        accept: vec![1],
                    }),
                    weight_channel: weighted.then_some(1),
                    min_spacing: rule.min_spacing,
                    scale_min: rule.scale[0],
                    scale_max: rule.scale[1],
                    asset_count: palette.len(),
                    ..Default::default()
                },
            );
            let mut placed: Vec<(String, Vec2)> = Vec::new();
            for candidate in candidates {
                let key = PlacementKey {
                    generator_revision: self.generator_revision.clone(),
                    seed: self.seed_value()?.to_string(),
                    content_revision: self.content_revision.clone(),
                    scene: self.scene.clone(),
                    rule: rule.id.clone(),
                    rule_revision: rule.revision,
                    region: rule.region.clone(),
                    cell: candidate.cell,
                };
                seen.insert(key.clone());
                let mut pose = PlacementPose {
                    position: candidate.position.to_array(),
                    yaw: candidate.yaw,
                    scale: candidate.scale,
                };
                match edits.get(&key) {
                    Some(PlacementEdit::Suppress { .. }) => continue,
                    Some(PlacementEdit::Move { pose: edited, .. }) => pose = edited.clone(),
                    None => (),
                }
                let point = Vec3::from_array(pose.position).xz();
                let in_bounds =
                    point.cmpge(map.origin).all() && point.cmplt(map.origin + map.size).all();
                // Test the final footprint as well as the source-cell mask.
                // Out-of-region edits never reach integer grid arithmetic.
                let masked = !in_bounds
                    || rule.blocked_cells.iter().any(|cell| {
                        let grid = Vec2::new(
                            (cell % rule.resolution) as f32,
                            (cell / rule.resolution) as f32,
                        );
                        let min = map.origin + grid * map.cell_size();
                        let max = min + map.cell_size();
                        ProtectedArea {
                            min: min.to_array(),
                            max: max.to_array(),
                        }
                        .bounds(rule.clearance * pose.scale)
                        .closest_point(point)
                            == point
                    });
                let protected = self
                    .protected_areas
                    .iter()
                    .chain(authored_clearance)
                    .any(|a| a.bounds(rule.clearance * pose.scale).closest_point(point) == point);
                // The exact position, not just its cell, meets the distances, and keeps
                // clear of every earlier placement (both clearances).
                let reach = rule.clearance * pose.scale;
                let crowded = !rule
                    .distances
                    .iter()
                    .all(|d| d.weigh(&anchors, point).is_some())
                    || output.placements.iter().any(|p| {
                        Vec3::from_array(p.pose.position).xz().distance(point) < p.clearance + reach
                    });
                if masked || protected || crowded {
                    if edits.contains_key(&key) {
                        output
                            .diagnostics
                            .push(format!("invalid placement override: {key:?}"));
                    }
                    continue;
                }
                let asset = palette[candidate.asset_index.min(palette.len() - 1)].to_owned();
                placed.push((model_stem(&asset).to_owned(), point));
                output.placements.push(PlacementRecord {
                    key,
                    asset,
                    pose,
                    clearance: reach,
                });
            }
            anchors.extend(placed);
        }
        for key in edits.keys().filter(|key| !seen.contains(**key)) {
            output
                .diagnostics
                .push(format!("stale placement edit retained: {key:?}"));
        }
        Ok(output)
    }
}
