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

    /// Check and reserve a slot for `key` in one step under a single lock, so
    /// concurrent requests for the same key cannot all pass the check before
    /// any of them records it. The slot is reserved regardless of what the
    /// caller does with the result, so failed or rejected attempts count
    /// against the limit too, not just successful ones.
    ///
    /// Returns `Ok(())` when the action is allowed (and now recorded), or the
    /// number of seconds the caller has to wait when the window hasn't
    /// elapsed yet.
    pub fn try_acquire(&self, key: &str, now: DateTime<Utc>) -> Result<(), i64> {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|_, last| now.signed_duration_since(*last).num_seconds() < self.window_secs);

        if let Some(last) = entries.get(key) {
            let elapsed = now.signed_duration_since(*last).num_seconds();
            if elapsed < self.window_secs {
                return Err((self.window_secs - elapsed).max(1));
            }
        }

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
        Ok(())
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
        assert_eq!(limiter.try_acquire("1.1.1.1", at(0)), Ok(()));
    }

    #[test]
    fn blocks_a_second_attempt_inside_the_window() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(0)), Ok(()));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(60)), Err(240));
    }

    #[test]
    fn allows_again_after_the_window() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(0)), Ok(()));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(300)), Ok(()));
    }

    #[test]
    fn keys_are_independent() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(0)), Ok(()));
        assert_eq!(limiter.try_acquire("2.2.2.2", at(1)), Ok(()));
    }

    #[test]
    fn prunes_expired_entries() {
        let limiter = RateLimiter::new(Duration::from_secs(300));
        limiter.try_acquire("1.1.1.1", at(0)).unwrap();
        limiter.try_acquire("2.2.2.2", at(400)).unwrap();
        assert_eq!(limiter.len(), 1);
    }

    #[test]
    fn every_attempt_reserves_the_slot_even_a_rejected_one() {
        // The slot is reserved before the caller knows whether the request is
        // valid, so a burst of junk attempts from one key still only gets one
        // slot per window - it can't be used to bypass the limit for free.
        let limiter = RateLimiter::new(Duration::from_secs(300));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(0)), Ok(()));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(1)), Err(299));
        assert_eq!(limiter.try_acquire("1.1.1.1", at(2)), Err(298));
    }
}
