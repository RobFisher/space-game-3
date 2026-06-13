use color_eyre::eyre::{eyre, Result};
use futures::{stream::Stream, Sink, SinkExt, StreamExt};
use space_game_protocol::{
    ClientToServer, DistanceResultDto, ObjectSummaryDto, ServerToClient, StatusDto,
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
        ServerToClient::Status {
            seq: Some(seq),
            status,
        } if *seq == expected_seq => vec![format_status(status)],
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
        | ServerToClient::Pong { seq } => *seq == expected_seq,
        ServerToClient::Status { seq: Some(seq), .. }
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

fn format_status(status: &StatusDto) -> String {
    format!(
        "Status: connected={} server={} game_time={} observer={} frame={} objects={}",
        status.connected,
        status.server,
        status.game_time,
        status.observer_label,
        status.observer_frame,
        status.object_count
    )
}

#[cfg(test)]
mod tests {
    use space_game_protocol::{ErrorDto, ObjectSummaryDto};

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
                    observer_label: "demo-observer".to_string(),
                    observer_frame: "solar_system_barycentric_j2000".to_string(),
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
                    observer_label: "demo-observer".to_string(),
                    observer_frame: "solar_system_barycentric_j2000".to_string(),
                    object_count: 8,
                },
            },
            4,
        );

        assert_eq!(
            lines,
            vec!["Status: connected=true server=127.0.0.1:4000 game_time=2097-01-01T00:00:00Z observer=demo-observer frame=solar_system_barycentric_j2000 objects=8"]
        );
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
