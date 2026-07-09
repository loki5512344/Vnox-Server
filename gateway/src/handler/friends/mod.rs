mod decline;
mod list;
mod manage;
mod requests;

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    net::io,
    proto::{ErrorPayload, PacketId, SessionCrypto, to_payload},
};

async fn send_err(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    seq: &mut u32,
    code: crate::proto::ErrorCode,
    msg: &str,
    crypto: &SessionCrypto,
) -> Result<()> {
    io::send_encrypted(
        stream,
        PacketId::Error,
        seq,
        &to_payload(&ErrorPayload {
            code: code as u32,
            message: msg.into(),
        }),
        crypto,
    )
    .await
}

pub use decline::handle_friend_decline;
pub use list::handle_friend_list;
pub use manage::{handle_block_list, handle_block_user, handle_friend_remove, handle_unblock_user};
pub use requests::{handle_friend_accept, handle_friend_request};
