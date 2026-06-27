use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use space_client_tui::{app::ClientApp, plain::run_plain};
use space_server::{config::ServerConfig, web::app};
use tokio::{
    io::{AsyncWrite, BufReader},
    net::TcpListener,
};

async fn spawn_server() -> String {
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

async fn run_command(server_url: &str, command: &str) -> String {
    let mut output = VecWriter::default();
    run_plain(
        ClientApp::with_server_url(server_url),
        Some(command.to_string()),
        BufReader::new(tokio::io::empty()),
        &mut output,
    )
    .await
    .unwrap();
    String::from_utf8(output.into_inner()).unwrap()
}

#[tokio::test]
async fn plain_client_smoke_lists_real_objects_and_distances() {
    let server_url = spawn_server().await;

    let objects = run_command(&server_url, "objects").await;
    assert!(objects.contains("Mars (mars)"));
    assert!(objects.contains("Moon (moon)"));
    assert!(objects.contains("Pluto (pluto)"));
    assert!(!objects.contains("Ceres (ceres)"));
    assert!(!objects.contains("Luna (luna)"));

    let distance = run_command(&server_url, "distance mars").await;
    assert!(
        distance.contains("Mars:") && distance.contains("AU") && distance.contains("km"),
        "unexpected distance output: {distance}"
    );
}

#[derive(Default)]
struct VecWriter {
    bytes: Vec<u8>,
}

impl VecWriter {
    fn into_inner(self) -> Vec<u8> {
        self.bytes
    }
}

impl AsyncWrite for VecWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.bytes.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
