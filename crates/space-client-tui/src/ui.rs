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

    let output = app
        .output_lines
        .iter()
        .rev()
        .take(main[0].height.saturating_sub(2) as usize)
        .rev()
        .map(|line| Line::from(line.as_str()))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(output)
            .block(Block::default().title("Output").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        main[0],
    );

    let status = vec![
        format!("Connected: {}", if app.connected { "yes" } else { "no" }),
        format!("Server: {}", app.status.server),
        format!("Game time: {}", app.display_game_time()),
        format!("Observer: {}", app.status.observer_label),
        format!("Frame: {}", app.status.observer_frame),
        format!("Objects: {}", app.status.object_count),
        format!("Last update: {}", app.status.last_update),
    ]
    .join("\n");
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
}
