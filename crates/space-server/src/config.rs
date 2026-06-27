use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use space_game_ephemeris::{
    resolved_asset_root, KernelManifest, ObjectRegistry, SolarSystem, SolarSystemBuilder,
};

use crate::query::SolarSystemQueryService;

pub const DEFAULT_WS_PATH: &str = "/ws";
pub const DEFAULT_SERVER_LABEL: &str = "127.0.0.1:4000";
pub const DEFAULT_GAME_TIME: &str = "2097-01-01T00:00:00Z";

const DEMO_REGISTRY_TOML: &str = include_str!("../data/demo_registry.toml");
const EPHEMERIS_MANIFEST_TOML: &str = include_str!("../../../data/ephemeris/manifest.toml");
const DEFAULT_EPHEMERIS_PROFILE: &str = "minimal";

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub ws_path: String,
    pub server_label: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);
        Self {
            bind_addr,
            ws_path: DEFAULT_WS_PATH.to_string(),
            server_label: DEFAULT_SERVER_LABEL.to_string(),
        }
    }
}

impl ServerConfig {
    pub fn query_service(
        &self,
    ) -> Result<SolarSystemQueryService, space_game_ephemeris::EphemerisError> {
        let registry = ObjectRegistry::from_toml_str(DEMO_REGISTRY_TOML)?;
        let manifest = KernelManifest::from_toml_str(EPHEMERIS_MANIFEST_TOML)?;
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .kernel_manifest_data(manifest)
            .kernel_dir(resolved_asset_root())
            .asset_profile(DEFAULT_EPHEMERIS_PROFILE)
            .build()?;
        Ok(SolarSystemQueryService::new(
            self.server_label.clone(),
            world,
        ))
    }
}

pub fn demo_world() -> Result<SolarSystem, space_game_ephemeris::EphemerisError> {
    let registry = ObjectRegistry::from_toml_str(DEMO_REGISTRY_TOML)?;
    let manifest = KernelManifest::from_toml_str(EPHEMERIS_MANIFEST_TOML)?;
    SolarSystemBuilder::new()
        .object_registry_data(registry)
        .kernel_manifest_data(manifest)
        .kernel_dir(resolved_asset_root())
        .asset_profile(DEFAULT_EPHEMERIS_PROFILE)
        .build()
}
