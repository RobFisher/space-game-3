use crate::time::GameTime;
use std::path::PathBuf;
use thiserror::Error;

/// Errors returned by ephemeris registry loading and state queries.
#[derive(Debug, Error)]
pub enum EphemerisError {
    #[error("unknown object: {0}")]
    UnknownObject(String),

    #[error("kernel not found: {0}")]
    KernelNotFound(String),

    #[error("object {object} is outside available ephemeris coverage at {epoch}")]
    OutOfCoverage { object: String, epoch: GameTime },

    #[error("frame transform unavailable: {0}")]
    FrameTransformUnavailable(String),

    #[error("invalid object definition: {0}")]
    InvalidObjectDefinition(String),

    #[error("cyclic object dependency involving: {0}")]
    CyclicDependency(String),

    #[error("download failed: {0}")]
    DownloadFailed(String),

    #[error(
        "asset {asset_id} size mismatch at {path}: expected {expected} bytes, got {actual} bytes"
    )]
    AssetSizeMismatch {
        asset_id: String,
        path: PathBuf,
        expected: u64,
        actual: u64,
    },

    #[error("asset {asset_id} checksum mismatch at {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        asset_id: String,
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("backend error: {0}")]
    Backend(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
}
