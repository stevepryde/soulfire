//! Per-key single-flight coalescing (`CHAT-13`, `AI-14`).
//!
//! Reproduces Soulfire-OG's `CharacterStateUpdater` scheduling: only one update
//! runs at a time per character, and updates requested while one is running
//! collapse to at most one pending run that re-reads fresh data. This is the pure
//! state machine; the engine spawns the actual work around it.

use std::collections::HashMap;
use std::sync::Mutex;

/// The scheduling decision for a requested run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// No run is in flight for this key — the caller should start one.
    Start,
    /// A run is already in flight — a pending run has been recorded (coalesced).
    Coalesced,
}

#[derive(Debug, Default, Clone, Copy)]
struct Slot {
    running: bool,
    pending: bool,
}

/// Single-flight, single-pending coalescer keyed by a string (e.g. a character
/// id). `Send + Sync`.
#[derive(Debug, Default)]
pub struct Coalescer {
    slots: Mutex<HashMap<String, Slot>>,
}

impl Coalescer {
    pub fn new() -> Self {
        Coalescer::default()
    }

    /// Request a run for `key`. Returns [`Decision::Start`] if the caller should
    /// begin work now, or [`Decision::Coalesced`] if a run is already in flight
    /// (at most one pending run is retained).
    pub fn request(&self, key: &str) -> Decision {
        let mut slots = self.slots.lock().unwrap();
        let slot = slots.entry(key.to_string()).or_default();
        if slot.running {
            slot.pending = true;
            Decision::Coalesced
        } else {
            slot.running = true;
            Decision::Start
        }
    }

    /// Signal that a run for `key` has finished. Returns `true` if a pending run
    /// was queued (the caller should run again with fresh data), or `false` if
    /// the key is now idle.
    pub fn finish(&self, key: &str) -> bool {
        let mut slots = self.slots.lock().unwrap();
        if let Some(slot) = slots.get_mut(key) {
            if slot.pending {
                slot.pending = false;
                // stays running for the next pass
                return true;
            }
            slot.running = false;
        }
        false
    }

    /// Whether a run is currently in flight for `key` (for assertions/tests).
    pub fn is_running(&self, key: &str) -> bool {
        self.slots
            .lock()
            .unwrap()
            .get(key)
            .map(|s| s.running)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_request_starts_concurrent_requests_coalesce() {
        let c = Coalescer::new();
        assert_eq!(c.request("chr_a"), Decision::Start);
        // While running, further requests coalesce into one pending.
        assert_eq!(c.request("chr_a"), Decision::Coalesced);
        assert_eq!(c.request("chr_a"), Decision::Coalesced);
        // One pending run remains: finishing returns true once, then idle.
        assert!(c.finish("chr_a"));
        assert!(c.is_running("chr_a"));
        assert!(!c.finish("chr_a"));
        assert!(!c.is_running("chr_a"));
    }

    #[test]
    fn different_keys_are_independent() {
        let c = Coalescer::new();
        assert_eq!(c.request("chr_a"), Decision::Start);
        assert_eq!(c.request("chr_b"), Decision::Start);
    }

    #[test]
    fn finish_without_pending_goes_idle() {
        let c = Coalescer::new();
        c.request("k");
        assert!(!c.finish("k"));
        // A fresh request starts again.
        assert_eq!(c.request("k"), Decision::Start);
    }
}
