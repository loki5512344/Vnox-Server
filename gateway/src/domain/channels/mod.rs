pub mod ops;

pub use ops::{create, delete, get_channel, join, leave, list, members, rename};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelKind {
    Text,
    Voice,
}

impl ChannelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Voice => "voice",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub kind: ChannelKind,
    pub members: HashSet<String>,
}

pub type ChannelStore = Arc<RwLock<HashMap<String, Channel>>>;

pub fn new_store() -> ChannelStore {
    let mut m = HashMap::new();
    m.insert(
        "general".into(),
        Channel {
            id: "general".into(),
            name: "general".into(),
            kind: ChannelKind::Text,
            members: HashSet::new(),
        },
    );
    m.insert(
        "voice".into(),
        Channel {
            id: "voice".into(),
            name: "voice".into(),
            kind: ChannelKind::Voice,
            members: HashSet::new(),
        },
    );
    Arc::new(RwLock::new(m))
}
