use std::time::Instant;

use chrono::{DateTime, SecondsFormat, Utc};
use space_game_protocol::{
    ClientToServer, DistanceResultDto, ObjectSummaryDto, ServerToClient, SimulationTimeDto,
    StatusDto,
};

use crate::command_input::{CommandInputController, ReverseSearchView};

pub const DEFAULT_SERVER_URL: &str = "ws://127.0.0.1:4000/ws";
pub const SILENT_TIME_SYNC_SEQ: u64 = 0;
const MAX_OUTPUT_LINES: usize = 500;

#[derive(Debug, Clone)]
pub struct ClientApp {
    pub connected: bool,
    pub server_url: String,
    pub output_lines: Vec<String>,
    pub status: ClientStatusView,
    pub command_input: CommandInputController,
    pub next_seq: u64,
    pub should_quit: bool,
    clock_sample: Option<ClientClockSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientStatusView {
    pub server: String,
    pub game_time: String,
    pub observer_label: String,
    pub observer_frame: String,
    pub object_count: usize,
    pub last_update: String,
}

#[derive(Debug, Clone)]
struct ClientClockSample {
    current_time: String,
    received_at: Instant,
    running: bool,
    rate: f64,
}

impl Default for ClientApp {
    fn default() -> Self {
        Self::with_server_url(DEFAULT_SERVER_URL)
    }
}

impl ClientApp {
    pub fn with_server_url(server_url: impl Into<String>) -> Self {
        let server_url = server_url.into();
        Self {
            connected: false,
            status: ClientStatusView {
                server: server_url.clone(),
                game_time: "-".to_string(),
                observer_label: "-".to_string(),
                observer_frame: "-".to_string(),
                object_count: 0,
                last_update: "-".to_string(),
            },
            server_url,
            output_lines: vec!["Connecting to ws://127.0.0.1:4000/ws".to_string()],
            command_input: CommandInputController::default(),
            next_seq: 1,
            should_quit: false,
            clock_sample: None,
        }
    }

    pub fn input_value(&self) -> &str {
        self.command_input.value()
    }

    pub fn input_cursor_byte(&self) -> usize {
        self.command_input.cursor_byte()
    }

    pub fn input_visual_cursor(&self) -> usize {
        self.command_input.visual_cursor()
    }

    pub fn input_visual_scroll(&self, width: usize) -> usize {
        self.command_input.visual_scroll(width)
    }

    pub fn set_input(&mut self, value: impl Into<String>) {
        self.command_input.set_value(value.into());
    }

    pub fn reverse_search_view(&self) -> Option<ReverseSearchView> {
        self.command_input.reverse_search_view()
    }

    pub fn insert_char(&mut self, c: char) {
        self.command_input.insert_char(c);
    }

    pub fn backspace(&mut self) {
        self.command_input.backspace();
    }

    pub fn move_left(&mut self) {
        self.command_input.move_left();
    }

    pub fn move_right(&mut self) {
        self.command_input.move_right();
    }

    pub fn history_previous(&mut self) {
        self.command_input.history_previous();
    }

    pub fn history_next(&mut self) {
        self.command_input.history_next();
    }

    pub fn start_or_repeat_reverse_search(&mut self) {
        if self.command_input.reverse_search_view().is_some() {
            self.command_input.repeat_reverse_search();
        } else {
            self.command_input.start_reverse_search();
        }
    }

    pub fn cancel_input_mode(&mut self) -> bool {
        self.command_input.cancel_reverse_search()
    }

    pub fn complete_local_command(&mut self) -> bool {
        self.command_input.complete_local_command()
    }

    pub fn submit_input(&mut self) -> Option<ClientToServer> {
        let text = self.command_input.submit()?;

        self.push_output(format!("> {text}"));

        if matches!(text.as_str(), "quit" | "exit") {
            self.should_quit = true;
            return None;
        }

        let seq = self.next_seq;
        self.next_seq += 1;
        Some(ClientToServer::Command { seq, text })
    }

