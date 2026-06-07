//! A coarse reactivity bridge between the imperative store and Dioxus.
//!
//! Screens read the store synchronously (cheap local SQLite reads). To re-render
//! when data changes, a screen calls [`subscribe`] (subscribing to a global
//! version signal) and any mutation calls [`touch`] to bump it. This is a
//! pragmatic single-process pattern; per-entity precision can come later.

use dioxus::prelude::*;

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
