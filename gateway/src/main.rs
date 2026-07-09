use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("VNOX_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let cfg = Arc::new(vnox_gateway::domain::config::load()?);
    vnox_gateway::run(cfg).await
}
