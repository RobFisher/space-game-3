use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use crate::{EphemerisError, ObjectId, ObjectKind};

pub const ASSET_ROOT_ENV: &str = "SPACE_GAME_EPHEMERIS_DATA_DIR";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EphemerisAssetManifest {
    pub version: u32,
    #[serde(default)]
    pub profiles: BTreeMap<String, AssetProfile>,
    #[serde(default)]
    pub assets: BTreeMap<String, AssetEntry>,
}

impl EphemerisAssetManifest {
    pub fn from_toml_str(input: &str) -> Result<Self, EphemerisError> {
        let manifest: Self = toml::from_str(input)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn from_toml_path(path: impl AsRef<Path>) -> Result<Self, EphemerisError> {
        let input = fs::read_to_string(path)?;
        Self::from_toml_str(&input)
    }

    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.version != 1 {
            return invalid_manifest(format!(
                "unsupported ephemeris asset manifest version {}",
                self.version
            ));
        }
        if self.profiles.is_empty() {
            return invalid_manifest("ephemeris asset manifest must define at least one profile");
        }
        if self.assets.is_empty() {
            return invalid_manifest("ephemeris asset manifest must define at least one asset");
        }

        for (profile_id, profile) in &self.profiles {
            validate_id("profile", profile_id)?;
            if profile.description.trim().is_empty() {
                return invalid_manifest(format!(
                    "profile {} description must not be empty",
                    profile_id
                ));
            }
            if profile.assets.is_empty() {
                return invalid_manifest(format!(
                    "profile {} must reference at least one asset",
                    profile_id
                ));
            }

            let mut seen = HashSet::new();
            for asset_id in &profile.assets {
                if !seen.insert(asset_id.as_str()) {
                    return invalid_manifest(format!(
                        "profile {} references asset {} more than once",
                        profile_id, asset_id
                    ));
                }
                if !self.assets.contains_key(asset_id) {
                    return invalid_manifest(format!(
                        "profile {} references unknown asset {}",
                        profile_id, asset_id
                    ));
                }
            }
        }

        let mut covered_objects: BTreeMap<ObjectId, CoveredObject> = BTreeMap::new();
        for (asset_id, asset) in &self.assets {
            validate_id("asset", asset_id)?;
            asset.validate(asset_id)?;
            for covered in &asset.covers {
                covered.validate(asset_id)?;
                if let Some(existing) = covered_objects.get(&covered.id) {
                    if existing != covered {
                        return invalid_manifest(format!(
                            "asset {} coverage for object {} conflicts with another asset",
                            asset_id, covered.id
                        ));
                    }
                } else {
                    covered_objects.insert(covered.id.clone(), covered.clone());
                }
            }
        }

        Ok(())
    }

    pub fn profile_assets(
        &self,
        profile_id: &str,
    ) -> Result<Vec<SelectedAsset<'_>>, EphemerisError> {
        let profile = self.profiles.get(profile_id).ok_or_else(|| {
            EphemerisError::InvalidObjectDefinition(format!(
                "unknown ephemeris asset profile {}",
                profile_id
            ))
        })?;

        let mut selected = Vec::with_capacity(profile.assets.len());
        for asset_id in &profile.assets {
            let asset = self.assets.get(asset_id).ok_or_else(|| {
                EphemerisError::InvalidObjectDefinition(format!(
                    "profile {} references unknown asset {}",
                    profile_id, asset_id
                ))
            })?;
            selected.push(SelectedAsset {
                id: asset_id.as_str(),
                asset,
            });
        }

