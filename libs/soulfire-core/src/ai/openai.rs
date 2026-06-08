//! The OpenAI adapter (`AI-2`, `AI-4`, `AI-5`): maps the provider seam onto the
//! vendored `ai-client` OpenAI Responses API. The instructions block is sent as
//! the cacheable prefix (`AI-4`); structured output uses a strict JSON schema
//! (`AI-5`); `top_k` is dropped (OpenAI ignores it, `AI-6`).

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream;

use crate::model::ai_model::{AiModel, AiVendor};
use ai_client::error::AiError;
use ai_client::openai::create_response::{
    OpenAIResponseContentPart, OpenAIResponseOutputItem, OpenAIResponseUsage,
    OpenAIResponsesCreateRequest, OpenAIResponsesInput, OpenAIResponsesInputContent,
    OpenAIResponsesInputItem, OpenAIResponsesReasoning, OpenAIResponsesStreamEvent,
    OpenAIResponsesTextConfig, OpenAIResponsesTextFormat, OpenAIResponsesTool,
};
use ai_client::openai::{OpenAIClient, OpenAIJsonSchema, OpenAIModel, OpenAIReasoningEffort};

use super::provider::{AiProvider, ApiKeySource, EventStream};
use super::types::{
    GenerationRequest, GenerationResponse, ImageRequest, ImageResponse, JsonMode, ProviderError,
    ReasoningEffort, Role, StreamEvent, Usage,
};

/// The OpenAI provider. Fetches the API key from the key source on each call so a
/// freshly-entered key takes effect immediately, and never holds the key beyond
/// the request (`SEC-9`).
pub struct OpenAiProvider {
    keys: Arc<dyn ApiKeySource>,
}

impl OpenAiProvider {
    pub fn new(keys: Arc<dyn ApiKeySource>) -> Self {
        OpenAiProvider { keys }
    }

    fn client(&self) -> Result<OpenAIClient, ProviderError> {
        let key = self
            .keys
            .api_key(AiVendor::OpenAI)
            .ok_or(ProviderError::MissingApiKey)?;
        OpenAIClient::builder()
            .api_key(key.expose().clone())
            .build()
            .map_err(map_ai_error)
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse, ProviderError> {
        let client = self.client()?;
        let req = build_request(&request, false)?;
        let resp = client.generate_response(req).await.map_err(map_ai_error)?;
        let text = extract_text(&resp.output);
        Ok(GenerationResponse {
            text,
            usage: map_usage(&resp.usage),
        })
    }

    async fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> Result<EventStream, ProviderError> {
        let client = self.client()?;
        let req = build_request(&request, true)?;
        let raw = client
            .generate_response_streamed(req)
            .await
            .map_err(map_ai_error)?;

        // Map each provider SSE event to zero or more seam events.
        let mapped = raw.flat_map(|item| {
            let events: Vec<StreamEvent> = match item {
                Ok(OpenAIResponsesStreamEvent::OutputTextDelta(d)) => {
                    vec![StreamEvent::Delta(d.delta)]
                }
                Ok(OpenAIResponsesStreamEvent::OutputTextDone(d)) => {
                    vec![StreamEvent::Full(d.text)]
                }
                Ok(OpenAIResponsesStreamEvent::ResponseDone(ev)) => match ev.response.usage {
                    Some(u) => vec![StreamEvent::Usage(map_usage(&u))],
                    None => vec![],
                },
                Ok(OpenAIResponsesStreamEvent::Error(e)) => {
                    vec![StreamEvent::Error(ProviderError::Other(
                        e.error.to_string(),
                    ))]
                }
                Err(e) => vec![StreamEvent::Error(ProviderError::Transient(e.to_string()))],
                _ => vec![], // lifecycle / image / unknown events are not surfaced
            };
            stream::iter(events)
        });
        Ok(mapped.boxed())
    }

    async fn generate_image(&self, request: ImageRequest) -> Result<ImageResponse, ProviderError> {
        let client = self.client()?;
        let model = map_model(request.model)?;
        let req = OpenAIResponsesCreateRequest::builder()
            .model(model)
            .input(OpenAIResponsesInput::Text(request.prompt))
            .tools(vec![OpenAIResponsesTool::image_generation()])
            .build();
        let resp = client.generate_response(req).await.map_err(map_ai_error)?;

        for item in &resp.output {
            if let OpenAIResponseOutputItem::ImageGenerationCall(call) = item {
                let bytes = call
                    .decode_image()
                    .map_err(|e| ProviderError::Other(format!("image decode failed: {e}")))?;
                return Ok(ImageResponse {
                    bytes,
                    mime: "image/png".to_string(),
                    usage: map_usage(&resp.usage),
                });
            }
        }
        Err(ProviderError::Other(
            "provider returned no image".to_string(),
        ))
    }
}

/// Map our model to the ai-client OpenAI model via its stable id (`AI-7`).
fn map_model(model: AiModel) -> Result<OpenAIModel, ProviderError> {
    model
        .id()
        .parse::<OpenAIModel>()
        .map_err(|_| ProviderError::Other(format!("unsupported OpenAI model: {}", model.id())))
}

fn map_role(role: Role) -> &'static str {
    match role {
        Role::Developer => "developer",
        Role::User => "user",
        Role::Model => "assistant",
    }
}

