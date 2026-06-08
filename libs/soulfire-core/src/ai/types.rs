//! The provider-contract types (`AI-1`, `AI-5`, `AI-6`, `AI-10`, `AI-12`).

use serde::{Deserialize, Serialize};

use crate::model::ai_model::AiModel;

/// A role-tagged message role (`AI-1`). Roles are developer/system, user, and
/// model/assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Developer/system instructions.
    Developer,
    /// The player / user.
    User,
    /// The model / assistant.
    Model,
}

/// A role-tagged message in a generation request (`AI-1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: Role,
    pub content: String,
}

impl PromptMessage {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        PromptMessage {
            role,
            content: content.into(),
        }
    }
    pub fn developer(content: impl Into<String>) -> Self {
        PromptMessage::new(Role::Developer, content)
    }
    pub fn user(content: impl Into<String>) -> Self {
        PromptMessage::new(Role::User, content)
    }
    pub fn model(content: impl Into<String>) -> Self {
        PromptMessage::new(Role::Model, content)
    }
}

/// Reasoning-effort level for models that support it (`AI-6`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
}

/// JSON output mode for a request (`AI-5`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonMode {
    /// Plain JSON text requested (no schema).
    Json,
    /// Strict JSON constrained to a schema (object schemas disallow unspecified
    /// properties).
    Schema(serde_json::Value),
}

/// Generation configuration (`AI-6`). Parameters a provider does not support are
/// ignored without error (e.g. OpenAI ignores `top_k`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GenerationConfig {
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub json: Option<JsonMode>,
    /// Hint that the provider should use prompt-prefix caching where supported
    /// (`AI-4`).
    pub cache_hint: bool,
}

impl GenerationConfig {
    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_output_tokens = Some(n);
        self
    }
    pub fn with_temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }
    pub fn with_json(mut self, mode: JsonMode) -> Self {
        self.json = Some(mode);
        self
    }
}

/// A text/structured generation request (`AI-1`).
#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub model: AiModel,
    /// The stable, cache-eligible instructions prefix (`AI-4`).
    pub instructions: Option<String>,
    /// Role-tagged conversation messages (durable → volatile order preserved).
    pub messages: Vec<PromptMessage>,
    pub config: GenerationConfig,
}

/// Token usage reported by the provider (`AI-15`, `STAT-3`). `cached_input_tokens`
/// is a subset of `input_tokens` when reported.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: Option<u64>,
}

/// A completed one-shot generation (`AI-1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationResponse {
    pub text: String,
    pub usage: Usage,
}

/// A streamed generation event (`AI-10`). A stream yields zero or more `Delta`s,
/// then a terminal `Full` (the complete text) and a `Usage`, or an `Error`.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    /// An incremental text delta.
    Delta(String),
    /// The terminal full-text event.
    Full(String),
    /// The terminal usage/metadata event.
    Usage(Usage),
    /// An error event.
    Error(ProviderError),
}

/// A request to generate an image (`IMG-1`).
#[derive(Debug, Clone)]
pub struct ImageRequest {
    pub model: AiModel,
    pub prompt: String,
}

/// A generated image (`IMG-1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageResponse {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub usage: Usage,
}

/// Errors a provider can surface (`AI-12`). Feature code maps these to clear,
/// user-actionable messages and leaves local state consistent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum ProviderError {
    /// No API key configured for the required provider (`AI-3`).
    #[error("no API key configured — add your API key to continue")]
    MissingApiKey,
    /// The configured key was rejected (authentication failure).
    #[error("the provider rejected the API key")]
    InvalidApiKey,
    /// Rate-limit or quota exceeded on the user's provider account (`PROD-13`).
    #[error("provider rate limit or quota exceeded: {0}")]
    RateLimited(String),
    /// The provider blocked the content on policy grounds.
    #[error("the provider blocked this request on content-policy grounds")]
    ContentBlocked,
    /// Transient unavailability (e.g. HTTP 503); retryable (`AI-13`).
    #[error("provider temporarily unavailable: {0}")]
    Transient(String),
    /// The stream produced no first token within the idle timeout (`AI-11`).
    #[error("timed out waiting for the provider to respond")]
    Timeout,
    /// Any other provider error.
    #[error("provider error: {0}")]
    Other(String),
}

impl ProviderError {
    /// Whether the error is transient and worth retrying with backoff (`AI-13`).
    pub fn is_transient(&self) -> bool {
        matches!(self, ProviderError::Transient(_))
    }
}
