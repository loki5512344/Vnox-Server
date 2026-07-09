pub mod channel;
pub mod content;
pub mod deliver;
pub mod direct_message;
pub mod dispatch;
pub mod friends;
pub mod guild;
pub mod run;

pub use dispatch::dispatch;
pub use run::run_session;

use tokio::net::TcpStream;

use crate::{net::state::State, proto::SessionCrypto};

pub struct Ctx<'a> {
    pub stream: &'a mut TcpStream,
    pub seq: &'a mut u32,
    pub crypto: &'a SessionCrypto,
    pub state: &'a State,
}
