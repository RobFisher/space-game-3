use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use space_game_ephemeris::GameTime;
use space_game_protocol::{
    ClientToServer, CompletionCandidateDto, CompletionCandidateKindDto, CompletionRequestDto,
    CompletionResponseDto, DistanceSort, ErrorDto, ReplacementSpanDto, ServerToClient, TimeUnit,
};

use crate::ship::DEFAULT_FLIGHT_ACCELERATION_KM_S2;
use crate::{clock::SimulationClock, query::SolarSystemQueryService};

pub type SharedSimulationClock = Arc<RwLock<SimulationClock>>;

const SERVER_COMMANDS: &[&str] = &[
    "advance",
    "distance",
    "distances",
    "flight",
    "help",
    "objects",
    "ship",
    "status",
    "time",
    "where",
];
const DISTANCE_OPTIONS: &[&str] = &["--at"];
const DISTANCES_OPTIONS: &[&str] = &["--at", "--limit", "--sort"];
const FLIGHT_PLAN_OPTIONS: &[&str] = &["--accel"];

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
    #[error("missing ship name")]
    MissingShipName,
    #[error("missing flight subcommand")]
    MissingFlightSubcommand,
    #[error("missing acceleration")]
    MissingAcceleration,
    #[error("invalid acceleration: {0}")]
    InvalidAcceleration(String),
    #[error(transparent)]
    InvalidShipName(#[from] crate::ship::ShipNameError),
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
            | Self::MissingShipName
            | Self::MissingFlightSubcommand
            | Self::MissingAcceleration
            | Self::InvalidAcceleration(_)
            | Self::InvalidShipName(_)
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
                    Self::MissingShipName => "missing_ship_name",
                    Self::MissingFlightSubcommand => "missing_flight_subcommand",
                    Self::MissingAcceleration => "missing_acceleration",
                    Self::InvalidAcceleration(_) => "invalid_acceleration",
                    Self::InvalidShipName(_) => "invalid_ship_name",
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
        ClientToServer::Hello { .. } => match status_message(service, clock, None) {
            Ok(message) => vec![message],
            Err(err) => vec![ServerToClient::Error {
                seq: None,
                error: err.to_error_dto(),
            }],
        },
        ClientToServer::Command { seq, text } => handle_command_message(service, clock, seq, &text),
        ClientToServer::CompletionRequest(request) => {
            vec![handle_completion_request(service, request)]
        }
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
        ClientToServer::RequestStatus { seq } => {
            response_or_error(seq, || status_message(service, clock, Some(seq)))
        }
        ClientToServer::Ping { seq } => vec![ServerToClient::Pong { seq }],
    }
}

pub fn handle_completion_request(
    service: &SolarSystemQueryService,
    request: CompletionRequestDto,
) -> ServerToClient {
    let completion = complete_input(service, &request.input, request.cursor);
    ServerToClient::CompletionResponse(CompletionResponseDto {
        seq: request.seq,
        replacement: completion.replacement,
        candidates: completion.candidates,
    })
}

struct CompletionResult {
    replacement: ReplacementSpanDto,
    candidates: Vec<CompletionCandidateDto>,
}

#[derive(Debug, Clone, Copy)]
struct TokenSpan<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

