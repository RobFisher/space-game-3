use serde::{Deserialize, Serialize};
use std::fmt;

use crate::EphemerisSource;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ObjectId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ObjectId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for ObjectId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Star,
    Planet,
    DwarfPlanet,
    Moon,
    Asteroid,
    Comet,
    Station,
    SurfaceBase,
    Ship,
    JumpGate,
    Region,
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicalProperties {
    pub mean_radius_km: Option<f64>,
    pub gravitational_parameter_km3_s2: Option<f64>,
    pub rotation_period_seconds: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameplayMetadata {
    pub description: Option<String>,
    pub faction: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_discoverable")]
    pub discoverable: bool,
}

fn default_discoverable() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectDefinition {
    pub id: ObjectId,
    pub name: String,
    pub kind: ObjectKind,
    pub source: EphemerisSource,
    pub physical: Option<PhysicalProperties>,
    pub gameplay: Option<GameplayMetadata>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectSummary {
    pub id: ObjectId,
    pub name: String,
    pub kind: ObjectKind,
}

impl From<&ObjectDefinition> for ObjectSummary {
    fn from(value: &ObjectDefinition) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            kind: value.kind.clone(),
        }
    }
}
