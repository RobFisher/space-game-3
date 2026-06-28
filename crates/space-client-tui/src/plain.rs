use color_eyre::eyre::{eyre, Result};
use futures::{stream::Stream, Sink, SinkExt, StreamExt};
use space_game_protocol::{
    ArrivalOrbitDto, ClientToServer, DistanceResultDto, FlightPlanDto, FlightPlanStatusDto,
    FlightPlanTargetDto, LocationSummaryDto, ObjectSummaryDto, ServerToClient, ShipStateDto,
    SimulationTimeDto, StatusDto,
};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite, tungstenite::Message};

use crate::app::ClientApp;

pub async fn run_plain<R, W>(
    mut app: ClientApp,
    command: Option<String>,
    input: R,
    mut output: W,
) -> Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let (socket, _) = connect_async(app.server_url.as_str()).await?;
    let (mut writer, mut reader) = socket.split();

    writer
        .send(Message::Text(serde_json::to_string(
            &ClientToServer::Hello {
                client_name: "space-client-tui".to_string(),
                client_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        )?))
        .await?;

    if let Some(command) = command {
        if let Some(seq) = send_plain_command(&mut app, &mut writer, command.trim()).await? {
            read_command_response(&mut app, &mut reader, &mut output, seq).await?;
        }
        output.flush().await?;
        return Ok(());
    }

    let mut lines = input.lines();
    while let Some(line) = lines.next_line().await? {
        let command = line.trim();
        let Some(seq) = send_plain_command(&mut app, &mut writer, command).await? else {
            if app.should_quit {
                break;
            }
            continue;
        };
        read_command_response(&mut app, &mut reader, &mut output, seq).await?;
    }

    output.flush().await?;
    Ok(())
}

pub fn plain_output_lines(message: &ServerToClient, expected_seq: u64) -> Vec<String> {
    match message {
        ServerToClient::Objects { seq, objects } if *seq == expected_seq => {
            vec![format!("Known objects: {}", format_objects(objects))]
        }
        ServerToClient::Distance { seq, result } if *seq == expected_seq => {
            vec![format_distance(result)]
        }
        ServerToClient::Distances { seq, results } if *seq == expected_seq => {
            let mut lines = vec!["Distances:".to_string()];
            lines.extend(results.iter().map(format_distance));
            lines
        }
        ServerToClient::ShipState { seq, ship } if *seq == expected_seq => {
            vec![format_ship_state(ship)]
        }
        ServerToClient::FlightPlan { seq, plan } if *seq == expected_seq => {
            vec![format_flight_plan(plan.as_ref())]
        }
        ServerToClient::LocationSummary { seq, summary } if *seq == expected_seq => {
            vec![format_location(summary)]
        }
        ServerToClient::Status {
            seq: Some(seq),
            status,
        } if *seq == expected_seq => vec![format_status(status)],
        ServerToClient::SimulationTime {
            seq: Some(seq),
            state,
        } if *seq == expected_seq => vec![format_simulation_time(state)],
        ServerToClient::OutputLine {
            seq: Some(seq),
            line,
        } if *seq == expected_seq => vec![line.clone()],
        ServerToClient::Error {
            seq: Some(seq),
            error,
        } if *seq == expected_seq => vec![format!("Error [{}]: {}", error.code, error.message)],
        ServerToClient::Pong { seq } if *seq == expected_seq => vec![format!("Pong {seq}")],
        _ => Vec::new(),
    }
}

async fn send_plain_command<W>(
    app: &mut ClientApp,
    writer: &mut W,
    command: &str,
) -> Result<Option<u64>>
where
    W: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let Some(message) = prepare_plain_command(app, command) else {
        return Ok(None);
    };
    let ClientToServer::Command { seq, .. } = &message else {
        unreachable!("plain mode only prepares command messages");
    };
    let seq = *seq;
    writer
        .send(Message::Text(serde_json::to_string(&message)?))
        .await?;
    Ok(Some(seq))
}

