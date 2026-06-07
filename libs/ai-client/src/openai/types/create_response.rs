use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::openai::{
    sanitise_request_params, OpenAIJsonSchema, OpenAIModel, OpenAIReasoningEffort,
};

// ============================================================================
// OpenAI Response Generation Types
// ============================================================================

/// POST /v1/responses
#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIResponsesCreateRequest {
    pub model: OpenAIModel,

    /// Text, image, or file inputs. Can be a string or an array of items.
    pub input: OpenAIResponsesInput,

    /// System/developer message inserted into the model context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Upper bound for generated tokens (includes reasoning tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// For better cache hit rates (replaces legacy `user`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,

    /// Set to "24h" to keep cached prefixes around longer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,

    /// Structured outputs etc. are configured here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<OpenAIResponsesTextConfig>,

    /// If you want multi-turn without resending all context, you can use this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,

    /// Stored by default; you can disable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// Reasoning config is an object in Responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<OpenAIResponsesReasoning>,

    /// Tools available to the model (e.g., image generation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAIResponsesTool>>,
}

impl OpenAIResponsesCreateRequest {
    pub(crate) fn sanitise(&mut self) {
        let mut effort = self.reasoning.as_ref().and_then(|r| r.effort.clone());
        sanitise_request_params(
            &self.model,
            &mut self.temperature,
            &mut effort,
            &mut self.prompt_cache_key,
            &mut self.prompt_cache_retention,
        );
        if let Some(reasoning) = self.reasoning.as_mut() {
            reasoning.effort = effort;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIResponsesInput {
    Text(String),
    Items(Vec<OpenAIResponsesInputItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIResponsesInputItem {
    /// "user", "assistant", "system", "developer" are seen in docs/guides.
    pub role: String,
    /// Content can be a simple string or an array of content parts (text, images).
    pub content: OpenAIResponsesInputContent,
}

/// Input content can be a simple string or an array of content parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIResponsesInputContent {
    Text(String),
    Parts(Vec<OpenAIResponsesInputContentPart>),
}

impl From<String> for OpenAIResponsesInputContent {
    fn from(s: String) -> Self {
        OpenAIResponsesInputContent::Text(s)
    }
}

impl From<&str> for OpenAIResponsesInputContent {
    fn from(s: &str) -> Self {
        OpenAIResponsesInputContent::Text(s.to_string())
    }
}

impl From<Vec<OpenAIResponsesInputContentPart>> for OpenAIResponsesInputContent {
    fn from(parts: Vec<OpenAIResponsesInputContentPart>) -> Self {
        OpenAIResponsesInputContent::Parts(parts)
    }
}

/// Input content parts for multimodal requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIResponsesInputContentPart {
    /// Text input.
    InputText { text: String },
    /// Image input via URL or base64 data URI.
    InputImage {
        /// URL or base64 data URI (e.g., "data:image/jpeg;base64,...")
        image_url: String,
        /// Optional detail level: "low", "high", or "auto"
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

impl OpenAIResponsesInputContentPart {
    /// Create a text input part.
    pub fn text(text: impl Into<String>) -> Self {
        OpenAIResponsesInputContentPart::InputText { text: text.into() }
    }

    /// Create an image input part from a URL.
    pub fn image_url(url: impl Into<String>) -> Self {
        OpenAIResponsesInputContentPart::InputImage {
            image_url: url.into(),
            detail: None,
        }
    }

    /// Create an image input part from base64 data.
    pub fn image_base64(mime_type: &str, base64_data: &str) -> Self {
        OpenAIResponsesInputContentPart::InputImage {
            image_url: format!("data:{};base64,{}", mime_type, base64_data),
            detail: None,
        }
    }

    /// Create an image input part with detail level.
    pub fn image_with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        OpenAIResponsesInputContentPart::InputImage {
            image_url: url.into(),
            detail: Some(detail.into()),
        }
    }
}

/// Responses uses `text` config instead of chat-completions `response_format`.
#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIResponsesTextConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OpenAIResponsesTextFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIResponsesTextFormat {
    Text,
    JsonSchema(OpenAIJsonSchema),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponsesReasoning {
    /// Your existing enum can map into a string here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<OpenAIReasoningEffort>,
}

// ============================================================================
// Tools
// ============================================================================

/// Tools available in the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIResponsesTool {
    /// Image generation tool using GPT Image models.
    ImageGeneration(OpenAIImageGenerationTool),
}

impl OpenAIResponsesTool {
    /// Create an image generation tool with default settings.
    pub fn image_generation() -> Self {
        OpenAIResponsesTool::ImageGeneration(OpenAIImageGenerationTool::default())
    }

    /// Create an image generation tool with a specific model.
    pub fn image_generation_with_model(model: OpenAIImageModel) -> Self {
        OpenAIResponsesTool::ImageGeneration(OpenAIImageGenerationTool {
            model: Some(model),
            ..Default::default()
        })
    }
}

/// Configuration for the image generation tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIImageGenerationTool {
    /// Model to use: gpt-image-1-mini, gpt-image-1, gpt-image-1.5
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<OpenAIImageModel>,

    /// Image size: 1024x1024, 1536x1024 (landscape), 1024x1536 (portrait), or auto
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<OpenAIImageSize>,

    /// Quality: low, medium, high, or auto
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<OpenAIImageQuality>,

    /// Background: transparent, opaque, or auto
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<OpenAIImageBackground>,

    /// Output format: png, webp, or jpeg
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<OpenAIImageFormat>,

    /// Number of partial images for streaming (0-3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<u8>,

    /// Action: auto, generate, or edit (only for gpt-image-1.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<OpenAIImageAction>,

    /// Input fidelity for image editing: high or low
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<OpenAIImageInputFidelity>,
}

/// GPT Image models for image generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OpenAIImageModel {
    /// Fast and cheap model (default)
    #[serde(rename = "gpt-image-1-mini")]
    #[default]
    GptImage1Mini,
    /// Standard model
    #[serde(rename = "gpt-image-1")]
    GptImage1,
    /// Best quality model
    #[serde(rename = "gpt-image-1.5")]
    GptImage1_5,
}

/// Image size options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OpenAIImageSize {
    #[serde(rename = "1024x1024")]
    Square1024,
    #[serde(rename = "1536x1024")]
    Landscape,
    #[serde(rename = "1024x1536")]
    Portrait,
    #[serde(rename = "auto")]
    #[default]
    Auto,
}

