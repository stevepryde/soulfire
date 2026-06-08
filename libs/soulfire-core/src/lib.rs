//! Soulfire's pure-Rust engine.
//!
//! Houses the store (encrypted SQLCipher), the AI provider seam and OpenAI
//! adapter, prompt assembly, and the chat/world turn engines — everything the
//! app shell drives. Kept free of UI dependencies so it is unit- and
//! integration-testable offline against substituted seams (`specs/13-testing.md`
//! TEST-2/TEST-5).
//!
//! The current app-shell rebuild direction is tracked in
//! `docs/OG_PARITY_ROADMAP.md`.

pub mod ai;
pub mod character;
pub mod chat;
pub mod clock;
pub mod error;
pub mod image;
pub mod keychain;
pub mod prompt;
pub mod seed;
pub mod stats;
pub mod store;
pub mod world;

pub use clock::{Clock, MockClock, SystemClock};
pub use error::{CoreError, CoreResult};
pub use keychain::{InMemoryKeychain, Keychain};
pub use store::{ImageOwnerKind, Store, StorePaths, StoredImage};
