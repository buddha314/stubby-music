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

    /// Resolve data only. `authored_clearance` comes from the caller's saved scene
    /// world bounds; this function never edits its entities or resource ledger.
    pub fn resolve_layout(&self, authored_clearance: &[ProtectedArea]) -> Result<ResolvedLayout> {
        self.validate()?;
        if authored_clearance.iter().any(|a| !a.valid()) {
            return Err("invalid authored clearance bounds".into());
        }
        let edits: BTreeMap<_, _> = self.edits.iter().map(|e| (e.key(), e)).collect();
        let mut seen = BTreeSet::new();
        let mut output = ResolvedLayout::default();
        // Stable rule order without coupling independent rules' RNG streams.
        let mut rules: Vec<_> = self.scatter.iter().collect();
        rules.sort_by_key(|r| (&r.id, &r.region));
        for rule in rules {
            let map = ground_heightmap(rule.resolution, rule.size.into(), rule.origin.into())?;
            let mut channel = ChannelData::new("eligible", ChannelElement::U8, rule.resolution);
            channel.values.fill(1);
            for cell in &rule.blocked_cells {
                channel.values[*cell as usize] = 0;
            }
            let candidates = jackdaw_terrain::scatter(
                &map,
                &[channel],
                &ScatterParams {
                    seed: self.seed_value()?,
                    density: rule.density,
                    mask: Some(ScatterMask {
                        channel: 0,
                        accept: vec![1],
                    }),
                    min_spacing: 0.,
                    asset_count: 1,
                    ..Default::default()
                },
            );
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
                if masked || protected {
                    if edits.contains_key(&key) {
                        output
                            .diagnostics
                            .push(format!("invalid placement override: {key:?}"));
                    }
                    continue;
                }
                output.placements.push(PlacementRecord {
                    key,
                    asset: rule.asset.clone(),
                    pose,
                });
            }
        }
        for key in edits.keys().filter(|key| !seen.contains(**key)) {
            output
                .diagnostics
                .push(format!("stale placement edit retained: {key:?}"));
        }
        Ok(output)
    }
}
