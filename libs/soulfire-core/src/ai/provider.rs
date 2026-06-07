//! The AI provider seam (`AI-1`, `TEST-5`) and the streaming collector that
//! enforces the idle timeout (`AI-11`, `CHAT-6`).

use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::BoxStream;

use lib_soulfire::ai_model::AiVendor;
use sp_core::secret::Secret;

use super::types::{
    GenerationRequest, GenerationResponse, ImageRequest, ImageResponse, ProviderError, StreamEvent,
    Usage,
};

/// A stream of generation events (`AI-10`).
pub type EventStream = BoxStream<'static, StreamEvent>;

/// The single internal contract every AI provider implements (`AI-1`). Feature
/// code depends only on this trait; production wires the OpenAI adapter, tests
/// wire a deterministic recording fake (`TEST-5`).
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// One-shot text (or structured-JSON) generation.
    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse, ProviderError>;

    /// Streamed text generation (`AI-10`).
    async fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> Result<EventStream, ProviderError>;

    /// Image generation (`IMG-1`).
    async fn generate_image(&self, request: ImageRequest) -> Result<ImageResponse, ProviderError>;
}

/// A source of provider API keys, decrypted from the store on demand (`SEC-9`).
/// Kept separate from the request so keys never flow through generic plumbing.
pub trait ApiKeySource: Send + Sync {
    /// The current API key for a vendor, or `None` if none is configured (`AI-3`).
    fn api_key(&self, vendor: AiVendor) -> Option<Secret<String>>;
}

/// Consume a streamed generation, forwarding each text delta to `on_delta` for
/// live rendering and enforcing the idle timeout (`AI-10`, `AI-11`, `CHAT-6`):
///
/// - If no first token arrives within `idle_timeout`, fail with
///   [`ProviderError::Timeout`] and return nothing (caller saves nothing).
/// - If the stream goes idle *after* partial text, finalize and return the
///   partial text rather than discarding it.
pub async fn collect_streamed<F>(
    mut stream: EventStream,
    idle_timeout: Duration,
    mut on_delta: F,
) -> Result<GenerationResponse, ProviderError>
where
    F: FnMut(&str),
{
    let mut accumulated = String::new();
    let mut got_first = false;
    let mut usage = Usage::default();
    let mut full: Option<String> = None;

    loop {
        match tokio::time::timeout(idle_timeout, stream.next()).await {
            // Idle timeout elapsed with no new event.
            Err(_) => {
                if got_first {
                    break; // finalize partial text (AI-11)
                }
                return Err(ProviderError::Timeout); // no first token (AI-11)
            }
            // Stream ended.
            Ok(None) => break,
            Ok(Some(event)) => match event {
                StreamEvent::Delta(delta) => {
                    got_first = true;
                    on_delta(&delta);
                    accumulated.push_str(&delta);
                }
                StreamEvent::Full(text) => {
                    got_first = true;
                    full = Some(text);
                }
                StreamEvent::Usage(u) => usage = u,
                StreamEvent::Error(e) => return Err(e),
            },
        }
    }

    Ok(GenerationResponse {
        text: full.unwrap_or(accumulated),
        usage,
    })
}