fn map_effort(effort: ReasoningEffort) -> OpenAIReasoningEffort {
    match effort {
        ReasoningEffort::Minimal => OpenAIReasoningEffort::Minimal,
        ReasoningEffort::Low => OpenAIReasoningEffort::Low,
        ReasoningEffort::Medium => OpenAIReasoningEffort::Medium,
        ReasoningEffort::High => OpenAIReasoningEffort::High,
    }
}

/// Build the Responses request from a seam request (`AI-1`, `AI-4`, `AI-5`).
fn build_request(
    request: &GenerationRequest,
    stream: bool,
) -> Result<OpenAIResponsesCreateRequest, ProviderError> {
    let model = map_model(request.model)?;
    let items: Vec<OpenAIResponsesInputItem> = request
        .messages
        .iter()
        .map(|m| OpenAIResponsesInputItem {
            role: map_role(m.role).to_string(),
            content: OpenAIResponsesInputContent::Text(m.content.clone()),
        })
        .collect();

    let cfg = &request.config;
    let text_config = cfg.json.as_ref().map(|json| match json {
        JsonMode::Json => OpenAIResponsesTextConfig {
            format: Some(OpenAIResponsesTextFormat::Text),
        },
        JsonMode::Schema(schema) => OpenAIResponsesTextConfig {
            format: Some(OpenAIResponsesTextFormat::JsonSchema(OpenAIJsonSchema {
                name: "response".to_string(),
                description: String::new(),
                schema: schema.clone(),
                strict: Some(true),
            })),
        },
    });
    let reasoning = cfg.reasoning_effort.map(|e| OpenAIResponsesReasoning {
        effort: Some(map_effort(e)),
    });

    Ok(OpenAIResponsesCreateRequest::builder()
        .model(model)
        .input(OpenAIResponsesInput::Items(items))
        .maybe_instructions(request.instructions.clone())
        .maybe_max_output_tokens(cfg.max_output_tokens.map(|n| n as u64))
        .maybe_temperature(cfg.temperature)
        .maybe_top_p(cfg.top_p)
        .maybe_text(text_config)
        .maybe_reasoning(reasoning)
        .stream(stream)
        .build())
}

/// Concatenate the assistant message text from the output items.
fn extract_text(output: &[OpenAIResponseOutputItem]) -> String {
    let mut text = String::new();
    for item in output {
        if let OpenAIResponseOutputItem::Message(msg) = item {
            for part in &msg.content {
                if let OpenAIResponseContentPart::OutputText { text: t, .. } = part {
                    text.push_str(t);
                }
            }
        }
    }
    text
}

fn map_usage(u: &OpenAIResponseUsage) -> Usage {
    Usage {
        input_tokens: u.input_tokens,
        output_tokens: u.output_tokens,
        cached_input_tokens: u.input_tokens_details.as_ref().map(|d| d.cached_tokens),
    }
}

/// Map an `ai-client` error to a provider error (`AI-12`, `AI-13`).
fn map_ai_error(err: AiError) -> ProviderError {
    match err {
        AiError::MissingApiKey => ProviderError::MissingApiKey,
        AiError::InvalidApiKey => ProviderError::InvalidApiKey,
        AiError::ApiError(status, msg) => match status.as_u16() {
            401 | 403 => ProviderError::InvalidApiKey,
            429 => ProviderError::RateLimited(msg),
            500 | 502 | 503 | 504 => ProviderError::Transient(msg),
            400 if msg.to_lowercase().contains("content")
                || msg.to_lowercase().contains("policy") =>
            {
                ProviderError::ContentBlocked
            }
            _ => ProviderError::Other(format!("[{status}] {msg}")),
        },
        // Network-level failures are transient and worth retrying (AI-13).
        AiError::Request(e) | AiError::Response(e) => ProviderError::Transient(e.to_string()),
        AiError::InvalidModel => ProviderError::Other("invalid model".to_string()),
        AiError::InvalidClient(m) => ProviderError::Other(m),
        AiError::Json(e) => ProviderError::Other(format!("json error: {e}")),
    }
}
