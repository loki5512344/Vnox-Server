use anyhow::Result;
use vnox_voice_node::{load_config, runner};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("VNOX_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let config = load_config()?;
    runner::run(config).await
}
