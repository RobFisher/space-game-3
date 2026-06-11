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
        format!("Game time: {}", app.status.game_time),
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

    frame.render_widget(
        Paragraph::new(app.input.as_str())
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().title("Command").borders(Borders::ALL)),
        vertical[1],
    );

    let cursor_x =
        vertical[1].x + 1 + app.cursor.min(vertical[1].width.saturating_sub(2) as usize) as u16;
    let cursor_y = vertical[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}
