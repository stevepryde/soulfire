//! AI provider client.
//!
//! Vendored and trimmed from the owner's `ai-client` crate: **OpenAI only**. The
//! unmaintained/untested Gemini path is dropped (per `specs/03-ai-integration.md`
//! AI-2 and the vendoring decision in `specs/12-platform-packaging.md`). Sits
//! behind the provider seam (AI-1) and is substitutable in tests (TEST-5).

pub mod error;
pub mod openai;

pub(crate) mod utils;

pub mod prelude {
    pub use crate::error::{AiError, AiResult};
}
