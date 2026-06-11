use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use crate::EphemerisError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KernelManifest {
    pub schema_version: u32,
    pub profile: String,
    #[serde(default)]
    pub kernels: Vec<KernelEntry>,
}

impl KernelManifest {
    pub fn from_toml_str(input: &str) -> Result<Self, EphemerisError> {
        let manifest: Self = toml::from_str(input)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn from_toml_path(path: impl AsRef<Path>) -> Result<Self, EphemerisError> {
        let input = std::fs::read_to_string(path)?;
        Self::from_toml_str(&input)
    }

    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.schema_version != 1 {
            return invalid_manifest(format!(
                "unsupported kernel manifest schema version {}",
                self.schema_version
            ));
        }
        if self.profile.trim().is_empty() {
            return invalid_manifest("kernel manifest profile must not be empty");
        }

        let mut ids = HashSet::new();
        for kernel in &self.kernels {
            if kernel.id.trim().is_empty() {
                return invalid_manifest("kernel id must not be empty");
            }
            if !ids.insert(kernel.id.as_str()) {
                return invalid_manifest(format!("duplicate kernel id {}", kernel.id));
            }
            if kernel.filename.trim().is_empty() {
                return invalid_manifest(format!(
                    "kernel {} filename must not be empty",
                    kernel.id
                ));
            }
            if !kernel.url.starts_with("https://") && !kernel.url.starts_with("http://") {
                return invalid_manifest(format!("kernel {} URL must be HTTP(S)", kernel.id));
            }
            if kernel.sha256.trim().is_empty() {
                return invalid_manifest(format!(
                    "kernel {} checksum must not be empty",
                    kernel.id
                ));
            }
            if let Some(size) = kernel.size_bytes {
                if size == 0 {
                    return invalid_manifest(format!("kernel {} size must be positive", kernel.id));
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KernelEntry {
    pub id: String,
    pub kind: KernelKind,
    pub filename: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub required: bool,
    pub covers: Option<String>,
    pub preferred_range: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelKind {
    Lsk,
    Pck,
    Spk,
    Fk,
}

fn invalid_manifest<T>(message: impl Into<String>) -> Result<T, EphemerisError> {
    Err(EphemerisError::InvalidObjectDefinition(message.into()))
}
