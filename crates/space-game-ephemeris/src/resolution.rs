use std::collections::HashSet;
use std::f64::consts::TAU;

use crate::{
    EphemerisError, EphemerisQuality, EphemerisSource, FrameId, GameTime, InterpolationMode,
    ObjectRegistry, StateVector, Vec3Km, Vec3KmPerSec,
};

pub(crate) fn resolve_global_state(
    registry: &ObjectRegistry,
    object_id: &str,
    epoch: &GameTime,
) -> Result<StateVector, EphemerisError> {
    let mut visited = HashSet::new();
    resolve_global_state_inner(registry, object_id, epoch, &mut visited)
}

fn resolve_global_state_inner(
    registry: &ObjectRegistry,
    object_id: &str,
    epoch: &GameTime,
    visited: &mut HashSet<String>,
) -> Result<StateVector, EphemerisError> {
    if !visited.insert(object_id.to_string()) {
        return Err(EphemerisError::CyclicDependency(object_id.to_string()));
    }

    let object = registry.get(object_id)?;
    let state = match &object.source {
        EphemerisSource::StaticState {
            position_km,
            velocity_km_s,
            frame,
        } => Ok(StateVector::new(
            *position_km,
            *velocity_km_s,
            frame.clone(),
            epoch.clone(),
            EphemerisQuality::Fictional,
        )),
        EphemerisSource::SpiceBody { .. } => Err(EphemerisError::Backend(format!(
            "SPICE provider is not implemented for object {}",
            object.id
        ))),
        EphemerisSource::BodyFixed { parent, .. } => {
            let _parent_state =
                resolve_global_state_inner(registry, parent.as_str(), epoch, visited)?;
            Err(EphemerisError::FrameTransformUnavailable(format!(
                "body-fixed transforms are not implemented for object {}",
                object.id
            )))
        }
        EphemerisSource::FixedOffset {
            parent, offset_km, ..
        } => {
            let parent_state =
                resolve_global_state_inner(registry, parent.as_str(), epoch, visited)?;
            let local = StateVector::new(
                *offset_km,
                Vec3KmPerSec::ZERO,
                FrameId::ParentCenteredInertial(parent.clone()),
                epoch.clone(),
                EphemerisQuality::Fictional,
            );
            Ok(StateVector::combine_parent_local(&parent_state, &local))
        }
        EphemerisSource::CircularOrbit {
            parent,
            radius_km,
            period_seconds,
            inclination_deg,
            raan_deg,
            phase_at_epoch_deg,
            epoch: orbit_epoch,
        } => {
            let parent_state =
                resolve_global_state_inner(registry, parent.as_str(), epoch, visited)?;
            let local = circular_orbit_state(
                *radius_km,
                *period_seconds,
                *inclination_deg,
                *raan_deg,
                *phase_at_epoch_deg,
                orbit_epoch,
                epoch,
                parent.clone(),
            )?;
            Ok(StateVector::combine_parent_local(&parent_state, &local))
        }
        EphemerisSource::SampledTrajectory {
            centre,
            frame,
            samples,
            interpolation,
        } => {
            let centre_state =
                resolve_global_state_inner(registry, centre.as_str(), epoch, visited)?;
            let local =
                sampled_trajectory_state(object_id, frame.clone(), samples, *interpolation, epoch)?;
            Ok(StateVector::combine_parent_local(&centre_state, &local))
        }
    };

    visited.remove(object_id);
    state
}

