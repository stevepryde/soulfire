use std::fmt::Debug;
use std::time::Duration;

#[cfg(feature = "stream")]
use futures::Stream;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
#[cfg(feature = "stream")]
use reqwest_streams::error::StreamBodyError;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(all(feature = "chat-completions", feature = "stream"))]
use crate::openai::create_chat_completion::OpenAIStreamChunk;
#[cfg(feature = "chat-completions")]
use crate::openai::create_chat_completion::{
    OpenAIGenerateContentRequest, OpenAIGenerateContentResponse,
};
#[cfg(feature = "stream")]
use crate::openai::create_response::OpenAIResponsesStreamEvent;
use crate::{
    openai::{
        create_response::{OpenAIResponsesCreateRequest, OpenAIResponsesCreateResponse},
        list_models::{OpenAIModelInfo, OpenAIModelsListResponse},
    },
    prelude::{AiError, AiResult},
    utils::Url,
};

use super::OpenAIModel;

const BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Clone, Default)]
pub struct OpenAIClientBuilder {
    api_key: Option<String>,
    timeout: Option<u64>,
}

impl Debug for OpenAIClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIClientBuilder")
            .field("api_key", &"*** redacted ***")
            .field(
                "timeout",
                &self
                    .timeout
                    .map(|t| format!("{t} seconds"))
                    .unwrap_or_else(|| "not set".to_string()),
            )
            .finish()
    }
}

impl OpenAIClientBuilder {
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> AiResult<OpenAIClient> {
        let api_key = self.api_key.ok_or(AiError::MissingApiKey)?;

        // Add default HTTP headers.
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {api_key}")
                .parse()
                .map_err(|_| AiError::InvalidApiKey)?,
        );
        headers.insert(
            USER_AGENT,
            env!("CARGO_PKG_NAME")
                .parse()
                .unwrap_or_else(|_| "reqwest".parse().unwrap()),
        );
        let mut builder = reqwest::Client::builder().default_headers(headers);

        // Default timeout.
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(Duration::from_secs(timeout));
        }

        let client = builder
            .build()
            .map_err(|e| AiError::InvalidClient(e.to_string()))?;
        Ok(OpenAIClient { api_key, client })
    }
}

pub async fn parse_response<T>(response: reqwest::Response) -> AiResult<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if status.is_success() {
        let text = response.text().await.map_err(AiError::Response)?;
        match serde_json::from_str(&text) {
            Ok(data) => Ok(data),
            Err(e) => {
                tracing::error!("failed to parse response body: {e:#}");
                tracing::error!("response body: {text}");
                Err(AiError::ApiError(
                    status,
                    "unrecognised API response".to_string(),
                ))
            }
        }
    } else {
        Err(AiError::ApiError(
            response.status(),
            response
                .text()
                .await
                .unwrap_or_else(|_| "failed to decode response body".to_string()),
        ))
    }
}

#[cfg(feature = "stream")]
/// Helper function to parse SSE streams from OpenAI API.
/// This is used by OpenAI streaming endpoints.
async fn parse_sse_stream<T>(
    response: reqwest::Response,
) -> impl Stream<Item = Result<T, StreamBodyError>>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    use futures::{stream, StreamExt};

    let byte_stream = response.bytes_stream();

    // Create a stateful stream that accumulates bytes and parses SSE events.
    // We keep a byte remainder buffer for incomplete UTF-8 sequences that can
    // occur when HTTP/2 chunks split in the middle of a multi-byte character.
    let remainder: Vec<u8> = Vec::new();
    stream::unfold(
        (byte_stream, String::new(), remainder),
        |(mut byte_stream, mut buffer, mut remainder)| async move {
            loop {
                // Try to parse a complete event from the buffer first
                if let Some(double_newline_pos) = buffer.find("\n\n") {
                    let event = buffer[..double_newline_pos].to_string();
                    buffer.drain(..double_newline_pos + 2);

                    // Parse the SSE event
                    for line in event.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            // Skip [DONE] messages
                            if data.trim() == "[DONE]" {
                                continue;
                            }

                            // Try to parse the JSON chunk
                            tracing::trace!("Received SSE data chunk: {data}");
                            match serde_json::from_str::<T>(data) {
                                Ok(chunk) => {
                                    return Some((Ok(chunk), (byte_stream, buffer, remainder)));
                                }
                                Err(e) => {
                                    tracing::error!("Failed to parse OpenAI stream chunk: {e:#}");
                                    tracing::error!("Chunk data: {data}");
                                    return Some((
                                        Err(StreamBodyError::new(
                                            reqwest_streams::error::StreamBodyKind::CodecError,
                                            None,
                                            Some(format!("Failed to parse JSON: {e}")),
                                        )),
                                        (byte_stream, buffer, remainder),
                                    ));
                                }
                            }
                        }
                    }
                }

                // Need more data from the stream
                match byte_stream.next().await {
                    Some(Ok(bytes)) => {
                        // Prepend any leftover bytes from previous incomplete UTF-8 sequence
                        let combined = if remainder.is_empty() {
                            bytes.to_vec()
                        } else {
                            let mut combined = std::mem::take(&mut remainder);
                            combined.extend_from_slice(&bytes);
                            combined
                        };

                        // Convert bytes to string, handling incomplete trailing UTF-8
                        match std::str::from_utf8(&combined) {
                            Ok(text) => {
                                buffer.push_str(text);
                            }
                            Err(e) => {
                                // Add the valid prefix to the buffer
                                let valid_up_to = e.valid_up_to();
                                if valid_up_to > 0 {
                                    // Safety: valid_up_to guarantees this slice is valid UTF-8
                                    let valid = unsafe {
                                        std::str::from_utf8_unchecked(&combined[..valid_up_to])
                                    };
                                    buffer.push_str(valid);
                                }
                                // Save the trailing incomplete bytes for the next chunk
                                remainder = combined[valid_up_to..].to_vec();
                            }
                        }
                    }
                    Some(Err(e)) => {
                        return Some((
                            Err(StreamBodyError::new(
                                reqwest_streams::error::StreamBodyKind::InputOutputError,
                                Some(Box::new(e)),
                                None,
                            )),
                            (byte_stream, buffer, remainder),
                        ));
                    }
                    None => {
                        // Stream ended
                        return None;
                    }
                }
            }
        },
    )
}

