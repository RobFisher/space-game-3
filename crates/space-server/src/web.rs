use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use space_game_protocol::{ServerToClient, StatusDto};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{
    clock::SimulationClock,
    command::{handle_client_message, SharedSimulationClock},
    config::{ServerConfig, DEFAULT_GAME_TIME},
    query::SolarSystemQueryService,
};

#[derive(Clone)]
pub struct AppState {
    service: Arc<SolarSystemQueryService>,
    clock: SharedSimulationClock,
}

pub fn app(config: ServerConfig) -> Result<Router, space_game_ephemeris::EphemerisError> {
    let ws_path = config.ws_path.clone();
    let state = AppState {
        service: Arc::new(config.query_service()?),
        clock: Arc::new(RwLock::new(SimulationClock::new(
            space_game_ephemeris::GameTime::from_utc_iso8601(DEFAULT_GAME_TIME)?,
            Instant::now(),
        ))),
    };

    Ok(Router::new()
        .route(&ws_path, get(ws_handler))
        .with_state(state))
}

pub async fn run(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(config.bind_addr).await?;
    let router = app(config)?;
    tracing::info!("space-server listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let welcome = ServerToClient::Welcome {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        session_id: Uuid::new_v4().to_string(),
    };

    if send_protocol(&mut sender, &welcome).await.is_err() {
        return;
    }

    let at = state
        .clock
        .read()
        .expect("simulation clock lock poisoned")
        .snapshot(Instant::now())
        .current_time;
    let (_, status) = state.service.status(None, &at);
    if send_protocol(&mut sender, &ServerToClient::Status { seq: None, status })
        .await
        .is_err()
    {
        return;
    }

    while let Some(message) = receiver.next().await {
        let message = match message {
            Ok(message) => message,
            Err(err) => {
                tracing::debug!("websocket receive error: {err}");
                break;
            }
        };

        let Message::Text(text) = message else {
            if matches!(message, Message::Close(_)) {
                break;
            }
            continue;
        };

        let incoming = match serde_json::from_str(&text) {
            Ok(incoming) => incoming,
            Err(err) => {
                let response = ServerToClient::Error {
                    seq: None,
                    error: space_game_protocol::ErrorDto {
                        code: "invalid_json".to_string(),
                        message: format!("invalid protocol JSON: {err}"),
                    },
                };
                if send_protocol(&mut sender, &response).await.is_err() {
                    break;
                }
                continue;
            }
        };

        for response in handle_client_message(&state.service, &state.clock, incoming) {
            if send_protocol(&mut sender, &response).await.is_err() {
                return;
            }
        }
    }
}

async fn send_protocol(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    message: &ServerToClient,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(message).expect("protocol messages serialize");
    sender.send(Message::Text(text)).await
}

#[allow(dead_code)]
fn _status_is_sendable(_: StatusDto) {}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt};
    use space_game_protocol::{ClientToServer, ServerToClient};
    use tokio::net::TcpListener;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    use super::*;

    async fn spawn_test_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let router = app(ServerConfig {
            bind_addr: addr,
            ws_path: "/ws".to_string(),
            server_label: addr.to_string(),
        })
        .unwrap();

        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        format!("ws://{addr}/ws")
    }

    async fn recv_protocol(
        socket: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) -> ServerToClient {
        let msg = socket.next().await.unwrap().unwrap();
        let Message::Text(text) = msg else {
            panic!("expected text websocket message");
        };
        serde_json::from_str(&text).unwrap()
    }

    async fn send_protocol(
        socket: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        message: ClientToServer,
    ) {
        socket
            .send(Message::Text(serde_json::to_string(&message).unwrap()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn websocket_serves_welcome_status_and_queries() {
        let url = spawn_test_server().await;
        let (mut socket, _) = connect_async(url).await.unwrap();

        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::Welcome { .. }
        ));
        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::Status { seq: None, .. }
        ));

        send_protocol(&mut socket, ClientToServer::RequestObjects { seq: 1 }).await;
        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::Objects { seq: 1, objects } if !objects.is_empty()
        ));

        send_protocol(
            &mut socket,
            ClientToServer::RequestDistance {
                seq: 2,
                object_query: "mars".to_string(),
                at_game_time: None,
            },
        )
        .await;
        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::Distance { seq: 2, result } if result.object_id == "mars"
        ));

        send_protocol(
            &mut socket,
            ClientToServer::RequestDistances {
                seq: 3,
                limit: Some(2),
                sort: space_game_protocol::DistanceSort::Distance,
                at_game_time: None,
            },
        )
        .await;
        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::Distances { seq: 3, results } if results.len() == 2
        ));

        send_protocol(&mut socket, ClientToServer::RequestStatus { seq: 4 }).await;
        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::Status { seq: Some(4), .. }
        ));

        send_protocol(&mut socket, ClientToServer::RequestSimulationTime { seq: 5 }).await;
        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::SimulationTime {
                seq: Some(5),
                state
            } if state.running && state.rate == 1.0
        ));

        send_protocol(
            &mut socket,
            ClientToServer::AdvanceSimulationTime {
                seq: 6,
                amount: 1,
                unit: space_game_protocol::TimeUnit::Days,
            },
        )
        .await;
        assert!(matches!(
            recv_protocol(&mut socket).await,
            ServerToClient::SimulationTime {
                seq: Some(6),
                state
            } if state.current_time.starts_with("2097-01-02T")
        ));
    }
}
