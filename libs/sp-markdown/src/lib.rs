//! Markdown rendering for Dioxus.
//!
//! Vendored verbatim from the owner's `sp-markdown` crate; renders chat-message
//! markdown (`specs/09-ui.md` UI-15). The renderer is already platform-agnostic
//! (pure RSX, no web APIs), so it works under the Dioxus desktop/mobile renderers
//! unchanged. See the vendoring decision in `specs/12-platform-packaging.md`.

pub mod ast;
pub mod parser;

#[cfg(feature = "dioxus")]
pub mod classes;
#[cfg(feature = "dioxus")]
mod render;

pub use ast::{Block, Inline};
pub use parser::parse;

#[cfg(feature = "dioxus")]
pub use render::{ChatMarkdown, Markdown};
