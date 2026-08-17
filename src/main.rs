use bg3_mcp::{Config, run};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    tracing_subscriber::registry()
        .with(EnvFilter::try_new(&config.log_filter).unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    run(config).await?;
    Ok(())
}
