use crate::domain::channels::{Channel, ChannelKind, ChannelStore};

pub async fn join(store: &ChannelStore, channel_id: &str, session_id: &str) -> bool {
    let mut l = store.write().await;
    if let Some(ch) = l.get_mut(channel_id) {
        ch.members.insert(session_id.into());
        true
    } else {
        false
    }
}

pub async fn leave(store: &ChannelStore, channel_id: &str, session_id: &str) {
    if let Some(ch) = store.write().await.get_mut(channel_id) {
        ch.members.remove(session_id);
    }
}

pub async fn members(store: &ChannelStore, channel_id: &str) -> Vec<String> {
    store
        .read()
        .await
        .get(channel_id)
        .map(|ch| ch.members.iter().cloned().collect())
        .unwrap_or_default()
}

pub async fn get_channel(store: &ChannelStore, channel_id: &str) -> Option<Channel> {
    store.read().await.get(channel_id).cloned()
}

/// Create a new channel in the store. Returns `false` if a channel with the
/// same id already exists.
pub async fn create(
    store: &ChannelStore,
    channel_id: &str,
    channel_name: &str,
    kind: ChannelKind,
    guild_id: Option<String>,
) -> bool {
    let mut l = store.write().await;
    if l.contains_key(channel_id) {
        return false;
    }
    l.insert(
        channel_id.to_string(),
        Channel {
            id: channel_id.to_string(),
            name: channel_name.to_string(),
            kind,
            guild_id,
            members: std::collections::HashSet::new(),
        },
    );
    true
}

/// Remove a channel from the store. Returns `true` if it existed.
pub async fn delete(store: &ChannelStore, channel_id: &str) -> bool {
    store.write().await.remove(channel_id).is_some()
}

/// Rename a channel in the store. Returns `true` if it existed.
pub async fn rename(store: &ChannelStore, channel_id: &str, new_name: &str) -> bool {
    let mut l = store.write().await;
    if let Some(ch) = l.get_mut(channel_id) {
        ch.name = new_name.to_string();
        true
    } else {
        false
    }
}

/// List all channels in the store.
pub async fn list(store: &ChannelStore) -> Vec<Channel> {
    store.read().await.values().cloned().collect()
}
