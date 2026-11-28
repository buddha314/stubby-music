//! The City kit's collision proxies (add-city-kit 5.3). City pieces are mostly single-level
//! models, so the trimesh path for flattened levels (`model_part_bindings::solid_boulders`)
//! never sees them, and the player walked through every building. Their simple shapes are
//! authored as data, `assets/structures/city/colliders.json` (written by
//! `tools/blender/create_city_kit.py index`); each placed City model gets its shapes as
//! static `Solid` children, which the player's movement slides along.
//!
//! City pieces are scenery and are deliberately not bridged to `PlacedAsset`: bridging the
//! reference district's ~4,900 placements stalled the first frame past 20 s (2026-11-28),
//! and their acoustic extras already reach Steam Audio through `prefab_presentation`. So this
//! reads the placed model's `GltfSource` path directly.
//!
//! Shapes marked `walkable` (paving, decks, slabs, bridges, stairs) are left out: the
//! controller cannot stand on a collider yet (add-prefab-colliders), so a deck would be a
//! wall. Only walls, piers, columns, kerbs and furniture-sized solids block.
use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use jackdaw_runtime::GltfSource;
use std::collections::HashMap;

pub struct CityCollidersPlugin;
impl Plugin for CityCollidersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CityColliders::load())
            .add_systems(Update, build_city_colliders);
    }
}

/// Solid shapes by model path (`structures/city/<id>/<id>.glb`), in the model's frame.
#[derive(Resource, Default)]
struct CityColliders(HashMap<String, Vec<Shape>>);

#[derive(Clone, Debug, PartialEq)]
struct Shape {
    kind: Kind,
    translation: Vec3,
    tilt: f32,
}

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Cuboid(Vec3),
    Cylinder { radius: f32, height: f32 },
}

impl CityColliders {
    fn load() -> Self {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/structures/city/colliders.json"
        );
        match std::fs::read_to_string(path) {
            Ok(text) => Self(parse(&text)),
            Err(error) => {
                warn!("City colliders not loaded ({path}): {error}");
                Self::default()
            }
        }
    }
}

fn number(value: &serde_json::Value, key: &str) -> Option<f32> {
    value.get(key)?.as_f64().map(|v| v as f32)
}

/// The blocking shapes of every model in the colliders file; walkable ones are skipped.
fn parse(text: &str) -> HashMap<String, Vec<Shape>> {
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(text) else {
        return HashMap::new();
    };
    let Some(models) = doc.get("models").and_then(|m| m.as_object()) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (model, entry) in models {
        let shapes: Vec<Shape> = entry
            .get("colliders")
            .and_then(|c| c.as_array())
            .into_iter()
            .flatten()
            .filter(|c| !c.get("walkable").and_then(|w| w.as_bool()).unwrap_or(false))
            .filter_map(|c| {
                let t = c.get("translation")?.as_array()?;
                let translation = Vec3::new(
                    t.first()?.as_f64()? as f32,
                    t.get(1)?.as_f64()? as f32,
                    t.get(2)?.as_f64()? as f32,
                );
                let kind = match c.get("shape")?.as_str()? {
                    "Cuboid" => Kind::Cuboid(Vec3::new(
                        number(c, "x_length")?,
                        number(c, "y_length")?,
                        number(c, "z_length")?,
                    )),
                    "Cylinder" => Kind::Cylinder {
                        radius: number(c, "radius")?,
                        height: number(c, "height")?,
                    },
                    _ => return None,
                };
                let tilt = number(c, "tilt_about_x_radians").unwrap_or(0.0);
                Some(Shape {
                    kind,
                    translation,
                    tilt,
                })
            })
            .collect();
        if !shapes.is_empty() {
            out.insert(model.clone(), shapes);
        }
    }
    out
}

#[derive(Component)]
struct CityCollisionBuilt;

fn build_city_colliders(
    mut commands: Commands,
    colliders: Res<CityColliders>,
    placed: Query<(Entity, &GltfSource), Without<CityCollisionBuilt>>,
) {
    for (entity, source) in &placed {
        if !source.path.starts_with("structures/city/") {
            continue;
        }
        let mut root = commands.entity(entity);
        root.insert(CityCollisionBuilt);
        let Some(shapes) = colliders.0.get(&source.path) else {
            continue;
        };
        root.with_children(|parent| {
            for shape in shapes {
                let collider = match shape.kind {
                    Kind::Cuboid(size) => Collider::cuboid(size.x, size.y, size.z),
                    Kind::Cylinder { radius, height } => Collider::cylinder(radius, height),
                };
                parent.spawn((
                    Name::new("City collider"),
                    collider,
                    RigidBody::Static,
                    stubby_core::Solid,
                    Transform::from_translation(shape.translation)
                        .with_rotation(Quat::from_rotation_x(shape.tilt)),
                ));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walls_block_and_walkable_shapes_do_not() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/structures/city/colliders.json"
        ))
        .unwrap();
        let shapes = parse(&text);
        // A facade bay is a wall; paving is walkable and stays out; a door bay keeps its piers.
        assert!(shapes.contains_key("structures/city/city_facade_window/city_facade_window.glb"));
        assert!(!shapes.contains_key("structures/city/city_paving_4/city_paving_4.glb"));
        let door = &shapes["structures/city/city_facade_door/city_facade_door.glb"];
        assert_eq!(door.len(), 3);
        assert!(
            shapes["structures/city/city_column/city_column.glb"]
                .iter()
                .all(|s| matches!(s.kind, Kind::Cylinder { .. }))
        );
    }
}
