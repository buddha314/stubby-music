//! Shared world inputs and the adapter to Jackdaw's existing scatter sampler.
//! This does not spawn resources or replace the authored opening.
use bevy::prelude::*;
use jackdaw_terrain::{Heightmap, Placement, ScatterParams};
mod placements;
pub use placements::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// v2: rules resolve in configuration order, each with its own seed, and may keep
/// distances from what is already placed (add-hybrid-world-placement 2b).
pub const GENERATOR_REVISION: &str = "mudflat-v2";

/// The seeded opening: its rules, protected areas and authored footprints
/// (`assets/world/opening.json`, a whole world configuration).
const SHIPPED: &str = include_str!("../../../assets/world/opening.json");

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
    /// Clearance kept around authored objects, by asset (metres of XZ half extent); an
    /// asset not listed protects only its point. Scatter candidates also add their own.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub authored_footprints: std::collections::BTreeMap<String, f32>,
    /// Things that form along every laid stream (ripple pools), at seeded random places.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stream_features: Vec<StreamFeature>,
}

/// A model that forms along laid streams: a Poisson process in arc length (gaps drawn
/// from an exponential distribution with mean `mean_spacing_m`), on either bank by the
/// seed (`add-ripple-pool` D1). The model sits in the stream segment's frame (flow +X,
/// origin at the channel's centre); half a turn puts it on the other bank.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamFeature {
    pub id: String,
    pub asset: String,
    pub mean_spacing_m: f32,
    /// Kept clear of a segment's ends, metres, so features do not straddle joins.
    #[serde(default)]
    pub end_margin_m: f32,
}

/// Where a stream feature forms along one laid segment: offsets in metres from the
/// segment's start, and whether it is on the far bank. Seeded from the world seed, the
/// feature and the segment's own key (`segment`: anything stable for that segment, such as
/// its path and index), so it never moves when the stream is extended or other streams
/// are laid, and laying the same segment again gives the same places.
pub fn stream_feature_places(
    seed: u64,
    feature: &StreamFeature,
    segment: u64,
    span: f32,
) -> Vec<(f32, bool)> {
    use rand_chacha::{
        ChaCha8Rng,
        rand_core::{RngCore, SeedableRng},
    };
    if !(feature.mean_spacing_m > 0. && span > 2. * feature.end_margin_m) {
        return Vec::new();
    }
    let key = feature.id.bytes().fold(
        seed ^ segment.rotate_left(17) ^ 0xcbf29ce484222325,
        |h, b| (h ^ u64::from(b)).wrapping_mul(0x100000001b3),
    );
    let mut rng = ChaCha8Rng::seed_from_u64(key);
    let mut unit = || (rng.next_u32() as f64 + 0.5) / 4_294_967_296.;
    // The margins leave less of the segment eligible; shorten the gaps to match, so a
    // stream still averages one per `mean_spacing_m` of its length.
    let usable = span - 2. * feature.end_margin_m;
    let mean = f64::from(feature.mean_spacing_m * usable / span);
    let mut places = Vec::new();
    let mut s = feature.end_margin_m;
    loop {
        s += (-unit().ln() * mean) as f32;
        if s > span - feature.end_margin_m {
            return places;
        }
        places.push((s, unit() < 0.5));
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            generator_revision: GENERATOR_REVISION.into(),
            seed: "42".into(),
            scene: "scene.bsn".into(),
            content_revision: "development".into(),
            scatter: Vec::new(),
            protected_areas: Vec::new(),
            edits: Vec::new(),
            authored_footprints: Default::default(),
            stream_features: Vec::new(),
        }
    }
}

impl WorldConfig {
    /// The game's own configuration: the seeded opening's rules. Both entry points start
    /// from it unless `--world-config` names another.
    pub fn shipped() -> Self {
        let config: Self = serde_json::from_str(SHIPPED).expect("assets/world/opening.json parses");
        config
            .validate()
            .expect("assets/world/opening.json is valid");
        config
    }

