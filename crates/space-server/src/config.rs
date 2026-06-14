use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use space_game_ephemeris::{
    EphemerisQuality, FrameId, ObjectRegistry, SolarSystem, SolarSystemBuilder, Vec3Km,
    Vec3KmPerSec,
};

use crate::query::{ObserverLocation, SolarSystemQueryService};

pub const DEFAULT_WS_PATH: &str = "/ws";
pub const DEFAULT_SERVER_LABEL: &str = "127.0.0.1:4000";
pub const DEFAULT_GAME_TIME: &str = "2097-01-01T00:00:00Z";

const DEMO_REGISTRY_TOML: &str = include_str!("../data/demo_registry.toml");

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
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()?;
        Ok(SolarSystemQueryService::new(
            self.server_label.clone(),
            world,
            ObserverLocation {
                label: "demo-observer".to_string(),
                frame: FrameId::SolarSystemBarycentricJ2000,
                position_km: Vec3Km::new(149_597_870.7, 0.0, 0.0),
                velocity_km_s: Vec3KmPerSec::ZERO,
                quality: EphemerisQuality::Fictional,
            },
        ))
    }
}

pub fn demo_world() -> Result<SolarSystem, space_game_ephemeris::EphemerisError> {
    let registry = ObjectRegistry::from_toml_str(DEMO_REGISTRY_TOML)?;
    SolarSystemBuilder::new()
        .object_registry_data(registry)
        .build()
}