    pub fn apply_server_message(&mut self, message: ServerToClient) {
        match message {
            ServerToClient::Welcome {
                server_version,
                session_id,
            } => {
                self.connected = true;
                self.push_output(format!(
                    "Connected to server {server_version} session {session_id}"
                ));
            }
            ServerToClient::CommandAck {
                accepted: false,
                message,
                ..
            } => {
                if let Some(message) = message {
                    self.push_output(format!("Command rejected: {message}"));
                }
            }
            ServerToClient::CommandAck { .. } => {}
            ServerToClient::CompletionResponse(_) => {}
            ServerToClient::Status { status, .. } => self.apply_status(status),
            ServerToClient::Objects { objects, .. } => self.display_objects(objects),
            ServerToClient::Distance { result, .. } => self.display_distance(result),
            ServerToClient::Distances { results, .. } => {
                self.push_output("Distances:".to_string());
                for result in results {
                    self.display_distance(result);
                }
            }
            ServerToClient::SimulationTime { seq, state } => {
                self.apply_simulation_time(&state);
                if seq != Some(SILENT_TIME_SYNC_SEQ) {
                    self.display_simulation_time(&state);
                }
            }
            ServerToClient::OutputLine { line, .. } => self.push_output(line),
            ServerToClient::Error { error, .. } => {
                self.push_output(format!("Error [{}]: {}", error.code, error.message));
            }
            ServerToClient::Pong { seq } => self.push_output(format!("Pong {seq}")),
        }
    }

    pub fn push_output(&mut self, line: String) {
        self.output_lines.push(line);
        if self.output_lines.len() > MAX_OUTPUT_LINES {
            let excess = self.output_lines.len() - MAX_OUTPUT_LINES;
            self.output_lines.drain(0..excess);
        }
    }

    fn apply_status(&mut self, status: StatusDto) {
        self.clock_sample = Some(ClientClockSample {
            current_time: status.game_time.clone(),
            received_at: Instant::now(),
            running: true,
            rate: 1.0,
        });
        self.connected = status.connected;
        self.status = ClientStatusView {
            server: status.server,
            game_time: status.game_time.clone(),
            observer_label: status.observer_label,
            observer_frame: status.observer_frame,
            object_count: status.object_count,
            last_update: status.game_time,
        };
    }

    fn apply_simulation_time(&mut self, state: &SimulationTimeDto) {
        self.clock_sample = Some(ClientClockSample {
            current_time: state.current_time.clone(),
            received_at: Instant::now(),
            running: state.running,
            rate: state.rate,
        });
        self.status.game_time = state.current_time.clone();
        self.status.last_update = state.current_time.clone();
    }

    pub fn display_game_time(&self) -> String {
        self.display_game_time_at(Instant::now())
    }

    pub fn display_game_time_at(&self, now: Instant) -> String {
        let Some(sample) = &self.clock_sample else {
            return self.status.game_time.clone();
        };
        if !sample.running {
            return sample.current_time.clone();
        }
        let Ok(parsed) = DateTime::parse_from_rfc3339(&sample.current_time) else {
            return sample.current_time.clone();
        };

        let elapsed_seconds = if now >= sample.received_at {
            now.duration_since(sample.received_at).as_secs_f64()
        } else {
            -sample.received_at.duration_since(now).as_secs_f64()
        };
        let nanos = (elapsed_seconds * sample.rate * 1_000_000_000.0).round() as i64;
        let projected = parsed.with_timezone(&Utc) + chrono::Duration::nanoseconds(nanos);
        projected.to_rfc3339_opts(SecondsFormat::Secs, true)
    }

    fn display_objects(&mut self, objects: Vec<ObjectSummaryDto>) {
        let names = objects
            .into_iter()
            .map(|object| format!("{} ({})", object.display_name, object.id))
            .collect::<Vec<_>>()
            .join(", ");
        self.push_output(format!("Known objects: {names}"));
    }

    fn display_distance(&mut self, result: DistanceResultDto) {
        self.push_output(format!(
            "{}: {:.3} AU / {:.0} km",
            result.display_name, result.distance_au, result.distance_km
        ));
    }

    fn display_simulation_time(&mut self, state: &SimulationTimeDto) {
        self.push_output(format!(
            "Simulation time: {} (running={}, rate={:.1})",
            state.current_time, state.running, state.rate
        ));
    }
}

#[cfg(test)]
mod tests {
    use space_game_protocol::{
        DistanceResultDto, ErrorDto, ObjectSummaryDto, ServerToClient, SimulationTimeDto,
        StatusDto,
    };

