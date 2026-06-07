//! Character chat behavior (`CHAT`).
//!
//! Pure helpers (reply sanitisation, prompt-history construction, per-character
//! coalescing) plus the chat engine that orchestrates opening a chat, streaming a
//! reply, reactions, the rolling summary, and the background character-state
//! updater.

pub mod coalesce;
pub mod engine;
pub mod history;
pub mod prompts;
pub mod sanitise;

pub use coalesce::{Coalescer, Decision};
pub use engine::{ChatEngine, SendOutcome};
pub use history::to_history_messages;
pub use sanitise::sanitise_reply;
