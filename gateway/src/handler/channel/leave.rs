use anyhow::Result;
use tokio::net::TcpStream;
use tracing::info;

use crate::{domain::channels, net::state::State, proto::SessionCrypto};

use super::{broadcast_leave, set_channel};

pub async fn leave(
    _stream: &mut TcpStream,
    _seq: &mut u32,
    session_id: &str,
    channel_id: &str,
    _crypto: &SessionCrypto,
    state: &State,
) -> Result<()> {
    channels::leave(&state.channels, channel_id, session_id).await;
    set_channel(state, session_id, None).await;
    broadcast_leave(state, channel_id, session_id).await;
    info!("session {} left {channel_id}", &session_id[..8]);
    Ok(())
}