    pub fn seed_value(&self) -> Result<u64> {
        if self.seed.is_empty() || !self.seed.bytes().all(|b| b.is_ascii_digit()) {
            return Err("world seed must be a decimal u64".into());
        }
        Ok(self.seed.parse()?)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || self.generator_revision != GENERATOR_REVISION {
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
            Self::shipped()
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
            map.set_height(x, z, crate::terrain::height(position));
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
        let mut config = WorldConfig::from_args(["--seed=17".into()]).unwrap();
        // A bare layout: none of the shipped opening's rules, areas or footprints.
        config.scatter.clear();
        config.protected_areas.clear();
        config.authored_footprints.clear();
        config.scatter.push(ScatterRule {
            id: "desert-decoration".into(),
            revision: 1,
            region: "opening".into(),
            asset: "props/desert/desert_debris_0/desert_debris_0.glb".into(),
            resolution: 9,
            origin: [-4.; 2],
            size: [8.; 2],
            density: 1.,
            clearance: 0.,
            blocked_cells: vec![],
            variants: vec![],
            min_spacing: 0.,
            scale: [1., 1.],
            distances: vec![],
        });
        let native =
            ground_scatter(config.rule_seed(&config.scatter[0]).unwrap(), &map, 1.).unwrap();
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

    #[test]
    fn authored_footprints_clear_scatter_around_authored_objects() {
        let mut config = WorldConfig::from_args(["--seed=17".into()]).unwrap();
        // A bare layout: none of the shipped opening's rules, areas or footprints.
        config.scatter.clear();
        config.protected_areas.clear();
        config.authored_footprints.clear();
        config.scatter.push(ScatterRule {
            id: "desert-decoration".into(),
            revision: 1,
            region: "opening".into(),
            asset: "props/desert/desert_debris_0/desert_debris_0.glb".into(),
            resolution: 9,
            origin: [-4.; 2],
            size: [8.; 2],
            density: 1.,
            clearance: 0.,
            blocked_cells: vec![],
            variants: vec![],
            min_spacing: 0.,
            scale: [1., 1.],
            distances: vec![],
        });
        let all = config.resolve_layout(&[]).unwrap().placements;
        // An authored burbler one metre from a candidate.
        let near = Vec3::from_array(all[0].pose.position);
        let burbler = near + Vec3::X;
        // Unlisted: only its point is protected, so the candidate stays.
        let point = config.authored_clearance([("mudpit_burbler", burbler)]);
        assert!(
            config
                .resolve_layout(&point)
                .unwrap()
                .placements
                .contains(&all[0])
        );
        // A 1.5 m footprint reaches it; the burbler itself is only read, never moved.
        config
            .authored_footprints
            .insert("mudpit_burbler".into(), 1.5);
        let area = config.authored_clearance([("mudpit_burbler", burbler)]);
        let resolved = config.resolve_layout(&area).unwrap();
        assert!(!resolved.placements.iter().any(|p| p.key == all[0].key));
        assert!(resolved.placements.len() < all.len());
        // Footprints round-trip; an invalid one is refused.
        let loaded: WorldConfig =
            serde_json::from_slice(&serde_json::to_vec(&config).unwrap()).unwrap();
        assert_eq!(loaded.authored_footprints, config.authored_footprints);
        config
            .authored_footprints
            .insert("mudpit_burbler".into(), f32::NAN);
        assert!(config.resolve_layout(&area).is_err());
    }

    /// The shipped opening on seed 17, around the opening scene's authored burblers and
    /// the grotto (add-opening-region "Playable placement", add-hybrid-world-placement 3.1).
    #[test]
    fn seed_17_opening_meets_its_placement_constraints() {
        let mut config = WorldConfig::shipped();
        config.seed = "17".into();
        let authored: Vec<(String, Vec3)> = [
            ("mudpit_burbler", Vec3::new(39.33, 0., -6.05)),
            ("mudpit_burbler", Vec3::new(69.44, 0., -59.2)),
            ("boulder_0", Vec3::new(70.5, 0., -60.3)),
        ]
        .into_iter()
        .map(|(s, p)| (s.to_owned(), p))
        .collect();
        let layout = config.resolve_opening(&authored).unwrap();
        assert_eq!(
            layout,
            config.resolve_opening(&authored).unwrap(),
            "same seed, same layout"
        );
        assert!(layout.diagnostics.is_empty(), "{:?}", layout.diagnostics);
        let of = |rule: &str| -> Vec<Vec2> {
            layout
                .placements
                .iter()
                .filter(|p| p.key.rule == rule)
                .map(|p| Vec3::from_array(p.pose.position).xz())
                .collect()
        };
        let (burblers, reeds, oaks, rocks, props) = (
            of("burblers"),
            of("reed_beds"),
            of("oak_groves"),
            of("rock_formations"),
            of("desert_props"),
        );
        eprintln!(
            "seed 17: {} burblers, {} reed clumps, {} oaks, {} rock formations, {} props",
            burblers.len(),
            reeds.len(),
            oaks.len(),
            rocks.len(),
            props.len()
        );
        assert!(burblers.len() >= 2 && !oaks.is_empty() && !rocks.is_empty() && !props.is_empty());
        let all_burblers: Vec<Vec2> = burblers
            .iter()
            .copied()
            .chain(
                authored
                    .iter()
                    .filter(|(s, _)| s == "mudpit_burbler")
                    .map(|(_, p)| p.xz()),
            )
            .collect();
        let nearest = |at: Vec2, to: &[Vec2]| {
            to.iter()
                .map(|p| p.distance(at))
                .fold(f32::INFINITY, f32::min)
        };
        for (i, b) in burblers.iter().enumerate() {
            let others: Vec<Vec2> = all_burblers.iter().copied().filter(|p| p != b).collect();
            assert!(nearest(*b, &others) >= 65., "burbler {i} crowds another");
            // A first stream to an oak (at most 40 m, 10 segments of 6 reeds) is affordable.
            let near = reeds.iter().filter(|r| r.distance(*b) <= 40.).count();
            assert!(
                near >= 10,
                "burbler {i} has only {near} reed clumps within 40 m"
            );
        }
        for oak in &oaks {
            let d = nearest(*oak, &all_burblers);
            assert!(
                (20.0..=40.5).contains(&d),
                "an oak {d} m from its nearest burbler"
            );
        }
        // The first filter can be made: a formation with an opening (variants 2 to 5).
        assert!(layout.placements.iter().any(|p| {
            p.key.rule == "rock_formations"
                && ["2", "3", "4", "5"]
                    .iter()
                    .any(|k| p.asset.ends_with(&format!("rock_formation_{k}.glb")))
        }));
        let spawn = &config.protected_areas[0];
        assert!(layout.placements.iter().all(|p| {
            let at = Vec3::from_array(p.pose.position).xz();
            !(at.cmpge(Vec2::from_array(spawn.min)).all()
                && at.cmple(Vec2::from_array(spawn.max)).all())
        }));
    }

    /// Ripple pools along streams: the same segment gives the same places, and over many
    /// segments they average one per mean spacing (add-ripple-pool D1).
    #[test]
    fn stream_features_are_seeded_and_poisson() {
        let config = WorldConfig::shipped();
        let pools = &config.stream_features[0];
        assert_eq!(pools.id, "ripple_pools");
        let a = stream_feature_places(17, pools, 42, 4.);
        assert_eq!(a, stream_feature_places(17, pools, 42, 4.));
        let total: usize = (0..2000)
            .map(|k| stream_feature_places(17, pools, k, 4.).len())
            .sum();
        let per_metre = total as f32 / (2000. * 4.); // per metre of stream
        assert!(
            (per_metre * pools.mean_spacing_m - 1.).abs() < 0.12,
            "{per_metre} per metre"
        );
        assert!(
            a.iter()
                .all(|(s, _)| (pools.end_margin_m..=4. - pools.end_margin_m).contains(s))
        );
    }
}
