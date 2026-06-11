use space_server::{config::ServerConfig, web::run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run(ServerConfig::default()).await?;
    Ok(())
}
