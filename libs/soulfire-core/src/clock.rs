//! The clock seam (`TEST-3`, `TEST-5`).
//!
//! Time-dependent behavior (timeouts, summary cadences, stale-lock expiry,
//! timestamps) depends on a [`Clock`] rather than the system clock directly, so
//! tests can inject a deterministic clock that advances on command.

use std::sync::Mutex;

use sp_core::datetime::SpDateTime;

/// A source of the current time. Production wires [`SystemClock`]; tests wire
/// [`MockClock`].
pub trait Clock: Send + Sync {
    fn now(&self) -> SpDateTime;
}

/// The real system clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SpDateTime {
        SpDateTime::now()
    }
}

/// A deterministic clock for tests: starts at a fixed instant and advances only
/// when told to (`TEST-3`).
#[derive(Debug)]
pub struct MockClock {
    current: Mutex<SpDateTime>,
}

impl MockClock {
    /// Create a mock clock at `start`.
    pub fn new(start: SpDateTime) -> Self {
        MockClock {
            current: Mutex::new(start),
        }
    }

    /// Create a mock clock at a fixed, reproducible epoch instant.
    pub fn at_epoch() -> Self {
        MockClock::new(SpDateTime::from_timestamp(1_700_000_000).expect("valid fixed test instant"))
    }

    /// Advance the clock by `seconds`.
    pub fn advance_secs(&self, seconds: i64) {
        let mut cur = self.current.lock().unwrap();
        *cur = cur.add_seconds(seconds);
    }

    /// Advance the clock by `millis`.
    pub fn advance_millis(&self, millis: i64) {
        let mut cur = self.current.lock().unwrap();
        *cur = cur.add_millis(millis);
    }

    /// Set the clock to an absolute instant.
    pub fn set(&self, t: SpDateTime) {
        *self.current.lock().unwrap() = t;
    }
}

impl Clock for MockClock {
    fn now(&self) -> SpDateTime {
        *self.current.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_clock_is_stable_until_advanced() {
        let clock = MockClock::at_epoch();
        let t0 = clock.now();
        assert_eq!(clock.now(), t0); // does not move on its own
        clock.advance_secs(90);
        assert_eq!(clock.now().seconds_since(t0), 90);
    }
}
