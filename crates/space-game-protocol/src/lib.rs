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
    RequestObjects {
        seq: u64,
    },
    RequestDistance {
        seq: u64,
        object_query: String,
    },
    RequestDistances {
        seq: u64,
        limit: Option<usize>,
        sort: DistanceSort,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusDto {
    pub connected: bool,
    pub server: String,
    pub game_time: String,
    pub observer_label: String,
    pub observer_frame: String,
    pub object_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDto {
    pub code: String,
    pub message: String,
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
    fn status_round_trips_without_sequence() {
        let msg = ServerToClient::Status {
            seq: None,
            status: StatusDto {
                connected: true,
                server: "127.0.0.1:4000".to_string(),
                game_time: "2097-01-01T00:00:00Z".to_string(),
                observer_label: "demo-observer".to_string(),
                observer_frame: "solar_system_barycentric_j2000".to_string(),
                object_count: 8,
            },
        };

        assert_eq!(round_trip(&msg), msg);
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
