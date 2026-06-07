//! The AI provider seam and OpenAI adapter (`AI`).
//!
//! `types` defines the provider contract; `provider` the trait + streaming
//! collector; `service` the key-guarded, retrying entry point feature code uses;
//! `fake` a deterministic recording provider for tests; `fence` tolerant JSON
//! parsing; `registry` model-selection precedence and token estimation. The real
//! OpenAI adapter lives in `openai`.

pub mod fake;
pub mod fence;
pub mod provider;
pub mod registry;
pub mod service;
pub mod types;

pub use fence::{parse_lenient, rescue_json_block, strip_json_fence};
pub use provider::{AiProvider, ApiKeySource, EventStream, collect_streamed};
pub use registry::{estimate_tokens, resolve_model};
pub use service::AiService;
pub use types::{
    GenerationConfig, GenerationRequest, GenerationResponse, ImageRequest, ImageResponse, JsonMode,
    PromptMessage, ProviderError, ReasoningEffort, Role, StreamEvent, Usage,
};
