use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use space_game_ephemeris::GameTime;
use space_game_protocol::{
    ClientToServer, DistanceSort, ErrorDto, ServerToClient, TimeUnit,
};

use crate::{clock::SimulationClock, query::SolarSystemQueryService};

pub type SharedSimulationClock = Arc<RwLock<SimulationClock>>;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("invalid limit: {0}")]
    InvalidLimit(String),
    #[error("unsupported sort: {0}")]
    UnsupportedSort(String),
    #[error("missing object query")]
    MissingObjectQuery,
    #[error("missing time advance amount")]
    MissingTimeAdvanceAmount,
    #[error("invalid time advance amount: {0}")]
    InvalidTimeAdvanceAmount(String),
    #[error("unsupported time unit: {0}")]
    UnsupportedTimeUnit(String),
    #[error("missing timestamp")]
    MissingTimestamp,
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error(transparent)]
    Query(#[from] crate::query::QueryError),
    #[error(transparent)]
    Time(#[from] space_game_ephemeris::EphemerisError),
}

impl CommandError {
    pub fn to_error_dto(&self) -> ErrorDto {
        match self {
            Self::Query(err) => err.to_error_dto(),
            Self::InvalidLimit(_)
            | Self::UnsupportedSort(_)
            | Self::MissingObjectQuery
            | Self::MissingTimeAdvanceAmount
            | Self::InvalidTimeAdvanceAmount(_)
            | Self::UnsupportedTimeUnit(_)
            | Self::MissingTimestamp
            | Self::UnknownCommand(_)
            | Self::Time(_) => ErrorDto {
                code: match self {
                    Self::InvalidLimit(_) => "invalid_limit",
                    Self::UnsupportedSort(_) => "unsupported_sort",
                    Self::MissingObjectQuery => "missing_object_query",
                    Self::MissingTimeAdvanceAmount => "missing_time_advance_amount",
                    Self::InvalidTimeAdvanceAmount(_) => "invalid_time_advance_amount",
                    Self::UnsupportedTimeUnit(_) => "unsupported_time_unit",
                    Self::MissingTimestamp => "missing_timestamp",
                    Self::UnknownCommand(_) => "unknown_command",
                    Self::Time(_) => "invalid_game_time",
                    Self::Query(_) => unreachable!("handled above"),
                }
                .to_string(),
                message: self.to_string(),
            },
        }
    }
}

pub fn handle_client_message(
    service: &SolarSystemQueryService,
    clock: &SharedSimulationClock,
    message: ClientToServer,
) -> Vec<ServerToClient> {
    match message {
        ClientToServer::Hello { .. } => vec![status_message(service, clock, None)],
        ClientToServer::Command { seq, text } => handle_command_message(service, clock, seq, &text),
        ClientToServer::RequestObjects { seq } => {
            vec![ServerToClient::Objects {
                seq,
                objects: service.list_objects(),
            }]
        }
        ClientToServer::RequestDistance {
            seq,
            object_query,
            at_game_time,
        } => response_or_error(seq, || {
            Ok(ServerToClient::Distance {
                seq,
                result: service.distance_to(&object_query, effective_time(clock, at_game_time)?)?,
            })
        }),
        ClientToServer::RequestDistances {
            seq,
            limit,
            sort,
            at_game_time,
        } => response_or_error(seq, || {
            Ok(ServerToClient::Distances {
                seq,
                results: service.distances(effective_time(clock, at_game_time)?, sort, limit)?,
            })
        }),
        ClientToServer::RequestSimulationTime { seq } => {
            vec![simulation_time_message(clock, Some(seq))]
        }
        ClientToServer::AdvanceSimulationTime { seq, amount, unit } => {
            vec![advance_time_message(clock, seq, amount, unit)]
        }
        ClientToServer::RequestStatus { seq } => vec![status_message(service, clock, Some(seq))],
        ClientToServer::Ping { seq } => vec![ServerToClient::Pong { seq }],
    }
}

pub fn handle_command_message(
    service: &SolarSystemQueryService,
    clock: &SharedSimulationClock,
    seq: u64,
    text: &str,
) -> Vec<ServerToClient> {
    let mut responses = vec![ServerToClient::CommandAck {
        seq,
        accepted: true,
        message: None,
    }];

    match handle_command(service, clock, seq, text) {
        Ok(mut command_responses) => responses.append(&mut command_responses),
        Err(err) => {
            responses[0] = ServerToClient::CommandAck {
                seq,
                accepted: false,
                message: Some(err.to_string()),
            };
            responses.push(ServerToClient::Error {
                seq: Some(seq),
                error: err.to_error_dto(),
            });
        }
    }

    responses
}

fn handle_command(
    service: &SolarSystemQueryService,
    clock: &SharedSimulationClock,
    seq: u64,
    text: &str,
) -> Result<Vec<ServerToClient>, CommandError> {
    let words: Vec<_> = text.split_whitespace().collect();
    let command = words
        .first()
        .ok_or_else(|| CommandError::UnknownCommand(text.to_string()))?
        .to_ascii_lowercase();

    match command.as_str() {
        "help" => Ok(vec![ServerToClient::OutputLine {
            seq: Some(seq),
            line: "Commands: help, objects, distance <object> [--at timestamp], distances [--limit n] [--sort name|distance] [--at timestamp], status, time, advance <amount> <seconds|minutes|hours|days>, quit".to_string(),
        }]),
        "objects" => Ok(vec![ServerToClient::Objects {
            seq,
            objects: service.list_objects(),
        }]),
        "distance" => {
            let (query, at_game_time) = parse_distance_args(&words[1..])?;
            Ok(vec![ServerToClient::Distance {
                seq,
                result: service.distance_to(&query, effective_time(clock, at_game_time)?)?,
            }])
        }
        "distances" => {
            let (limit, sort, at_game_time) = parse_distances_args(&words[1..])?;
            Ok(vec![ServerToClient::Distances {
                seq,
                results: service.distances(effective_time(clock, at_game_time)?, sort, limit)?,
            }])
        }
        "status" => Ok(vec![status_message(service, clock, Some(seq))]),
        "time" => Ok(vec![simulation_time_message(clock, Some(seq))]),
        "advance" => {
            let (amount, unit) = parse_advance_args(&words[1..])?;
            Ok(vec![advance_time_message(clock, seq, amount, unit)])
        }
        _ => Err(CommandError::UnknownCommand(command)),
    }
}

fn parse_distance_args(words: &[&str]) -> Result<(String, Option<String>), CommandError> {
    let at_index = words.iter().position(|word| *word == "--at");
    let (query_words, at_game_time) = match at_index {
        Some(index) => {
            let timestamp = words.get(index + 1).ok_or(CommandError::MissingTimestamp)?;
            if words.len() > index + 2 {
                return Err(CommandError::UnknownCommand(format!(
                    "distance {}",
                    words[index + 2]
                )));
            }
            (&words[..index], Some((*timestamp).to_string()))
        }
        None => (words, None),
    };

    let query = (!query_words.is_empty())
        .then(|| query_words.join(" "))
        .ok_or(CommandError::MissingObjectQuery)?;
    Ok((query, at_game_time))
}

fn parse_distances_args(
    words: &[&str],
) -> Result<(Option<usize>, DistanceSort, Option<String>), CommandError> {
    let mut limit = None;
    let mut sort = DistanceSort::Name;
    let mut at_game_time = None;
    let mut index = 0;

    while index < words.len() {
        match words[index] {
            "--limit" => {
                let value = words
                    .get(index + 1)
                    .ok_or_else(|| CommandError::InvalidLimit("missing value".to_string()))?;
                limit = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| CommandError::InvalidLimit((*value).to_string()))?,
                );
                index += 2;
            }
            "--sort" => {
                let value = words
                    .get(index + 1)
                    .ok_or_else(|| CommandError::UnsupportedSort("missing value".to_string()))?;
                sort = match value.to_ascii_lowercase().as_str() {
                    "name" => DistanceSort::Name,
                    "distance" => DistanceSort::Distance,
                    _ => return Err(CommandError::UnsupportedSort((*value).to_string())),
                };
                index += 2;
            }
            "--at" => {
                let value = words.get(index + 1).ok_or(CommandError::MissingTimestamp)?;
                at_game_time = Some((*value).to_string());
                index += 2;
            }
            other => return Err(CommandError::UnknownCommand(format!("distances {other}"))),
        }
    }

    Ok((limit, sort, at_game_time))
}