#[non_exhaustive]
pub struct OpenAIClient {
    pub api_key: String,
    pub client: reqwest::Client,
}

impl OpenAIClient {
    pub fn builder() -> OpenAIClientBuilder {
        OpenAIClientBuilder::default()
    }

    pub async fn get<T>(&self, url: &str) -> AiResult<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(AiError::Request)?;

        parse_response(response).await
    }

    pub async fn post<Req, Res>(&self, url: &str, request: Req) -> AiResult<Res>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(AiError::Request)?;

        parse_response(response).await
    }

    pub async fn list_models(&self) -> AiResult<OpenAIModelsListResponse> {
        let url = Url::new(format!("{BASE_URL}/models")).build();
        self.get(&url).await
    }

    pub async fn get_model(&self, model: OpenAIModel) -> AiResult<OpenAIModelInfo> {
        // NOTE: Model serializes with the `models/` prefix.
        let url = Url::new(format!("{BASE_URL}/models/{model}")).build();
        self.get(&url).await
    }

    /// This method uses the Chat Completions API.
    /// The chat completions API is not recommended for new code.
    /// Please consider using the Responses API instead (generate_response).
    #[cfg(feature = "chat-completions")]
    pub async fn generate_content(
        &self,
        mut request: OpenAIGenerateContentRequest,
    ) -> AiResult<OpenAIGenerateContentResponse> {
        request.sanitise();
        let url = Url::new(format!("{BASE_URL}/chat/completions")).build();
        self.post(&url, request).await
    }

    /// This method uses the Chat Completions API.
    /// The chat completions API is not recommended for new code.
    /// Please consider using the Responses API instead (generate_response_streamed).
    #[cfg(all(feature = "chat-completions", feature = "stream"))]
    pub async fn generate_content_streamed(
        &self,
        mut request: OpenAIGenerateContentRequest,
    ) -> AiResult<impl Stream<Item = Result<OpenAIStreamChunk, StreamBodyError>>> {
        request.sanitise();

        let url = Url::new(format!("{BASE_URL}/chat/completions")).build();
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(AiError::Request)?;

        Ok(parse_sse_stream(response).await)
    }

    /// Generate a response using OpenAI's Responses API.
    pub async fn generate_response(
        &self,
        mut request: OpenAIResponsesCreateRequest,
    ) -> AiResult<OpenAIResponsesCreateResponse> {
        request.sanitise();
        let url = Url::new(format!("{BASE_URL}/responses")).build();
        self.post(&url, request).await
    }

    #[cfg(feature = "stream")]
    /// Generate a response using OpenAI's Responses API with streaming support.
    pub async fn generate_response_streamed(
        &self,
        mut request: OpenAIResponsesCreateRequest,
    ) -> AiResult<impl Stream<Item = Result<OpenAIResponsesStreamEvent, StreamBodyError>>> {
        request.sanitise();

        let url = Url::new(format!("{BASE_URL}/responses")).build();
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(AiError::Request)?;

        Ok(parse_sse_stream(response).await)
    }
}
