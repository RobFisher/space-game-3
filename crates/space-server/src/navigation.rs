use std::f64::consts::TAU;

use space_game_ephemeris::{
    EphemerisQuality, FrameId, GameTime, ObjectDefinition, ObjectId, PhysicalProperties,
    StateVector, Vec3Km, Vec3KmPerSec,
};
use thiserror::Error;

pub const STANDARD_GRAVITY_KM_S2: f64 = 0.009_806_65;
pub const DEFAULT_LOW_ORBIT_ALTITUDE_KM: f64 = 400.0;
pub const DEFAULT_ORBIT_ENTRY_DURATION_SECONDS: f64 = 600.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerationInput {
    pub km_s2: f64,
    pub g: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationProfile {
    pub max_acceleration_km_s2: f64,
    pub specific_impulse_seconds: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrivalOrbitRequest {
    Default,
    Low,
    Stationary,
    CustomAltitudeKm(f64),
    CustomRadiusKm(f64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrivalOrbitKind {
    Default,
    Low,
    Stationary,
    Custom,
}

impl ArrivalOrbitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Low => "low",
            Self::Stationary => "stationary",
            Self::Custom => "custom",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedArrivalOrbit {
    pub kind: ArrivalOrbitKind,
    pub radius_km: f64,
    pub altitude_km: Option<f64>,
    pub period_seconds: Option<f64>,
    pub circular_speed_km_s: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationPhase {
    FlightPlan,
    EnteringOrbit,
    Orbiting,
}

impl NavigationPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FlightPlan => "flight_plan",
            Self::EnteringOrbit => "entering_orbit",
            Self::Orbiting => "orbiting",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BodyNavigationConstants {
    pub mean_radius_km: Option<f64>,
    pub gravitational_parameter_km3_s2: Option<f64>,
    pub rotation_period_seconds: Option<f64>,
    pub low_orbit_altitude_km: Option<f64>,
}

#[derive(Debug, Error, PartialEq)]
pub enum NavigationError {
    #[error("invalid acceleration: {0}")]
    InvalidAcceleration(f64),
    #[error("invalid specific impulse: {0}")]
    InvalidSpecificImpulse(f64),
    #[error("invalid orbit value: {0}")]
    InvalidOrbitValue(f64),
    #[error("unsupported stationary orbit for object: {0}")]
    UnsupportedStationaryOrbit(String),
    #[error("missing body radius for object: {0}")]
    MissingBodyRadius(String),
}

impl NavigationProfile {
    pub fn new(
        max_acceleration_km_s2: f64,
        specific_impulse_seconds: Option<f64>,
    ) -> Result<Self, NavigationError> {
        validate_acceleration(max_acceleration_km_s2)?;
        if let Some(value) = specific_impulse_seconds {
            if !value.is_finite() || value <= 0.0 {
                return Err(NavigationError::InvalidSpecificImpulse(value));
            }
        }
        Ok(Self {
            max_acceleration_km_s2,
            specific_impulse_seconds,
        })
    }
}

pub fn parse_acceleration_input(input: &str) -> Result<AccelerationInput, NavigationError> {
    let input = input.trim();
    let lowercase = input.to_ascii_lowercase();
    if let Some(value) = lowercase.strip_suffix('g') {
        let g = value
            .trim()
            .parse::<f64>()
            .map_err(|_| NavigationError::InvalidAcceleration(f64::NAN))?;
        let km_s2 = g * STANDARD_GRAVITY_KM_S2;
        validate_acceleration(km_s2)?;
        Ok(AccelerationInput { km_s2, g: Some(g) })
    } else {
        let km_s2 = input
            .parse::<f64>()
            .map_err(|_| NavigationError::InvalidAcceleration(f64::NAN))?;
        validate_acceleration(km_s2)?;
        Ok(AccelerationInput { km_s2, g: None })
    }
}

pub fn acceleration_to_g(acceleration_km_s2: f64) -> f64 {
    acceleration_km_s2 / STANDARD_GRAVITY_KM_S2
}

pub fn validate_acceleration(acceleration_km_s2: f64) -> Result<(), NavigationError> {
    if acceleration_km_s2.is_finite() && acceleration_km_s2 > 0.0 {
        Ok(())
    } else {
        Err(NavigationError::InvalidAcceleration(acceleration_km_s2))
    }
}

pub fn constants_for_object(object: &ObjectDefinition) -> BodyNavigationConstants {
    let mut constants = object
        .physical
        .as_ref()
        .map(constants_from_physical)
        .unwrap_or_default();

    if let Some(known) = known_body_constants(object.id.as_str()) {
        constants.mean_radius_km = constants.mean_radius_km.or(known.mean_radius_km);
        constants.gravitational_parameter_km3_s2 = constants
            .gravitational_parameter_km3_s2
            .or(known.gravitational_parameter_km3_s2);
        constants.rotation_period_seconds = constants
            .rotation_period_seconds
            .or(known.rotation_period_seconds);
        constants.low_orbit_altitude_km = constants
            .low_orbit_altitude_km
            .or(known.low_orbit_altitude_km);
    }

    constants
}

pub fn resolve_arrival_orbit(
    object_id: &ObjectId,
    constants: BodyNavigationConstants,
    request: &ArrivalOrbitRequest,
    default_radius_km: f64,
) -> Result<ResolvedArrivalOrbit, NavigationError> {
    let (kind, radius_km) = match request {
        ArrivalOrbitRequest::Default => (ArrivalOrbitKind::Default, default_radius_km),
        ArrivalOrbitRequest::Low => {
            let radius = required_body_radius(object_id, constants)?
                + constants
                    .low_orbit_altitude_km
                    .unwrap_or(DEFAULT_LOW_ORBIT_ALTITUDE_KM);
            (ArrivalOrbitKind::Low, radius)
        }
        ArrivalOrbitRequest::Stationary => {
            let mu = constants.gravitational_parameter_km3_s2.ok_or_else(|| {
                NavigationError::UnsupportedStationaryOrbit(object_id.to_string())
            })?;
            let period = constants.rotation_period_seconds.ok_or_else(|| {
                NavigationError::UnsupportedStationaryOrbit(object_id.to_string())
            })?;
            if !mu.is_finite() || mu <= 0.0 || !period.is_finite() || period <= 0.0 {
                return Err(NavigationError::UnsupportedStationaryOrbit(
                    object_id.to_string(),
                ));
            }
            let radius = (mu * (period / TAU).powi(2)).cbrt();
            (ArrivalOrbitKind::Stationary, radius)
        }
        ArrivalOrbitRequest::CustomAltitudeKm(altitude_km) => {
            validate_orbit_value(*altitude_km)?;
            let radius = required_body_radius(object_id, constants)? + altitude_km;
            (ArrivalOrbitKind::Custom, radius)
        }
        ArrivalOrbitRequest::CustomRadiusKm(radius_km) => {
            validate_orbit_value(*radius_km)?;
            (ArrivalOrbitKind::Custom, *radius_km)
        }
    };

    validate_orbit_value(radius_km)?;
    let altitude_km = constants.mean_radius_km.map(|mean| radius_km - mean);
    let (period_seconds, circular_speed_km_s) =
        if let Some(mu) = constants.gravitational_parameter_km3_s2 {
            if mu.is_finite() && mu > 0.0 {
                (
                    Some(TAU * (radius_km.powi(3) / mu).sqrt()),
                    Some((mu / radius_km).sqrt()),
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

    Ok(ResolvedArrivalOrbit {
        kind,
        radius_km,
        altitude_km,
        period_seconds,
        circular_speed_km_s,
    })
}

pub fn circular_orbit_local_state(
    parent_object_id: &ObjectId,
    orbit: &ResolvedArrivalOrbit,
    at: GameTime,
    epoch: &GameTime,
    phase_radians: f64,
    quality: EphemerisQuality,
) -> StateVector {
    let elapsed_seconds = at.seconds_since(epoch);
    let angular_rate = orbit
        .period_seconds
        .filter(|period| *period > 0.0)
        .map(|period| TAU / period)
        .unwrap_or(0.0);
    let theta = phase_radians + angular_rate * elapsed_seconds;
    let (sin_theta, cos_theta) = theta.sin_cos();
    let speed = orbit
        .circular_speed_km_s
        .unwrap_or(orbit.radius_km * angular_rate);
    StateVector::new(
        Vec3Km::new(
            orbit.radius_km * cos_theta,
            orbit.radius_km * sin_theta,
            0.0,
        ),
        Vec3KmPerSec::new(-speed * sin_theta, speed * cos_theta, 0.0),
        FrameId::ParentCenteredInertial(parent_object_id.clone()),
        at,
        quality,
    )
}

pub fn orbit_insertion_state(
    parent_state: &StateVector,
    parent_object_id: &ObjectId,
    orbit: &ResolvedArrivalOrbit,
    at: GameTime,
    quality: EphemerisQuality,
) -> StateVector {
    let local = circular_orbit_local_state(
        parent_object_id,
        orbit,
        at,
        &parent_state.epoch,
        0.0,
        quality,
    );
    StateVector::combine_parent_local(parent_state, &local)
}

pub fn transfer_duration_seconds(distance_km: f64, acceleration_km_s2: f64) -> f64 {
    if distance_km <= 0.0 {
        0.0
    } else {
        2.0 * (distance_km / acceleration_km_s2).sqrt()
    }
}

pub fn transfer_state_between(
    origin_state: &StateVector,
    target_state: &StateVector,
    departure_time: &GameTime,
    arrival_time: &GameTime,
    duration_seconds: f64,
    at: GameTime,
) -> StateVector {
    if at <= *departure_time {
        let mut state = origin_state.clone();
        state.epoch = at;
        return state;
    }
    if at >= *arrival_time || duration_seconds <= 0.0 {
        let mut state = target_state.clone();
        state.epoch = at;
        return state;
    }

    let normalized = (at.seconds_since(departure_time) / duration_seconds).clamp(0.0, 1.0);
    let progress = ease_in_out_accel_decel(normalized);
    let velocity_factor = ease_in_out_accel_decel_derivative(normalized) / duration_seconds;
    let delta = target_state.position_km - origin_state.position_km;
    StateVector::new(
        origin_state.position_km + delta * progress,
        Vec3KmPerSec::new(
            delta.x * velocity_factor,
            delta.y * velocity_factor,
            delta.z * velocity_factor,
        ),
        origin_state.frame.clone(),
        at,
        target_state.quality,
    )
}

pub fn blend_states(
    from: &StateVector,
    to: &StateVector,
    normalized: f64,
    at: GameTime,
) -> StateVector {
    let progress = normalized.clamp(0.0, 1.0);
    let position_delta = to.position_km - from.position_km;
    let velocity_delta = to.velocity_km_s - from.velocity_km_s;
    StateVector::new(
        from.position_km + position_delta * progress,
        from.velocity_km_s + velocity_delta * progress,
        to.frame.clone(),
        at,
        to.quality,
    )
}

fn constants_from_physical(physical: &PhysicalProperties) -> BodyNavigationConstants {
    BodyNavigationConstants {
        mean_radius_km: physical.mean_radius_km,
        gravitational_parameter_km3_s2: physical.gravitational_parameter_km3_s2,
        rotation_period_seconds: physical.rotation_period_seconds,
        low_orbit_altitude_km: None,
    }
}

fn known_body_constants(object_id: &str) -> Option<BodyNavigationConstants> {
    let (mean_radius_km, gravitational_parameter_km3_s2, rotation_period_seconds) = match object_id
    {
        "mercury" => (2_439.7, 22_032.080, 5_067_031.7),
        "venus" => (6_051.8, 324_858.592, 20_996_731.0),
        "earth" => (6_371.0, 398_600.435_436, 86_164.090_5),
        "moon" => (1_737.4, 4_902.800_066, 2_360_591.5),
        "mars" => (3_389.5, 42_828.375_214, 88_642.663),
        "jupiter" => (69_911.0, 126_686_534.0, 35_729.7),
        "saturn" => (58_232.0, 37_931_207.8, 38_036.0),
        "uranus" => (25_362.0, 5_793_951.3, 62_064.0),
        "neptune" => (24_622.0, 6_835_099.5, 57_996.0),
        "pluto" => (1_188.3, 869.61, 551_856.7),
        _ => return None,
    };
    Some(BodyNavigationConstants {
        mean_radius_km: Some(mean_radius_km),
        gravitational_parameter_km3_s2: Some(gravitational_parameter_km3_s2),
        rotation_period_seconds: Some(rotation_period_seconds),
        low_orbit_altitude_km: Some(DEFAULT_LOW_ORBIT_ALTITUDE_KM),
    })
}

fn required_body_radius(
    object_id: &ObjectId,
    constants: BodyNavigationConstants,
) -> Result<f64, NavigationError> {
    constants
        .mean_radius_km
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| NavigationError::MissingBodyRadius(object_id.to_string()))
}

fn validate_orbit_value(value: f64) -> Result<(), NavigationError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(NavigationError::InvalidOrbitValue(value))
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

    fn approx_eq(left: f64, right: f64, tolerance: f64) {
        assert!(
            (left - right).abs() <= tolerance,
            "{left} not within {tolerance} of {right}"
        );
    }

    #[test]
    fn parses_acceleration_units() {
        let raw = parse_acceleration_input("0.02").unwrap();
        assert_eq!(raw.km_s2, 0.02);
        assert_eq!(raw.g, None);

        let g = parse_acceleration_input("0.5g").unwrap();
        approx_eq(g.km_s2, 0.004_903_325, 1e-12);
        assert_eq!(g.g, Some(0.5));
        approx_eq(acceleration_to_g(g.km_s2), 0.5, 1e-12);
    }

    #[test]
    fn rejects_invalid_navigation_values() {
        assert!(parse_acceleration_input("0g").is_err());
        assert!(parse_acceleration_input("bad").is_err());
        assert!(NavigationProfile::new(0.01, Some(300.0)).is_ok());
        assert_eq!(
            NavigationProfile::new(0.0, None),
            Err(NavigationError::InvalidAcceleration(0.0))
        );
        assert_eq!(
            NavigationProfile::new(0.01, Some(0.0)),
            Err(NavigationError::InvalidSpecificImpulse(0.0))
        );
    }

    #[test]
    fn resolves_low_orbit_with_known_body_constants() {
        let orbit = resolve_arrival_orbit(
            &ObjectId::from("mars"),
            known_body_constants("mars").unwrap(),
            &ArrivalOrbitRequest::Low,
            42_164.0,
        )
        .unwrap();

        assert_eq!(orbit.kind, ArrivalOrbitKind::Low);
        approx_eq(orbit.radius_km, 3_789.5, 1e-9);
        approx_eq(orbit.altitude_km.unwrap(), 400.0, 1e-9);
        assert!(orbit.period_seconds.unwrap() > 0.0);
        assert!(orbit.circular_speed_km_s.unwrap() > 0.0);
    }

    #[test]
    fn resolves_stationary_orbit_when_constants_are_available() {
        let orbit = resolve_arrival_orbit(
            &ObjectId::from("earth"),
            known_body_constants("earth").unwrap(),
            &ArrivalOrbitRequest::Stationary,
            42_164.0,
        )
        .unwrap();

        assert_eq!(orbit.kind, ArrivalOrbitKind::Stationary);
        approx_eq(orbit.period_seconds.unwrap(), 86_164.090_5, 1e-6);
        approx_eq(orbit.radius_km, 42_164.0, 1.0);
    }

    #[test]
    fn rejects_stationary_orbit_without_constants() {
        assert_eq!(
            resolve_arrival_orbit(
                &ObjectId::from("demo-station"),
                BodyNavigationConstants::default(),
                &ArrivalOrbitRequest::Stationary,
                42_164.0,
            ),
            Err(NavigationError::UnsupportedStationaryOrbit(
                "demo-station".to_string()
            ))
        );
    }

    #[test]
    fn resolves_custom_orbits() {
        let constants = known_body_constants("earth").unwrap();
        let altitude = resolve_arrival_orbit(
            &ObjectId::from("earth"),
            constants,
            &ArrivalOrbitRequest::CustomAltitudeKm(1_000.0),
            42_164.0,
        )
        .unwrap();
        approx_eq(altitude.radius_km, 7_371.0, 1e-9);

        let radius = resolve_arrival_orbit(
            &ObjectId::from("earth"),
            constants,
            &ArrivalOrbitRequest::CustomRadiusKm(10_000.0),
            42_164.0,
        )
        .unwrap();
        approx_eq(radius.radius_km, 10_000.0, 1e-9);
    }

    #[test]
    fn transfer_duration_uses_acceleration() {
        assert_eq!(transfer_duration_seconds(0.0, 0.02), 0.0);
        assert_eq!(transfer_duration_seconds(8.0, 2.0), 4.0);
    }
}
