use space_game_ephemeris::{
    EphemerisError, EphemerisQuality, FrameId, GameTime, ObjectKind, ObjectSummary, SolarSystem,
    StateVector, Vec3Km, Vec3KmPerSec,
};
use space_game_protocol::{
    DistanceResultDto, DistanceSort, ErrorDto, LocationSummaryDto, ObjectSummaryDto, StatusDto,
};
use thiserror::Error;

pub const AU_KM: f64 = 149_597_870.7;

#[derive(Clone, Debug)]
pub struct ObserverLocation {
    pub label: String,
    pub frame: FrameId,
    pub position_km: Vec3Km,
    pub velocity_km_s: Vec3KmPerSec,
    pub quality: EphemerisQuality,
}

impl ObserverLocation {
    pub fn state_at(&self, at: GameTime) -> StateVector {
        StateVector::new(
            self.position_km,
            self.velocity_km_s,
            self.frame.clone(),
            at,
            self.quality,
        )
    }
}

#[derive(Clone, Debug)]
pub struct SolarSystemQueryService {
    server_label: String,
    world: SolarSystem,
    observer: ObserverLocation,
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("unknown object: {0}")]
    UnknownObject(String),
    #[error("object query '{query}' is ambiguous: {matches}")]
    AmbiguousObject { query: String, matches: String },
    #[error("cannot compare observer frame '{observer_frame}' with object '{object}' frame '{object_frame}'")]
    IncompatibleFrame {
        observer_frame: String,
        object: String,
        object_frame: String,
    },
    #[error(transparent)]
    Ephemeris(#[from] EphemerisError),
}

impl QueryError {
    pub fn to_error_dto(&self) -> ErrorDto {
        let code = match self {
            Self::UnknownObject(_) => "unknown_object",
            Self::AmbiguousObject { .. } => "ambiguous_object",
            Self::IncompatibleFrame { .. } => "incompatible_frame",
            Self::Ephemeris(_) => "ephemeris_error",
        };
        ErrorDto {
            code: code.to_string(),
            message: self.to_string(),
        }
    }
}

impl SolarSystemQueryService {
    pub fn new(server_label: String, world: SolarSystem, observer: ObserverLocation) -> Self {
        Self {
            server_label,
            world,
            observer,
        }
    }

    pub fn list_objects(&self) -> Vec<ObjectSummaryDto> {
        self.world
            .list_objects()
            .into_iter()
            .map(summary_to_dto)
            .collect()
    }

    pub fn resolve_object(&self, query: &str) -> Result<ObjectSummary, QueryError> {
        let normalized = query.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(QueryError::UnknownObject(query.to_string()));
        }

        let objects = self.world.list_objects();
        if let Some(exact) = objects
            .iter()
            .find(|object| object.id.as_str() == query.trim())
        {
            return Ok(exact.clone());
        }

        let matches: Vec<_> = objects
            .into_iter()
            .filter(|object| {
                object.id.as_str().to_ascii_lowercase() == normalized
                    || object.name.to_ascii_lowercase() == normalized
            })
            .collect();