fn parse_advance_args(words: &[&str]) -> Result<(i64, TimeUnit), CommandError> {
    let amount_text = words.first().ok_or(CommandError::MissingTimeAdvanceAmount)?;
    let amount = amount_text
        .parse::<i64>()
        .map_err(|_| CommandError::InvalidTimeAdvanceAmount((*amount_text).to_string()))?;
    let unit_text = words
        .get(1)
        .ok_or_else(|| CommandError::UnsupportedTimeUnit("missing value".to_string()))?;
    if words.len() > 2 {
        return Err(CommandError::UnknownCommand(format!("advance {}", words[2])));
    }
    Ok((amount, parse_time_unit(unit_text)?))
}

fn parse_time_unit(unit: &str) -> Result<TimeUnit, CommandError> {
    match unit.to_ascii_lowercase().as_str() {
        "second" | "seconds" => Ok(TimeUnit::Seconds),
        "minute" | "minutes" => Ok(TimeUnit::Minutes),
        "hour" | "hours" => Ok(TimeUnit::Hours),
        "day" | "days" => Ok(TimeUnit::Days),
        _ => Err(CommandError::UnsupportedTimeUnit(unit.to_string())),
    }
}

fn status_message(
    service: &SolarSystemQueryService,
    clock: &SharedSimulationClock,
    seq: Option<u64>,
) -> ServerToClient {
    let at = clock_snapshot(clock).current_time;
    let (seq, status) = service.status(seq, &at);
    ServerToClient::Status { seq, status }
}

