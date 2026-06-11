use serde::{Deserialize, Serialize};

use crate::{GameTime, ObjectId, Vec3Km, Vec3KmPerSec};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum FrameId {
    SolarSystemBarycentricJ2000,
    ParentCenteredInertial(ObjectId),
    BodyFixed(ObjectId),
    Custom(String),
}

impl Default for FrameId {
    fn default() -> Self {
        Self::SolarSystemBarycentricJ2000
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EphemerisQuality {
    RealKernel,
    Fictional,
    Approximate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateVector {
    pub position_km: Vec3Km,
    pub velocity_km_s: Vec3KmPerSec,
    pub frame: FrameId,
    pub epoch: GameTime,
    pub quality: EphemerisQuality,
}

impl StateVector {
    pub fn new(
        position_km: Vec3Km,
        velocity_km_s: Vec3KmPerSec,
        frame: FrameId,
        epoch: GameTime,
        quality: EphemerisQuality,
    ) -> Self {
        Self {
            position_km,
            velocity_km_s,
            frame,
            epoch,
            quality,
        }
    }

    pub fn combine_parent_local(parent: &Self, local: &Self) -> Self {
        Self {
            position_km: parent.position_km + local.position_km,
            velocity_km_s: parent.velocity_km_s + local.velocity_km_s,
            frame: FrameId::SolarSystemBarycentricJ2000,
            epoch: parent.epoch.clone(),
            quality: combine_quality(parent.quality, local.quality),
        }
    }

    pub fn relative_to(&self, observer: &Self) -> Self {
        Self {
            position_km: self.position_km - observer.position_km,
            velocity_km_s: self.velocity_km_s - observer.velocity_km_s,
            frame: FrameId::ParentCenteredInertial(ObjectId::from("observer")),
            epoch: self.epoch.clone(),
            quality: combine_quality(self.quality, observer.quality),
        }
    }
}

fn combine_quality(a: EphemerisQuality, b: EphemerisQuality) -> EphemerisQuality {
    match (a, b) {
        (EphemerisQuality::Approximate, _) | (_, EphemerisQuality::Approximate) => {
            EphemerisQuality::Approximate
        }
        (EphemerisQuality::Fictional, _) | (_, EphemerisQuality::Fictional) => {
            EphemerisQuality::Fictional
        }
        (EphemerisQuality::RealKernel, EphemerisQuality::RealKernel) => {
            EphemerisQuality::RealKernel
        }
    }
}