        match matches.len() {
            0 => Err(QueryError::UnknownObject(query.to_string())),
            1 => Ok(matches.into_iter().next().expect("one match")),
            _ => Err(QueryError::AmbiguousObject {
                query: query.to_string(),
                matches: matches
                    .iter()
                    .map(|object| format!("{} ({})", object.name, object.id))
                    .collect::<Vec<_>>()
                    .join(", "),
            }),
        }
    }

    pub fn status(&self, seq: Option<u64>, at: &GameTime) -> (Option<u64>, StatusDto) {
        (
            seq,
            StatusDto {
                connected: true,
                server: self.server_label.clone(),
                game_time: at.to_string(),
                observer_label: self.observer.label.clone(),
                observer_frame: frame_label(&self.observer.state_at(at.clone()).frame),
                object_count: self.world.list_objects().len(),
            },
        )
    }

    pub fn distance_to(
        &self,
        object_query: &str,
        at: GameTime,
    ) -> Result<DistanceResultDto, QueryError> {
        let object = self.resolve_object(object_query)?;
        self.distance_for_summary(&object, at)
    }

    pub fn distances(
        &self,
        at: GameTime,
        sort: DistanceSort,
        limit: Option<usize>,
    ) -> Result<Vec<DistanceResultDto>, QueryError> {
        let mut results = self
            .world
            .list_objects()
            .into_iter()
            .map(|object| self.distance_for_summary(&object, at.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        match sort {
            DistanceSort::Name => {
                results.sort_by(|a, b| a.display_name.cmp(&b.display_name));
            }
            DistanceSort::Distance => {
                results.sort_by(|a, b| a.distance_km.total_cmp(&b.distance_km));
            }
        }

        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    fn distance_for_summary(
        &self,
        object: &ObjectSummary,
        at: GameTime,
    ) -> Result<DistanceResultDto, QueryError> {
        let observer_state = self.observer.state_at(at.clone());
        let target_state = self.world.state(object.id.as_str(), at.clone())?;
        let (distance_km, quality) =
            self.distance_between_states(object, &observer_state, &target_state)?;
        Ok(DistanceResultDto {
            object_id: object.id.to_string(),
            display_name: object.name.clone(),
            distance_km,
            distance_au: distance_km / AU_KM,
            at_game_time: at.to_string(),
            quality: Some(quality_label(quality)),
        })
    }

    pub fn location_summary(&self, at: GameTime) -> Result<LocationSummaryDto, QueryError> {
        let observer_state = self.observer.state_at(at.clone());
        let mut nearest: Option<(ObjectSummary, f64, EphemerisQuality)> = None;

        for object in self.world.list_objects() {
            let target_state = self.world.state(object.id.as_str(), at.clone())?;
            let (distance_km, quality) =
                self.distance_between_states(&object, &observer_state, &target_state)?;
            let replace = nearest
                .as_ref()
                .map(|(_, nearest_distance, _)| distance_km < *nearest_distance)
                .unwrap_or(true);
            if replace {
                nearest = Some((object, distance_km, quality));
            }
        }

        let (nearest_object, distance_km, quality) =
            nearest.ok_or_else(|| QueryError::UnknownObject("nearest object".to_string()))?;
        Ok(LocationSummaryDto {
            observer_label: self.observer.label.clone(),
            frame: frame_label(&observer_state.frame),
            game_time: at.to_string(),
            nearest_object_id: nearest_object.id.to_string(),
            nearest_object_name: nearest_object.name,
            distance_km,
            distance_au: distance_km / AU_KM,
            quality: Some(quality_label(quality)),
        })
    }

    fn distance_between_states(
        &self,
        object: &ObjectSummary,
        observer_state: &StateVector,
        target_state: &StateVector,
    ) -> Result<(f64, EphemerisQuality), QueryError> {
        if observer_state.frame != target_state.frame {
            return Err(QueryError::IncompatibleFrame {
                observer_frame: frame_label(&observer_state.frame),
                object: object.id.to_string(),
                object_frame: frame_label(&target_state.frame),
            });
        }

        let relative_state = target_state.relative_to(observer_state);
        Ok((
            relative_state.position_km.magnitude(),
            relative_state.quality,
        ))
    }
}

pub fn summary_to_dto(summary: ObjectSummary) -> ObjectSummaryDto {
    ObjectSummaryDto {
        id: summary.id.to_string(),
        display_name: summary.name,
        kind: kind_label(&summary.kind),
    }
}

fn kind_label(kind: &ObjectKind) -> String {
    format!("{kind:?}").to_ascii_lowercase()
}

fn frame_label(frame: &FrameId) -> String {
    match frame {
        FrameId::SolarSystemBarycentricJ2000 => "solar_system_barycentric_j2000".to_string(),
        FrameId::ParentCenteredInertial(id) => format!("parent_centered_inertial:{id}"),
        FrameId::BodyFixed(id) => format!("body_fixed:{id}"),
        FrameId::Custom(id) => id.clone(),
    }
}

fn quality_label(quality: EphemerisQuality) -> String {
    format!("{quality:?}").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use space_game_ephemeris::{EphemerisQuality, FrameId, ObjectRegistry, SolarSystemBuilder};

    use super::*;
    use crate::config::{demo_world, DEFAULT_GAME_TIME};

    fn epoch() -> GameTime {
        GameTime::from_utc_iso8601(DEFAULT_GAME_TIME).unwrap()
    }

    fn service() -> SolarSystemQueryService {
        SolarSystemQueryService::new(
            "test-server".to_string(),
            demo_world().unwrap(),
            ObserverLocation {
                label: "demo-observer".to_string(),
                frame: FrameId::SolarSystemBarycentricJ2000,
                position_km: Vec3Km::new(AU_KM, 0.0, 0.0),
                velocity_km_s: Vec3KmPerSec::ZERO,
                quality: EphemerisQuality::Fictional,
            },
        )
    }

    fn observer_at(position_km: Vec3Km, frame: FrameId) -> ObserverLocation {
        ObserverLocation {
            label: "test-observer".to_string(),
            frame,
            position_km,
            velocity_km_s: Vec3KmPerSec::ZERO,
            quality: EphemerisQuality::Fictional,
        }
    }

    #[test]
    fn lists_demo_objects() {
        let objects = service().list_objects();
        assert!(objects.iter().any(|object| object.id == "mars"));
        assert!(objects.iter().any(|object| object.id == "demo-station"));
    }

    #[test]
    fn resolves_lowercase_id_and_display_name() {
        let service = service();

        assert_eq!(service.resolve_object("mars").unwrap().id.as_str(), "mars");
        assert_eq!(service.resolve_object("Mars").unwrap().id.as_str(), "mars");
    }

    #[test]
    fn reports_ambiguous_object_query() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "alpha-one"
name = "Alpha"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "alpha-two"
name = "Alpha"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 1.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
"#,
        )
        .unwrap();
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()
            .unwrap();
        let service = SolarSystemQueryService::new(
            "test-server".to_string(),
            world,
            ObserverLocation {
                label: "origin".to_string(),
                frame: FrameId::SolarSystemBarycentricJ2000,
                position_km: Vec3Km::ZERO,
                velocity_km_s: Vec3KmPerSec::ZERO,
                quality: EphemerisQuality::Fictional,
            },
        );

        assert!(matches!(
            service.resolve_object("alpha"),
            Err(QueryError::AmbiguousObject { .. })
        ));
    }

    #[test]
    fn calculates_single_distance() {
        let result = service().distance_to("mars", epoch()).unwrap();

        assert_eq!(result.object_id, "mars");
        assert!(result.distance_km.is_finite());
        assert!(result.distance_au.is_finite());
    }

    #[test]
    fn keeps_demo_observer_distance_compatibility() {
        let result = service().distance_to("sun", epoch()).unwrap();

        assert_eq!(result.distance_km, AU_KM);
        assert_eq!(result.distance_au, 1.0);
    }

    #[test]
    fn observer_resolves_to_state_at_requested_time() {
        let at = epoch();
        let observer = observer_at(
            Vec3Km::new(1.0, 2.0, 3.0),
            FrameId::SolarSystemBarycentricJ2000,
        );
        let state = observer.state_at(at.clone());

        assert_eq!(state.position_km, Vec3Km::new(1.0, 2.0, 3.0));
        assert_eq!(state.velocity_km_s, Vec3KmPerSec::ZERO);
        assert_eq!(state.frame, FrameId::SolarSystemBarycentricJ2000);
        assert_eq!(state.epoch, at);
        assert_eq!(state.quality, EphemerisQuality::Fictional);
    }

    #[test]
    fn calculates_distance_from_observer_and_target_states() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "target"
name = "Target"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 13.0, y = 24.0, z = 30.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
"#,
        )
        .unwrap();
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()
            .unwrap();
        let service = SolarSystemQueryService::new(
            "test-server".to_string(),
            world,
            observer_at(
                Vec3Km::new(10.0, 20.0, 30.0),
                FrameId::SolarSystemBarycentricJ2000,
            ),
        );

        let result = service.distance_to("target", epoch()).unwrap();
        assert_eq!(result.distance_km, 5.0);
    }

    #[test]
    fn rejects_incompatible_distance_frames() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "target"