fn simulation_time_message(clock: &SharedSimulationClock, seq: Option<u64>) -> ServerToClient {
    ServerToClient::SimulationTime {
        seq,
        state: clock_snapshot(clock).to_dto(),
    }
}

fn advance_time_message(
    clock: &SharedSimulationClock,
    seq: u64,
    amount: i64,
    unit: TimeUnit,
) -> ServerToClient {
    let state = clock
        .write()
        .expect("simulation clock lock poisoned")
        .advance(amount, unit, Instant::now())
        .to_dto();
    ServerToClient::SimulationTime {
        seq: Some(seq),
        state,
    }
}

fn effective_time(
    clock: &SharedSimulationClock,
    at_game_time: Option<String>,
) -> Result<GameTime, space_game_ephemeris::EphemerisError> {
    match at_game_time {
        Some(at) => GameTime::from_utc_iso8601(&at),
        None => Ok(clock_snapshot(clock).current_time),
    }
}

fn clock_snapshot(clock: &SharedSimulationClock) -> crate::clock::SimulationClockSnapshot {
    clock
        .read()
        .expect("simulation clock lock poisoned")
        .snapshot(Instant::now())
}

fn response_or_error(
    seq: u64,
    f: impl FnOnce() -> Result<ServerToClient, CommandError>,
) -> Vec<ServerToClient> {
    match f() {
        Ok(response) => vec![response],
        Err(err) => vec![ServerToClient::Error {
            seq: Some(seq),
            error: err.to_error_dto(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use space_game_protocol::DistanceSort;

    use super::*;
    use crate::{config::{ServerConfig, DEFAULT_GAME_TIME}, query::AU_KM};

    fn service() -> SolarSystemQueryService {
        ServerConfig::default().query_service().unwrap()
    }

    fn clock() -> SharedSimulationClock {
        Arc::new(RwLock::new(SimulationClock::new(
            GameTime::from_utc_iso8601(DEFAULT_GAME_TIME).unwrap(),
            Instant::now(),
        )))
    }

    #[test]
    fn handles_objects_command_with_sequence() {
        let responses = handle_command_message(&service(), &clock(), 7, "objects");

        assert!(matches!(
            &responses[0],
            ServerToClient::CommandAck {
                seq: 7,
                accepted: true,
                ..
            }
        ));
        assert!(matches!(
            &responses[1],
            ServerToClient::Objects { seq: 7, objects } if !objects.is_empty()
        ));
    }

    #[test]
    fn handles_distance_command_with_sequence() {
        let responses = handle_command_message(&service(), &clock(), 8, "distance mars");

        assert!(matches!(
            &responses[1],
            ServerToClient::Distance { seq: 8, result } if result.object_id == "mars"
        ));
    }

    #[test]
    fn handles_limited_and_sorted_distances() {
        let responses =
            handle_command_message(&service(), &clock(), 9, "distances --limit 3 --sort distance");

        match &responses[1] {
            ServerToClient::Distances { seq, results } => {
                assert_eq!(*seq, 9);
                assert_eq!(results.len(), 3);
                assert!(results[0].distance_km <= results[1].distance_km);
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn handles_status_command() {
        let responses = handle_command_message(&service(), &clock(), 10, "status");

        assert!(matches!(
            &responses[1],
            ServerToClient::Status {
                seq: Some(10),
                status
            } if status.observer_label == "demo-observer"
        ));
    }

    #[test]
    fn returns_error_for_unknown_command() {
        let responses = handle_command_message(&service(), &clock(), 11, "launch");

        assert!(matches!(
            &responses[0],
            ServerToClient::CommandAck {
                seq: 11,
                accepted: false,
                ..
            }
        ));
        assert!(matches!(
            &responses[1],
            ServerToClient::Error {
                seq: Some(11),
                error
            } if error.code == "unknown_command"
        ));
    }

    #[test]
    fn parses_distance_args() {
        assert_eq!(
            parse_distances_args(&["--limit", "5", "--sort", "distance"]).unwrap(),
            (Some(5), DistanceSort::Distance, None)
        );
    }

    #[test]
    fn observer_is_one_au_from_origin() {
        let responses = handle_command_message(&service(), &clock(), 12, "distance sun");

        match &responses[1] {
            ServerToClient::Distance { result, .. } => assert_eq!(result.distance_km, AU_KM),
            other => panic!("unexpected response: {other:?}"),
        }
    }
}
