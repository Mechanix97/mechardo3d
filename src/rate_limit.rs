use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};

/// Hard cap on tracked clients, so a flood of unique IPs cannot grow the map
/// without bound.
const MAX_ENTRIES: usize = 10_000;

/// Fixed-window limiter: one action per key per window.
pub struct RateLimiter {
    window_secs: i64,
    entries: Mutex<HashMap<String, DateTime<Utc>>>,
}

impl RateLimiter {
    pub fn new(window: Duration) -> Self {
        Self {
            window_secs: window.as_secs().min(i64::MAX as u64) as i64,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Seconds the caller has to wait, or `None` when the action is allowed.
    pub fn retry_after(&self, key: &str, now: DateTime<Utc>) -> Option<i64> {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let last = entries.get(key)?;
        let elapsed = now.signed_duration_since(*last).num_seconds();
        if elapsed < self.window_secs {
            Some((self.window_secs - elapsed).max(1))
        } else {
            None
        }
    }

    /// Record an action and drop entries whose window has already elapsed.
    pub fn record(&self, key: &str, now: DateTime<Utc>) {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|_, last| now.signed_duration_since(*last).num_seconds() < self.window_secs);

        if entries.len() >= MAX_ENTRIES && !entries.contains_key(key) {
            // Still over capacity after pruning: drop the oldest entry.
            if let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, last)| **last)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest);
            }
        }

        entries.insert(key.to_string(), now);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs, 0).expect("valid timestamp")
    }

    #[test]
    fn allows_the_first_action() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        assert_eq!(limiter.retry_after("1.1.1.1", at(0)), None);
    }

    #[test]
    fn blocks_inside_the_window() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        limiter.record("1.1.1.1", at(0));
        assert_eq!(limiter.retry_after("1.1.1.1", at(60)), Some(240));
    }

    #[test]
    fn allows_again_after_the_window() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        limiter.record("1.1.1.1", at(0));
        assert_eq!(limiter.retry_after("1.1.1.1", at(300)), None);
    }

    #[test]
    fn keys_are_independent() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        limiter.record("1.1.1.1", at(0));
        assert_eq!(limiter.retry_after("2.2.2.2", at(1)), None);
    }

    #[test]
    fn prunes_expired_entries() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        limiter.record("1.1.1.1", at(0));
        limiter.record("2.2.2.2", at(400));
        assert_eq!(limiter.len(), 1);
    }
}
