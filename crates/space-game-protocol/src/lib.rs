//! Shared JSON-serializable protocol types for the networked space game slice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientToServer {
    Hello {
        client_name: String,
        client_version: String,
    },
    Command {
        seq: u64,
        text: String,
    },
    CompletionRequest(CompletionRequestDto),
    RequestObjects {
        seq: u64,
    },
    RequestDistance {
        seq: u64,
        object_query: String,
        #[serde(default)]
        at_game_time: Option<String>,
    },
    RequestDistances {
        seq: u64,
        limit: Option<usize>,
        sort: DistanceSort,
        #[serde(default)]
        at_game_time: Option<String>,
    },
    RequestSimulationTime {
        seq: u64,
    },
    AdvanceSimulationTime {
        seq: u64,
        amount: i64,
        unit: TimeUnit,
    },
    RequestStatus {
        seq: u64,
    },
    Ping {
        seq: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClient {
    Welcome {
        server_version: String,
        session_id: String,
    },
    CommandAck {
        seq: u64,
        accepted: bool,
        message: Option<String>,
    },
    CompletionResponse(CompletionResponseDto),
    Status {
        seq: Option<u64>,
        status: StatusDto,
    },
    Objects {
        seq: u64,
        objects: Vec<ObjectSummaryDto>,
    },
    Distance {
        seq: u64,
        result: DistanceResultDto,
    },
    Distances {
        seq: u64,
        results: Vec<DistanceResultDto>,
    },
    ShipState {
        seq: u64,
        ship: ShipStateDto,
    },
    FlightPlan {
        seq: u64,
        plan: Option<FlightPlanDto>,
    },
    LocationSummary {
        seq: u64,
        summary: LocationSummaryDto,
    },
    SimulationTime {
        seq: Option<u64>,
        state: SimulationTimeDto,
    },
    OutputLine {
        seq: Option<u64>,
        line: String,
    },
    Error {
        seq: Option<u64>,
        error: ErrorDto,
    },
    Pong {
        seq: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceSort {
    Name,
    Distance,
}

impl Default for DistanceSort {
    fn default() -> Self {
        Self::Name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectSummaryDto {
    pub id: String,
    pub display_name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistanceResultDto {
    pub object_id: String,
    pub display_name: String,
    pub distance_km: f64,
    pub distance_au: f64,
    pub at_game_time: String,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationSummaryDto {
    pub subject_id: Option<String>,
    pub subject_label: String,
    pub subject_type: String,
    pub frame: String,
    pub game_time: String,
    pub nearest_object_id: String,
    pub nearest_object_name: String,
    pub distance_km: f64,
    pub distance_au: f64,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusDto {
    pub connected: bool,
    pub server: String,
    pub game_time: String,
    pub ship_id: String,
    pub ship_name: String,
    pub ship_frame: String,
    pub ship_motion: String,
    pub object_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipStateDto {
    pub ship_id: String,
    pub ship_name: String,
    pub motion_mode: String,
    pub frame: String,
    pub game_time: String,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlightPlanDto {
    pub plan_id: String,
    pub ship_id: String,
    pub target: FlightPlanTargetDto,
    pub departure_time: String,
    pub arrival_time: String,
    pub orbit_entry_time: String,
    pub duration_seconds: f64,
    pub acceleration_km_s2: f64,
    pub acceleration_g: Option<f64>,
    pub status: FlightPlanStatusDto,
    pub navigation_phase: String,
    pub arrival_orbit: Option<ArrivalOrbitDto>,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrivalOrbitDto {
    pub kind: String,
    pub radius_km: f64,
    pub altitude_km: Option<f64>,
    pub period_seconds: Option<f64>,
    pub circular_speed_km_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FlightPlanTargetDto {
    Object {
        object_id: String,
        display_name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlightPlanStatusDto {
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationTimeDto {
    pub current_time: String,
    pub running: bool,
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDto {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionRequestDto {
    pub seq: u64,
    pub input: String,
    pub cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionResponseDto {
    pub seq: u64,
    pub replacement: ReplacementSpanDto,
    pub candidates: Vec<CompletionCandidateDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplacementSpanDto {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionCandidateDto {
    pub insertion: String,
    pub display: String,
    pub kind: CompletionCandidateKindDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionCandidateKindDto {
    Command,
    Object,
    Option,
    LocalCommand,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(value: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let json = serde_json::to_string(value).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn command_round_trips() {
        let msg = ClientToServer::Command {
            seq: 42,
            text: "distance mars".to_string(),
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn completion_request_round_trips() {
        let msg = ClientToServer::CompletionRequest(CompletionRequestDto {
            seq: 12,
            input: "distance ma".to_string(),
            cursor: 11,
        });

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn completion_response_round_trips() {
        let msg = ServerToClient::CompletionResponse(CompletionResponseDto {
            seq: 12,
            replacement: ReplacementSpanDto { start: 9, end: 11 },
            candidates: vec![CompletionCandidateDto {
                insertion: "mars".to_string(),
                display: "Mars".to_string(),
                kind: CompletionCandidateKindDto::Object,
            }],
        });

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn empty_completion_response_preserves_sequence() {
        let msg = ServerToClient::CompletionResponse(CompletionResponseDto {
            seq: 15,
            replacement: ReplacementSpanDto { start: 0, end: 0 },
            candidates: Vec::new(),
        });

        let round_tripped = round_trip(&msg);
        assert_eq!(round_tripped, msg);
        assert!(matches!(
            round_tripped,
            ServerToClient::CompletionResponse(CompletionResponseDto {
                seq: 15,
                candidates,
                ..
            }) if candidates.is_empty()
        ));
    }

    #[test]
    fn status_round_trips_without_sequence() {
        let msg = ServerToClient::Status {
            seq: None,
            status: StatusDto {
                connected: true,
                server: "127.0.0.1:4000".to_string(),
                game_time: "2097-01-01T00:00:00Z".to_string(),
                ship_id: "player-ship".to_string(),
                ship_name: "Wayfarer".to_string(),
                ship_frame: "solar_system_barycentric_j2000".to_string(),
                ship_motion: "orbiting".to_string(),
                object_count: 8,
            },
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn ship_state_round_trips_with_sequence() {
        let msg = ServerToClient::ShipState {
            seq: 11,
            ship: ShipStateDto {
                ship_id: "player-ship".to_string(),
                ship_name: "Wayfarer".to_string(),
                motion_mode: "orbiting".to_string(),
                frame: "solar_system_barycentric_j2000".to_string(),
                game_time: "2097-01-01T00:00:00Z".to_string(),
                quality: Some("fictional".to_string()),
            },
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn active_flight_plan_response_round_trips() {
        let msg = ServerToClient::FlightPlan {
            seq: 14,
            plan: Some(FlightPlanDto {
                plan_id: "flight-1".to_string(),
                ship_id: "player-ship".to_string(),
                target: FlightPlanTargetDto::Object {
                    object_id: "mars".to_string(),
                    display_name: "Mars".to_string(),
                },
                departure_time: "2097-01-01T00:00:00Z".to_string(),
                arrival_time: "2097-01-01T03:00:00Z".to_string(),
                orbit_entry_time: "2097-01-01T03:10:00Z".to_string(),
                duration_seconds: 10_800.0,
                acceleration_km_s2: 0.02,
                acceleration_g: Some(2.039_432_426),
                status: FlightPlanStatusDto::Active,
                navigation_phase: "flight_plan".to_string(),
                arrival_orbit: Some(ArrivalOrbitDto {
                    kind: "low".to_string(),
                    radius_km: 3_789.5,
                    altitude_km: Some(400.0),
                    period_seconds: Some(7_113.0),
                    circular_speed_km_s: Some(3.362),
                }),
                quality: Some("fictional".to_string()),
            }),
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn entering_orbit_ship_state_round_trips() {
        let msg = ServerToClient::ShipState {
            seq: 16,
            ship: ShipStateDto {
                ship_id: "player-ship".to_string(),
                ship_name: "Wayfarer".to_string(),
                motion_mode: "entering_orbit".to_string(),
                frame: "solar_system_barycentric_j2000".to_string(),
                game_time: "2097-01-01T03:05:00Z".to_string(),
                quality: Some("fictional".to_string()),
            },
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn no_active_flight_plan_response_preserves_sequence() {
        let msg = ServerToClient::FlightPlan {
            seq: 15,
            plan: None,
        };

        let round_tripped = round_trip(&msg);
        assert_eq!(round_tripped, msg);
        assert!(matches!(
            round_tripped,
            ServerToClient::FlightPlan {
                seq: 15,
                plan: None
            }
        ));
    }

    #[test]
    fn object_list_round_trips_with_sequence() {
        let msg = ServerToClient::Objects {
            seq: 7,
            objects: vec![ObjectSummaryDto {
                id: "mars".to_string(),
                display_name: "Mars".to_string(),
                kind: "planet".to_string(),
            }],
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn distance_round_trips() {
        let msg = ServerToClient::Distance {
            seq: 9,
            result: DistanceResultDto {
                object_id: "mars".to_string(),
                display_name: "Mars".to_string(),
                distance_km: 78_000_000.0,
                distance_au: 0.521,
                at_game_time: "2097-01-01T00:00:00Z".to_string(),
                quality: Some("fictional".to_string()),
            },
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn location_summary_round_trips() {
        let msg = ServerToClient::LocationSummary {
            seq: 9,
            summary: LocationSummaryDto {
                subject_id: None,
                subject_label: "demo-observer".to_string(),
                subject_type: "observer".to_string(),
                frame: "solar_system_barycentric_j2000".to_string(),
                game_time: "2097-01-01T00:00:00Z".to_string(),
                nearest_object_id: "earth".to_string(),
                nearest_object_name: "Earth".to_string(),
                distance_km: 42_000.0,
                distance_au: 0.000_280_753,
                quality: Some("fictional".to_string()),
            },
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn location_summary_omits_raw_coordinates() {
        let msg = ServerToClient::LocationSummary {
            seq: 9,
            summary: LocationSummaryDto {
                subject_id: Some("mars".to_string()),
                subject_label: "Mars".to_string(),
                subject_type: "object".to_string(),
                frame: "solar_system_barycentric_j2000".to_string(),
                game_time: "2097-01-01T00:00:00Z".to_string(),
                nearest_object_id: "phobos".to_string(),
                nearest_object_name: "Phobos".to_string(),
                distance_km: 42_000.0,
                distance_au: 0.000_280_753,
                quality: Some("fictional".to_string()),
            },
        };
        let json = serde_json::to_value(&msg).unwrap();
        let summary = json.get("summary").unwrap();

        assert!(summary.get("x").is_none());
        assert!(summary.get("y").is_none());
        assert!(summary.get("z").is_none());
        assert!(summary.get("position_km").is_none());
        assert!(summary.get("velocity_km_s").is_none());
    }

    #[test]
    fn simulation_time_request_round_trips() {
        let msg = ClientToServer::RequestSimulationTime { seq: 4 };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn simulation_time_response_round_trips() {
        let msg = ServerToClient::SimulationTime {
            seq: Some(6),
            state: SimulationTimeDto {
                current_time: "2097-01-01T00:00:07Z".to_string(),
                running: true,
                rate: 1.0,
            },
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn simulation_time_advance_round_trips() {
        let msg = ClientToServer::AdvanceSimulationTime {
            seq: 5,
            amount: 2,
            unit: TimeUnit::Hours,
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn distance_request_round_trips_without_explicit_time() {
        let msg = ClientToServer::RequestDistance {
            seq: 10,
            object_query: "mars".to_string(),
            at_game_time: None,
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn distance_request_round_trips_with_explicit_time() {
        let msg = ClientToServer::RequestDistance {
            seq: 10,
            object_query: "mars".to_string(),
            at_game_time: Some("2097-01-02T00:00:00Z".to_string()),
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn distances_request_round_trips_with_explicit_time() {
        let msg = ClientToServer::RequestDistances {
            seq: 11,
            limit: Some(3),
            sort: DistanceSort::Distance,
            at_game_time: Some("2097-01-02T00:00:00Z".to_string()),
        };

        assert_eq!(round_trip(&msg), msg);
    }

    #[test]
    fn error_round_trips_with_sequence() {
        let msg = ServerToClient::Error {
            seq: Some(8),
            error: ErrorDto {
                code: "unknown_command".to_string(),
                message: "Unknown command".to_string(),
            },
        };

        assert_eq!(round_trip(&msg), msg);
    }
}
