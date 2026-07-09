//! Per-session token-bucket rate limiter.
//!
//! Each authenticated session gets its own bucket. The bucket refills at
//! `rate_per_sec` tokens per second, up to a maximum of `burst` tokens.
//! A message consumes 1 token; if the bucket is empty, the message is
//! rejected with `ErrorCode::RateLimited`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::domain::session::SessionStore;

pub struct RateLimiter {
    inner: Mutex<HashMap<String, Bucket>>,
    rate_per_sec: f32,
    burst: u32,
}

struct Bucket {
    tokens: f32,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(rate_per_sec: f32, burst: u32) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            rate_per_sec,
            burst,
        }
    }

    /// Try to consume one token. Returns `true` if allowed, `false` if rate-limited.
    /// Lazily creates a bucket for new sessions.
    pub fn try_consume(&self, session_id: &str) -> bool {
        if self.rate_per_sec <= 0.0 {
            return true;
        }
        let mut guard = self.inner.lock().unwrap();
        let now = Instant::now();
        let bucket = guard
            .entry(session_id.to_string())
            .or_insert_with(|| Bucket {
                tokens: self.burst as f32,
                last_refill: now,
            });
        // Refill
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f32();
        bucket.tokens = (bucket.tokens + elapsed * self.rate_per_sec).min(self.burst as f32);
        bucket.last_refill = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Remove a session's bucket on disconnect.
    pub fn remove(&self, session_id: &str) {
        let mut guard = self.inner.lock().unwrap();
        guard.remove(session_id);
    }

    /// Periodically prune buckets for sessions that no longer exist.
    /// Returns the number of buckets pruned.
    #[allow(dead_code)]
    pub async fn prune_dead(&self, sessions: &SessionStore) -> usize {
        let live: Vec<String> = sessions.read().await.keys().cloned().collect();
        let live_set: std::collections::HashSet<String> = live.into_iter().collect();
        let mut guard = self.inner.lock().unwrap();
        let before = guard.len();
        guard.retain(|sid, _| live_set.contains(sid));
        before - guard.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_within_burst() {
        let r = RateLimiter::new(10.0, 3);
        assert!(r.try_consume("s1"));
        assert!(r.try_consume("s1"));
        assert!(r.try_consume("s1"));
        assert!(!r.try_consume("s1")); // bucket empty
    }

    #[test]
    fn independent_buckets_per_session() {
        let r = RateLimiter::new(10.0, 1);
        assert!(r.try_consume("s1"));
        assert!(!r.try_consume("s1"));
        assert!(r.try_consume("s2"));
    }

    #[test]
    fn disabled_when_rate_zero() {
        let r = RateLimiter::new(0.0, 1);
        for _ in 0..100 {
            assert!(r.try_consume("s1"));
        }
    }

    #[test]
    fn remove_clears_bucket() {
        let r = RateLimiter::new(0.1, 1);
        assert!(r.try_consume("s1"));
        assert!(!r.try_consume("s1"));
        r.remove("s1");
        // After removal, a fresh bucket is created with full burst.
        assert!(r.try_consume("s1"));
    }
}
