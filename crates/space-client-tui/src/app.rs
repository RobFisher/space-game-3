use std::{io, time::Instant};

use chrono::{DateTime, SecondsFormat, Utc};
use space_game_protocol::{
    ArrivalOrbitDto, ClientToServer, CompletionCandidateDto, DistanceResultDto, LocationSummaryDto,
    ObjectSummaryDto, ServerToClient, ShipStateDto, SimulationTimeDto, StatusDto,
};
use space_game_protocol::{FlightPlanDto, FlightPlanStatusDto, FlightPlanTargetDto};

use crate::{
    command_input::{CommandInputController, ReverseSearchView},
    history::CommandHistoryStore,
};

pub const DEFAULT_SERVER_URL: &str = "ws://127.0.0.1:4000/ws";
pub const SILENT_TIME_SYNC_SEQ: u64 = 0;
pub const SILENT_FLIGHT_STATUS_SEQ: u64 = u64::MAX;
const AU_KM: f64 = 149_597_870.7;
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
    active_flight_plan: Option<FlightPlanDto>,
    history_store: CommandHistoryStore,
    clock_sample: Option<ClientClockSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientStatusView {
    pub server: String,
    pub game_time: String,
    pub ship_id: String,
    pub ship_name: String,
    pub ship_frame: String,
    pub ship_motion: String,
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
                ship_id: "-".to_string(),
                ship_name: "-".to_string(),
                ship_frame: "-".to_string(),
                ship_motion: "-".to_string(),
                object_count: 0,
                last_update: "-".to_string(),
            },
            server_url,
            output_lines: vec!["Connecting to ws://127.0.0.1:4000/ws".to_string()],
            command_input: CommandInputController::default(),
            next_seq: 1,
            should_quit: false,
            active_flight_plan: None,
            history_store: CommandHistoryStore::disabled(),
            clock_sample: None,
        }
    }

    pub fn with_history_store(
        server_url: impl Into<String>,
        history_store: CommandHistoryStore,
    ) -> io::Result<Self> {
        let mut app = Self::with_server_url(server_url);
        let history = history_store.load()?;
        app.command_input.set_history(history);
        app.history_store = history_store;
        Ok(app)
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
        self.command_input.cancel_reverse_search() || self.command_input.cancel_pending_completion()
    }

    pub fn complete_local_command(&mut self) -> bool {
        self.command_input.complete_local_command()
    }

    pub fn request_completion(&mut self, now: Instant) -> ClientToServer {
        let seq = self.next_seq;
        self.next_seq += 1;
        ClientToServer::CompletionRequest(self.command_input.completion_request(seq, now))
    }

    pub fn completion_candidates(&self) -> &[CompletionCandidateDto] {
        self.command_input.completion_candidates()
    }

    pub fn show_completion_pending(&self, now: Instant) -> bool {
        self.command_input.show_completion_pending(now)
    }

    pub fn submit_input(&mut self) -> Option<ClientToServer> {
        let text = self.command_input.submit()?;

        self.push_output(format!("> {text}"));

        if matches!(text.as_str(), "quit" | "exit") {
            self.should_quit = true;
            return None;
        }

        if self.command_input.take_history_dirty() {
            let history = self.command_input.history().to_vec();
            if let Err(err) = self.history_store.save(&history) {
                self.push_output(format!("Unable to save command history: {err}"));
            }
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
            ServerToClient::CompletionResponse(response) => {
                self.command_input.apply_completion_response(response);
            }
            ServerToClient::Status { status, .. } => self.apply_status(status),
            ServerToClient::Objects { objects, .. } => self.display_objects(objects),
            ServerToClient::Distance { result, .. } => self.display_distance(result),
            ServerToClient::Distances { results, .. } => {
                self.push_output("Distances:".to_string());
                for result in results {
                    self.display_distance(result);
                }
            }
            ServerToClient::ShipState { ship, .. } => self.display_ship_state(ship),
            ServerToClient::FlightPlan { seq, plan } => {
                self.active_flight_plan = plan
                    .clone()
                    .filter(|plan| plan.status == FlightPlanStatusDto::Active);
                if seq != SILENT_FLIGHT_STATUS_SEQ {
                    self.push_output(format_flight_plan(plan.as_ref()));
                }
            }
            ServerToClient::LocationSummary { summary, .. } => self.display_location(summary),
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
            ship_id: status.ship_id,
            ship_name: status.ship_name,
            ship_frame: status.ship_frame,
            ship_motion: status.ship_motion,
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

    pub fn active_flight_status_lines(&self, now: Instant) -> Vec<String> {
        let Some(plan) = &self.active_flight_plan else {
            return Vec::new();
        };
        let FlightPlanTargetDto::Object {
            object_id,
            display_name,
        } = &plan.target;
        vec![
            format!("Flight: {display_name} ({object_id})"),
            format!("Phase: {}", plan.navigation_phase),
            format!("ETA: {}", plan.arrival_time),
            format!(
                "Countdown: {}",
                format_countdown(&self.display_game_time_at(now), &plan.arrival_time)
            ),
            format!(
                "Distance: {}",
                format_remaining_distance(&self.display_game_time_at(now), plan)
            ),
            format!(
                "Navigation: {}",
                format_phase_detail(&self.display_game_time_at(now), plan)
            ),
        ]
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

    fn display_location(&mut self, summary: LocationSummaryDto) {
        self.push_output(format!(
            "{} is nearest {} at {:.3} AU / {:.0} km (frame {}, time {})",
            summary.subject_label,
            summary.nearest_object_name,
            summary.distance_au,
            summary.distance_km,
            summary.frame,
            summary.game_time
        ));
    }

    fn display_ship_state(&mut self, ship: ShipStateDto) {
        self.status.ship_id = ship.ship_id;
        self.status.ship_name = ship.ship_name.clone();
        self.status.ship_frame = ship.frame.clone();
        self.status.ship_motion = ship.motion_mode.clone();
        self.status.game_time = ship.game_time.clone();
        self.status.last_update = ship.game_time.clone();
        self.push_output(format!(
            "Ship: {} ({}, frame {}, time {})",
            ship.ship_name, ship.motion_mode, ship.frame, ship.game_time
        ));
    }

    fn display_simulation_time(&mut self, state: &SimulationTimeDto) {
        self.push_output(format!(
            "Simulation time: {} (running={}, rate={:.1})",
            state.current_time, state.running, state.rate
        ));
    }
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
        "Flight plan {}: {} to {} ({}) phase={} acceleration={} departure={} arrival={} orbit_entry={} duration={:.0}s navigation={}",
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
        format_phase_detail(&plan.departure_time, plan)
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

fn format_phase_detail(current_time: &str, plan: &FlightPlanDto) -> String {
    match plan.navigation_phase.as_str() {
        "flight_plan" => format_transfer_dynamics(current_time, plan),
        "entering_orbit" => "Entering orbit".to_string(),
        "orbiting" => format_arrival_orbit(plan.arrival_orbit.as_ref()),
        "cancelled" => "Cancelled".to_string(),
        _ => "-".to_string(),
    }
}

fn format_transfer_dynamics(current_time: &str, plan: &FlightPlanDto) -> String {
    let acceleration = format_current_acceleration(current_time, plan);
    let speed = format_transfer_speed(current_time, plan);
    format!("acceleration={acceleration} path_speed={speed}")
}

fn format_current_acceleration(current_time: &str, plan: &FlightPlanDto) -> String {
    if plan.duration_seconds <= 0.0 || plan.acceleration_km_s2 <= 0.0 {
        return "0.000 km/s^2".to_string();
    }
    let Some(normalized) = normalized_flight_progress(current_time, plan) else {
        return "-".to_string();
    };
    let direction = if normalized < 0.5 {
        "accelerating"
    } else {
        "decelerating"
    };
    format!("{} {direction}", format_acceleration(plan))
}

fn format_transfer_speed(current_time: &str, plan: &FlightPlanDto) -> String {
    let Some(speed_km_s) = transfer_speed_km_s(current_time, plan) else {
        return "-".to_string();
    };
    format!("{speed_km_s:.3} km/s")
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

fn format_countdown(current_time: &str, arrival_time: &str) -> String {
    let Ok(current) = DateTime::parse_from_rfc3339(current_time) else {
        return "-".to_string();
    };
    let Ok(arrival) = DateTime::parse_from_rfc3339(arrival_time) else {
        return "-".to_string();
    };
    let seconds = arrival
        .with_timezone(&Utc)
        .signed_duration_since(current.with_timezone(&Utc))
        .num_seconds();
    if seconds <= 0 {
        return "arrived".to_string();
    }
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if days > 0 {
        format!("{days}d {hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

fn format_remaining_distance(current_time: &str, plan: &FlightPlanDto) -> String {
    let Some(distance_km) = remaining_distance_km(current_time, plan) else {
        return "-".to_string();
    };
    if distance_km >= 0.01 * AU_KM {
        format!("{:.3} AU / {:.0} km", distance_km / AU_KM, distance_km)
    } else {
        format!("{distance_km:.0} km")
    }
}

fn remaining_distance_km(current_time: &str, plan: &FlightPlanDto) -> Option<f64> {
    if plan.duration_seconds <= 0.0 || plan.acceleration_km_s2 <= 0.0 {
        return Some(0.0);
    }
    let normalized = normalized_flight_progress(current_time, plan)?;
    let total_distance_km =
        plan.acceleration_km_s2 * (plan.duration_seconds / 2.0) * (plan.duration_seconds / 2.0);
    Some(total_distance_km * (1.0 - ease_in_out_accel_decel(normalized)))
}

fn transfer_speed_km_s(current_time: &str, plan: &FlightPlanDto) -> Option<f64> {
    if plan.duration_seconds <= 0.0 || plan.acceleration_km_s2 <= 0.0 {
        return Some(0.0);
    }
    let normalized = normalized_flight_progress(current_time, plan)?;
    let total_distance_km =
        plan.acceleration_km_s2 * (plan.duration_seconds / 2.0) * (plan.duration_seconds / 2.0);
    Some(total_distance_km * ease_in_out_accel_decel_derivative(normalized) / plan.duration_seconds)
}

fn normalized_flight_progress(current_time: &str, plan: &FlightPlanDto) -> Option<f64> {
    let current = DateTime::parse_from_rfc3339(current_time)
        .ok()?
        .with_timezone(&Utc);
    let departure = DateTime::parse_from_rfc3339(&plan.departure_time)
        .ok()?
        .with_timezone(&Utc);
    let elapsed_seconds =
        current.signed_duration_since(departure).num_milliseconds() as f64 / 1_000.0;
    Some((elapsed_seconds / plan.duration_seconds).clamp(0.0, 1.0))
}

fn ease_in_out_accel_decel(normalized: f64) -> f64 {
    if normalized < 0.5 {
        2.0 * normalized * normalized
    } else {
        1.0 - 2.0 * (1.0 - normalized) * (1.0 - normalized)
    }
}

fn ease_in_out_accel_decel_derivative(normalized: f64) -> f64 {
    if normalized < 0.5 {
        4.0 * normalized
    } else {
        4.0 * (1.0 - normalized)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use space_game_protocol::{
        ArrivalOrbitDto, DistanceResultDto, ErrorDto, FlightPlanDto, FlightPlanStatusDto,
        FlightPlanTargetDto, LocationSummaryDto, ObjectSummaryDto, ServerToClient, ShipStateDto,
        SimulationTimeDto, StatusDto,
    };

    use super::*;

    fn temp_history_path(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("space-game-app-{name}-{id}.history"))
    }

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
    fn submits_where_command() {
        let mut app = ClientApp::default();
        app.set_input("where");

        assert_eq!(
            app.submit_input(),
            Some(ClientToServer::Command {
                seq: 1,
                text: "where".to_string()
            })
        );
    }

    #[test]
    fn submits_where_object_command() {
        let mut app = ClientApp::default();
        app.set_input("where mars --at 2097-01-02T00:00:00Z");

        assert_eq!(
            app.submit_input(),
            Some(ClientToServer::Command {
                seq: 1,
                text: "where mars --at 2097-01-02T00:00:00Z".to_string()
            })
        );
    }

    #[test]
    fn submits_flight_commands() {
        let cases = [
            "flight plan mars --accel 0.02",
            "flight plan mars --accel 0.5g --orbit low",
            "flight plan mars --orbit-radius 10000",
            "flight status",
            "flight cancel",
        ];

        for command in cases {
            let mut app = ClientApp::default();
            app.set_input(command);

            assert_eq!(
                app.submit_input(),
                Some(ClientToServer::Command {
                    seq: 1,
                    text: command.to_string()
                })
            );
        }
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
                ship_id: "player-ship".to_string(),
                ship_name: "Wayfarer".to_string(),
                ship_frame: "solar_system_barycentric_j2000".to_string(),
                ship_motion: "orbiting".to_string(),
                object_count: 8,
            },
        });

        assert_eq!(app.input_value(), "distance mars");
        assert_eq!(app.status.object_count, 8);
        assert_eq!(app.status.ship_name, "Wayfarer");
        assert_eq!(app.display_game_time(), "2097-01-01T00:00:00Z");
    }

    #[test]
    fn loads_history_from_injected_store() {
        let path = temp_history_path("load");
        fs::write(&path, "objects\nstatus\n").unwrap();

        let mut app =
            ClientApp::with_history_store(DEFAULT_SERVER_URL, CommandHistoryStore::path(&path))
                .unwrap();
        app.history_previous();

        assert_eq!(app.input_value(), "status");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn saves_submitted_history_to_injected_store() {
        let path = temp_history_path("save");
        let mut app =
            ClientApp::with_history_store(DEFAULT_SERVER_URL, CommandHistoryStore::path(&path))
                .unwrap();
        app.set_input("objects");

        assert!(matches!(
            app.submit_input(),
            Some(ClientToServer::Command { text, .. }) if text == "objects"
        ));

        assert_eq!(fs::read_to_string(&path).unwrap(), "objects");

        let _ = fs::remove_file(path);
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
        app.apply_server_message(ServerToClient::LocationSummary {
            seq: 4,
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
        });
        app.apply_server_message(ServerToClient::ShipState {
            seq: 5,
            ship: ShipStateDto {
                ship_id: "player-ship".to_string(),
                ship_name: "Wayfarer".to_string(),
                motion_mode: "orbiting".to_string(),
                frame: "solar_system_barycentric_j2000".to_string(),
                game_time: "2097-01-01T00:00:00Z".to_string(),
                quality: Some("fictional".to_string()),
            },
        });
        app.apply_server_message(ServerToClient::FlightPlan {
            seq: 6,
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
            .any(|line| line.contains("Ship: Wayfarer")));
        assert!(app.output_lines.iter().any(|line| {
            line.contains("Flight plan flight-1: active to Mars")
                && line.contains("phase=flight_plan")
                && line.contains("acceleration=0.020 km/s^2 (2.039g)")
                && line.contains("navigation=acceleration=0.020 km/s^2 (2.039g) accelerating")
                && line.contains("path_speed=0.000 km/s")
                && !line.contains("orbit=low")
        }));
        let location_line = app
            .output_lines
            .iter()
            .find(|line| line.contains("Mars is nearest Phobos"))
            .expect("location summary output");
        assert!(location_line.contains("solar_system_barycentric_j2000"));
        assert!(location_line.contains("2097-01-01T00:00:00Z"));
        assert!(!location_line.contains(" x"));
        assert!(!location_line.contains(" y"));
        assert!(!location_line.contains(" z"));
        assert!(app
            .output_lines
            .iter()
            .any(|line| line.contains("unknown command")));
    }

    #[test]
    fn presents_cancelled_and_no_active_flight_plan_responses() {
        let mut app = ClientApp::default();

        app.apply_server_message(ServerToClient::FlightPlan {
            seq: 1,
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
        });
        app.apply_server_message(ServerToClient::FlightPlan { seq: 2, plan: None });

        assert!(app
            .output_lines
            .iter()
            .any(|line| line.contains("Flight plan flight-1: cancelled to Mars")));
        assert!(app
            .output_lines
            .iter()
            .any(|line| line == "No active flight plan."));
    }

    #[test]
    fn active_flight_status_lines_include_eta_and_countdown() {
        let mut app = ClientApp::default();
        let received_at = Instant::now();
        app.clock_sample = Some(ClientClockSample {
            current_time: "2097-01-01T00:00:00Z".to_string(),
            received_at,
            running: true,
            rate: 1.0,
        });

        app.apply_server_message(ServerToClient::FlightPlan {
            seq: 1,
            plan: Some(FlightPlanDto {
                plan_id: "flight-1".to_string(),
                ship_id: "player-ship".to_string(),
                target: FlightPlanTargetDto::Object {
                    object_id: "mars".to_string(),
                    display_name: "Mars".to_string(),
                },
                departure_time: "2097-01-01T00:00:00Z".to_string(),
                arrival_time: "2097-01-01T01:02:03Z".to_string(),
                orbit_entry_time: "2097-01-01T01:12:03Z".to_string(),
                duration_seconds: 3_723.0,
                acceleration_km_s2: 0.02,
                acceleration_g: None,
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
        });

        assert_eq!(
            app.active_flight_status_lines(received_at),
            vec![
                "Flight: Mars (mars)".to_string(),
                "Phase: flight_plan".to_string(),
                "ETA: 2097-01-01T01:02:03Z".to_string(),
                "Countdown: 01:02:03".to_string(),
                "Distance: 69304 km".to_string(),
                "Navigation: acceleration=0.020 km/s^2 accelerating path_speed=0.000 km/s"
                    .to_string(),
            ]
        );

        app.apply_server_message(ServerToClient::FlightPlan { seq: 2, plan: None });
        assert!(app.active_flight_status_lines(received_at).is_empty());
    }

    #[test]
    fn active_flight_status_lines_are_phase_specific() {
        let mut app = ClientApp::default();
        let received_at = Instant::now();
        app.clock_sample = Some(ClientClockSample {
            current_time: "2097-01-01T01:05:00Z".to_string(),
            received_at,
            running: false,
            rate: 1.0,
        });

        let base_plan = FlightPlanDto {
            plan_id: "flight-1".to_string(),
            ship_id: "player-ship".to_string(),
            target: FlightPlanTargetDto::Object {
                object_id: "mars".to_string(),
                display_name: "Mars".to_string(),
            },
            departure_time: "2097-01-01T00:00:00Z".to_string(),
            arrival_time: "2097-01-01T01:00:00Z".to_string(),
            orbit_entry_time: "2097-01-01T01:10:00Z".to_string(),
            duration_seconds: 3_600.0,
            acceleration_km_s2: 0.02,
            acceleration_g: None,
            status: FlightPlanStatusDto::Active,
            navigation_phase: "entering_orbit".to_string(),
            arrival_orbit: Some(ArrivalOrbitDto {
                kind: "low".to_string(),
                radius_km: 3_789.5,
                altitude_km: Some(400.0),
                period_seconds: Some(7_113.0),
                circular_speed_km_s: Some(3.362),
            }),
            quality: Some("fictional".to_string()),
        };

        app.apply_server_message(ServerToClient::FlightPlan {
            seq: 1,
            plan: Some(base_plan.clone()),
        });
        assert!(app
            .active_flight_status_lines(received_at)
            .contains(&"Navigation: Entering orbit".to_string()));

        let mut orbiting_plan = base_plan;
        orbiting_plan.navigation_phase = "orbiting".to_string();
        app.apply_server_message(ServerToClient::FlightPlan {
            seq: 2,
            plan: Some(orbiting_plan),
        });
        assert!(app.active_flight_status_lines(received_at).contains(
            &"Navigation: low radius=3790 km altitude=400 km period=1h 58m 33s speed=3.362 km/s"
                .to_string()
        ));
    }

    #[test]
    fn active_flight_status_lines_include_remaining_distance() {
        let mut app = ClientApp::default();
        let received_at = Instant::now();
        app.clock_sample = Some(ClientClockSample {
            current_time: "2097-01-01T00:00:05Z".to_string(),
            received_at,
            running: false,
            rate: 1.0,
        });
        app.apply_server_message(ServerToClient::FlightPlan {
            seq: 1,
            plan: Some(FlightPlanDto {
                plan_id: "flight-1".to_string(),
                ship_id: "player-ship".to_string(),
                target: FlightPlanTargetDto::Object {
                    object_id: "mars".to_string(),
                    display_name: "Mars".to_string(),
                },
                departure_time: "2097-01-01T00:00:00Z".to_string(),
                arrival_time: "2097-01-01T00:00:20Z".to_string(),
                orbit_entry_time: "2097-01-01T00:10:20Z".to_string(),
                duration_seconds: 20.0,
                acceleration_km_s2: 2.0,
                acceleration_g: None,
                status: FlightPlanStatusDto::Active,
                navigation_phase: "flight_plan".to_string(),
                arrival_orbit: None,
                quality: Some("fictional".to_string()),
            }),
        });

        assert!(app
            .active_flight_status_lines(received_at)
            .contains(&"Distance: 175 km".to_string()));
    }

    #[test]
    fn silent_flight_status_updates_without_output() {
        let mut app = ClientApp::default();
        let line_count = app.output_lines.len();

        app.apply_server_message(ServerToClient::FlightPlan {
            seq: SILENT_FLIGHT_STATUS_SEQ,
            plan: Some(FlightPlanDto {
                plan_id: "flight-1".to_string(),
                ship_id: "player-ship".to_string(),
                target: FlightPlanTargetDto::Object {
                    object_id: "mars".to_string(),
                    display_name: "Mars".to_string(),
                },
                departure_time: "2097-01-01T00:00:00Z".to_string(),
                arrival_time: "2097-01-01T00:00:20Z".to_string(),
                orbit_entry_time: "2097-01-01T00:10:20Z".to_string(),
                duration_seconds: 20.0,
                acceleration_km_s2: 2.0,
                acceleration_g: None,
                status: FlightPlanStatusDto::Active,
                navigation_phase: "flight_plan".to_string(),
                arrival_orbit: None,
                quality: Some("fictional".to_string()),
            }),
        });

        assert_eq!(app.output_lines.len(), line_count);
        assert!(!app.active_flight_status_lines(Instant::now()).is_empty());
    }
}
