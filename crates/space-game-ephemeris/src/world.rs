use std::path::PathBuf;

use crate::{
    providers::spice::SpiceProvider, resolution, EphemerisError, FrameId, GameTime, KernelManifest,
    ObjectDefinition, ObjectRegistry, ObjectSummary, StateVector, Vec3Km,
};

const SPEED_OF_LIGHT_KM_S: f64 = 299_792.458;

#[derive(Clone, Debug, Default)]
pub struct SolarSystemBuilder {
    kernel_dir: Option<PathBuf>,
    manifest: Option<KernelManifest>,
    manifest_path: Option<PathBuf>,
    object_registry_path: Option<PathBuf>,
    asset_profile: Option<String>,
    allow_approximate_fallbacks: bool,
    registry: Option<ObjectRegistry>,
}

impl SolarSystemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn kernel_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.kernel_dir = Some(path.into());
        self
    }

    pub fn kernel_manifest(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest_path = Some(path.into());
        self
    }

    pub fn kernel_manifest_data(mut self, manifest: KernelManifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn asset_profile(mut self, profile: impl Into<String>) -> Self {
        self.asset_profile = Some(profile.into());
        self
    }

    pub fn object_registry(mut self, path: impl Into<PathBuf>) -> Self {
        self.object_registry_path = Some(path.into());
        self
    }

    pub fn object_registry_data(mut self, registry: ObjectRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn allow_approximate_fallbacks(mut self, enabled: bool) -> Self {
        self.allow_approximate_fallbacks = enabled;
        self
    }

    pub fn build(self) -> Result<SolarSystem, EphemerisError> {
        let registry = match (self.registry, self.object_registry_path) {
            (Some(registry), _) => registry,
            (None, Some(path)) => ObjectRegistry::from_toml_path(path)?,
            (None, None) => ObjectRegistry::default(),
        };

        let manifest = match (self.manifest, self.manifest_path) {
            (Some(manifest), _) => Some(manifest),
            (None, Some(path)) => Some(KernelManifest::from_toml_path(path)?),
            (None, None) => None,
        };

        let spice_provider = SpiceProvider::new(
            manifest.clone(),
            self.kernel_dir.clone(),
            self.asset_profile.clone(),
        );

        Ok(SolarSystem {
            registry,
            manifest,
            kernel_dir: self.kernel_dir,
            asset_profile: self.asset_profile,
            allow_approximate_fallbacks: self.allow_approximate_fallbacks,
            spice_provider,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SolarSystem {
    registry: ObjectRegistry,
    manifest: Option<KernelManifest>,
    kernel_dir: Option<PathBuf>,
    asset_profile: Option<String>,
    allow_approximate_fallbacks: bool,
    spice_provider: SpiceProvider,
}

impl SolarSystem {
    pub fn state(
        &self,
        object: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<StateVector, EphemerisError> {
        resolution::resolve_global_state_with_spice(
            &self.registry,
            object.as_ref(),
            &epoch,
            &self.spice_provider,
        )
    }

    pub fn state_relative_to(
        &self,
        target: impl AsRef<str>,
        observer: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<StateVector, EphemerisError> {
        let target = self.state(target, epoch.clone())?;
        let observer = self.state(observer, epoch)?;
        Ok(target.relative_to(&observer))
    }

    pub fn position(
        &self,
        object: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<Vec3Km, EphemerisError> {
        Ok(self.state(object, epoch)?.position_km)
    }

    pub fn distance(
        &self,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<f64, EphemerisError> {
        let a = self.position(a, epoch.clone())?;
        let b = self.position(b, epoch)?;
        Ok(a.distance(b))
    }

    pub fn light_time_seconds(
        &self,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
        epoch: GameTime,
    ) -> Result<f64, EphemerisError> {
        Ok(self.distance(a, b, epoch)? / SPEED_OF_LIGHT_KM_S)
    }

    pub fn list_objects(&self) -> Vec<ObjectSummary> {
        self.registry.list_objects()
    }

    pub fn object_metadata(
        &self,
        object: impl AsRef<str>,
    ) -> Result<ObjectDefinition, EphemerisError> {
        Ok(self.registry.get(object)?.clone())
    }

    pub fn manifest(&self) -> Option<&KernelManifest> {
        self.manifest.as_ref()
    }

    pub fn kernel_dir(&self) -> Option<&PathBuf> {
        self.kernel_dir.as_ref()
    }

    pub fn asset_profile(&self) -> Option<&str> {
        self.asset_profile.as_deref()
    }

    pub fn allow_approximate_fallbacks(&self) -> bool {
        self.allow_approximate_fallbacks
    }

    pub fn default_frame(&self) -> FrameId {
        FrameId::SolarSystemBarycentricJ2000
    }
}
