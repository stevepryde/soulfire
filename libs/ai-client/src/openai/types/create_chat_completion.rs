use serde::{Deserialize, Serialize};

use crate::openai::{
    types::sanitise_request_params, OpenAIJsonSchema, OpenAIModel, OpenAIPrompt,
    OpenAIReasoningEffort, OpenAIRole,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum OpenAIResponseFormat {
    Text,
    JsonSchema { json_schema: OpenAIJsonSchema },
    JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIGenerateContentResponse {
    pub choices: Vec<OpenAIResponseChoice>,
    pub created: u64,
    pub id: String,
    pub model: String,
    pub object: String,
    pub usage: OpenAIResponseUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseChoice {
    pub index: u64,
    pub finish_reason: String,
    pub message: OpenAiMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiMessage {
    pub role: OpenAIRole,
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub annotations: Option<Vec<OpenAIAnnotation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAnnotation {
    pub r#type: String,
    pub url_citation: OpenAIUrlCitation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIUrlCitation {
    pub start_index: u64,
    pub end_index: u64,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseUsage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
    // TODO: Add other fields as needed
}

/// Streaming response chunk from OpenAI API.
/// This is the response format when stream=true is set in the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIStreamChoice>,
    /// Usage information is only included in the final chunk
    pub usage: Option<OpenAIResponseUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamChoice {
    pub index: u64,
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIDelta {
    pub role: Option<OpenAIRole>,
    pub content: Option<String>,
    pub refusal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIGenerateContentRequest {
    /// Model ID used to generate the response, like gpt-4o or o3.
    /// OpenAI offers a wide range of models with different capabilities, performance
    /// characteristics, and price points. Refer to the model guide to browse and compare
    /// available models.
    pub model: OpenAIModel,
    /// A list of messages comprising the conversation so far. Depending on the model you use,
    /// different message types (modalities) are supported, like text, images, and audio.
    pub messages: Vec<OpenAIPrompt>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their
    /// existing frequency in the text so far, decreasing the model's likelihood to repeat
    /// the same line verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    /// An upper bound for the number of tokens that can be generated for a completion, including
    /// visible output tokens and reasoning tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,
    /// How many chat completion choices to generate for each input message. Note that you will be
    /// charged based on the number of generated tokens across all of the choices. Keep n as 1 to
    /// minimize costs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u64>,
    /// Output types that you would like the model to generate. Most models are capable of
    /// generating text, which is the default: ["text"].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,
    /// An object specifying the format that the model must output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<OpenAIResponseFormat>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the
    /// output more random, while lower values like 0.2 will make it more focused and deterministic.
    /// We generally recommend altering this or top_p but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// An alternative to sampling with temperature, called nucleus sampling, where the model
    /// considers the results of the tokens with top_p probability mass.
    /// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    ///
    /// We generally recommend altering this or temperature but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// If set, partial message deltas will be sent, like in ChatGPT. Tokens will be sent as
    /// data-only server-sent events as they become available, with the stream terminated by a
    /// `data: [DONE]` message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Constrains effort on reasoning for reasoning models.
    /// Currently supported values are none, minimal, low, medium, high, and xhigh.
    /// Reducing reasoning effort can result in faster responses and fewer tokens
    /// used on reasoning in a response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<OpenAIReasoningEffort>,
}

impl OpenAIGenerateContentRequest {
    pub(crate) fn sanitise(&mut self) {
        sanitise_request_params(
            &self.model,
            &mut self.temperature,
            &mut self.reasoning_effort,
            &mut None,
            &mut None,
        );
    }
}
