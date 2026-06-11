use space_game_protocol::{
    ClientToServer, DistanceResultDto, ObjectSummaryDto, ServerToClient, StatusDto,
};

const MAX_OUTPUT_LINES: usize = 500;

#[derive(Debug, Clone)]
pub struct ClientApp {
    pub connected: bool,
    pub server_url: String,
    pub output_lines: Vec<String>,
    pub status: ClientStatusView,
    pub input: String,
    pub cursor: usize,
    pub next_seq: u64,
    pub should_quit: bool,
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

impl Default for ClientApp {
    fn default() -> Self {
        let server_url = "ws://127.0.0.1:4000/ws".to_string();
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
            input: String::new(),
            cursor: 0,
            next_seq: 1,
            should_quit: false,
        }
    }
}

impl ClientApp {
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some((previous, _)) = self.input[..self.cursor].char_indices().last() {
            self.input.drain(previous..self.cursor);
            self.cursor = previous;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some((previous, _)) = self.input[..self.cursor].char_indices().last() {
            self.cursor = previous;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor >= self.input.len() {
            return;
        }
        if let Some((offset, ch)) = self.input[self.cursor..].char_indices().next() {
            self.cursor += offset + ch.len_utf8();
        }
    }

    pub fn submit_input(&mut self) -> Option<ClientToServer> {
        let text = self.input.trim().to_string();
        self.input.clear();
        self.cursor = 0;

        if text.is_empty() {
            return None;
        }

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
            ServerToClient::Status { status, .. } => self.apply_status(status),
            ServerToClient::Objects { objects, .. } => self.display_objects(objects),
            ServerToClient::Distance { result, .. } => self.display_distance(result),
            ServerToClient::Distances { results, .. } => {
                self.push_output("Distances:".to_string());
                for result in results {
                    self.display_distance(result);
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
}

#[cfg(test)]
mod tests {
    use space_game_protocol::{
        DistanceResultDto, ErrorDto, ObjectSummaryDto, ServerToClient, StatusDto,
    };

    use super::*;

    #[test]
    fn command_submission_increments_sequence() {
        let mut app = ClientApp::default();
        app.input = "objects".to_string();
        app.cursor = app.input.len();

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
        app.input = "quit".to_string();
        app.cursor = app.input.len();

        assert_eq!(app.submit_input(), None);
        assert!(app.should_quit);
    }

    #[test]
    fn status_update_preserves_input() {
        let mut app = ClientApp::default();
        app.input = "distance mars".to_string();
        app.cursor = app.input.len();

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

        assert_eq!(app.input, "distance mars");
        assert_eq!(app.status.object_count, 8);
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
