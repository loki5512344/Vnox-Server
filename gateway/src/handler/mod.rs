pub mod channel;
pub mod content;
pub mod deliver;
pub mod direct_message;
pub mod dispatch;
pub mod e2ee;
pub mod friends;
pub mod guild;
pub mod run;

pub use dispatch::dispatch;
pub use run::run_session;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{net::state::State, proto::SessionCrypto};

pub struct Ctx<'a, S: AsyncRead + AsyncWrite + Unpin> {
    pub stream: &'a mut S,
    pub seq: &'a mut u32,
    pub crypto: &'a SessionCrypto,
    pub state: &'a State,
}
