use std::f64::consts::TAU;

use space_game_ephemeris::{
    EphemerisQuality, FrameId, GameTime, ObjectId, StateVector, Vec3Km, Vec3KmPerSec,
};
use thiserror::Error;

pub const DEFAULT_SHIP_ID: &str = "player-ship";
pub const DEFAULT_SHIP_NAME: &str = "Wayfarer";
pub const DEFAULT_SHIP_PARENT_ID: &str = "earth";
pub const DEFAULT_SHIP_ORBIT_RADIUS_KM: f64 = 42_164.0;
pub const DEFAULT_SHIP_ORBIT_PERIOD_SECONDS: f64 = 86_164.0;

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerShip {
    id: String,
    display_name: String,
    motion: ShipMotionMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShipMotionMode {
    Orbiting(OrbitingMotion),
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrbitingMotion {
    pub parent_object_id: ObjectId,
    pub radius_km: f64,
    pub period_seconds: f64,
    pub phase_radians: f64,
    pub epoch: GameTime,
    pub quality: EphemerisQuality,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedShipState {
    pub ship_id: String,
    pub display_name: String,
    pub motion_mode: String,
    pub parent_object_id: Option<ObjectId>,
    pub state: StateVector,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShipNameError {
    #[error("ship name cannot be empty")]
    Empty,
}

impl PlayerShip {
    pub fn default_near_earth(epoch: GameTime) -> Self {
        Self {
            id: DEFAULT_SHIP_ID.to_string(),
            display_name: DEFAULT_SHIP_NAME.to_string(),
            motion: ShipMotionMode::Orbiting(OrbitingMotion {
                parent_object_id: ObjectId::from(DEFAULT_SHIP_PARENT_ID),
                radius_km: DEFAULT_SHIP_ORBIT_RADIUS_KM,
                period_seconds: DEFAULT_SHIP_ORBIT_PERIOD_SECONDS,
                phase_radians: 0.0,
                epoch,
                quality: EphemerisQuality::Fictional,
            }),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn motion_mode_label(&self) -> &'static str {
        match self.motion {
            ShipMotionMode::Orbiting(_) => "orbiting",
        }
    }

    pub fn orbit_parent_id(&self) -> Option<&ObjectId> {
        match &self.motion {
            ShipMotionMode::Orbiting(motion) => Some(&motion.parent_object_id),
        }
    }

    pub fn rename(&mut self, display_name: &str) -> Result<(), ShipNameError> {
        let display_name = display_name.trim();
        if display_name.is_empty() {
            return Err(ShipNameError::Empty);
        }
        self.display_name = display_name.to_string();
        Ok(())
    }

    pub fn resolve_orbiting_state(
        &self,
        at: GameTime,
        parent_state: StateVector,
    ) -> ResolvedShipState {
        match &self.motion {
            ShipMotionMode::Orbiting(motion) => {
                let local_state = motion.local_state_at(at);
                let state = StateVector::combine_parent_local(&parent_state, &local_state);
                ResolvedShipState {
                    ship_id: self.id.clone(),
                    display_name: self.display_name.clone(),
                    motion_mode: self.motion_mode_label().to_string(),
                    parent_object_id: Some(motion.parent_object_id.clone()),
                    state,
                }
            }
        }
    }
}

impl OrbitingMotion {
    fn local_state_at(&self, at: GameTime) -> StateVector {
        let elapsed_seconds = at.seconds_since(&self.epoch);
        let angular_rate = TAU / self.period_seconds;
        let theta = self.phase_radians + angular_rate * elapsed_seconds;
        let (sin_theta, cos_theta) = theta.sin_cos();
        StateVector::new(
            Vec3Km::new(self.radius_km * cos_theta, self.radius_km * sin_theta, 0.0),
            Vec3KmPerSec::new(
                -self.radius_km * angular_rate * sin_theta,
                self.radius_km * angular_rate * cos_theta,
                0.0,
            ),
            FrameId::ParentCenteredInertial(self.parent_object_id.clone()),
            at,
            self.quality,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn epoch() -> GameTime {
        GameTime::from_utc_iso8601("2097-01-01T00:00:00Z").unwrap()
    }

    fn parent_state(at: GameTime) -> StateVector {
        StateVector::new(
            Vec3Km::new(10.0, 20.0, 30.0),
            Vec3KmPerSec::new(1.0, 2.0, 3.0),
            FrameId::SolarSystemBarycentricJ2000,
            at,
            EphemerisQuality::Fictional,
        )
    }

    #[test]
    fn default_ship_has_stable_identity_and_orbit_metadata() {
        let ship = PlayerShip::default_near_earth(epoch());

        assert_eq!(ship.id(), DEFAULT_SHIP_ID);
        assert_eq!(ship.display_name(), DEFAULT_SHIP_NAME);
        assert_eq!(ship.motion_mode_label(), "orbiting");
        assert_eq!(
            ship.orbit_parent_id().map(ObjectId::as_str),
            Some(DEFAULT_SHIP_PARENT_ID)
        );
    }

    #[test]
    fn orbiting_state_resolves_from_parent_state() {
        let at = epoch();
        let ship = PlayerShip::default_near_earth(at.clone());
        let state = ship.resolve_orbiting_state(at.clone(), parent_state(at));

        assert_eq!(state.ship_id, DEFAULT_SHIP_ID);
        assert_eq!(state.display_name, DEFAULT_SHIP_NAME);
        assert_eq!(state.motion_mode, "orbiting");
        assert_eq!(
            state.parent_object_id.as_ref().map(ObjectId::as_str),
            Some("earth")
        );
        assert_eq!(state.state.frame, FrameId::SolarSystemBarycentricJ2000);
        assert_eq!(state.state.epoch, epoch());
        assert_eq!(
            state.state.position_km,
            Vec3Km::new(10.0 + DEFAULT_SHIP_ORBIT_RADIUS_KM, 20.0, 30.0)
        );
    }

    #[test]
    fn orbiting_state_changes_with_time() {
        let start = epoch();
        let later = start.add_seconds(DEFAULT_SHIP_ORBIT_PERIOD_SECONDS / 4.0);
        let ship = PlayerShip::default_near_earth(start.clone());

        let first = ship.resolve_orbiting_state(start.clone(), parent_state(start));
        let second = ship.resolve_orbiting_state(later.clone(), parent_state(later.clone()));

        assert_eq!(second.state.epoch, later);
        assert_ne!(first.state.position_km, second.state.position_km);
    }

    #[test]
    fn renaming_trims_name_and_preserves_identity() {
        let mut ship = PlayerShip::default_near_earth(epoch());
        ship.rename("  Wayfarer II  ").unwrap();

        assert_eq!(ship.id(), DEFAULT_SHIP_ID);
        assert_eq!(ship.display_name(), "Wayfarer II");
    }

    #[test]
    fn renaming_rejects_empty_name() {
        let mut ship = PlayerShip::default_near_earth(epoch());

        assert_eq!(ship.rename("   "), Err(ShipNameError::Empty));
        assert_eq!(ship.display_name(), DEFAULT_SHIP_NAME);
    }
}
