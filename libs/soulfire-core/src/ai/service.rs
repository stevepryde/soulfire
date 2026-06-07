//! The AI service: the layer feature code calls. Enforces the missing-key
//! condition (`AI-3`) before any request is sent and applies bounded retry with
//! backoff for transient provider errors (`AI-13`). Metering (`AI-15`) is done by
//! the calling engine, which holds the entity context and the store.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use lib_soulfire::ai_model::AiModel;

use super::provider::{AiProvider, ApiKeySource, EventStream};
use super::types::{
    GenerationRequest, GenerationResponse, ImageRequest, ImageResponse, ProviderError,
};

/// Maximum number of retries for transient provider errors (`AI-13`).
pub const MAX_TRANSIENT_RETRIES: u32 = 5;

/// The AI service feature code depends on (`AI-1`). Wraps a provider and the
/// API-key source.
#[derive(Clone)]
pub struct AiService {
    provider: Arc<dyn AiProvider>,
    keys: Arc<dyn ApiKeySource>,
}

impl AiService {
    pub fn new(provider: Arc<dyn AiProvider>, keys: Arc<dyn ApiKeySource>) -> Self {
        AiService { provider, keys }
    }

    /// Fail with [`ProviderError::MissingApiKey`] if no key is configured for the
    /// model's vendor, *before* any request is sent (`AI-3`).
    fn ensure_key(&self, model: AiModel) -> Result<(), ProviderError> {
        if self.keys.api_key(model.vendor()).is_none() {
            Err(ProviderError::MissingApiKey)
        } else {
            Ok(())
        }
    }

    /// One-shot text/structured generation with the missing-key guard and
    /// transient retry.
    pub async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse, ProviderError> {
        self.ensure_key(request.model)?;
        with_retry(MAX_TRANSIENT_RETRIES, || {
            let req = request.clone();
            let provider = self.provider.clone();
            async move { provider.generate(req).await }
        })
        .await
    }

    /// Begin a streamed generation. The key guard applies up front; the caller
    /// consumes the stream (e.g. via `collect_streamed`) and enforces the idle
    /// timeout (`AI-11`).
    pub async fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> Result<EventStream, ProviderError> {
        self.ensure_key(request.model)?;
        // Retry only the initial connection; mid-stream handling is the caller's.
        with_retry(MAX_TRANSIENT_RETRIES, || {
            let req = request.clone();
            let provider = self.provider.clone();
            async move { provider.generate_stream(req).await }
        })
        .await
    }

    /// Image generation with the missing-key guard and transient retry (`IMG-1`).
    pub async fn generate_image(
        &self,
        request: ImageRequest,
    ) -> Result<ImageResponse, ProviderError> {
        self.ensure_key(request.model)?;
        with_retry(MAX_TRANSIENT_RETRIES, || {
            let req = request.clone();
            let provider = self.provider.clone();
            async move { provider.generate_image(req).await }
        })
        .await
    }
}

