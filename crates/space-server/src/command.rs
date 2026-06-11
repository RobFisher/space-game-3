use space_game_ephemeris::GameTime;
use space_game_protocol::{ClientToServer, DistanceSort, ErrorDto, ServerToClient};

use crate::{config::DEFAULT_GAME_TIME, query::SolarSystemQueryService};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("invalid limit: {0}")]
    InvalidLimit(String),
    #[error("unsupported sort: {0}")]
    UnsupportedSort(String),
    #[error("missing object query")]
    MissingObjectQuery,
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
            | Self::UnknownCommand(_)
            | Self::Time(_) => ErrorDto {
                code: match self {
                    Self::InvalidLimit(_) => "invalid_limit",
                    Self::UnsupportedSort(_) => "unsupported_sort",
                    Self::MissingObjectQuery => "missing_object_query",
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
    message: ClientToServer,
) -> Vec<ServerToClient> {
    match message {
        ClientToServer::Hello { .. } => vec![status_message(service, None)],
        ClientToServer::Command { seq, text } => handle_command_message(service, seq, &text),
        ClientToServer::RequestObjects { seq } => {
            vec![ServerToClient::Objects {
                seq,
                objects: service.list_objects(),
            }]
        }
        ClientToServer::RequestDistance { seq, object_query } => response_or_error(seq, || {
            Ok(ServerToClient::Distance {
                seq,
                result: service.distance_to(&object_query, game_time()?)?,
            })
        }),
        ClientToServer::RequestDistances { seq, limit, sort } => response_or_error(seq, || {
            Ok(ServerToClient::Distances {
                seq,
                results: service.distances(game_time()?, sort, limit)?,
            })
        }),
        ClientToServer::RequestStatus { seq } => vec![status_message(service, Some(seq))],
        ClientToServer::Ping { seq } => vec![ServerToClient::Pong { seq }],
    }
}

pub fn handle_command_message(
    service: &SolarSystemQueryService,
    seq: u64,
    text: &str,
) -> Vec<ServerToClient> {
    let mut responses = vec![ServerToClient::CommandAck {
        seq,
        accepted: true,
        message: None,
    }];

    match handle_command(service, seq, text) {
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
            line: "Commands: help, objects, distance <object>, distances [--limit n] [--sort name|distance], status, quit".to_string(),
        }]),
        "objects" => Ok(vec![ServerToClient::Objects {
            seq,
            objects: service.list_objects(),
        }]),
        "distance" => {
            let query = words
                .get(1..)
                .filter(|tail| !tail.is_empty())
                .map(|tail| tail.join(" "))
                .ok_or(CommandError::MissingObjectQuery)?;
            Ok(vec![ServerToClient::Distance {
                seq,
                result: service.distance_to(&query, game_time()?)?,
            }])
        }
        "distances" => {
            let (limit, sort) = parse_distances_args(&words[1..])?;
            Ok(vec![ServerToClient::Distances {
                seq,
                results: service.distances(game_time()?, sort, limit)?,
            }])
        }
        "status" => Ok(vec![status_message(service, Some(seq))]),
        _ => Err(CommandError::UnknownCommand(command)),
    }
}

fn parse_distances_args(words: &[&str]) -> Result<(Option<usize>, DistanceSort), CommandError> {
    let mut limit = None;
    let mut sort = DistanceSort::Name;
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
            other => return Err(CommandError::UnknownCommand(format!("distances {other}"))),
        }
    }

    Ok((limit, sort))
}

fn status_message(service: &SolarSystemQueryService, seq: Option<u64>) -> ServerToClient {
    let at = game_time().expect("default game time is valid");
    let (seq, status) = service.status(seq, &at);
    ServerToClient::Status { seq, status }
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

fn game_time() -> Result<GameTime, space_game_ephemeris::EphemerisError> {
    GameTime::from_utc_iso8601(DEFAULT_GAME_TIME)
}

#[cfg(test)]
mod tests {
    use space_game_protocol::DistanceSort;

    use super::*;
    use crate::{config::ServerConfig, query::AU_KM};

    fn service() -> SolarSystemQueryService {
        ServerConfig::default().query_service().unwrap()
    }

    #[test]
    fn handles_objects_command_with_sequence() {
        let responses = handle_command_message(&service(), 7, "objects");

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
        let responses = handle_command_message(&service(), 8, "distance mars");

        assert!(matches!(
            &responses[1],
            ServerToClient::Distance { seq: 8, result } if result.object_id == "mars"
        ));
    }

    #[test]
    fn handles_limited_and_sorted_distances() {
        let responses =
            handle_command_message(&service(), 9, "distances --limit 3 --sort distance");

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
        let responses = handle_command_message(&service(), 10, "status");

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
        let responses = handle_command_message(&service(), 11, "launch");

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
            (Some(5), DistanceSort::Distance)
        );
    }

    #[test]
    fn observer_is_one_au_from_origin() {
        let responses = handle_command_message(&service(), 12, "distance sun");

        match &responses[1] {
            ServerToClient::Distance { result, .. } => assert_eq!(result.distance_km, AU_KM),
            other => panic!("unexpected response: {other:?}"),
        }
    }
}
