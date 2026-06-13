use std::time::{Duration, Instant};

use color_eyre::eyre::{eyre, Result};
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::{SinkExt, StreamExt};
use space_game_protocol::{ClientToServer, ServerToClient};
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    app::{ClientApp, SILENT_TIME_SYNC_SEQ},
    terminal::TerminalGuard,
    ui,
};

pub async fn run_client(mut app: ClientApp, terminal: &mut TerminalGuard) -> Result<()> {
    let (socket, _) = connect_async(app.server_url.as_str()).await?;
    let (mut writer, mut reader) = socket.split();
    let mut events = EventStream::new();
    let mut render_tick = time::interval(Duration::from_millis(100));
    let mut time_sync_tick = time::interval(Duration::from_secs(5));

    writer
        .send(Message::Text(serde_json::to_string(
            &ClientToServer::Hello {
                client_name: "space-client-tui".to_string(),
                client_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        )?))
        .await?;

    loop {
        tokio::select! {
            _ = render_tick.tick() => {
                terminal.terminal_mut().draw(|frame| ui::draw(frame, &app))?;
            }
            _ = time_sync_tick.tick() => {
                writer
                    .send(Message::Text(serde_json::to_string(
                        &ClientToServer::RequestSimulationTime {
                            seq: SILENT_TIME_SYNC_SEQ,
                        },
                    )?))
                    .await?;
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Some(message) = handle_terminal_event(&mut app, event) {
                            writer.send(Message::Text(serde_json::to_string(&message)?)).await?;
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Err(eyre!("terminal event stream ended")),
                }
            }
            maybe_message = reader.next() => {
                match maybe_message {
                    Some(Ok(Message::Text(text))) => {
                        let message: ServerToClient = serde_json::from_str(&text)?;
                        app.apply_server_message(message);
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        app.connected = false;
                        app.push_output("Server connection closed".to_string());
                        app.should_quit = true;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(err)) => return Err(err.into()),
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

pub fn handle_terminal_event(app: &mut ClientApp, event: Event) -> Option<ClientToServer> {
    let Event::Key(key) = event else {
        return None;
    };
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            None
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.start_or_repeat_reverse_search();
            None
        }
        KeyCode::Char(ch) => {
            app.insert_char(ch);
            None
        }
        KeyCode::Backspace => {
            app.backspace();
            None
        }
        KeyCode::Left => {
            app.move_left();
            None
        }
        KeyCode::Right => {
            app.move_right();
            None
        }
        KeyCode::Up => {
            app.history_previous();
            None
        }
        KeyCode::Down => {
            app.history_next();
            None
        }
        KeyCode::Tab => {
            if app.complete_local_command() {
                None
            } else {
                Some(app.request_completion(Instant::now()))
            }
        }
        KeyCode::Enter => app.submit_input(),
        KeyCode::Esc => {
            app.cancel_input_mode();
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEvent, KeyEventState};

    use super::*;

    #[test]
    fn esc_does_not_quit_tui() {
        let mut app = ClientApp::default();

        let message = handle_terminal_event(
            &mut app,
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }),
        );

        assert_eq!(message, None);
        assert!(!app.should_quit);
    }

    #[test]
    fn ctrl_c_still_quits_tui() {
        let mut app = ClientApp::default();

        let message = handle_terminal_event(
            &mut app,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }),
        );

        assert_eq!(message, None);
        assert!(app.should_quit);
    }
}