/// Run `f`, retrying on transient errors up to `max_retries` with linear backoff;
/// non-transient errors fail fast (`AI-13`).
async fn with_retry<T, F, Fut>(max_retries: u32, mut f: F) -> Result<T, ProviderError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ProviderError>>,
{
    let mut attempt = 0u32;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if e.is_transient() && attempt < max_retries => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::fake::{RecordingProvider, Scripted};
    use crate::ai::provider::collect_streamed;
    use crate::ai::types::{GenerationConfig, PromptMessage};
    use lib_soulfire::ai_model::AiVendor;
    use sp_core::secret::Secret;

    /// A simple key source for tests.
    struct Keys(Option<String>);
    impl ApiKeySource for Keys {
        fn api_key(&self, _vendor: AiVendor) -> Option<Secret<String>> {
            self.0.clone().map(Secret::new)
        }
    }

    fn req() -> GenerationRequest {
        GenerationRequest {
            model: AiModel::Gpt5_1,
            instructions: None,
            messages: vec![PromptMessage::user("hi")],
            config: GenerationConfig::default(),
        }
    }

    #[tokio::test]
    async fn missing_key_sends_no_request() {
        // AC-AI-a / AI-3: with no key, the action reports MissingApiKey and sends
        // nothing.
        let provider = Arc::new(RecordingProvider::new());
        let svc = AiService::new(provider.clone(), Arc::new(Keys(None)));
        let err = svc.generate(req()).await.unwrap_err();
        assert_eq!(err, ProviderError::MissingApiKey);
        assert_eq!(provider.request_count(), 0);
    }

    #[tokio::test]
    async fn with_key_generates_and_records_request() {
        let provider = Arc::new(RecordingProvider::new());
        provider.push(Scripted::text("hello!", 10, 3));
        let svc = AiService::new(provider.clone(), Arc::new(Keys(Some("sk-x".into()))));
        let resp = svc.generate(req()).await.unwrap();
        assert_eq!(resp.text, "hello!");
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(provider.request_count(), 1);
        assert_eq!(provider.last_request().unwrap().messages[0].content, "hi");
    }

    #[tokio::test(start_paused = true)]
    async fn transient_error_retries_then_succeeds() {
        // AC-AI-e: a simulated 503 retries then succeeds.
        let provider = Arc::new(RecordingProvider::new());
        provider.push(Scripted::Error(ProviderError::Transient("503".into())));
        provider.push(Scripted::text("recovered", 1, 1));
        let svc = AiService::new(provider.clone(), Arc::new(Keys(Some("sk".into()))));
        let resp = svc.generate(req()).await.unwrap();
        assert_eq!(resp.text, "recovered");
        assert_eq!(provider.request_count(), 2);
    }

    #[tokio::test]
    async fn non_transient_error_fails_fast() {
        let provider = Arc::new(RecordingProvider::new());
        provider.push(Scripted::Error(ProviderError::RateLimited("429".into())));
        let svc = AiService::new(provider.clone(), Arc::new(Keys(Some("sk".into()))));
        let err = svc.generate(req()).await.unwrap_err();
        assert!(matches!(err, ProviderError::RateLimited(_)));
        assert_eq!(provider.request_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn streaming_delivers_deltas_then_finalizes() {
        // AC-AI-d: streamed reply renders token-by-token then finalizes.
        let provider = Arc::new(RecordingProvider::new());
        provider.push(Scripted::stream(vec!["Hel", "lo ", "there"], 5, 4));
        let svc = AiService::new(provider, Arc::new(Keys(Some("sk".into()))));
        let stream = svc.generate_stream(req()).await.unwrap();
        let mut seen = String::new();
        let resp = collect_streamed(stream, Duration::from_secs(30), |d| seen.push_str(d))
            .await
            .unwrap();
        assert_eq!(seen, "Hello there");
        assert_eq!(resp.text, "Hello there");
        assert_eq!(resp.usage.output_tokens, 4);
    }

    #[tokio::test(start_paused = true)]
    async fn streaming_no_first_token_times_out_and_saves_nothing() {
        // AC-AI-d: a stream with no first token within the idle timeout errors.
        let provider = Arc::new(RecordingProvider::new());
        provider.push(Scripted::Stream {
            deltas: vec![],
            usage: Default::default(),
            stall: true,
        });
        let svc = AiService::new(provider, Arc::new(Keys(Some("sk".into()))));
        let stream = svc.generate_stream(req()).await.unwrap();
        let err = collect_streamed(stream, Duration::from_secs(30), |_| {})
            .await
            .unwrap_err();
        assert_eq!(err, ProviderError::Timeout);
    }

    #[tokio::test(start_paused = true)]
    async fn streaming_mid_stall_keeps_partial_text() {
        // AC-AI-d: a stream that stalls mid-reply keeps the partial text.
        let provider = Arc::new(RecordingProvider::new());
        provider.push(Scripted::Stream {
            deltas: vec!["partial ".into(), "text".into()],
            usage: Default::default(),
            stall: true,
        });
        let svc = AiService::new(provider, Arc::new(Keys(Some("sk".into()))));
        let stream = svc.generate_stream(req()).await.unwrap();
        let resp = collect_streamed(stream, Duration::from_secs(30), |_| {})
            .await
            .unwrap();
        assert_eq!(resp.text, "partial text");
    }
}
