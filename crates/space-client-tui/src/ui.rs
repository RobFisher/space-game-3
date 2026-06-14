use std::time::Instant;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, Paragraph, Wrap},
    Frame,
};

use crate::app::ClientApp;

pub fn draw(frame: &mut Frame<'_>, app: &ClientApp) {
    let root = frame.area();
    let candidate_count = app.completion_candidates().len().min(4) as u16;
    let command_height = if candidate_count > 0 {
        5 + candidate_count
    } else {
        3
    };
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(command_height)])
        .split(root);
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(28)])
        .split(vertical[0]);

    let output = visible_output_lines(
        &app.output_lines,
        main[0].width.saturating_sub(2) as usize,
        main[0].height.saturating_sub(2) as usize,
    );
    frame.render_widget(
        Paragraph::new(output)
            .block(Block::default().title("Output").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        main[0],
    );

    let now = Instant::now();
    let mut status_lines = vec![
        format!("Connected: {}", if app.connected { "yes" } else { "no" }),
        format!("Server: {}", app.status.server),
        format!("Game time: {}", app.display_game_time_at(now)),
        format!("Ship: {}", app.status.ship_name),
        format!("Motion: {}", app.status.ship_motion),
        format!("Frame: {}", app.status.ship_frame),
        format!("Objects: {}", app.status.object_count),
        format!("Last update: {}", app.status.last_update),
    ];
    status_lines.extend(app.active_flight_status_lines(now));
    let status = status_lines.join("\n");
    frame.render_widget(
        Paragraph::new(status)
            .block(Block::default().title("Status").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        main[1],
    );

    let command_chunks = if candidate_count > 0 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(vertical[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3)])
            .split(vertical[1])
    };
    let input_area = command_chunks[0];
    let input_area_width = input_area.width.saturating_sub(2) as usize;
    let scroll = app.input_visual_scroll(input_area_width);
    let (title, input_text) = if let Some(search) = app.reverse_search_view() {
        (
            format!("Reverse search: {}", search.query),
            search.current_match.unwrap_or_default(),
        )
    } else {
        let title = if app.show_completion_pending(Instant::now()) {
            "Command (completing...)"
        } else {
            "Command"
        };
        (title.to_string(), app.input_value().to_string())
    };

    frame.render_widget(
        Paragraph::new(input_text)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .scroll((0, scroll as u16))
            .block(Block::default().title(title).borders(Borders::ALL)),
        input_area,
    );

    if command_chunks.len() > 1 {
        let candidates = app
            .completion_candidates()
            .iter()
            .take(4)
            .map(|candidate| candidate.display.as_str())
            .collect::<Vec<_>>();
        frame.render_widget(
            List::new(candidates)
                .block(Block::default().title("Completions").borders(Borders::ALL)),
            command_chunks[1],
        );
    }

    let cursor_x = input_area.x
        + 1
        + app
            .input_visual_cursor()
            .saturating_sub(scroll)
            .min(input_area_width) as u16;
    let cursor_y = input_area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn visible_output_lines(lines: &[String], width: usize, height: usize) -> Vec<Line<'static>> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let mut visual_lines = Vec::new();
    for line in lines {
        visual_lines.extend(wrap_output_line(line, width));
    }
    let start = visual_lines.len().saturating_sub(height);
    visual_lines[start..]
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect()
}

fn wrap_output_line(line: &str, width: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let chars = line.chars().collect::<Vec<_>>();
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use ratatui::{backend::TestBackend, Terminal};

    use super::*;

    #[test]
    fn renders_completion_candidates_and_pending_state() {
        let mut pending_app = ClientApp::default();
        pending_app.set_input("distance ma");
        let _ = pending_app.request_completion(Instant::now() - Duration::from_millis(250));
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &pending_app)).unwrap();
        let rendered = format!("{:?}", terminal.backend().buffer());

        assert!(rendered.contains("completing"));

        let mut app = ClientApp::default();
        app.set_input("distance ma");
        let request = app.request_completion(Instant::now() - Duration::from_millis(250));
        let space_game_protocol::ClientToServer::CompletionRequest(request) = request else {
            panic!("expected completion request");
        };
        app.apply_server_message(space_game_protocol::ServerToClient::CompletionResponse(
            space_game_protocol::CompletionResponseDto {
                seq: request.seq,
                replacement: space_game_protocol::ReplacementSpanDto { start: 9, end: 11 },
                candidates: vec![
                    space_game_protocol::CompletionCandidateDto {
                        insertion: "mars".to_string(),
                        display: "Mars".to_string(),
                        kind: space_game_protocol::CompletionCandidateKindDto::Object,
                    },
                    space_game_protocol::CompletionCandidateDto {
                        insertion: "martian-station".to_string(),
                        display: "Martian Station".to_string(),
                        kind: space_game_protocol::CompletionCandidateKindDto::Object,
                    },
                ],
            },
        ));

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
        let rendered = format!("{:?}", terminal.backend().buffer());

        assert!(rendered.contains("Completions"));
        assert!(rendered.contains("Mars"));
        assert!(rendered.contains("Martian Station"));
    }

    #[test]
    fn output_pane_shows_last_visual_rows_when_wrapped() {
        let mut app = ClientApp::default();
        app.output_lines.clear();
        app.push_output("first line".to_string());
        app.push_output("second line is long enough to wrap in a narrow pane".to_string());
        app.push_output("final visible line".to_string());

        let backend = TestBackend::new(42, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
        let rendered = format!("{:?}", terminal.backend().buffer());

        assert!(rendered.contains("final visible line"));
    }

    #[test]
    fn visible_output_lines_tails_wrapped_rows() {
        let lines = vec![
            "alpha".to_string(),
            "bravo-charlie".to_string(),
            "delta".to_string(),
        ];

        let visible = visible_output_lines(&lines, 5, 3)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        assert_eq!(visible, vec!["-char", "lie", "delta"]);
    }

    #[test]
    fn status_pane_renders_active_flight_eta() {
        let mut app = ClientApp::default();
        app.apply_server_message(space_game_protocol::ServerToClient::SimulationTime {
            seq: Some(1),
            state: space_game_protocol::SimulationTimeDto {
                current_time: "2097-01-01T00:00:00Z".to_string(),
                running: false,
                rate: 1.0,
            },
        });
        app.apply_server_message(space_game_protocol::ServerToClient::FlightPlan {
            seq: 2,
            plan: Some(space_game_protocol::FlightPlanDto {
                plan_id: "flight-1".to_string(),
                ship_id: "player-ship".to_string(),
                target: space_game_protocol::FlightPlanTargetDto::Object {
                    object_id: "mars".to_string(),
                    display_name: "Mars".to_string(),
                },
                departure_time: "2097-01-01T00:00:00Z".to_string(),
                arrival_time: "2097-01-01T00:05:00Z".to_string(),
                duration_seconds: 300.0,
                acceleration_km_s2: 0.02,
                status: space_game_protocol::FlightPlanStatusDto::Active,
                quality: Some("fictional".to_string()),
            }),
        });

        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
        let rendered = format!("{:?}", terminal.backend().buffer());

        assert!(rendered.contains("Flight: Mars"));
        assert!(rendered.contains("ETA: 2097-01-01T00:05:00Z"));
        assert!(rendered.contains("Countdown: 00:05:00"));
        assert!(rendered.contains("Distance: 450 km"));
    }
}
