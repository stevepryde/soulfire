//! Soulfire's pure-Rust engine.
//!
//! Houses the store (encrypted SQLCipher), the AI provider seam and OpenAI
//! adapter, prompt assembly, and the chat/world turn engines — everything the
//! Dioxus UI shell drives. Kept free of UI dependencies so it is unit- and
//! integration-testable offline against substituted seams (`specs/13-testing.md`
//! TEST-2/TEST-5).
//!
//! Modules are filled in layer by layer per `docs/BUILD_PLAN.md`.