fn prepare_plain_command(app: &mut ClientApp, command: &str) -> Option<ClientToServer> {
    if command.is_empty() {
        return None;
    }
    if matches!(command, "quit" | "exit") {
        app.should_quit = true;
        return None;
    }

    let seq = app.next_seq;
    app.next_seq += 1;
    Some(ClientToServer::Command {
        seq,
        text: command.to_string(),
    })
}

async fn read_command_response<S, W>(
    app: &mut ClientApp,
    reader: &mut S,
    output: &mut W,
    expected_seq: u64,
) -> Result<()>
where
    S: Stream<Item = Result<Message, tungstenite::Error>> + Unpin,
    W: AsyncWrite + Unpin,
{
    while let Some(message) = reader.next().await {
        let message = message?;
        let Message::Text(text) = message else {
            if matches!(message, Message::Close(_)) {
                return Err(eyre!("server connection closed"));
            }
            continue;
        };
        let protocol_message: ServerToClient = serde_json::from_str(&text)?;
        let output_lines = plain_output_lines(&protocol_message, expected_seq);
        let is_complete = is_command_completion(&protocol_message, expected_seq);
        app.apply_server_message(protocol_message);

        for line in output_lines {
            output.write_all(line.as_bytes()).await?;
            output.write_all(b"\n").await?;
        }

        if is_complete {
            return Ok(());
        }
    }

    Err(eyre!("server connection closed"))
}

fn is_command_completion(message: &ServerToClient, expected_seq: u64) -> bool {
    match message {
        ServerToClient::Objects { seq, .. }
        | ServerToClient::Distance { seq, .. }
        | ServerToClient::Distances { seq, .. }
        | ServerToClient::ShipState { seq, .. }
        | ServerToClient::FlightPlan { seq, .. }
        | ServerToClient::LocationSummary { seq, .. }
        | ServerToClient::Pong { seq } => *seq == expected_seq,
        ServerToClient::Status { seq: Some(seq), .. }
        | ServerToClient::SimulationTime { seq: Some(seq), .. }
        | ServerToClient::OutputLine { seq: Some(seq), .. }
        | ServerToClient::Error { seq: Some(seq), .. } => *seq == expected_seq,
        _ => false,
    }
}