fn complete_input(
    service: &SolarSystemQueryService,
    input: &str,
    cursor: usize,
) -> CompletionResult {
    let cursor = cursor.min(input.len());
    if !input.is_char_boundary(cursor) {
        return empty_completion(cursor);
    }

    let tokens = token_spans(input);
    let (current_index, replacement) = current_token(&tokens, cursor);
    let prefix = token_text(input, &replacement);

    if current_index == 0 {
        return CompletionResult {
            replacement,
            candidates: complete_words(
                SERVER_COMMANDS,
                prefix,
                CompletionCandidateKindDto::Command,
            ),
        };
    }

    let Some(command) = tokens.first().map(|token| token.text.to_ascii_lowercase()) else {
        return empty_completion(cursor);
    };
    match command.as_str() {
        "distance" | "where" => {
            if prefix.starts_with("--") {
                return CompletionResult {
                    replacement,
                    candidates: complete_words(
                        DISTANCE_OPTIONS,
                        prefix,
                        CompletionCandidateKindDto::Option,
                    ),
                };
            }
            if tokens
                .iter()
                .take(current_index)
                .any(|token| token.text == "--at")
            {
                return CompletionResult {
                    replacement,
                    candidates: Vec::new(),
                };
            }
            CompletionResult {
                replacement,
                candidates: object_candidates(service, prefix),
            }
        }
        "flight" => complete_flight(service, &tokens, current_index, replacement, prefix),
        "distances" => {
            if prefix.starts_with("--") {
                CompletionResult {
                    replacement,
                    candidates: complete_words(
                        DISTANCES_OPTIONS,
                        prefix,
                        CompletionCandidateKindDto::Option,
                    ),
                }
            } else {
                CompletionResult {
                    replacement,
                    candidates: Vec::new(),
                }
            }
        }
        _ => CompletionResult {
            replacement,
            candidates: Vec::new(),
        },
    }
}

fn complete_flight(
    service: &SolarSystemQueryService,
    tokens: &[TokenSpan<'_>],
    current_index: usize,
    replacement: ReplacementSpanDto,
    prefix: &str,
) -> CompletionResult {
    if current_index == 1 {
        return CompletionResult {
            replacement,
            candidates: complete_words(
                &["plan", "status", "cancel"],
                prefix,
                CompletionCandidateKindDto::Command,
            ),
        };
    }

    if tokens.get(1).map(|token| token.text) != Some("plan") {
        return CompletionResult {
            replacement,
            candidates: Vec::new(),
        };
    }

    if prefix.starts_with("--") {
        return CompletionResult {
            replacement,
            candidates: complete_words(
                FLIGHT_PLAN_OPTIONS,
                prefix,
                CompletionCandidateKindDto::Option,
            ),
        };
    }
    if tokens
        .iter()
        .take(current_index)
        .any(|token| token.text == "--accel")
    {
        return CompletionResult {
            replacement,
            candidates: Vec::new(),
        };
    }

    CompletionResult {
        replacement,
        candidates: object_candidates(service, prefix),
    }
}

fn empty_completion(cursor: usize) -> CompletionResult {
    CompletionResult {
        replacement: ReplacementSpanDto {
            start: cursor,
            end: cursor,
        },
        candidates: Vec::new(),
    }
}

fn token_spans(input: &str) -> Vec<TokenSpan<'_>> {
    let mut tokens = Vec::new();
    let mut token_start = None;

    for (index, ch) in input.char_indices() {
        if ch.is_whitespace() {
            if let Some(start) = token_start.take() {
                tokens.push(TokenSpan {
                    text: &input[start..index],
                    start,
                    end: index,
                });
            }
        } else if token_start.is_none() {
            token_start = Some(index);
        }
    }

    if let Some(start) = token_start {
        tokens.push(TokenSpan {
            text: &input[start..],
            start,
            end: input.len(),
        });
    }

    tokens
}

fn current_token(tokens: &[TokenSpan<'_>], cursor: usize) -> (usize, ReplacementSpanDto) {
    for (index, token) in tokens.iter().enumerate() {
        if (token.start..=token.end).contains(&cursor) {
            return (
                index,
                ReplacementSpanDto {
                    start: token.start,
                    end: token.end,
                },
            );
        }
        if cursor < token.start {
            return (
                index,
                ReplacementSpanDto {
                    start: cursor,
                    end: cursor,
                },
            );
        }
    }

    (
        tokens.len(),
        ReplacementSpanDto {
            start: cursor,
            end: cursor,
        },
    )
}

fn token_text<'a>(input: &'a str, replacement: &ReplacementSpanDto) -> &'a str {
    input
        .get(replacement.start..replacement.end)
        .expect("replacement span comes from input token boundaries")
}

