//! Prometheus-compatible metrics counters.
//!
//! All counters are atomic and lock-free. The text format output follows the
//! Prometheus exposition format v0.0.4 so it can be scraped directly.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

use crate::admin::AdminState;

/// Container for all gateway metrics. Cheap to clone (Arc-backed).
#[derive(Debug, Default)]
pub struct Metrics {
    pub messages_sent: AtomicU64,
    pub dm_messages_sent: AtomicU64,
    pub voice_packets_relayed: AtomicU64,
    pub connections_total: AtomicU64,
    pub auth_failures: AtomicU64,
    pub rate_limited_events: AtomicU64,
    pub errors_total: AtomicU64,
    pub guilds_total: AtomicU64,
    pub friends_requests_total: AtomicU64,
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn inc(&self, field: &AtomicU64) {
        field.fetch_add(1, Ordering::Relaxed);
    }

    /// Render metrics in Prometheus text exposition format.
    pub fn render_prometheus(&self, s: &AdminState) -> String {
        let uptime = s.started_at.elapsed().as_secs();
        let sessions = s.sessions_count.load(Ordering::Relaxed);
        let channels = s.channels_count.load(Ordering::Relaxed);
        let mut out = String::with_capacity(2048);

        out.push_str("# HELP vnox_uptime_seconds Gateway uptime in seconds.\n");
        out.push_str("# TYPE vnox_uptime_seconds counter\n");
        out.push_str(&format!("vnox_uptime_seconds {uptime}\n\n"));

        out.push_str("# HELP vnox_sessions_active Currently active sessions.\n");
        out.push_str("# TYPE vnox_sessions_active gauge\n");
        out.push_str(&format!("vnox_sessions_active {sessions}\n\n"));

        out.push_str("# HELP vnox_channels_active Currently active channels.\n");
        out.push_str("# TYPE vnox_channels_active gauge\n");
        out.push_str(&format!("vnox_channels_active {channels}\n\n"));

        out.push_str("# HELP vnox_messages_sent_total Total chat messages sent.\n");
        out.push_str("# TYPE vnox_messages_sent_total counter\n");
        out.push_str(&format!(
            "vnox_messages_sent_total {}\n\n",
            self.messages_sent.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_dm_messages_sent_total Total DM messages sent.\n");
        out.push_str("# TYPE vnox_dm_messages_sent_total counter\n");
        out.push_str(&format!(
            "vnox_dm_messages_sent_total {}\n\n",
            self.dm_messages_sent.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_voice_packets_relayed_total Total UDP voice packets relayed.\n");
        out.push_str("# TYPE vnox_voice_packets_relayed_total counter\n");
        out.push_str(&format!(
            "vnox_voice_packets_relayed_total {}\n\n",
            self.voice_packets_relayed.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_connections_total Total TCP connections accepted.\n");
        out.push_str("# TYPE vnox_connections_total counter\n");
        out.push_str(&format!(
            "vnox_connections_total {}\n\n",
            self.connections_total.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_auth_failures_total Total authentication failures.\n");
        out.push_str("# TYPE vnox_auth_failures_total counter\n");
        out.push_str(&format!(
            "vnox_auth_failures_total {}\n\n",
            self.auth_failures.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_rate_limited_events_total Total rate-limited requests.\n");
        out.push_str("# TYPE vnox_rate_limited_events_total counter\n");
        out.push_str(&format!(
            "vnox_rate_limited_events_total {}\n\n",
            self.rate_limited_events.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_errors_total Total error packets sent.\n");
        out.push_str("# TYPE vnox_errors_total counter\n");
        out.push_str(&format!(
            "vnox_errors_total {}\n\n",
            self.errors_total.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_guilds_total Total guilds tracked.\n");
        out.push_str("# TYPE vnox_guilds_total gauge\n");
        out.push_str(&format!(
            "vnox_guilds_total {}\n\n",
            self.guilds_total.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP vnox_friends_requests_total Total friend requests sent.\n");
        out.push_str("# TYPE vnox_friends_requests_total counter\n");
        out.push_str(&format!(
            "vnox_friends_requests_total {}\n\n",
            self.friends_requests_total.load(Ordering::Relaxed)
        ));

        out
    }
}

/// Helper to bump a metric from anywhere that holds an Arc<Metrics>.
#[allow(dead_code)]
pub fn bump(m: &Arc<Metrics>, field: fn(&Metrics) -> &AtomicU64) {
    m.inc(field(m));
}

#[allow(dead_code)]
fn _silence_unused_helper(_start: Instant) {}
