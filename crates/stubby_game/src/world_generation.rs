//! Shared world inputs and the adapter to Jackdaw's existing scatter sampler.
//! This does not spawn resources or replace the authored opening.
use bevy::prelude::*;
use jackdaw_terrain::{Heightmap, Placement, ScatterParams};
mod placements;
pub use placements::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Clone, Debug, PartialEq, Resource, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    pub schema_version: u32,
    pub generator_revision: String,
    /// Decimal text avoids losing u64 precision in JSON consumers.
    pub seed: String,
    pub scene: String,
    /// Caller-supplied content reference; not a computed content checksum.
    pub content_revision: String,
    #[serde(default)]
    pub scatter: Vec<ScatterRule>,
    #[serde(default)]
    pub protected_areas: Vec<ProtectedArea>,
    #[serde(default)]
    pub edits: Vec<PlacementEdit>,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            generator_revision: "mudflat-v1".into(),
            seed: "42".into(),
            scene: "scene.bsn".into(),
            content_revision: "development".into(),
            scatter: Vec::new(),
            protected_areas: Vec::new(),
            edits: Vec::new(),
        }
    }
}

impl WorldConfig {
    pub fn seed_value(&self) -> Result<u64> {
        if self.seed.is_empty() || !self.seed.bytes().all(|b| b.is_ascii_digit()) {
            return Err("world seed must be a decimal u64".into());
        }
        Ok(self.seed.parse()?)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || self.generator_revision != "mudflat-v1" {
            return Err("unsupported world schema or generator revision".into());
        }
        self.seed_value()?;
        if self.scene.trim().is_empty() || self.content_revision.trim().is_empty() {
            return Err("world scene and content revision must not be empty".into());
        }
        self.validate_placement_inputs()?;
        Ok(())
    }