fn complete_words(
    words: &[&str],
    prefix: &str,
    kind: CompletionCandidateKindDto,
) -> Vec<CompletionCandidateDto> {
    words
        .iter()
        .copied()
        .filter(|word| word.starts_with(prefix))
        .map(|word| CompletionCandidateDto {
            insertion: word.to_string(),
            display: word.to_string(),
            kind,
        })
        .collect()
}

fn object_candidates(
    service: &SolarSystemQueryService,
    prefix: &str,
) -> Vec<CompletionCandidateDto> {
    let normalized = prefix.to_ascii_lowercase();
    service
        .list_objects()
        .into_iter()
        .filter(|object| {
            object.id.to_ascii_lowercase().starts_with(&normalized)
                || object
                    .display_name
                    .to_ascii_lowercase()
                    .starts_with(&normalized)
        })
        .map(|object| CompletionCandidateDto {
            insertion: object.display_name.clone(),
            display: object.display_name,
            kind: CompletionCandidateKindDto::Object,
        })
        .collect()
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
            line: "Commands: help, objects, distance <object> [--at timestamp], distances [--limit n] [--sort name|distance] [--at timestamp], status, ship [status|name <name>], flight plan <object> [--accel km_per_s2], flight status, flight cancel, time, advance <amount> <seconds|minutes|hours|days>, where [object] [--at timestamp], quit".to_string(),
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
        "status" => Ok(vec![status_message(service, clock, Some(seq))?]),
        "ship" => handle_ship_command(service, clock, seq, &words[1..]),
        "flight" => handle_flight_command(service, clock, seq, &words[1..]),
        "time" => Ok(vec![simulation_time_message(clock, Some(seq))]),
        "where" => {
            if words.len() == 1 {
                return Ok(vec![ServerToClient::LocationSummary {
                    seq,
                    summary: service.location_summary(effective_time(clock, None)?)?,
                }]);
            }
            let (query, at_game_time) = parse_distance_args(&words[1..])?;
            Ok(vec![ServerToClient::LocationSummary {
                seq,
                summary: service
                    .object_location_summary(&query, effective_time(clock, at_game_time)?)?,
            }])
        }
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

fn handle_ship_command(
    service: &SolarSystemQueryService,
    clock: &SharedSimulationClock,
    seq: u64,
    words: &[&str],
) -> Result<Vec<ServerToClient>, CommandError> {
    match words.first().map(|word| word.to_ascii_lowercase()) {
        None => Ok(vec![ship_state_message(service, clock, seq)?]),
        Some(command) if command == "status" && words.len() == 1 => {
            Ok(vec![ship_state_message(service, clock, seq)?])
        }
        Some(command) if command == "name" => {
            let name_words = words
                .get(1..)
                .filter(|words| !words.is_empty())
                .ok_or(CommandError::MissingShipName)?;
            service.rename_player_ship(&name_words.join(" "))?;
            Ok(vec![ship_state_message(service, clock, seq)?])
        }
        Some(command) => Err(CommandError::UnknownCommand(format!("ship {command}"))),
    }
}

fn handle_flight_command(
    service: &SolarSystemQueryService,
    clock: &SharedSimulationClock,
    seq: u64,
    words: &[&str],
) -> Result<Vec<ServerToClient>, CommandError> {
    match words.first().map(|word| word.to_ascii_lowercase()) {
        None => Err(CommandError::MissingFlightSubcommand),
        Some(command) if command == "plan" => {
            let (query, acceleration_km_s2) = parse_flight_plan_args(&words[1..])?;
            let at = clock_snapshot(clock).current_time;
            Ok(vec![ServerToClient::FlightPlan {
                seq,
                plan: Some(service.create_flight_plan(&query, at, acceleration_km_s2)?),
            }])
        }
        Some(command) if command == "status" && words.len() == 1 => {
            let at = clock_snapshot(clock).current_time;
            Ok(vec![ServerToClient::FlightPlan {
                seq,
                plan: service.active_flight_plan(&at),
            }])
        }
        Some(command) if command == "cancel" && words.len() == 1 => {
            let at = clock_snapshot(clock).current_time;
            Ok(vec![ServerToClient::FlightPlan {
                seq,
                plan: service.cancel_flight_plan(&at),
            }])
        }
        Some(command) => Err(CommandError::UnknownCommand(format!("flight {command}"))),
    }
}

fn parse_flight_plan_args(words: &[&str]) -> Result<(String, f64), CommandError> {
    let accel_index = words.iter().position(|word| *word == "--accel");
    let (query_words, acceleration_km_s2) = match accel_index {
        Some(index) => {
            let value = words
                .get(index + 1)
                .ok_or(CommandError::MissingAcceleration)?;
            if words.len() > index + 2 {
                return Err(CommandError::UnknownCommand(format!(
                    "flight plan {}",
                    words[index + 2]
                )));
            }
            let acceleration = value
                .parse::<f64>()
                .map_err(|_| CommandError::InvalidAcceleration((*value).to_string()))?;
            (&words[..index], acceleration)
        }
        None => (words, DEFAULT_FLIGHT_ACCELERATION_KM_S2),
    };

    if !acceleration_km_s2.is_finite() || acceleration_km_s2 <= 0.0 {
        return Err(CommandError::InvalidAcceleration(
            acceleration_km_s2.to_string(),
        ));
    }

    let query = (!query_words.is_empty())
        .then(|| query_words.join(" "))
        .ok_or(CommandError::MissingObjectQuery)?;
    Ok((query, acceleration_km_s2))
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
    let amount_text = words
        .first()
        .ok_or(CommandError::MissingTimeAdvanceAmount)?;
    let amount = amount_text
        .parse::<i64>()
        .map_err(|_| CommandError::InvalidTimeAdvanceAmount((*amount_text).to_string()))?;
    let unit_text = words
        .get(1)
        .ok_or_else(|| CommandError::UnsupportedTimeUnit("missing value".to_string()))?;
    if words.len() > 2 {
        return Err(CommandError::UnknownCommand(format!(
            "advance {}",
            words[2]
        )));
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
) -> Result<ServerToClient, CommandError> {
    let at = clock_snapshot(clock).current_time;
    let (seq, status) = service.status(seq, &at)?;
    Ok(ServerToClient::Status { seq, status })
}

fn ship_state_message(
    service: &SolarSystemQueryService,
    clock: &SharedSimulationClock,
    seq: u64,
) -> Result<ServerToClient, CommandError> {
    Ok(ServerToClient::ShipState {
        seq,
        ship: service.ship_state(clock_snapshot(clock).current_time)?,
    })
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

    use space_game_protocol::{
        CompletionCandidateKindDto, CompletionRequestDto, DistanceSort, FlightPlanStatusDto,
    };

    use super::*;
    use crate::config::{ServerConfig, DEFAULT_GAME_TIME};

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
    fn handles_distance_command_with_explicit_time() {
        let responses = handle_command_message(
            &service(),
            &clock(),
            8,
            "distance mars --at 2097-01-02T00:00:00Z",
        );

        assert!(matches!(
            &responses[1],
            ServerToClient::Distance { seq: 8, result }
                if result.object_id == "mars"
                    && result.at_game_time == "2097-01-02T00:00:00Z"
        ));
    }

    #[test]
    fn handles_limited_and_sorted_distances() {
        let responses = handle_command_message(
            &service(),
            &clock(),
            9,
            "distances --limit 3 --sort distance",
        );

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
    fn handles_distances_with_explicit_time() {
        let responses = handle_command_message(
            &service(),
            &clock(),
            9,
            "distances --limit 2 --at 2097-01-02T00:00:00Z",
        );

        match &responses[1] {
            ServerToClient::Distances { seq, results } => {
                assert_eq!(*seq, 9);
                assert_eq!(results.len(), 2);
                assert!(results
                    .iter()
                    .all(|result| result.at_game_time == "2097-01-02T00:00:00Z"));
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
            } if status.ship_id == "player-ship"
                && status.ship_name == "Wayfarer"
                && status.ship_motion == "orbiting"
        ));
    }

    #[test]
    fn handles_ship_command_with_sequence() {
        let responses = handle_command_message(&service(), &clock(), 30, "ship");

        assert!(matches!(
            &responses[1],
            ServerToClient::ShipState { seq: 30, ship }
                if ship.ship_id == "player-ship"
                    && ship.ship_name == "Wayfarer"
                    && ship.motion_mode == "orbiting"
                    && ship.frame == "solar_system_barycentric_j2000"
        ));
    }

    #[test]
    fn handles_ship_status_command_with_sequence() {
        let responses = handle_command_message(&service(), &clock(), 31, "ship status");

        assert!(matches!(
            &responses[1],
            ServerToClient::ShipState { seq: 31, ship }
                if ship.ship_name == "Wayfarer"
        ));
    }

    #[test]
    fn handles_ship_name_command_and_status_uses_new_name() {
        let service = service();
        let responses = handle_command_message(&service, &clock(), 32, "ship name Wayfarer II");

        assert!(matches!(
            &responses[1],
            ServerToClient::ShipState { seq: 32, ship }
                if ship.ship_id == "player-ship" && ship.ship_name == "Wayfarer II"
        ));

        let responses = handle_command_message(&service, &clock(), 33, "status");
        assert!(matches!(
            &responses[1],
            ServerToClient::Status { status, .. } if status.ship_name == "Wayfarer II"
        ));
    }

    #[test]
    fn rejects_empty_ship_name_command() {
        let service = service();
        let responses = handle_command_message(&service, &clock(), 34, "ship name");

        assert!(matches!(
            &responses[1],
            ServerToClient::Error {
                seq: Some(34),
                error
            } if error.code == "missing_ship_name"
        ));
        assert_eq!(service.player_ship().display_name(), "Wayfarer");
    }

    #[test]
    fn handles_time_command() {
        let responses = handle_command_message(&service(), &clock(), 13, "time");

        assert!(matches!(
            &responses[1],
            ServerToClient::SimulationTime {
                seq: Some(13),
                state
            } if state.current_time == DEFAULT_GAME_TIME && state.running && state.rate == 1.0
        ));
    }

    #[test]
    fn handles_where_command_with_sequence() {
        let responses = handle_command_message(&service(), &clock(), 12, "where");

        assert!(matches!(
            &responses[1],
            ServerToClient::LocationSummary { seq: 12, summary }
                if summary.subject_id.as_deref() == Some("player-ship")
                    && summary.subject_label == "Wayfarer"
                    && summary.subject_type == "ship"
                    && summary.frame == "solar_system_barycentric_j2000"
                    && !summary.nearest_object_id.is_empty()
        ));
    }

    #[test]
    fn handles_where_object_command_with_sequence() {
        let responses = handle_command_message(&service(), &clock(), 12, "where mars");

        assert!(matches!(
            &responses[1],
            ServerToClient::LocationSummary { seq: 12, summary }
                if summary.subject_id.as_deref() == Some("mars")
                    && summary.subject_label == "Mars"
                    && summary.subject_type == "object"
                    && summary.nearest_object_id != "mars"
        ));
    }

    #[test]
    fn handles_where_object_command_with_explicit_time() {
        let responses = handle_command_message(
            &service(),
            &clock(),
            12,
            "where mars --at 2097-01-02T00:00:00Z",
        );

        assert!(matches!(
            &responses[1],
            ServerToClient::LocationSummary { seq: 12, summary }
                if summary.subject_id.as_deref() == Some("mars")
                    && summary.game_time == "2097-01-02T00:00:00Z"
        ));
    }

    #[test]
    fn default_where_uses_advanced_clock() {
        let clock = clock();
        let _ = handle_command_message(&service(), &clock, 19, "advance 1 day");
        let responses = handle_command_message(&service(), &clock, 20, "where");

        assert!(matches!(
            &responses[1],
            ServerToClient::LocationSummary { summary, .. }
                if summary.game_time == "2097-01-02T00:00:00Z"
        ));
    }

    #[test]
    fn handles_advance_command() {
        let clock = clock();
        let responses = handle_command_message(&service(), &clock, 14, "advance 1 day");

        assert!(matches!(
            &responses[1],
            ServerToClient::SimulationTime {
                seq: Some(14),
                state
            } if state.current_time == "2097-01-02T00:00:00Z"
        ));

        let responses = handle_command_message(&service(), &clock, 15, "time");
        assert!(matches!(
            &responses[1],
            ServerToClient::SimulationTime { state, .. }
                if state.current_time.starts_with("2097-01-02T00:00:")
        ));
    }

    #[test]
    fn rejects_invalid_advance_command() {
        let responses = handle_command_message(&service(), &clock(), 16, "advance 1 month");

        assert!(matches!(
            &responses[1],
            ServerToClient::Error {
                seq: Some(16),
                error
            } if error.code == "unsupported_time_unit"
        ));
    }

    #[test]
    fn handles_typed_simulation_time_request() {
        let responses = handle_client_message(
            &service(),
            &clock(),
            ClientToServer::RequestSimulationTime { seq: 17 },
        );

        assert!(matches!(
            &responses[0],
            ServerToClient::SimulationTime {
                seq: Some(17),
                state
            } if state.current_time == DEFAULT_GAME_TIME
        ));
    }

    #[test]
    fn completes_command_names_with_sequence() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 22,
                input: "di".to_string(),
                cursor: 2,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.seq, 22);
                assert_eq!(response.replacement.start, 0);
                assert_eq!(response.replacement.end, 2);
                assert!(response.candidates.iter().any(|candidate| {
                    candidate.insertion == "distance"
                        && candidate.kind == CompletionCandidateKindDto::Command
                }));
                assert!(response
                    .candidates
                    .iter()
                    .any(|candidate| candidate.insertion == "distances"));
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn completes_where_command_name() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 27,
                input: "wh".to_string(),
                cursor: 2,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.seq, 27);
                assert_eq!(response.replacement.start, 0);
                assert_eq!(response.replacement.end, 2);
                assert_eq!(
                    response
                        .candidates
                        .iter()
                        .map(|candidate| candidate.insertion.as_str())
                        .collect::<Vec<_>>(),
                    vec!["where"]
                );
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn completes_ship_command_name() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 29,
                input: "sh".to_string(),
                cursor: 2,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.replacement.start, 0);
                assert_eq!(response.replacement.end, 2);
                assert!(response.candidates.iter().any(|candidate| {
                    candidate.insertion == "ship"
                        && candidate.kind == CompletionCandidateKindDto::Command
                }));
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn completes_where_object_argument() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 28,
                input: "where ma".to_string(),
                cursor: 8,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.replacement.start, 6);
                assert_eq!(response.replacement.end, 8);
                assert!(response.candidates.iter().any(|candidate| {
                    candidate.insertion == "Mars"
                        && candidate.kind == CompletionCandidateKindDto::Object
                }));
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn completes_distance_object_argument() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 23,
                input: "distance ma".to_string(),
                cursor: 11,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.replacement.start, 9);
                assert_eq!(response.replacement.end, 11);
                assert!(response.candidates.iter().any(|candidate| {
                    candidate.insertion == "Mars"
                        && candidate.display == "Mars"
                        && candidate.kind == CompletionCandidateKindDto::Object
                }));
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn completes_multi_word_object_display_name() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 24,
                input: "distance demo".to_string(),
                cursor: 13,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.replacement.start, 9);
                assert_eq!(response.replacement.end, 13);
                assert!(response.candidates.iter().any(|candidate| {
                    candidate.insertion == "Demo Station"
                        && candidate.display == "Demo Station"
                        && candidate.kind == CompletionCandidateKindDto::Object
                }));
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn completes_distances_option_name() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 25,
                input: "distances --s".to_string(),
                cursor: 13,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.replacement.start, 10);
                assert_eq!(response.replacement.end, 13);
                assert_eq!(
                    response
                        .candidates
                        .iter()
                        .map(|candidate| candidate.insertion.as_str())
                        .collect::<Vec<_>>(),
                    vec!["--sort"]
                );
                assert_eq!(
                    response.candidates[0].kind,
                    CompletionCandidateKindDto::Option
                );
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn returns_empty_completion_for_unsupported_context() {
        let response = handle_completion_request(
            &service(),
            CompletionRequestDto {
                seq: 26,
                input: "advance 1".to_string(),
                cursor: 9,
            },
        );

        match response {
            ServerToClient::CompletionResponse(response) => {
                assert_eq!(response.seq, 26);
                assert!(response.candidates.is_empty());
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn handles_typed_simulation_time_advance() {
        let clock = clock();
        let responses = handle_client_message(
            &service(),
            &clock,
            ClientToServer::AdvanceSimulationTime {
                seq: 18,
                amount: 2,
                unit: TimeUnit::Hours,
            },
        );

        assert!(matches!(
            &responses[0],
            ServerToClient::SimulationTime {
                seq: Some(18),
                state
            } if state.current_time == "2097-01-01T02:00:00Z"
        ));
    }

    #[test]
    fn default_distance_uses_advanced_clock() {
        let clock = clock();
        let _ = handle_command_message(&service(), &clock, 19, "advance 1 day");
        let responses = handle_command_message(&service(), &clock, 20, "distance mars");

        assert!(matches!(
            &responses[1],
            ServerToClient::Distance { result, .. }
                if result.at_game_time == "2097-01-02T00:00:00Z"
        ));
    }

    #[test]
    fn explicit_distance_time_does_not_mutate_clock() {
        let clock = clock();
        let _ = handle_command_message(&service(), &clock, 21, "advance 1 day");
        let responses = handle_command_message(
            &service(),
            &clock,
            22,
            "distance mars --at 2097-01-01T00:00:00Z",
        );
        assert!(matches!(
            &responses[1],
            ServerToClient::Distance { result, .. }
                if result.at_game_time == "2097-01-01T00:00:00Z"
        ));

        let responses = handle_command_message(&service(), &clock, 23, "time");
        assert!(matches!(
            &responses[1],
            ServerToClient::SimulationTime { state, .. }
                if state.current_time.starts_with("2097-01-02T00:00:")
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
        assert_eq!(
            parse_distances_args(&["--at", "2097-01-02T00:00:00Z"]).unwrap(),
            (
                None,
                DistanceSort::Name,
                Some("2097-01-02T00:00:00Z".to_string())
            )
        );
    }

    #[test]
    fn distance_to_sun_uses_player_ship_origin() {
        let responses = handle_command_message(&service(), &clock(), 12, "distance sun");

        match &responses[1] {
            ServerToClient::Distance { result, .. } => {
                assert_eq!(result.object_id, "sun");
                assert!(result.distance_km.is_finite());
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn handles_flight_plan_command_with_explicit_acceleration() {
        let service = service();
        let responses =
            handle_command_message(&service, &clock(), 40, "flight plan mars --accel 0.03");

        assert!(matches!(
            &responses[1],
            ServerToClient::FlightPlan {
                seq: 40,
                plan: Some(plan)
            } if plan.plan_id == "flight-1"
                && plan.ship_id == "player-ship"
                && plan.acceleration_km_s2 == 0.03
                && plan.status == FlightPlanStatusDto::Active
                && plan.departure_time == DEFAULT_GAME_TIME
                && plan.duration_seconds.is_finite()
                && plan.duration_seconds > 0.0
        ));
    }

    #[test]
    fn handles_flight_plan_command_with_default_acceleration() {
        let service = service();
        let responses = handle_command_message(&service, &clock(), 41, "flight plan mars");

        assert!(matches!(
            &responses[1],
            ServerToClient::FlightPlan {
                plan: Some(plan), ..
            } if plan.acceleration_km_s2 == DEFAULT_FLIGHT_ACCELERATION_KM_S2
        ));
    }

    #[test]
    fn rejects_invalid_flight_acceleration_and_preserves_motion() {
        let service = service();
        let responses =
            handle_command_message(&service, &clock(), 42, "flight plan mars --accel 0");

        assert!(matches!(
            &responses[1],
            ServerToClient::Error {
                seq: Some(42),
                error
            } if error.code == "invalid_acceleration"
        ));
        let status = handle_command_message(&service, &clock(), 43, "flight status");
        assert!(matches!(
            &status[1],
            ServerToClient::FlightPlan { plan: None, .. }
        ));
    }

    #[test]
    fn flight_status_and_cancel_report_plan_state() {
        let service = service();
        let clock = clock();
        let _ = handle_command_message(&service, &clock, 44, "flight plan mars --accel 0.02");

        let status = handle_command_message(&service, &clock, 45, "flight status");
        assert!(matches!(
            &status[1],
            ServerToClient::FlightPlan {
                seq: 45,
                plan: Some(plan)
            } if plan.plan_id == "flight-1" && plan.status == FlightPlanStatusDto::Active
        ));

        let cancelled = handle_command_message(&service, &clock, 46, "flight cancel");
        assert!(matches!(
            &cancelled[1],
            ServerToClient::FlightPlan {
                seq: 46,
                plan: Some(plan)
            } if plan.plan_id == "flight-1" && plan.status == FlightPlanStatusDto::Cancelled
        ));

        let status = handle_command_message(&service, &clock, 47, "flight status");
        assert!(matches!(
            &status[1],
            ServerToClient::FlightPlan { plan: None, .. }
        ));
    }

    #[test]
    fn replacing_flight_plan_starts_new_plan_from_current_motion() {
        let service = service();
        let clock = clock();
        let _ = handle_command_message(&service, &clock, 48, "flight plan mars --accel 0.02");
        let _ = handle_command_message(&service, &clock, 49, "advance 10 minutes");
        let replacement =
            handle_command_message(&service, &clock, 50, "flight plan venus --accel 0.04");

        assert!(matches!(
            &replacement[1],
            ServerToClient::FlightPlan {
                seq: 50,
                plan: Some(plan)
            } if plan.plan_id == "flight-2"
                && plan.acceleration_km_s2 == 0.04
                && plan.departure_time.starts_with("2097-01-01T00:10:")
        ));
    }

    #[test]
    fn ship_status_and_distance_use_active_and_completed_flight_motion() {
        let service = service();
        let clock = clock();
        let _ = handle_command_message(&service, &clock, 51, "flight plan mars --accel 0.02");

        let ship = handle_command_message(&service, &clock, 52, "ship status");
        assert!(matches!(
            &ship[1],
            ServerToClient::ShipState { ship, .. } if ship.motion_mode == "flight_plan"
        ));

        let distance = handle_command_message(&service, &clock, 53, "distance mars");
        let before_arrival = match &distance[1] {
            ServerToClient::Distance { result, .. } => result.distance_km,
            other => panic!("unexpected response: {other:?}"),
        };

        let plan = service
            .active_flight_plan(&GameTime::from_utc_iso8601(DEFAULT_GAME_TIME).unwrap())
            .unwrap();
        let arrival = GameTime::from_utc_iso8601(&plan.arrival_time)
            .unwrap()
            .add_seconds(1.0);
        let completed = service.ship_state(arrival.clone()).unwrap();
        assert_eq!(completed.motion_mode, "orbiting");

        let arrived_distance = service.distance_to("mars", arrival).unwrap();
        assert!(arrived_distance.distance_km < before_arrival);
    }
}