fn circular_orbit_state(
    radius_km: f64,
    period_seconds: f64,
    inclination_deg: f64,
    raan_deg: f64,
    phase_at_epoch_deg: f64,
    orbit_epoch: &GameTime,
    query_epoch: &GameTime,
    parent: crate::ObjectId,
) -> Result<StateVector, EphemerisError> {
    if radius_km <= 0.0 || period_seconds <= 0.0 {
        return Err(EphemerisError::InvalidObjectDefinition(
            "circular orbit radius and period must be positive".to_string(),
        ));
    }

    let elapsed = query_epoch.seconds_since(orbit_epoch);
    let mean_motion = TAU / period_seconds;
    let theta = phase_at_epoch_deg.to_radians() + elapsed * mean_motion;
    let speed = radius_km * mean_motion;

    let orbital_position = Vec3Km::new(radius_km * theta.cos(), radius_km * theta.sin(), 0.0);
    let orbital_velocity = Vec3KmPerSec::new(-speed * theta.sin(), speed * theta.cos(), 0.0);
    let (position_km, velocity_km_s) = rotate_orbital_plane(
        orbital_position,
        orbital_velocity,
        inclination_deg,
        raan_deg,
    );

    Ok(StateVector::new(
        position_km,
        velocity_km_s,
        FrameId::ParentCenteredInertial(parent),
        query_epoch.clone(),
        EphemerisQuality::Fictional,
    ))
}

fn rotate_orbital_plane(
    position: Vec3Km,
    velocity: Vec3KmPerSec,
    inclination_deg: f64,
    raan_deg: f64,
) -> (Vec3Km, Vec3KmPerSec) {
    let inc = inclination_deg.to_radians();
    let raan = raan_deg.to_radians();

    let rotate_position = |v: Vec3Km| {
        let x1 = v.x;
        let y1 = v.y * inc.cos() - v.z * inc.sin();
        let z1 = v.y * inc.sin() + v.z * inc.cos();

        Vec3Km::new(
            x1 * raan.cos() - y1 * raan.sin(),
            x1 * raan.sin() + y1 * raan.cos(),
            z1,
        )
    };

    let rotate_velocity = |v: Vec3KmPerSec| {
        let x1 = v.x;
        let y1 = v.y * inc.cos() - v.z * inc.sin();
        let z1 = v.y * inc.sin() + v.z * inc.cos();

        Vec3KmPerSec::new(
            x1 * raan.cos() - y1 * raan.sin(),
            x1 * raan.sin() + y1 * raan.cos(),
            z1,
        )
    };

    (rotate_position(position), rotate_velocity(velocity))
}

fn sampled_trajectory_state(
    object_id: &str,
    frame: FrameId,
    samples: &[crate::TrajectorySample],
    interpolation: InterpolationMode,
    epoch: &GameTime,
) -> Result<StateVector, EphemerisError> {
    if interpolation != InterpolationMode::Linear {
        return Err(EphemerisError::InvalidObjectDefinition(
            "only linear sampled trajectory interpolation is supported".to_string(),
        ));
    }

    let first = samples.first().ok_or_else(|| {
        EphemerisError::InvalidObjectDefinition("sampled trajectory must have samples".to_string())
    })?;
    let last = samples.last().expect("checked first sample");
    if epoch < &first.epoch || epoch > &last.epoch {
        return Err(EphemerisError::OutOfCoverage {
            object: object_id.to_string(),
            epoch: epoch.clone(),
        });
    }

    if let Some(sample) = samples.iter().find(|sample| &sample.epoch == epoch) {
        return Ok(StateVector::new(
            sample.position_km,
            sample.velocity_km_s,
            frame,
            epoch.clone(),
            EphemerisQuality::Fictional,
        ));
    }

    for pair in samples.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        if &a.epoch <= epoch && epoch <= &b.epoch {
            let span = b.epoch.seconds_since(&a.epoch);
            if span <= 0.0 {
                return Err(EphemerisError::InvalidObjectDefinition(
                    "sample epochs must be strictly increasing".to_string(),
                ));
            }
            let amount = epoch.seconds_since(&a.epoch) / span;
            return Ok(StateVector::new(
                a.position_km + (b.position_km - a.position_km) * amount,
                a.velocity_km_s + (b.velocity_km_s - a.velocity_km_s) * amount,
                frame,
                epoch.clone(),
                EphemerisQuality::Fictional,
            ));
        }
    }

    Err(EphemerisError::OutOfCoverage {
        object: object_id.to_string(),
        epoch: epoch.clone(),
    })
}