/// Image quality options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageQuality {
    Low,
    Medium,
    High,
    #[default]
    Auto,
}

/// Image background options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageBackground {
    Transparent,
    Opaque,
    #[default]
    Auto,
}

/// Image output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageFormat {
    #[default]
    Png,
    Webp,
    Jpeg,
}

/// Action for image generation (gpt-image-1.5 only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageAction {
    /// Let the model decide (recommended)
    #[default]
    Auto,
    /// Always generate a new image
    Generate,
    /// Edit an existing image (requires image in context)
    Edit,
}

/// Input fidelity for image editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageInputFidelity {
    #[default]
    High,
    Low,
}

/// Response object returned by POST /v1/responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponsesCreateResponse {
    pub id: String,
    pub object: String, // always "response"
    pub created_at: u64,

    pub status: OpenAIResponseStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<OpenAIResponseError>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<OpenAIIncompleteDetails>,

    /// Generated items (messages, tool calls, etc).
    pub output: Vec<OpenAIResponseOutputItem>,

    pub usage: OpenAIResponseUsage,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    #[serde(default)]
    pub metadata: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIResponseStatus {
    Completed,
    Failed,
    InProgress,
    Cancelled,
    Queued,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseError {
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIIncompleteDetails {
    #[serde(flatten)]
    pub extra: Value,
}

/// Usage shape shown in the Responses docs example.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseUsage {
    pub input_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens_details: Option<OpenAIInputTokensDetails>,

    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OpenAIOutputTokensDetails>,

    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIInputTokensDetails {
    pub cached_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIOutputTokensDetails {
    pub reasoning_tokens: u64,
}

/// Output items: many types exist; for text generation you mostly care about "message".
/// Keep an Unknown fallback so you don't break when new item types appear.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIResponseOutputItem {
    /// Text message output.
    Message(OpenAIResponseMessageItem),

    /// Image generation tool call result.
    ImageGenerationCall(OpenAIImageGenerationCallItem),

    #[serde(other)]
    Unknown,
}

/// Image generation call output item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageGenerationCallItem {
    /// Unique ID for this output item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Status of the generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Base64-encoded image data.
    pub result: String,

    /// Size of the generated image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// Quality setting used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,

    /// Background setting used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

impl OpenAIImageGenerationCallItem {
    /// Decode the base64 result to raw image bytes.
    pub fn decode_image(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.decode(&self.result)
    }
}

/// "message" item shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseMessageItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub role: String, // "assistant" typically
    #[serde(alias = "text")]
    pub content: Vec<OpenAIResponseContentPart>,
}

/// Content parts. In the example output content is "output_text".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIResponseContentPart {
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<Value>,
    },

    InputText {
        text: String,
    },

    #[serde(other)]
    Unknown,
}

