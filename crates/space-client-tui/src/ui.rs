use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::ClientApp;

pub fn draw(frame: &mut Frame<'_>, app: &ClientApp) {
    let root = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
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

    let input_area_width = vertical[1].width.saturating_sub(2) as usize;
    let scroll = app.input_visual_scroll(input_area_width);
    let (title, input_text) = if let Some(search) = app.reverse_search_view() {
        (
            format!("Reverse search: {}", search.query),
            search.current_match.unwrap_or_default(),
        )
    } else {
        ("Command".to_string(), app.input_value().to_string())
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
        vertical[1],
    );

    let cursor_x = vertical[1].x
        + 1
        + app
            .input_visual_cursor()
            .saturating_sub(scroll)
            .min(input_area_width) as u16;
    let cursor_y = vertical[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}
