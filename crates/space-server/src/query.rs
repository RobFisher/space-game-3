use std::sync::RwLock;

use space_game_ephemeris::{
    EphemerisError, EphemerisQuality, FrameId, GameTime, ObjectKind, ObjectSummary, SolarSystem,
    StateVector,
};
use space_game_protocol::{
    DistanceResultDto, DistanceSort, ErrorDto, FlightPlanDto, FlightPlanStatusDto,
    FlightPlanTargetDto, LocationSummaryDto, ObjectSummaryDto, ShipStateDto, StatusDto,
};
use thiserror::Error;

use crate::ship::{
    transfer_duration_seconds, validate_acceleration, FlightPlan, FlightPlanError,
    FlightPlanStatus, FlightPlanTarget, PlayerShip, ResolvedShipState, ShipNameError,
};

pub const AU_KM: f64 = 149_597_870.7;
const INTERCEPT_ITERATIONS: usize = 8;

#[derive(Debug)]
pub struct SolarSystemQueryService {
    server_label: String,
    world: SolarSystem,
    player_ship: RwLock<PlayerShip>,
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("unknown object: {0}")]
    UnknownObject(String),
    #[error("object query '{query}' is ambiguous: {matches}")]
    AmbiguousObject { query: String, matches: String },
    #[error(
        "cannot compare ship frame '{ship_frame}' with object '{object}' frame '{object_frame}'"
    )]
    IncompatibleFrame {
        ship_frame: String,
        object: String,
        object_frame: String,
    },
    #[error(transparent)]
    Ephemeris(#[from] EphemerisError),
    #[error(transparent)]
    FlightPlan(#[from] FlightPlanError),
}

impl QueryError {
    pub fn to_error_dto(&self) -> ErrorDto {
        let code = match self {
            Self::UnknownObject(_) => "unknown_object",
            Self::AmbiguousObject { .. } => "ambiguous_object",
            Self::IncompatibleFrame { .. } => "incompatible_frame",
            Self::Ephemeris(_) => "ephemeris_error",
            Self::FlightPlan(_) => "invalid_acceleration",
        };
        ErrorDto {
            code: code.to_string(),
            message: self.to_string(),
        }
    }
}

impl SolarSystemQueryService {
    pub fn new(server_label: String, world: SolarSystem) -> Self {
        let player_ship = PlayerShip::default_near_earth(
            GameTime::from_utc_iso8601(crate::config::DEFAULT_GAME_TIME)
                .expect("default game time is valid"),
        );
        Self::with_player_ship(server_label, world, player_ship)
    }

    pub fn with_player_ship(
        server_label: String,
        world: SolarSystem,
        player_ship: PlayerShip,
    ) -> Self {
        Self {
            server_label,
            world,
            player_ship: RwLock::new(player_ship),
        }
    }

    pub fn player_ship(&self) -> PlayerShip {
        self.player_ship
            .read()
            .expect("player ship lock poisoned")
            .clone()
    }

    pub fn rename_player_ship(&self, display_name: &str) -> Result<PlayerShip, ShipNameError> {
        let mut ship = self.player_ship.write().expect("player ship lock poisoned");
        ship.rename(display_name)?;
        Ok(ship.clone())
    }

    pub fn player_ship_state(&self, at: GameTime) -> Result<ResolvedShipState, QueryError> {
        let ship = self.player_ship();
        if let Some(plan) = ship.active_flight_plan() {
            if at < plan.arrival_time {
                return Ok(ship.resolve_flight_plan_state(plan, at));
            }
            let target_state = self
                .world
                .state(plan.target.object_id().as_str(), at.clone())?;
            return Ok(ship.resolve_arrived_orbiting_state(plan, at, target_state));
        }

        let parent_id = ship
            .orbit_parent_id()
            .ok_or_else(|| QueryError::UnknownObject("ship parent".to_string()))?
            .to_string();
        let parent_state = self.world.state(&parent_id, at.clone())?;
        Ok(ship.resolve_orbiting_state(at, parent_state))
    }

    pub fn create_flight_plan(
        &self,
        object_query: &str,
        departure_time: GameTime,
        acceleration_km_s2: f64,
    ) -> Result<FlightPlanDto, QueryError> {
        validate_acceleration(acceleration_km_s2)?;
        let target = self.resolve_object(object_query)?;
        let origin_state = self.player_ship_state(departure_time.clone())?.state;
        let (arrival_time, arrival_state, duration_seconds) = self.estimate_object_intercept(
            &target,
            &origin_state,
            &departure_time,
            acceleration_km_s2,
        )?;
        let mut ship = self.player_ship.write().expect("player ship lock poisoned");
        let plan = ship.register_flight_plan(
            origin_state,
            FlightPlanTarget::Object {
                object_id: target.id,
                display_name: target.name,
                arrival_state,
            },
            departure_time.clone(),
            arrival_time,
            duration_seconds,
            acceleration_km_s2,
        )?;
        Ok(flight_plan_to_dto(&plan, &departure_time))
    }

    pub fn active_flight_plan(&self, at: &GameTime) -> Option<FlightPlanDto> {
        self.player_ship()
            .active_flight_plan()
            .map(|plan| flight_plan_to_dto(plan, at))
    }

    pub fn cancel_flight_plan(&self, at: &GameTime) -> Option<FlightPlanDto> {
        self.player_ship
            .write()
            .expect("player ship lock poisoned")
            .cancel_active_flight_plan()
            .map(|plan| flight_plan_to_dto(&plan, at))
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

    pub fn status(
        &self,
        seq: Option<u64>,
        at: &GameTime,
    ) -> Result<(Option<u64>, StatusDto), QueryError> {
        let ship = self.player_ship_state(at.clone())?;
        Ok((
            seq,
            StatusDto {
                connected: true,
                server: self.server_label.clone(),
                game_time: at.to_string(),
                ship_id: ship.ship_id,
                ship_name: ship.display_name,
                ship_frame: frame_label(&ship.state.frame),
                ship_motion: ship.motion_mode,
                object_count: self.world.list_objects().len(),
            },
        ))
    }

    pub fn ship_state(&self, at: GameTime) -> Result<ShipStateDto, QueryError> {
        let ship = self.player_ship_state(at)?;
        Ok(ship_state_to_dto(ship))
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
        let ship_state = self.player_ship_state(at.clone())?;
        let target_state = self.world.state(object.id.as_str(), at.clone())?;
        let (distance_km, quality) =
            self.distance_between_states(object, &ship_state.state, &target_state)?;
        Ok(DistanceResultDto {
            object_id: object.id.to_string(),
            display_name: object.name.clone(),
            distance_km,
            distance_au: distance_km / AU_KM,
            at_game_time: at.to_string(),
            quality: Some(quality_label(quality)),
        })
    }

    fn estimate_object_intercept(
        &self,
        object: &ObjectSummary,
        origin_state: &StateVector,
        departure_time: &GameTime,
        acceleration_km_s2: f64,
    ) -> Result<(GameTime, StateVector, f64), QueryError> {
        let mut target_state = self
            .world
            .state(object.id.as_str(), departure_time.clone())?;
        let mut duration_seconds = 0.0;
        let mut arrival_time = departure_time.clone();

        for _ in 0..INTERCEPT_ITERATIONS {
            if origin_state.frame != target_state.frame {
                return Err(QueryError::IncompatibleFrame {
                    ship_frame: frame_label(&origin_state.frame),
                    object: object.id.to_string(),
                    object_frame: frame_label(&target_state.frame),
                });
            }
            duration_seconds = transfer_duration_seconds(
                origin_state.position_km.distance(target_state.position_km),
                acceleration_km_s2,
            );
            arrival_time = departure_time.add_seconds(duration_seconds);
            target_state = self.world.state(object.id.as_str(), arrival_time.clone())?;
        }

        Ok((arrival_time, target_state, duration_seconds))
    }

    pub fn location_summary(&self, at: GameTime) -> Result<LocationSummaryDto, QueryError> {
        let ship_state = self.player_ship_state(at.clone())?;
        self.location_summary_for_state(
            Some(ship_state.ship_id),
            ship_state.display_name,
            "ship".to_string(),
            &ship_state.state,
            at,
        )
    }

    pub fn object_location_summary(
        &self,
        object_query: &str,
        at: GameTime,
    ) -> Result<LocationSummaryDto, QueryError> {
        let object = self.resolve_object(object_query)?;
        let subject_state = self.world.state(object.id.as_str(), at.clone())?;
        self.location_summary_for_state(
            Some(object.id.to_string()),
            object.name,
            "object".to_string(),
            &subject_state,
            at,
        )
    }

    fn location_summary_for_state(
        &self,
        subject_id: Option<String>,
        subject_label: String,
        subject_type: String,
        subject_state: &StateVector,
        at: GameTime,
    ) -> Result<LocationSummaryDto, QueryError> {
        let mut nearest: Option<(ObjectSummary, f64, EphemerisQuality)> = None;

        for object in self.world.list_objects() {
            if subject_id.as_deref() == Some(object.id.as_str()) {
                continue;
            }
            let target_state = self.world.state(object.id.as_str(), at.clone())?;
            let (distance_km, quality) =
                self.distance_between_states(&object, subject_state, &target_state)?;
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
            subject_id,
            subject_label,
            subject_type,
            frame: frame_label(&subject_state.frame),
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
        ship_state: &StateVector,
        target_state: &StateVector,
    ) -> Result<(f64, EphemerisQuality), QueryError> {
        if ship_state.frame != target_state.frame {
            return Err(QueryError::IncompatibleFrame {
                ship_frame: frame_label(&ship_state.frame),
                object: object.id.to_string(),
                object_frame: frame_label(&target_state.frame),
            });
        }

        let relative_state = target_state.relative_to(ship_state);
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

fn ship_state_to_dto(ship: ResolvedShipState) -> ShipStateDto {
    ShipStateDto {
        ship_id: ship.ship_id,
        ship_name: ship.display_name,
        motion_mode: ship.motion_mode,
        frame: frame_label(&ship.state.frame),
        game_time: ship.state.epoch.to_string(),
        quality: Some(quality_label(ship.state.quality)),
    }
}

pub fn flight_plan_to_dto(plan: &FlightPlan, at: &GameTime) -> FlightPlanDto {
    FlightPlanDto {
        plan_id: plan.plan_id.clone(),
        ship_id: plan.ship_id.clone(),
        target: match &plan.target {
            FlightPlanTarget::Object {
                object_id,
                display_name,
                ..
            } => FlightPlanTargetDto::Object {
                object_id: object_id.to_string(),
                display_name: display_name.clone(),
            },
        },
        departure_time: plan.departure_time.to_string(),
        arrival_time: plan.arrival_time.to_string(),
        duration_seconds: plan.duration_seconds,
        acceleration_km_s2: plan.acceleration_km_s2,
        status: match plan.effective_status_at(at) {
            FlightPlanStatus::Active => FlightPlanStatusDto::Active,
            FlightPlanStatus::Completed => FlightPlanStatusDto::Completed,
            FlightPlanStatus::Cancelled => FlightPlanStatusDto::Cancelled,
        },
        quality: Some(quality_label(plan.target_state().quality)),
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
    use space_game_ephemeris::{FrameId, ObjectRegistry, SolarSystemBuilder};

    use super::*;
    use crate::config::{demo_world, DEFAULT_GAME_TIME};

    fn epoch() -> GameTime {
        GameTime::from_utc_iso8601(DEFAULT_GAME_TIME).unwrap()
    }

    fn service() -> SolarSystemQueryService {
        SolarSystemQueryService::new("test-server".to_string(), demo_world().unwrap())
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
        let service = SolarSystemQueryService::new("test-server".to_string(), world);

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
    fn calculates_distance_from_player_ship() {
        let result = service().distance_to("sun", epoch()).unwrap();

        assert!(result.distance_km.is_finite());
        assert_ne!(result.distance_km, AU_KM);
    }

    #[test]
    fn resolves_ship_state_at_requested_time() {
        let at = epoch();
        let ship = service().player_ship_state(at.clone()).unwrap();

        assert_eq!(ship.ship_id, "player-ship");
        assert_eq!(ship.display_name, "Wayfarer");
        assert_eq!(ship.motion_mode, "orbiting");
        assert_eq!(
            ship.parent_object_id.as_ref().map(|id| id.as_str()),
            Some("earth")
        );
        assert_eq!(ship.state.frame, FrameId::SolarSystemBarycentricJ2000);
        assert_eq!(ship.state.epoch, at);
    }

    #[test]
    fn calculates_distance_from_ship_and_target_states() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "earth"
name = "Earth"
kind = "planet"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "target"
name = "Target"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 42169.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
"#,
        )
        .unwrap();
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()
            .unwrap();
        let service = SolarSystemQueryService::new("test-server".to_string(), world);

        let result = service.distance_to("target", epoch()).unwrap();
        assert_eq!(result.distance_km, 5.0);
    }

    #[test]
    fn rejects_incompatible_distance_frames() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "earth"
name = "Earth"
kind = "planet"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

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
        let service = SolarSystemQueryService::new("test-server".to_string(), world);

        assert!(matches!(
            service.distance_to("target", epoch()),
            Err(QueryError::IncompatibleFrame {
                ship_frame,
                object,
                object_frame
            }) if ship_frame == "solar_system_barycentric_j2000"
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
        let (_, status) = service().status(Some(4), &epoch()).unwrap();

        assert_eq!(status.server, "test-server");
        assert_eq!(status.ship_id, "player-ship");
        assert_eq!(status.ship_name, "Wayfarer");
        assert_eq!(status.ship_motion, "orbiting");
        assert_eq!(status.object_count, 8);
    }

    #[test]
    fn builds_location_summary_from_nearest_object() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "earth"
name = "Earth"
kind = "planet"
[objects.source]
type = "static_state"
position_km = { x = 0.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "near"
name = "Near Station"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 42166.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "far"
name = "Far Station"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 42214.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
"#,
        )
        .unwrap();
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()
            .unwrap();
        let service = SolarSystemQueryService::new("test-server".to_string(), world);

        let summary = service.location_summary(epoch()).unwrap();

        assert_eq!(summary.subject_id.as_deref(), Some("player-ship"));
        assert_eq!(summary.subject_label, "Wayfarer");
        assert_eq!(summary.subject_type, "ship");
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

    #[test]
    fn builds_object_location_summary_and_excludes_subject() {
        let registry = ObjectRegistry::from_toml_str(
            r#"
[[objects]]
id = "subject"
name = "Subject Station"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 10.0, y = 0.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }

[[objects]]
id = "near"
name = "Near Station"
kind = "station"
[objects.source]
type = "static_state"
position_km = { x = 13.0, y = 4.0, z = 0.0 }
velocity_km_s = { x = 0.0, y = 0.0, z = 0.0 }
"#,
        )
        .unwrap();
        let world = SolarSystemBuilder::new()
            .object_registry_data(registry)
            .build()
            .unwrap();
        let service = SolarSystemQueryService::new("test-server".to_string(), world);

        let summary = service.object_location_summary("subject", epoch()).unwrap();

        assert_eq!(summary.subject_id.as_deref(), Some("subject"));
        assert_eq!(summary.subject_label, "Subject Station");
        assert_eq!(summary.subject_type, "object");
        assert_eq!(summary.nearest_object_id, "near");
        assert_eq!(summary.nearest_object_name, "Near Station");
        assert_eq!(summary.distance_km, 5.0);
    }
}
