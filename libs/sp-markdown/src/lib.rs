//! Markdown parsing helpers.
//!
//! Vendored from the owner's `sp-markdown` crate. The old Dioxus renderer has
//! been removed with the Dioxus app; keep the parser/AST as reusable core logic
//! for future React rendering or prompt/display tests.

pub mod ast;
pub mod parser;

pub use ast::{Block, Inline};
pub use parser::parse;
