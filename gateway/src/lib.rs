pub mod admin;
pub mod bootstrap;
pub mod domain;
pub mod handler;
pub mod net;
pub mod proto;

use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use domain::{channels, config, session, storage};
use net::state::{State, VoiceMemberTx};

pub async fn run(cfg: Arc<config::Config>, voice_member_tx: Option<VoiceMemberTx>) -> Result<()> {
    let private_mode = cfg.is_private();
    if private_mode {
        info!("private mode enabled, federation disabled");
    } else {
        info!("federation enabled");
    }
    info!(
        "VNOX Gateway — node: {} addr: {} bind: {}",
        cfg.node.name, cfg.node.address, cfg.gateway.bind
    );
    if let Some(limit) = cfg.gateway.max_connections {
        info!("max_connections configured: {limit} (not enforced yet)");
    }
    match cfg.storage.backend.as_deref().unwrap_or("sqlite") {
        "sqlite" => {
            if cfg.storage.postgres_url.is_some() {
                warn!("storage.postgres_url is set but backend=sqlite; postgres_url is ignored");
            }
        }
        "postgres" => {
            warn!("backend=postgres is not implemented yet; sqlite storage will be used");
            if cfg.storage.postgres_url.is_none() {
                warn!("backend=postgres configured but storage.postgres_url is missing");
            }
        }
        other => warn!("unknown storage backend '{other}'; sqlite storage will be used"),
    }

    let sqlite = cfg
        .storage
        .sqlite_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "./dev/data/vnox.db".into());

    let server_identity = Arc::new(
        bootstrap::server_identity::ServerIdentity::load_or_generate(&cfg.storage.data_dir)?,
    );
    let server_pubkey = server_identity.pubkey_hex();
    info!(
        "server identity: {}…",
        &server_pubkey[..8.min(server_pubkey.len())]
    );

    let (tx, _) = broadcast::channel(256);
    let metrics = admin::metrics::Metrics::new();
    let sessions_count = Arc::new(AtomicUsize::new(0));
    let channels_count = Arc::new(AtomicUsize::new(0));
    let state = State::new(
        session::new_store(),
        channels::new_store(),
        Arc::new(storage::Storage::connect(&sqlite).await?),
        cfg.clone(),
        server_identity,
        tx,
        metrics.clone(),
        sessions_count.clone(),
        channels_count.clone(),
        voice_member_tx,
    );

    let admin_bind = cfg
        .gateway
        .admin_bind
        .clone()
        .unwrap_or_else(|| "0.0.0.0:7601".to_string());
    let admin_state = admin::AdminState {
        started_at: std::time::Instant::now(),
        node_name: cfg.node.name.clone(),
        node_address: cfg.node.address.clone(),
        private_mode,
        metrics: metrics.clone(),
        sessions_count,
        channels_count,
    };
    tokio::spawn(admin::run(admin_bind, admin_state));

    let listener = TcpListener::bind(&cfg.gateway.bind).await?;
    info!("listening on {}", cfg.gateway.bind);

    if cfg.gateway.tls_enabled {
        run_tls(listener, cfg, state).await
    } else {
        run_plain(listener, state).await
    }
}

async fn run_tls(listener: TcpListener, cfg: Arc<config::Config>, state: State) -> Result<()> {
    let (certs, key) = load_or_generate_tls_certs(&cfg.gateway)?;
    if cfg.gateway.tls_cert_path.is_none() {
        warn!("using self-signed TLS certificate — clients must accept it manually");
    }
    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("connection from {addr} (TLS)");
                state.metrics.inc(&state.metrics.connections_total);
                let s = state.clone();
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    match acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            if let Err(e) = handle(tls_stream, addr, s).await {
                                debug!("{addr} closed: {e}");
                            }
                        }
                        Err(e) => error!("TLS accept error from {addr}: {e}"),
                    }
                });
            }
            Err(e) => error!("accept: {e}"),
        }
    }
}

async fn run_plain(listener: TcpListener, state: State) -> Result<()> {
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("connection from {addr}");
                state.metrics.inc(&state.metrics.connections_total);
                let s = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle(stream, addr, s).await {
                        debug!("{addr} closed: {e}");
                    }
                });
            }
            Err(e) => error!("accept: {e}"),
        }
    }
}

async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
    mut stream: S,
    addr: std::net::SocketAddr,
    state: State,
) -> Result<()> {
    let mut seq = 0u32;
    let (sess, crypto) = net::handshake::run(&mut stream, addr, &state, &mut seq).await?;
    let sid = sess.session_id.clone();
    state
        .sessions_count
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut bcast_rx = state.broadcast.subscribe();

    let result = handler::run_session(
        &mut stream,
        addr,
        &mut seq,
        &sid,
        &crypto,
        &state,
        &mut bcast_rx,
    )
    .await;
    if result.is_err() {
        let p = proto::DisconnectPayload {
            reason: "session closed".into(),
        };
        let _ = net::io::send_encrypted(
            &mut stream,
            proto::PacketId::Disconnect,
            &mut seq,
            &proto::to_payload(&p),
            &crypto,
        )
        .await;
    }

    if let Some(ch) = session::get(&state.sessions, &sid)
        .await
        .and_then(|s| s.channel_id)
    {
        channels::leave(&state.channels, &ch, &sid).await;
        handler::channel::broadcast_leave(&state, &ch, &sid).await;
    }
    session::remove(&state.sessions, &sid).await;
    state
        .sessions_count
        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    state.rate_limiter.remove(&sid);
    debug!("{addr} cleaned up");
    result
}

fn load_or_generate_tls_certs(
    cfg: &config::GatewayConfig,
) -> Result<(
    Vec<rustls::pki_types::CertificateDer<'static>>,
    rustls::pki_types::PrivateKeyDer<'static>,
)> {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

    match (&cfg.tls_cert_path, &cfg.tls_key_path) {
        (Some(cert_path), Some(key_path)) => {
            let cert_pem = std::fs::read_to_string(cert_path)?;
            let key_pem = std::fs::read_to_string(key_path)?;
            let cert = CertificateDer::from_pem_reader(&mut cert_pem.as_bytes())?;
            let key = PrivateKeyDer::from_pem_reader(&mut key_pem.as_bytes())?;
            Ok((vec![cert], key))
        }
        _ => {
            info!("No TLS cert/key configured, generating self-signed certificate");
            let certified_key = rcgen::generate_simple_self_signed(vec!["VNOX Server".into()])?;
            let cert_der = certified_key.cert.der().clone();
            let key_der = PrivateKeyDer::from(PrivatePkcs8KeyDer::from(
                certified_key.key_pair.serialize_der(),
            ));
            Ok((vec![cert_der], key_der))
        }
    }
}
