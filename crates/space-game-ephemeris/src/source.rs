use serde::{Deserialize, Serialize};

use crate::{FrameId, GameTime, ObjectId, Vec3Km, Vec3KmPerSec};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EphemerisSource {
    StaticState {
        position_km: Vec3Km,
        velocity_km_s: Vec3KmPerSec,
        #[serde(default)]
        frame: FrameId,
    },
    SpiceBody {
        naif_id: i32,
        name: Option<String>,
        default_observer_naif_id: Option<i32>,
    },
    BodyFixed {
        parent: ObjectId,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_km: f64,
    },
    CircularOrbit {
        parent: ObjectId,
        radius_km: f64,
        period_seconds: f64,
        inclination_deg: f64,
        raan_deg: f64,
        phase_at_epoch_deg: f64,
        epoch: GameTime,
    },
    SampledTrajectory {
        centre: ObjectId,
        frame: FrameId,
        samples: Vec<TrajectorySample>,
        interpolation: InterpolationMode,
    },
    FixedOffset {
        parent: ObjectId,
        offset_km: Vec3Km,
        frame: FrameId,
    },
}

impl EphemerisSource {
    pub(crate) fn parent_ids(&self) -> Vec<&ObjectId> {
        match self {
            Self::StaticState { .. } | Self::SpiceBody { .. } => Vec::new(),
            Self::BodyFixed { parent, .. }
            | Self::CircularOrbit { parent, .. }
            | Self::FixedOffset { parent, .. } => vec![parent],
            Self::SampledTrajectory { centre, .. } => vec![centre],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrajectorySample {
    pub epoch: GameTime,
    pub position_km: Vec3Km,
    pub velocity_km_s: Vec3KmPerSec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationMode {
    Linear,
    CubicHermite,
}
