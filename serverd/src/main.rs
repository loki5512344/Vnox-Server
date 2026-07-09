use anyhow::Result;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("VNOX_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let config_path = std::env::args()
        .skip_while(|a| a != "--config")
        .nth(1)
        .unwrap_or_else(|| "/etc/vnox/config.toml".into());

    let text = std::fs::read_to_string(&config_path)?;
    let cfg: vnox_gateway::domain::config::Config =
        toml::from_str(&text).map_err(|e| anyhow::anyhow!("invalid config: {e}"))?;

    info!(
        "VNOX Server starting — node: {}",
        cfg.node.name
    );
    info!("Gateway TCP: {}", cfg.gateway.bind);
    info!("Voice UDP: {}", cfg.voice.bind);

    let node_name = cfg.node.name.clone();
    let voice_bind = cfg.voice.bind.clone();
    let gate_cfg = Arc::new(cfg);

    let voice_handle = tokio::spawn(async move {
        vnox_voice_node::runner::run_bind(&node_name, &voice_bind).await
    });

    let gate_handle = tokio::spawn(async move {
        vnox_gateway::run(gate_cfg).await
    });

    tokio::select! {
        r = voice_handle => {
            info!("voice node exited: {:?}", r);
        }
        r = gate_handle => {
            info!("gateway exited: {:?}", r);
        }
    }

    Ok(())
}