        Ok(selected)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetProfile {
    pub description: String,
    #[serde(default)]
    pub assets: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetEntry {
    pub kind: AssetKind,
    pub filename: String,
    pub source: String,
    pub url: String,
    pub local_path: PathBuf,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub approx_size: Option<String>,
    #[serde(default)]
    pub required: bool,
    pub description: Option<String>,
    pub source_notes: Option<String>,
    pub license_notes: Option<String>,
    #[serde(default)]
    pub covers: Vec<CoveredObject>,
}

impl AssetEntry {
    fn validate(&self, asset_id: &str) -> Result<(), EphemerisError> {
        if self.filename.trim().is_empty() {
            return invalid_manifest(format!("asset {} filename must not be empty", asset_id));
        }
        if self.source.trim().is_empty() {
            return invalid_manifest(format!("asset {} source must not be empty", asset_id));
        }
        if !self.url.starts_with("https://")
            && !self.url.starts_with("http://")
            && !self.url.starts_with("file://")
        {
            return invalid_manifest(format!("asset {} URL must be HTTP(S) or file", asset_id));
        }
        validate_local_path(asset_id, &self.local_path)?;
        if let Some(size) = self.size_bytes {
            if size == 0 {
                return invalid_manifest(format!("asset {} size_bytes must be positive", asset_id));
            }
        }
        if let Some(sha256) = &self.sha256 {
            validate_sha256(asset_id, sha256)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoveredObject {
    pub id: ObjectId,
    pub name: String,
    pub kind: ObjectKind,
    pub naif_id: Option<i32>,
    pub notes: Option<String>,
}

impl CoveredObject {
    fn validate(&self, asset_id: &str) -> Result<(), EphemerisError> {
        validate_id("covered object", self.id.as_str())?;
        if self.name.trim().is_empty() {
            return invalid_manifest(format!(
                "asset {} covered object {} name must not be empty",
                asset_id, self.id
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssetKind {
    Lsk,
    Pck,
    Spk,
    Fk,
    AnisePca,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedAsset<'a> {
    pub id: &'a str,
    pub asset: &'a AssetEntry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetVerification {
    pub id: String,
    pub path: PathBuf,
    pub status: AssetVerificationStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetVerificationStatus {
    Valid,
    OptionalMissing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadedProfileObjects {
    pub objects: Vec<DownloadedCoveredObject>,
    pub skipped_assets: Vec<SkippedProfileAsset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadedCoveredObject {
    pub object: CoveredObject,
    pub source_asset_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkippedProfileAsset {
    pub id: String,
    pub path: PathBuf,
    pub reason: SkippedAssetReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkippedAssetReason {
    Missing,
    Invalid(String),
    NoCoverage,
}

pub type KernelManifest = EphemerisAssetManifest;
pub type KernelEntry = AssetEntry;
pub type KernelKind = AssetKind;

pub fn default_asset_root() -> PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(crate_dir);
    repo_root.join("data").join("ephemeris")
}

pub fn resolved_asset_root() -> PathBuf {
    match env::var_os(ASSET_ROOT_ENV) {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => default_asset_root(),
    }
}

pub fn resolve_asset_path(root: impl AsRef<Path>, asset: &AssetEntry) -> PathBuf {
    root.as_ref().join(&asset.local_path)
}

pub fn verify_profile_assets(
    manifest: &EphemerisAssetManifest,
    profile_id: &str,
    root: impl AsRef<Path>,
) -> Result<Vec<AssetVerification>, EphemerisError> {
    let root = root.as_ref();
    let mut results = Vec::new();

    for selected in manifest.profile_assets(profile_id)? {
        match verify_asset(selected, root)? {
            Some(result) => results.push(result),
            None => {}
        }
    }

    Ok(results)
}

pub fn verify_asset(
    selected: SelectedAsset<'_>,
    root: &Path,
) -> Result<Option<AssetVerification>, EphemerisError> {
    let path = resolve_asset_path(root, selected.asset);
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if selected.asset.required {
                return Err(EphemerisError::KernelNotFound(format!(
                    "asset {} missing at {}",
                    selected.id,
                    path.display()
                )));
            }
            return Ok(Some(AssetVerification {
                id: selected.id.to_string(),
                path,
                status: AssetVerificationStatus::OptionalMissing,
            }));
        }
        Err(error) => return Err(error.into()),
    };

    if let Some(expected) = selected.asset.size_bytes {
        let actual = metadata.len();
        if actual != expected {
            return Err(EphemerisError::AssetSizeMismatch {
                asset_id: selected.id.to_string(),
                path,
                expected,
                actual,
            });
        }
    }

    if let Some(expected) = &selected.asset.sha256 {
        let actual = sha256_file(&path)?;
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(EphemerisError::ChecksumMismatch {
                asset_id: selected.id.to_string(),
                path,
                expected: expected.to_string(),
                actual,
            });
        }
    }

    Ok(Some(AssetVerification {
        id: selected.id.to_string(),
        path,
        status: AssetVerificationStatus::Valid,
    }))
}

pub fn downloaded_profile_objects(
    manifest: &EphemerisAssetManifest,
    profile_id: &str,
    root: impl AsRef<Path>,
) -> Result<DownloadedProfileObjects, EphemerisError> {
    let root = root.as_ref();
    let mut objects_by_id: BTreeMap<ObjectId, DownloadedCoveredObject> = BTreeMap::new();
    let mut skipped_assets = Vec::new();

    for selected in manifest.profile_assets(profile_id)? {
        let path = resolve_asset_path(root, selected.asset);
        match verify_asset(selected, root) {
            Ok(Some(AssetVerification {
                status: AssetVerificationStatus::Valid,
                ..
            })) => {
                if selected.asset.covers.is_empty() {
                    skipped_assets.push(SkippedProfileAsset {
                        id: selected.id.to_string(),
                        path,
                        reason: SkippedAssetReason::NoCoverage,
                    });
                    continue;
                }

                for covered in &selected.asset.covers {
                    objects_by_id.entry(covered.id.clone()).or_insert_with(|| {
                        DownloadedCoveredObject {
                            object: covered.clone(),
                            source_asset_id: selected.id.to_string(),
                        }
                    });
                }
            }
            Ok(Some(AssetVerification {
                status: AssetVerificationStatus::OptionalMissing,
                ..
            }))
            | Err(EphemerisError::KernelNotFound(_)) => {
                skipped_assets.push(SkippedProfileAsset {
                    id: selected.id.to_string(),
                    path,
                    reason: SkippedAssetReason::Missing,
                });
            }
            Ok(None) => {}
            Err(error) => {
                skipped_assets.push(SkippedProfileAsset {
                    id: selected.id.to_string(),
                    path,
                    reason: SkippedAssetReason::Invalid(error.to_string()),
                });
            }
        }
    }

    Ok(DownloadedProfileObjects {
        objects: objects_by_id.into_values().collect(),
        skipped_assets,
    })
}

pub fn fetch_profile_assets(
    manifest: &EphemerisAssetManifest,
    profile_id: &str,
    root: impl AsRef<Path>,
    force: bool,
) -> Result<Vec<AssetVerification>, EphemerisError> {
    let root = root.as_ref();
    let mut results = Vec::new();

    for selected in manifest.profile_assets(profile_id)? {
        if !force {
            match verify_asset(selected, root) {
                Ok(Some(
                    result @ AssetVerification {
                        status: AssetVerificationStatus::Valid,
                        ..
                    },
                )) => {
                    results.push(result);
                    continue;
                }
                Ok(_) | Err(EphemerisError::KernelNotFound(_)) => {}
                Err(_) => {}
            }
        }

        let path = resolve_asset_path(root, selected.asset);
        download_asset(selected.id, selected.asset, &path)?;
        if let Some(result) = verify_asset(selected, root)? {
            results.push(result);
        }
    }

    Ok(results)
}

fn download_asset(asset_id: &str, asset: &AssetEntry, path: &Path) -> Result<(), EphemerisError> {
    let parent = path.parent().ok_or_else(|| {
        EphemerisError::DownloadFailed(format!(
            "asset {} has no parent directory for {}",
            asset_id,
            path.display()
        ))
    })?;
    fs::create_dir_all(parent)?;

    let temp_path = path.with_extension(format!(
        "{}.download",
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("tmp")
    ));

    if let Some(source_path) = asset.url.strip_prefix("file://") {
        fs::copy(source_path, &temp_path).map_err(|error| {
            EphemerisError::DownloadFailed(format!(
                "asset {} copy from {} to {} failed: {}",
                asset_id,
                source_path,
                temp_path.display(),
                error
            ))
        })?;
    } else {
        let response = ureq::get(&asset.url).call().map_err(|error| {
            EphemerisError::DownloadFailed(format!(
                "asset {} download from {} failed: {}",
                asset_id, asset.url, error
            ))
        })?;
        let mut reader = response.into_reader();
        let mut file = fs::File::create(&temp_path)?;
        std::io::copy(&mut reader, &mut file)?;
        file.flush()?;
    }

    fs::rename(&temp_path, path)?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, EphemerisError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_id(kind: &str, id: &str) -> Result<(), EphemerisError> {
    if id.trim().is_empty() {
        return invalid_manifest(format!("{} id must not be empty", kind));
    }
    if id
        .chars()
        .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
    {
        return invalid_manifest(format!(
            "{} id {} must contain only ASCII letters, digits, '-' or '_'",
            kind, id
        ));
    }
    Ok(())
}

fn validate_local_path(asset_id: &str, path: &Path) -> Result<(), EphemerisError> {
    if path.as_os_str().is_empty() {
        return invalid_manifest(format!("asset {} local_path must not be empty", asset_id));
    }
    if path.is_absolute() {
        return invalid_manifest(format!("asset {} local_path must be relative", asset_id));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return invalid_manifest(format!(
                    "asset {} local_path must not escape the asset root",
                    asset_id
                ));
            }
        }
    }
    Ok(())
}

fn validate_sha256(asset_id: &str, sha256: &str) -> Result<(), EphemerisError> {
    if sha256.len() != 64 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return invalid_manifest(format!(
            "asset {} sha256 must be a 64-character hexadecimal digest",
            asset_id
        ));
    }
    Ok(())
}

fn invalid_manifest<T>(message: impl Into<String>) -> Result<T, EphemerisError> {
    Err(EphemerisError::InvalidObjectDefinition(message.into()))
}