    /// Existing CLI convention: --name=value. Unrelated game flags pass through.
    /// A config is authoritative: conflicting seed/scene flags are rejected.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let args: Vec<_> = args.into_iter().collect();
        let value = |name: &str| -> Result<Option<String>> {
            let prefix = format!("{name}=");
            let values: Vec<_> = args
                .iter()
                .filter_map(|arg| arg.strip_prefix(&prefix))
                .collect();
            if values.len() > 1 || args.iter().any(|arg| arg == name) {
                return Err(format!("use {name}=<value> exactly once").into());
            }
            Ok(values.first().map(|v| (*v).to_owned()))
        };
        let config_path = value("--world-config")?;
        let seed = value("--seed")?;
        let scene = value("--scene")?;
        let mut config = if let Some(path) = &config_path {
            serde_json::from_slice::<Self>(&std::fs::read(path)?)?
        } else {
            Self::default()
        };
        if let Some(seed) = seed {
            if config_path.is_some() && config.seed != seed {
                return Err("--seed conflicts with --world-config".into());
            }
            config.seed = seed;
        }
        if let Some(scene) = scene {
            if config_path.is_some() && config.scene != scene {
                return Err("--scene conflicts with --world-config".into());
            }
            config.scene = scene;
        }
        config.validate()?;
        Ok(config)
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        self.validate()?;
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

/// Sample the existing analytical ground into native scatter input. The rendered
/// cracked plates remain untouched. Heights are metres, not normalized values.
pub fn ground_heightmap(resolution: u32, size: Vec2, origin: Vec2) -> Result<Heightmap> {
    if !(2..=512).contains(&resolution)
        || !size.is_finite()
        || size.min_element() <= 0.
        || !origin.is_finite()
    {
        return Err("scatter heightmap needs 2..=512 samples and finite positive size".into());
    }
    let mut map = Heightmap::new_at(resolution, size, 1., origin);
    for z in 0..resolution {
        for x in 0..resolution {
            let position = origin + Vec2::new(x as f32, z as f32) * size / (resolution - 1) as f32;
            map.set_height(x, z, super::terrain::height(position));
        }
    }
    Ok(map)
}

/// Native sampler only; no runtime spawning or new random algorithm. Restrict
/// this first adapter to at most one candidate per cell, so native cell IDs can
/// later serve as stable keys without inventing candidate ordinal reconstruction.
pub fn ground_scatter(seed: u64, map: &Heightmap, density: f32) -> Result<Vec<Placement>> {
    if !(2..=512).contains(&map.resolution)
        || map.heights.len() != (map.resolution as usize).pow(2)
        || !map.size.is_finite()
        || map.size.min_element() <= 0.
        || !map.origin.is_finite()
        || map.heights.iter().any(|h| !h.is_finite())
    {
        return Err("invalid scatter heightmap".into());
    }
    let cell_area = map.cell_size().element_product();
    if !cell_area.is_finite()
        || cell_area <= 0.
        || !density.is_finite()
        || density < 0.
        || density * cell_area > 1.
    {
        return Err("initial scatter density must produce at most one candidate per cell".into());
    }
    Ok(jackdaw_terrain::scatter(
        map,
        &[],
        &ScatterParams {
            seed,
            density,
            min_spacing: 0.,
            asset_count: 1,
            ..Default::default()
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_17_configuration_roundtrip_and_input_errors() {
        let config = WorldConfig::from_args(["--seed=17".into()]).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("world.json");
        config.write(&path).unwrap();
        let flag = format!("--world-config={}", path.display());
        assert_eq!(config, WorldConfig::from_args([flag.clone()]).unwrap());
        assert!(WorldConfig::from_args([flag, "--seed=invalid".into()]).is_err());
        assert!(WorldConfig::from_args(["--seed=invalid".into()]).is_err());
        let mut unsupported = config;
        unsupported.schema_version = 2;
        assert!(unsupported.validate().is_err());
    }

    #[test]
    fn seed_17_native_scatter_and_saved_placement_edits() {
        let map = ground_heightmap(9, Vec2::splat(8.), Vec2::splat(-4.)).unwrap();
        let native = ground_scatter(17, &map, 1.).unwrap();
        assert_eq!(native, ground_scatter(17, &map, 1.).unwrap());
        let mut config = WorldConfig::from_args(["--seed=17".into()]).unwrap();
        config.scatter.push(ScatterRule {
            id: "desert-decoration".into(),
            revision: 1,
            region: "opening".into(),
            asset: "models/props/desert/desert_debris_0.glb".into(),
            resolution: 9,
            origin: [-4.; 2],
            size: [8.; 2],
            density: 1.,
            clearance: 0.,
            blocked_cells: vec![],
        });
        let original = config.resolve_layout(&[]).unwrap();
        assert_eq!(original.placements.len(), native.len());
        assert!(original.placements.len() > 4);
        let a = &original.placements[0];
        let b = &original.placements[1];
        let c = &original.placements[2];
        let d = &original.placements[3];
        let center = Vec3::from_array(c.pose.position).xz().to_array();
        let authored = [ProtectedArea {
            min: center,
            max: center,
        }];
        let mut moved = b.pose.clone();
        moved.position[1] += 0.25;
        moved.yaw += 0.2;
        config.edits = vec![
            PlacementEdit::Suppress { key: a.key.clone() },
            PlacementEdit::Move {
                key: b.key.clone(),
                pose: moved.clone(),
            },
        ];
        config.scatter[0].blocked_cells.push(d.key.cell);
        let resolved = config.resolve_layout(&authored).unwrap();
        assert_eq!(resolved.placements.len(), original.placements.len() - 3);
        assert!(resolved.diagnostics.is_empty());
        assert_eq!(
            resolved
                .placements
                .iter()
                .find(|p| p.key == b.key)
                .unwrap()
                .pose,
            moved
        );
        for unchanged in &original.placements[4..] {
            assert!(resolved.placements.contains(unchanged));
        }
        let saved = serde_json::to_vec(&config).unwrap();
        let loaded: WorldConfig = serde_json::from_slice(&saved).unwrap();
        assert_eq!(resolved, loaded.resolve_layout(&authored).unwrap());
        // Conflicting move is omitted, never relocated or committed over the author.
        config.edits[1] = PlacementEdit::Move {
            key: b.key.clone(),
            pose: c.pose.clone(),
        };
        let invalid = config.resolve_layout(&authored).unwrap();
        assert!(!invalid.placements.iter().any(|p| p.key == b.key));
        assert_eq!(invalid.diagnostics.len(), 1);
        // A removed rule leaves edits intact and reports them as stale.
        config.scatter.clear();
        let stale = config.resolve_layout(&authored).unwrap();
        assert_eq!(stale.diagnostics.len(), 2);
        assert_eq!(config.edits.len(), 2);
    }
}
