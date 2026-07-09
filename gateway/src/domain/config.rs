use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub gateway: GatewayConfig,
    pub voice: VoiceConfig,
    pub storage: StorageConfig,
    #[serde(default)]
    pub server: ServerConfig,
    pub federation: Option<FederationConfig>,
}

impl Config {
    /// Returns `true` when private mode is active — federation disabled,
    /// server is isolated, client shows a "PRIVATE" badge.
    pub fn is_private(&self) -> bool {
        match self.server.mode {
            Some(ref m) => m.is_private(),
            None => self
                .federation
                .as_ref()
                .map(|f| f.is_private())
                .unwrap_or(true),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ServerConfig {
    /// Explicit server mode: "private" or "federated".
    /// Overrides `[federation] enabled` when set.
    pub mode: Option<ServerMode>,
}

/// Server isolation mode.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerMode {
    /// Fully isolated — no federation, no node discovery.
    Private,
    /// Can discover, peer, and sync with other VNOX nodes.
    Federated,
}

impl ServerMode {
    pub fn is_private(&self) -> bool {
        matches!(self, ServerMode::Private)
    }
}

#[derive(Debug, Deserialize)]
pub struct FederationConfig {
    pub enabled: bool,
    /// If true, the server runs in private mode (federation disabled, client shows a badge).
    /// Defaults to the inverse of `enabled` when not set.
    pub private_mode: Option<bool>,
}

impl FederationConfig {
    /// Returns `true` when private mode is active — federation is disabled and the
    /// server does not exchange any data with other nodes.
    pub fn is_private(&self) -> bool {
        self.private_mode.unwrap_or(!self.enabled)
    }
}

#[derive(Debug, Deserialize)]
pub struct NodeConfig {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct GatewayConfig {
    pub bind: String,
    pub max_connections: Option<usize>,
    pub session_timeout: Option<u64>,
    /// Bind address for the admin HTTP server (/health, /metrics).
    /// Defaults to "0.0.0.0:7601" when not specified.
    #[serde(default)]
    pub admin_bind: Option<String>,
    /// Per-session chat/DM message rate limit (messages per second).
    /// Defaults to 5.0 when not specified. Set to 0 to disable.
    #[serde(default)]
    pub message_rate_per_sec: Option<f32>,
    /// Burst size for the token bucket (messages allowed in one tick).
    /// Defaults to 10.
    #[serde(default)]
    pub message_rate_burst: Option<u32>,
    /// Enable TLS 1.3 on the TCP listener.
    #[serde(default)]
    pub tls_enabled: bool,
    /// Path to TLS certificate file (PEM).
    #[serde(default)]
    pub tls_cert_path: Option<String>,
    /// Path to TLS private key file (PEM).
    #[serde(default)]
    pub tls_key_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VoiceConfig {
    pub bind: String,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub backend: Option<String>,
    pub sqlite_path: Option<PathBuf>,
    pub postgres_url: Option<String>,
}

pub fn load() -> Result<Config> {
    let path = std::env::args()
        .skip_while(|a| a != "--config")
        .nth(1)
        .unwrap_or_else(|| "/etc/vnox/config.toml".into());

    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("cannot read config {path}: {e}"))?;

    toml::from_str(&text).map_err(|e| anyhow::anyhow!("invalid config: {e}"))
}