/// One SSE "data: {...}" JSON object.
/// Each event includes a `type` discriminator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenAIResponsesStreamEvent {
    /// Incremental update to an output_text content part.
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta(OpenAIResponseOutputTextDelta),

    /// Final text for an output_text content part.
    #[serde(rename = "response.output_text.done")]
    OutputTextDone(OpenAIResponseOutputTextDone),

    #[serde(rename = "response.content_part.added")]
    ContentPartAdded(OpenAIResponseContentPartEvent),

    #[serde(rename = "response.content_part.done")]
    ContentPartDone(OpenAIResponseContentPartEvent),

    #[serde(rename = "response.output_item.added")]
    OutputItemAdded(OpenAIResponseOutputItemEvent),

    #[serde(rename = "response.output_item.done")]
    OutputItemDone(OpenAIResponseOutputItemEvent),

    #[serde(rename = "response.created")]
    ResponseCreated(OpenAIResponseEvent),

    #[serde(rename = "response.in_progress")]
    ResponseInProgress(OpenAIResponseEvent),

    /// End-of-stream / final lifecycle event (you can treat this as "stop").
    #[serde(rename = "response.completed")]
    ResponseDone(OpenAIResponseEvent),

    /// Some integrations may emit explicit errors in-stream.
    #[serde(rename = "error")]
    Error(OpenAIStreamError),

    // Image generation streaming events
    /// Partial image during generation (for progressive rendering).
    #[serde(rename = "response.image_generation_call.partial_image")]
    ImageGenerationPartialImage(OpenAIImageGenerationPartialEvent),

    /// Image generation started.
    #[serde(rename = "response.image_generation_call.generating")]
    ImageGenerationGenerating(OpenAIImageGenerationStatusEvent),

    /// Image generation completed.
    #[serde(rename = "response.image_generation_call.complete")]
    ImageGenerationComplete(OpenAIImageGenerationCompleteEvent),

    /// Anything else — keep for forward compatibility.
    #[serde(other)]
    Unknown,
}

// ============================================================================
// Image Generation Streaming Events
// ============================================================================

/// Partial image event during streaming generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageGenerationPartialEvent {
    pub item_id: String,
    pub sequence_number: u32,
    pub output_index: u32,
    /// Base64-encoded partial image data.
    pub partial_image: String,
    /// Size of the image being generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Quality setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Background setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

impl OpenAIImageGenerationPartialEvent {
    /// Decode the partial image to raw bytes.
    pub fn decode_image(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.decode(&self.partial_image)
    }
}

/// Image generation status event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageGenerationStatusEvent {
    pub item_id: String,
    pub sequence_number: u32,
    pub output_index: u32,
}

/// Image generation complete event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageGenerationCompleteEvent {
    pub item_id: String,
    pub sequence_number: u32,
    pub output_index: u32,
    /// The completed image generation call item.
    pub item: OpenAIImageGenerationCallItem,
}

/// response.output_text.delta event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseOutputTextDelta {
    pub item_id: String,
    pub sequence_number: u32,
    pub output_index: u32,
    pub content_index: u32,
    pub delta: String,
}

/// response.output_text.done event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseOutputTextDone {
    pub item_id: String,
    pub sequence_number: u32,
    pub output_index: u32,
    pub content_index: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseContentPartEvent {
    pub item_id: String,
    pub sequence_number: u32,
    pub output_index: u32,
    pub content_index: u32,
    pub part: OpenAIResponseContentPart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseOutputItemEvent {
    pub sequence_number: u32,
    pub output_index: u32,
    pub item: OpenAIResponseOutputItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseEvent {
    pub response: OpenAIResponse,
    pub sequence_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub status: String,
    pub model: String,
    pub output: Vec<OpenAIResponseMessageItem>,
    pub usage: Option<OpenAIResponseUsage>,
}

/// Generic error event (streaming). Keep permissive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    pub error: Value,
}
