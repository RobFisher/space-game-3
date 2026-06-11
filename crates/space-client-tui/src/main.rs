use space_client_tui::{app::ClientApp, net::run_client, terminal::TerminalGuard};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut terminal = TerminalGuard::enter()?;
    let app = ClientApp::default();
    run_client(app, &mut terminal).await
}
