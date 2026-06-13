use space_client_tui::{
    app::ClientApp,
    args::{ClientArgs, ClientMode},
    history::CommandHistoryStore,
    net::run_client,
    plain::run_plain,
    terminal::TerminalGuard,
};
use tokio::io::BufReader;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = ClientArgs::parse_env()?;

    match args.mode {
        ClientMode::Tui => {
            let app = ClientApp::with_history_store(
                args.server_url,
                CommandHistoryStore::default_path(),
            )?;
            let mut terminal = TerminalGuard::enter()?;
            run_client(app, &mut terminal).await
        }
        ClientMode::Plain { command } => {
            let app = ClientApp::with_server_url(args.server_url);
            let stdin = BufReader::new(tokio::io::stdin());
            let stdout = tokio::io::stdout();
            run_plain(app, command, stdin, stdout).await
        }
    }
}