fn format_objects(objects: &[ObjectSummaryDto]) -> String {
    objects
        .iter()
        .map(|object| format!("{} ({})", object.display_name, object.id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_distance(result: &DistanceResultDto) -> String {
    format!(
        "{}: {:.3} AU / {:.0} km",
        result.display_name, result.distance_au, result.distance_km
    )
}

fn format_location(summary: &LocationSummaryDto) -> String {
    format!(
        "Location: {} nearest {} ({}) at {:.3} AU / {:.0} km frame={} game_time={}",
        summary.subject_label,
        summary.nearest_object_name,
        summary.nearest_object_id,
        summary.distance_au,
        summary.distance_km,
        summary.frame,
        summary.game_time
    )
}

fn format_status(status: &StatusDto) -> String {
    format!(
        "Status: connected={} server={} game_time={} ship={} motion={} frame={} objects={}",
        status.connected,
        status.server,
        status.game_time,
        status.ship_name,
        status.ship_motion,
        status.ship_frame,
        status.object_count
    )
}

fn format_ship_state(ship: &ShipStateDto) -> String {
    format!(
        "Ship: {} motion={} frame={} game_time={}",
        ship.ship_name, ship.motion_mode, ship.frame, ship.game_time
    )
}

fn format_flight_plan(plan: Option<&FlightPlanDto>) -> String {
    let Some(plan) = plan else {
        return "No active flight plan.".to_string();
    };
    let FlightPlanTargetDto::Object {
        object_id,
        display_name,
    } = &plan.target;
    format!(
        "Flight plan {}: {} to {} ({}) phase={} acceleration={} departure={} arrival={} orbit_entry={} duration={:.0}s orbit={}",
        plan.plan_id,
        flight_status_label(plan.status),
        display_name,
        object_id,
        plan.navigation_phase,
        format_acceleration(plan),
        plan.departure_time,
        plan.arrival_time,
        plan.orbit_entry_time,
        plan.duration_seconds,
        format_arrival_orbit(plan.arrival_orbit.as_ref())
    )
}

fn format_acceleration(plan: &FlightPlanDto) -> String {
    match plan.acceleration_g {
        Some(g) => format!("{:.3} km/s^2 ({g:.3}g)", plan.acceleration_km_s2),
        None => format!("{:.3} km/s^2", plan.acceleration_km_s2),
    }
}

fn format_arrival_orbit(orbit: Option<&ArrivalOrbitDto>) -> String {
    let Some(orbit) = orbit else {
        return "-".to_string();
    };
    let mut parts = vec![format!("{} radius={:.0} km", orbit.kind, orbit.radius_km)];
    if let Some(altitude_km) = orbit.altitude_km {
        parts.push(format!("altitude={altitude_km:.0} km"));
    }
    if let Some(period_seconds) = orbit.period_seconds {
        parts.push(format!(
            "period={}",
            format_duration_seconds(period_seconds)
        ));
    }
    if let Some(speed) = orbit.circular_speed_km_s {
        parts.push(format!("speed={speed:.3} km/s"));
    }
    parts.join(" ")
}

fn format_duration_seconds(seconds: f64) -> String {
    if !seconds.is_finite() {
        return "-".to_string();
    }
    let seconds = seconds.round().max(0.0) as i64;
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m {seconds:02}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

fn flight_status_label(status: FlightPlanStatusDto) -> &'static str {
    match status {
        FlightPlanStatusDto::Active => "active",
        FlightPlanStatusDto::Completed => "completed",
        FlightPlanStatusDto::Cancelled => "cancelled",
    }
}

fn format_simulation_time(state: &SimulationTimeDto) -> String {
    format!(
        "Simulation time: {} running={} rate={}",
        state.current_time, state.running, state.rate
    )
}

#[cfg(test)]
mod tests {
    use space_game_protocol::{
        ArrivalOrbitDto, ErrorDto, FlightPlanDto, FlightPlanStatusDto, FlightPlanTargetDto,
        LocationSummaryDto, ObjectSummaryDto, ShipStateDto,
    };

    use super::*;

    #[test]
    fn formats_object_response_for_expected_sequence() {
        let lines = plain_output_lines(
            &ServerToClient::Objects {
                seq: 3,
                objects: vec![ObjectSummaryDto {
                    id: "mars".to_string(),
                    display_name: "Mars".to_string(),
                    kind: "planet".to_string(),
                }],
            },
            3,
        );

        assert_eq!(lines, vec!["Known objects: Mars (mars)"]);
    }

    #[test]
    fn ignores_uncorrelated_startup_status() {
        let lines = plain_output_lines(
            &ServerToClient::Status {
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
            },
            3,
        );

        assert!(lines.is_empty());
    }

    #[test]
    fn formats_status_response_for_expected_sequence() {
        let lines = plain_output_lines(
            &ServerToClient::Status {
                seq: Some(4),
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
            },
            4,
        );

        assert_eq!(
            lines,
            vec!["Status: connected=true server=127.0.0.1:4000 game_time=2097-01-01T00:00:00Z ship=Wayfarer motion=orbiting frame=solar_system_barycentric_j2000 objects=8"]
        );
    }

    #[test]
    fn formats_ship_response_for_expected_sequence() {
        let lines = plain_output_lines(
            &ServerToClient::ShipState {
                seq: 8,
                ship: ShipStateDto {
                    ship_id: "player-ship".to_string(),
                    ship_name: "Wayfarer".to_string(),
                    motion_mode: "orbiting".to_string(),
                    frame: "solar_system_barycentric_j2000".to_string(),
                    game_time: "2097-01-01T00:00:00Z".to_string(),
                    quality: Some("fictional".to_string()),
                },
            },
            8,
        );

        assert_eq!(
            lines,
            vec!["Ship: Wayfarer motion=orbiting frame=solar_system_barycentric_j2000 game_time=2097-01-01T00:00:00Z"]
        );
    }

    #[test]
    fn formats_flight_plan_response_for_expected_sequence() {
        let lines = plain_output_lines(
            &ServerToClient::FlightPlan {
                seq: 9,
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
                    acceleration_g: Some(2.039),
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
            },
            9,
        );

        assert_eq!(
            lines,
            vec!["Flight plan flight-1: active to Mars (mars) phase=flight_plan acceleration=0.020 km/s^2 (2.039g) departure=2097-01-01T00:00:00Z arrival=2097-01-01T03:00:00Z orbit_entry=2097-01-01T03:10:00Z duration=10800s orbit=low radius=3790 km altitude=400 km period=1h 58m 33s speed=3.362 km/s"]
        );
    }

    #[test]
    fn formats_cancelled_and_no_active_flight_plan_responses() {
        let cancelled = plain_output_lines(
            &ServerToClient::FlightPlan {
                seq: 10,
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
                    acceleration_g: None,
                    status: FlightPlanStatusDto::Cancelled,
                    navigation_phase: "cancelled".to_string(),
                    arrival_orbit: None,
                    quality: Some("fictional".to_string()),
                }),
            },
            10,
        );
        let none = plain_output_lines(
            &ServerToClient::FlightPlan {
                seq: 11,
                plan: None,
            },
            11,
        );

        assert!(cancelled[0].contains("Flight plan flight-1: cancelled to Mars"));
        assert_eq!(none, vec!["No active flight plan."]);
    }

    #[test]
    fn formats_simulation_time_response_for_expected_sequence() {
        let lines = plain_output_lines(
            &ServerToClient::SimulationTime {
                seq: Some(6),
                state: SimulationTimeDto {
                    current_time: "2097-01-01T00:00:00Z".to_string(),
                    running: true,
                    rate: 1.0,
                },
            },
            6,
        );

        assert_eq!(
            lines,
            vec!["Simulation time: 2097-01-01T00:00:00Z running=true rate=1"]
        );
    }

    #[test]
    fn formats_location_summary_for_expected_sequence_without_coordinates() {
        let lines = plain_output_lines(
            &ServerToClient::LocationSummary {
                seq: 7,
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
            },
            7,
        );

        assert_eq!(
            lines,
            vec!["Location: Mars nearest Phobos (phobos) at 0.000 AU / 42000 km frame=solar_system_barycentric_j2000 game_time=2097-01-01T00:00:00Z"]
        );
        assert!(!lines[0].contains(" x"));
        assert!(!lines[0].contains(" y"));
        assert!(!lines[0].contains(" z"));
    }

    #[test]
    fn formats_protocol_error_response() {
        let lines = plain_output_lines(
            &ServerToClient::Error {
                seq: Some(5),
                error: ErrorDto {
                    code: "unknown_command".to_string(),
                    message: "unknown command: launch".to_string(),
                },
            },
            5,
        );

        assert_eq!(
            lines,
            vec!["Error [unknown_command]: unknown command: launch"]
        );
    }

    #[test]
    fn prepares_plain_commands_with_monotonic_sequences() {
        let mut app = ClientApp::default();

        assert_eq!(
            prepare_plain_command(&mut app, "objects"),
            Some(ClientToServer::Command {
                seq: 1,
                text: "objects".to_string()
            })
        );
        assert_eq!(
            prepare_plain_command(&mut app, "status"),
            Some(ClientToServer::Command {
                seq: 2,
                text: "status".to_string()
            })
        );
    }

    #[test]
    fn ignores_empty_plain_commands() {
        let mut app = ClientApp::default();

        assert_eq!(prepare_plain_command(&mut app, ""), None);
        assert_eq!(app.next_seq, 1);
        assert!(!app.should_quit);
    }

    #[test]
    fn handles_quit_locally() {
        let mut app = ClientApp::default();

        assert_eq!(prepare_plain_command(&mut app, "quit"), None);
        assert_eq!(app.next_seq, 1);
        assert!(app.should_quit);
    }
}