name = "Target"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 1.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
frame = { type = "custom", value = "other" }
"#,
        )
        .unwrap();
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()
            .unwrap();
        let service = SolarSystemQueryService::new(
            "test-server".to_string(),
            world,
            observer_at(Vec3Km::ZERO, FrameId::SolarSystemBarycentricJ2000),
        );

        assert!(matches!(
            service.distance_to("target", epoch()),
            Err(QueryError::IncompatibleFrame {
                observer_frame,
                object,
                object_frame
            }) if observer_frame == "solar_system_barycentric_j2000"
                && object == "target"
                && object_frame == "other"
        ));
    }

    #[test]
    fn sorts_and_limits_distances() {
        let service = service();
        let results = service
            .distances(epoch(), DistanceSort::Distance, Some(3))
            .unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].distance_km <= results[1].distance_km);
        assert!(results[1].distance_km <= results[2].distance_km);

        let by_name = service
            .distances(epoch(), DistanceSort::Name, None)
            .unwrap();
        assert!(by_name
            .windows(2)
            .all(|pair| pair[0].display_name <= pair[1].display_name));
    }

    #[test]
    fn reports_status() {
        let (_, status) = service().status(Some(4), &epoch());

        assert_eq!(status.server, "test-server");
        assert_eq!(status.observer_label, "demo-observer");
        assert_eq!(status.object_count, 8);
    }

    #[test]
    fn builds_location_summary_from_nearest_object() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "near"
name = "Near Station"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 12.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "far"
name = "Far Station"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 50.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
"#,
        )
        .unwrap();
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()
            .unwrap();
        let service = SolarSystemQueryService::new(
            "test-server".to_string(),
            world,
            observer_at(
                Vec3Km::new(10.0, 0.0, 0.0),
                FrameId::SolarSystemBarycentricJ2000,
            ),
        );

        let summary = service.location_summary(epoch()).unwrap();

        assert_eq!(summary.observer_label, "test-observer");
        assert_eq!(summary.frame, "solar_system_barycentric_j2000");
        assert_eq!(summary.game_time, DEFAULT_GAME_TIME);
        assert_eq!(summary.nearest_object_id, "near");
        assert_eq!(summary.nearest_object_name, "Near Station");
        assert_eq!(summary.distance_km, 2.0);
        assert_eq!(summary.quality.as_deref(), Some("fictional"));

        let json = serde_json::to_value(&summary).unwrap();
        assert!(json.get("x").is_none());
        assert!(json.get("y").is_none());
        assert!(json.get("z").is_none());
    }
}
