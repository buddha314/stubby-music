//! Resource sources: finite pickups and infinite mines are different objects.
//! Production rates and environmental simulation are deliberately not implemented.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Exhaustible,
    InfiniteMine,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProductivityFactor {
    WaterQuality,
    LightQuality,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MineDefinition {
    pub id: String,
    pub name: String,
    /// A resource definition ID, not an item ID. Each produced bug is finite.
    pub produces_resource: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub productivity_factors: Vec<ProductivityFactor>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceObject {
    Exhaustible {
        resource: String,
        instance: String,
        #[serde(default)]
        collected: bool,
    },
    /// Infinite capacity; there is intentionally no collected flag or reserve.
    InfiniteMine { mine: String, instance: String },
}
impl SourceObject {
    pub fn kind(&self) -> SourceKind {
        match self {
            Self::Exhaustible { .. } => SourceKind::Exhaustible,
            Self::InfiniteMine { .. } => SourceKind::InfiniteMine,
        }
    }
    pub fn definition_id(&self) -> &str {
        match self {
            Self::Exhaustible { resource, .. } => resource,
            Self::InfiniteMine { mine, .. } => mine,
        }
    }
    pub fn instance_id(&self) -> &str {
        match self {
            Self::Exhaustible { instance, .. } | Self::InfiniteMine { instance, .. } => instance,
        }
    }
}
