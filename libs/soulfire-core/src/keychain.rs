//! The secure-credential (keychain) seam (`SEC-7`, `TEST-5`).
//!
//! "Remember unlock on this device" stores the store key (or a wrapping secret)
//! in the OS secure credential store. Feature code depends on the [`Keychain`]
//! trait; production wires the real OS keychain, tests wire an in-memory fake.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::CoreResult;

/// A platform secure-credential store. Items are opaque byte secrets keyed by a
/// stable name. An unavailable store (no usable backend, `SEC-7`) returns
/// `Ok(None)` from `get` and surfaces failures as `Err` from `set`/`delete`.
pub trait Keychain: Send + Sync {
    /// Retrieve the secret bytes stored under `key`, or `None` if absent.
    fn get(&self, key: &str) -> CoreResult<Option<Vec<u8>>>;
    /// Store `secret` under `key`, replacing any existing value.
    fn set(&self, key: &str, secret: &[u8]) -> CoreResult<()>;
    /// Remove any secret stored under `key` (idempotent).
    fn delete(&self, key: &str) -> CoreResult<()>;
}

/// An in-memory keychain for tests and for platforms with no usable secure store.
/// Holds secrets only in process memory; nothing is persisted.
#[derive(Debug, Default)]
pub struct InMemoryKeychain {
    items: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryKeychain {
    pub fn new() -> Self {
        InMemoryKeychain::default()
    }
}

impl Keychain for InMemoryKeychain {
    fn get(&self, key: &str) -> CoreResult<Option<Vec<u8>>> {
        Ok(self.items.lock().unwrap().get(key).cloned())
    }

    fn set(&self, key: &str, secret: &[u8]) -> CoreResult<()> {
        self.items
            .lock()
            .unwrap()
            .insert(key.to_string(), secret.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> CoreResult<()> {
        self.items.lock().unwrap().remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_keychain_round_trips_and_deletes() {
        let kc = InMemoryKeychain::new();
        assert_eq!(kc.get("k").unwrap(), None);
        kc.set("k", b"secret").unwrap();
        assert_eq!(kc.get("k").unwrap().as_deref(), Some(&b"secret"[..]));
        kc.delete("k").unwrap();
        assert_eq!(kc.get("k").unwrap(), None);
    }
}