    use super::*;

    #[test]
    fn command_submission_increments_sequence() {
        let mut app = ClientApp::default();
        app.set_input("objects");

        assert_eq!(
            app.submit_input(),
            Some(ClientToServer::Command {
                seq: 1,
                text: "objects".to_string()
            })
        );
        assert_eq!(app.next_seq, 2);
        assert!(app.output_lines.iter().any(|line| line == "> objects"));
    }

    #[test]
    fn quit_sets_quit_without_sending_command() {
        let mut app = ClientApp::default();
        app.set_input("quit");

        assert_eq!(app.submit_input(), None);
        assert!(app.should_quit);
    }

    #[test]
    fn status_update_preserves_input() {
        let mut app = ClientApp::default();
        app.set_input("distance mars");

        app.apply_server_message(ServerToClient::Status {
            seq: None,
            status: StatusDto {
                connected: true,
                server: "127.0.0.1:4000".to_string(),
                game_time: "2097-01-01T00:00:00Z".to_string(),
                observer_label: "demo-observer".to_string(),
                observer_frame: "solar_system_barycentric_j2000".to_string(),
                object_count: 8,
            },
        });

        assert_eq!(app.input_value(), "distance mars");
        assert_eq!(app.status.object_count, 8);
        assert_eq!(app.display_game_time(), "2097-01-01T00:00:00Z");
    }

    #[test]
    fn simulation_time_updates_clock_sample() {
        let mut app = ClientApp::default();

        app.apply_server_message(ServerToClient::SimulationTime {
            seq: Some(1),
            state: SimulationTimeDto {
                current_time: "2097-01-01T00:00:00Z".to_string(),
                running: true,
                rate: 1.0,
            },
        });

        assert_eq!(app.status.game_time, "2097-01-01T00:00:00Z");
        assert!(app
            .output_lines
            .iter()
            .any(|line| line.contains("Simulation time: 2097-01-01T00:00:00Z")));
    }

    #[test]
    fn silent_time_sync_updates_without_output() {
        let mut app = ClientApp::default();
        let line_count = app.output_lines.len();

        app.apply_server_message(ServerToClient::SimulationTime {
            seq: Some(SILENT_TIME_SYNC_SEQ),
            state: SimulationTimeDto {
                current_time: "2097-01-01T00:00:00Z".to_string(),
                running: true,
                rate: 1.0,
            },
        });

        assert_eq!(app.status.game_time, "2097-01-01T00:00:00Z");
        assert_eq!(app.output_lines.len(), line_count);
    }

    #[test]
    fn projects_running_clock_sample() {
        let mut app = ClientApp::default();
        let received_at = Instant::now();
        app.clock_sample = Some(ClientClockSample {
            current_time: "2097-01-01T00:00:00Z".to_string(),
            received_at,
            running: true,
            rate: 1.0,
        });

        assert_eq!(
            app.display_game_time_at(received_at + std::time::Duration::from_secs(7)),
            "2097-01-01T00:00:07Z"
        );
    }

    #[test]
    fn presents_server_messages() {
        let mut app = ClientApp::default();

        app.apply_server_message(ServerToClient::Objects {
            seq: 1,
            objects: vec![ObjectSummaryDto {
                id: "mars".to_string(),
                display_name: "Mars".to_string(),
                kind: "planet".to_string(),
            }],
        });
        app.apply_server_message(ServerToClient::Distance {
            seq: 2,
            result: DistanceResultDto {
                object_id: "mars".to_string(),
                display_name: "Mars".to_string(),
                distance_km: 78_000_000.0,
                distance_au: 0.521,
                at_game_time: "2097-01-01T00:00:00Z".to_string(),
                quality: Some("fictional".to_string()),
            },
        });
        app.apply_server_message(ServerToClient::Error {
            seq: Some(3),
            error: ErrorDto {
                code: "unknown_command".to_string(),
                message: "unknown command".to_string(),
            },
        });

        assert!(app
            .output_lines
            .iter()
            .any(|line| line.contains("Known objects")));
        assert!(app.output_lines.iter().any(|line| line.contains("Mars:")));
        assert!(app
            .output_lines
            .iter()
            .any(|line| line.contains("unknown command")));
    }
}
