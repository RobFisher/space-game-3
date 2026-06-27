//! Ephemeris calculations and data structures for space simulations.

mod error;
mod manifest;
mod object;
mod providers;
mod registry;
mod resolution;
mod source;
mod state;
mod time;
mod vector;
mod world;

pub use crate::error::EphemerisError;
pub use crate::manifest::{
    default_asset_root, downloaded_profile_objects, fetch_profile_assets, resolve_asset_path,
    resolved_asset_root, verify_asset, verify_profile_assets, AssetEntry, AssetKind, AssetProfile,
    AssetVerification, AssetVerificationStatus, CoveredObject, DownloadedCoveredObject,
    DownloadedProfileObjects, EphemerisAssetManifest, KernelEntry, KernelKind, KernelManifest,
    SelectedAsset, SkippedAssetReason, SkippedProfileAsset, ASSET_ROOT_ENV,
};
pub use crate::object::{
    GameplayMetadata, ObjectDefinition, ObjectId, ObjectKind, ObjectSummary, PhysicalProperties,
};
pub use crate::registry::ObjectRegistry;
pub use crate::source::{EphemerisSource, InterpolationMode, TrajectorySample};
pub use crate::state::{EphemerisQuality, FrameId, StateVector};
pub use crate::time::GameTime;
pub use crate::vector::{Vec3Km, Vec3KmPerSec};
pub use crate::world::{SolarSystem, SolarSystemBuilder};
