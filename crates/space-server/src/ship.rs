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
pub const DEFAULT_FLIGHT_ACCELERATION_KM_S2: f64 = 0.02;

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerShip {
    id: String,
    display_name: String,
    motion: ShipMotionMode,
    active_flight_plan: Option<FlightPlan>,
    next_flight_plan_number: u64,
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
pub struct FlightPlan {
    pub plan_id: String,
    pub ship_id: String,
    pub origin_state: StateVector,
    pub target: FlightPlanTarget,
    pub departure_time: GameTime,
    pub arrival_time: GameTime,
    pub duration_seconds: f64,
    pub acceleration_km_s2: f64,
    pub status: FlightPlanStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlightPlanTarget {
    Object {
        object_id: ObjectId,
        display_name: String,
        arrival_state: StateVector,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlightPlanStatus {
    Active,
    Completed,
    Cancelled,
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

#[derive(Debug, Error, PartialEq)]
pub enum FlightPlanError {
    #[error("invalid acceleration: {0}")]
    InvalidAcceleration(f64),
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
            active_flight_plan: None,
            next_flight_plan_number: 1,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn motion_mode_label(&self) -> &'static str {
        if self.active_flight_plan.is_some() {
            "flight_plan"
        } else {
            match self.motion {
                ShipMotionMode::Orbiting(_) => "orbiting",
            }
        }
    }

    pub fn orbit_parent_id(&self) -> Option<&ObjectId> {
        if let Some(plan) = &self.active_flight_plan {
            Some(plan.target.object_id())
        } else {
            match &self.motion {
                ShipMotionMode::Orbiting(motion) => Some(&motion.parent_object_id),
            }
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

    pub fn active_flight_plan(&self) -> Option<&FlightPlan> {
        self.active_flight_plan.as_ref()
    }

    pub fn register_flight_plan(
        &mut self,
        origin_state: StateVector,
        target: FlightPlanTarget,
        departure_time: GameTime,
        arrival_time: GameTime,
        duration_seconds: f64,
        acceleration_km_s2: f64,
    ) -> Result<FlightPlan, FlightPlanError> {
        validate_acceleration(acceleration_km_s2)?;
        let plan = FlightPlan {
            plan_id: format!("flight-{}", self.next_flight_plan_number),
            ship_id: self.id.clone(),
            origin_state,
            target,
            departure_time,
            arrival_time,
            duration_seconds,
            acceleration_km_s2,
            status: FlightPlanStatus::Active,
        };
        self.next_flight_plan_number += 1;
        self.active_flight_plan = Some(plan.clone());
        Ok(plan)
    }

    pub fn cancel_active_flight_plan(&mut self) -> Option<FlightPlan> {
        self.active_flight_plan.take().map(|mut plan| {
            plan.status = FlightPlanStatus::Cancelled;
            plan
        })
    }

    pub fn resolve_flight_plan_state(&self, plan: &FlightPlan, at: GameTime) -> ResolvedShipState {
        ResolvedShipState {
            ship_id: self.id.clone(),
            display_name: self.display_name.clone(),
            motion_mode: "flight_plan".to_string(),
            parent_object_id: Some(plan.target.object_id().clone()),
            state: plan.transfer_state_at(at),
        }
    }

    pub fn resolve_arrived_orbiting_state(
        &self,
        plan: &FlightPlan,
        at: GameTime,
        parent_state: StateVector,
    ) -> ResolvedShipState {
        let motion = OrbitingMotion {
            parent_object_id: plan.target.object_id().clone(),
            radius_km: DEFAULT_SHIP_ORBIT_RADIUS_KM,
            period_seconds: DEFAULT_SHIP_ORBIT_PERIOD_SECONDS,
            phase_radians: 0.0,
            epoch: plan.arrival_time.clone(),
            quality: EphemerisQuality::Fictional,
        };
        let local_state = motion.local_state_at(at);
        ResolvedShipState {
            ship_id: self.id.clone(),
            display_name: self.display_name.clone(),
            motion_mode: "orbiting".to_string(),
            parent_object_id: Some(motion.parent_object_id),
            state: StateVector::combine_parent_local(&parent_state, &local_state),
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

impl FlightPlan {
    pub fn effective_status_at(&self, at: &GameTime) -> FlightPlanStatus {
        if self.status == FlightPlanStatus::Active && at >= &self.arrival_time {
            FlightPlanStatus::Completed
        } else {
            self.status
        }
    }

    pub fn target_state(&self) -> &StateVector {
        match &self.target {
            FlightPlanTarget::Object { arrival_state, .. } => arrival_state,
        }
    }

    fn transfer_state_at(&self, at: GameTime) -> StateVector {
        if at <= self.departure_time {
            let mut state = self.origin_state.clone();
            state.epoch = at;
            return state;
        }
        if at >= self.arrival_time || self.duration_seconds <= 0.0 {
            let mut state = self.target_state().clone();
            state.epoch = at;
            return state;
        }

        let normalized =
            (at.seconds_since(&self.departure_time) / self.duration_seconds).clamp(0.0, 1.0);
        let progress = ease_in_out_accel_decel(normalized);
        let velocity_factor =
            ease_in_out_accel_decel_derivative(normalized) / self.duration_seconds;
        let delta = self.target_state().position_km - self.origin_state.position_km;
        StateVector::new(
            self.origin_state.position_km + delta * progress,
            Vec3KmPerSec::new(
                delta.x * velocity_factor,
                delta.y * velocity_factor,
                delta.z * velocity_factor,
            ),
            self.origin_state.frame.clone(),
            at,
            self.target_state().quality,
        )
    }
}

impl FlightPlanTarget {
    pub fn object_id(&self) -> &ObjectId {
        match self {
            Self::Object { object_id, .. } => object_id,
        }
    }
}

pub fn validate_acceleration(acceleration_km_s2: f64) -> Result<(), FlightPlanError> {
    if acceleration_km_s2.is_finite() && acceleration_km_s2 > 0.0 {
        Ok(())
    } else {
        Err(FlightPlanError::InvalidAcceleration(acceleration_km_s2))
    }
}

pub fn transfer_duration_seconds(distance_km: f64, acceleration_km_s2: f64) -> f64 {
    if distance_km <= 0.0 {
        0.0
    } else {
        2.0 * (distance_km / acceleration_km_s2).sqrt()
    }
}

fn ease_in_out_accel_decel(normalized: f64) -> f64 {
    if normalized < 0.5 {
        2.0 * normalized * normalized
    } else {
        1.0 - 2.0 * (1.0 - normalized) * (1.0 - normalized)
    }
}

fn ease_in_out_accel_decel_derivative(normalized: f64) -> f64 {
    if normalized < 0.5 {
        4.0 * normalized
    } else {
        4.0 * (1.0 - normalized)
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
    fn rejects_invalid_flight_acceleration() {
        assert!(validate_acceleration(0.02).is_ok());
        assert_eq!(
            validate_acceleration(0.0),
            Err(FlightPlanError::InvalidAcceleration(0.0))
        );
        assert!(validate_acceleration(f64::NAN).is_err());
    }

    #[test]
    fn transfer_duration_uses_acceleration() {
        assert_eq!(transfer_duration_seconds(0.0, 0.02), 0.0);
        assert_eq!(transfer_duration_seconds(8.0, 2.0), 4.0);
    }

    #[test]
    fn flight_plan_interpolates_between_origin_and_target() {
        let start = epoch();
        let arrival = start.add_seconds(10.0);
        let plan = FlightPlan {
            plan_id: "flight-1".to_string(),
            ship_id: DEFAULT_SHIP_ID.to_string(),
            origin_state: StateVector::new(
                Vec3Km::new(0.0, 0.0, 0.0),
                Vec3KmPerSec::ZERO,
                FrameId::SolarSystemBarycentricJ2000,
                start.clone(),
                EphemerisQuality::Fictional,
            ),
            target: FlightPlanTarget::Object {
                object_id: ObjectId::from("mars"),
                display_name: "Mars".to_string(),
                arrival_state: StateVector::new(
                    Vec3Km::new(100.0, 0.0, 0.0),
                    Vec3KmPerSec::ZERO,
                    FrameId::SolarSystemBarycentricJ2000,
                    arrival.clone(),
                    EphemerisQuality::Fictional,
                ),
            },
            departure_time: start.clone(),
            arrival_time: arrival.clone(),
            duration_seconds: 10.0,
            acceleration_km_s2: 2.0,
            status: FlightPlanStatus::Active,
        };

        let midpoint = plan.transfer_state_at(start.add_seconds(5.0));
        assert_eq!(midpoint.position_km, Vec3Km::new(50.0, 0.0, 0.0));
        assert_eq!(midpoint.epoch, start.add_seconds(5.0));

        let arrived = plan.transfer_state_at(arrival.clone());
        assert_eq!(arrived.position_km, Vec3Km::new(100.0, 0.0, 0.0));
        assert_eq!(arrived.epoch, arrival);
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
