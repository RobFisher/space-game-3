use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::{
    EphemerisError, EphemerisSource, InterpolationMode, ObjectDefinition, ObjectId, ObjectSummary,
};

#[derive(Clone, Debug, Default)]
pub struct ObjectRegistry {
    objects: HashMap<ObjectId, ObjectDefinition>,
}

impl ObjectRegistry {
    pub fn new(objects: Vec<ObjectDefinition>) -> Result<Self, EphemerisError> {
        let mut registry = Self {
            objects: HashMap::with_capacity(objects.len()),
        };

        for object in objects {
            validate_object_definition(&object)?;
            if registry.objects.insert(object.id.clone(), object).is_some() {
                return Err(EphemerisError::InvalidObjectDefinition(
                    "duplicate object id".to_string(),
                ));
            }
        }

        registry.validate_parent_references()?;
        Ok(registry)
    }

    pub fn from_toml_str(input: &str) -> Result<Self, EphemerisError> {
        let raw: RegistryFile = toml::from_str(input)?;
        Self::new(raw.objects)
    }

    pub fn from_toml_path(path: impl AsRef<Path>) -> Result<Self, EphemerisError> {
        let input = std::fs::read_to_string(path)?;
        Self::from_toml_str(&input)
    }

    pub fn get(&self, id: impl AsRef<str>) -> Result<&ObjectDefinition, EphemerisError> {
        let id = id.as_ref();
        self.objects
            .get(&ObjectId::from(id))
            .ok_or_else(|| EphemerisError::UnknownObject(id.to_string()))
    }

    pub fn list_objects(&self) -> Vec<ObjectSummary> {
        let mut objects: Vec<_> = self.objects.values().map(ObjectSummary::from).collect();
        objects.sort_by(|a, b| a.id.cmp(&b.id));
        objects
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    fn validate_parent_references(&self) -> Result<(), EphemerisError> {
        for object in self.objects.values() {
            for parent in object.source.parent_ids() {
                if !self.objects.contains_key(parent) {
                    return Err(EphemerisError::UnknownObject(parent.to_string()));
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct RegistryFile {
    objects: Vec<ObjectDefinition>,
}

fn validate_object_definition(object: &ObjectDefinition) -> Result<(), EphemerisError> {
    if object.id.as_str().trim().is_empty() {
        return invalid("object id must not be empty");
    }
    if object.name.trim().is_empty() {
        return invalid(format!("object {} name must not be empty", object.id));
    }

    match &object.source {
        EphemerisSource::StaticState {
            position_km,
            velocity_km_s,
            ..
        } => {
            if position_km.is_finite() && velocity_km_s.is_finite() {
                Ok(())
            } else {
                invalid(format!(
                    "static state {} contains non-finite values",
                    object.id
                ))
            }
        }
        EphemerisSource::SpiceBody { .. } => Ok(()),
        EphemerisSource::BodyFixed {
            latitude_deg,
            longitude_deg,
            altitude_km,
            ..
        } => {
            finite(*latitude_deg, "latitude_deg")?;
            finite(*longitude_deg, "longitude_deg")?;
            finite(*altitude_km, "altitude_km")
        }
        EphemerisSource::CircularOrbit {
            radius_km,
            period_seconds,
            inclination_deg,
            raan_deg,
            phase_at_epoch_deg,
            ..
        } => {
            finite(*radius_km, "radius_km")?;
            finite(*period_seconds, "period_seconds")?;
            finite(*inclination_deg, "inclination_deg")?;
            finite(*raan_deg, "raan_deg")?;
            finite(*phase_at_epoch_deg, "phase_at_epoch_deg")?;
            if *radius_km <= 0.0 || *period_seconds <= 0.0 {
                return invalid(format!(
                    "circular orbit {} must have positive radius and period",
                    object.id
                ));
            }
            Ok(())
        }
        EphemerisSource::SampledTrajectory {
            samples,
            interpolation,
            ..
        } => {
            if *interpolation != InterpolationMode::Linear {
                return invalid("only linear sampled trajectory interpolation is supported");
            }
            if samples.is_empty() {
                return invalid(format!(
                    "sampled trajectory {} must have samples",
                    object.id
                ));
            }
            for pair in samples.windows(2) {
                if pair[1].epoch <= pair[0].epoch {
                    return invalid(format!(
                        "sampled trajectory {} epochs must be strictly increasing",
                        object.id
                    ));
                }
            }
            for sample in samples {
                if !sample.position_km.is_finite() || !sample.velocity_km_s.is_finite() {
                    return invalid(format!(
                        "sampled trajectory {} contains non-finite sample",
                        object.id
                    ));
                }
            }
            Ok(())
        }
        EphemerisSource::FixedOffset { offset_km, .. } => {
            if offset_km.is_finite() {
                Ok(())
            } else {
                invalid(format!(
                    "fixed offset {} contains non-finite offset",
                    object.id
                ))
            }
        }
    }
}

fn finite(value: f64, field: &str) -> Result<(), EphemerisError> {
    if value.is_finite() {
        Ok(())
    } else {
        invalid(format!("{field} must be finite"))
    }
}

fn invalid<T>(message: impl Into<String>) -> Result<T, EphemerisError> {
    Err(EphemerisError::InvalidObjectDefinition(message.into()))
}
