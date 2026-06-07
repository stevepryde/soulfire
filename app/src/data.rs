//! A coarse reactivity bridge between the imperative store and Dioxus.
//!
//! Store access is serialized and encrypted, so GUI components must not do
//! repository reads in render. Screens subscribe to the global version signal,
//! then load snapshots through [`blocking`] from effects/resources.

use dioxus::prelude::*;
use std::fmt::Display;

static DATA_VERSION: GlobalSignal<u64> = Signal::global(|| 0);

/// Subscribe the current component to data changes (call at the top of a screen
/// that reads the store).
pub fn subscribe() -> u64 {
    DATA_VERSION()
}

/// Signal that store data changed, re-rendering all subscribers.
pub fn touch() {
    *DATA_VERSION.write() += 1;
}

/// Run blocking store/KDF work away from the Dioxus render thread.
pub async fn blocking<T, E, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    E: Display + Send + 'static,
    F: FnOnce() -> Result<T, E> + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| format!("background task failed: {e}"))?
        .map_err(|e| e.to_string())
}
