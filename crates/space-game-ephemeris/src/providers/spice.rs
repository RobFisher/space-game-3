use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anise::constants::frames::SSB_J2000;
use anise::frames::Frame;
use anise::prelude::Almanac;
use anise::time::Epoch;

use crate::{
    resolve_asset_path, verify_asset, AssetKind, EphemerisError, EphemerisQuality, EphemerisSource,
    FrameId, GameTime, KernelManifest, ObjectDefinition, SelectedAsset, StateVector, Vec3Km,
    Vec3KmPerSec,
};

const DEFAULT_PROFILE: &str = "minimal";

#[derive(Clone)]
pub(crate) struct SpiceProvider {
    inner: Arc<SpiceProviderInner>,
}

struct SpiceProviderInner {
    manifest: Option<KernelManifest>,
    asset_root: Option<PathBuf>,
    profile: String,
    almanac: OnceLock<Result<Almanac, String>>,
}

impl SpiceProvider {
    pub(crate) fn new(
        manifest: Option<KernelManifest>,
        asset_root: Option<PathBuf>,
        profile: Option<String>,
    ) -> Self {
        Self {
            inner: Arc::new(SpiceProviderInner {
                manifest,
                asset_root,
                profile: profile.unwrap_or_else(|| DEFAULT_PROFILE.to_string()),
                almanac: OnceLock::new(),
            }),
        }
    }

    pub(crate) fn state(
        &self,
        object: &ObjectDefinition,
        epoch: &GameTime,
    ) -> Result<StateVector, EphemerisError> {
        let EphemerisSource::SpiceBody { naif_id, .. } = &object.source else {
            return Err(EphemerisError::InvalidObjectDefinition(format!(
                "object {} does not use a SPICE body source",
                object.id
            )));
        };

        let almanac = self.almanac()?;
        let hifitime_epoch = game_time_to_epoch(epoch)?;
        let target = Frame::from_ephem_j2000(*naif_id);
        let state = almanac
            .translate(target, SSB_J2000, hifitime_epoch, None)
            .map_err(|err| map_anise_state_error(object, epoch, err))?;

        Ok(StateVector::new(
            Vec3Km::new(state.radius_km.x, state.radius_km.y, state.radius_km.z),
            Vec3KmPerSec::new(
                state.velocity_km_s.x,
                state.velocity_km_s.y,
                state.velocity_km_s.z,
            ),
            FrameId::SolarSystemBarycentricJ2000,
            epoch.clone(),
            EphemerisQuality::RealKernel,
        ))
    }

    fn almanac(&self) -> Result<&Almanac, EphemerisError> {
        let loaded = self.inner.almanac.get_or_init(|| self.load_almanac());
        loaded
            .as_ref()
            .map_err(|message| EphemerisError::Backend(message.clone()))
    }

    fn load_almanac(&self) -> Result<Almanac, String> {
        let manifest = self.inner.manifest.as_ref().ok_or_else(|| {
            "SPICE body requested but no ephemeris asset manifest is configured".to_string()
        })?;
        let asset_root = self.inner.asset_root.as_ref().ok_or_else(|| {
            "SPICE body requested but no ephemeris asset root is configured".to_string()
        })?;

        let mut almanac = Almanac::default();
        let selected = manifest
            .profile_assets(&self.inner.profile)
            .map_err(|err| err.to_string())?;
        for asset in selected {
            if !matches!(
                asset.asset.kind,
                AssetKind::Spk | AssetKind::Pck | AssetKind::AnisePca
            ) {
                continue;
            }
            match verify_asset(asset, asset_root) {
                Ok(Some(_)) => {
                    let path = resolve_asset_path(asset_root, asset.asset);
                    almanac = almanac
                        .load(path.to_string_lossy().as_ref())
                        .map_err(|err| {
                            format!(
                                "failed to load asset {} at {}: {err}",
                                asset.id,
                                path.display()
                            )
                        })?;
                }
                Ok(None) => {}
                Err(err) => {
                    return Err(format_asset_error(asset, asset_root, err));
                }
            }
        }

        Ok(almanac)
    }
}

impl fmt::Debug for SpiceProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpiceProvider")
            .field("profile", &self.inner.profile)
            .field("asset_root", &self.inner.asset_root)
            .field("has_manifest", &self.inner.manifest.is_some())
            .field("loaded", &self.inner.almanac.get().is_some())
            .finish()
    }
}

fn format_asset_error(
    selected: SelectedAsset<'_>,
    asset_root: &std::path::Path,
    err: EphemerisError,
) -> String {
    let path = resolve_asset_path(asset_root, selected.asset);
    format!(
        "asset {} at {} is unavailable: {err}",
        selected.id,
        path.display()
    )
}

fn game_time_to_epoch(epoch: &GameTime) -> Result<Epoch, EphemerisError> {
    let timestamp = epoch
        .as_utc()
        .format("%Y-%m-%dT%H:%M:%S%.f UTC")
        .to_string();
    timestamp.parse::<Epoch>().map_err(|err| {
        EphemerisError::InvalidObjectDefinition(format!(
            "failed to convert UTC timestamp '{}' for ANISE: {err}",
            epoch
        ))
    })
}

fn map_anise_state_error(
    object: &ObjectDefinition,
    epoch: &GameTime,
    err: anise::ephemerides::EphemerisError,
) -> EphemerisError {
    let message = err.to_string();
    if message.contains("valid from") || message.contains("requested") {
        EphemerisError::OutOfCoverage {
            object: object.id.to_string(),
            epoch: epoch.clone(),
        }
    } else {
        EphemerisError::Backend(format!(
            "failed to resolve SPICE body {}: {message}",
            object.id
        ))
    }
}
