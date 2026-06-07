//! A deterministic, recording fake provider for tests (`TEST-5`).
//!
//! Returns scripted responses (text, structured JSON, streamed deltas, errors,
//! images, and a no-/partial-token stall) and records the exact requests sent so
//! tests can assert what was transmitted.

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};

use super::provider::{AiProvider, EventStream};
use super::types::{
    GenerationRequest, GenerationResponse, ImageRequest, ImageResponse, ProviderError, StreamEvent,
    Usage,
};

/// One scripted provider response.
#[derive(Debug, Clone)]
pub enum Scripted {
    /// A one-shot text/JSON response.
    Text { text: String, usage: Usage },
    /// A streamed response: yields each delta, then a terminal full text and
    /// usage. When `stall` is set the stream instead hangs after the deltas
    /// (drives the idle-timeout path, `AI-11`).
    Stream {
        deltas: Vec<String>,
        usage: Usage,
        stall: bool,
    },
    /// An error response.
    Error(ProviderError),
    /// A generated image.
    Image {
        bytes: Vec<u8>,
        mime: String,
        usage: Usage,
    },
}

impl Scripted {
    /// Convenience: a plain text reply with the given token usage.
    pub fn text(text: impl Into<String>, input: u64, output: u64) -> Self {
        Scripted::Text {
            text: text.into(),
            usage: Usage {
                input_tokens: input,
                output_tokens: output,
                cached_input_tokens: None,
            },
        }
    }

    /// Convenience: a streamed reply that yields `deltas` then finalizes.
    pub fn stream(deltas: Vec<&str>, input: u64, output: u64) -> Self {
        Scripted::Stream {
            deltas: deltas.into_iter().map(String::from).collect(),
            usage: Usage {
                input_tokens: input,
                output_tokens: output,
                cached_input_tokens: None,
            },
            stall: false,
        }
    }
}

/// A provider that returns scripted responses in order and records requests.
#[derive(Debug, Default)]
pub struct RecordingProvider {
    script: Mutex<VecDeque<Scripted>>,
    requests: Mutex<Vec<GenerationRequest>>,
    image_requests: Mutex<Vec<ImageRequest>>,
}

impl RecordingProvider {
    pub fn new() -> Self {
        RecordingProvider::default()
    }

    /// Queue a scripted response.
    pub fn push(&self, response: Scripted) {
        self.script.lock().unwrap().push_back(response);
    }

    /// All text/structured requests recorded, in order (`TEST-5`).
    pub fn requests(&self) -> Vec<GenerationRequest> {
        self.requests.lock().unwrap().clone()
    }

    /// The most recent text/structured request, if any.
    pub fn last_request(&self) -> Option<GenerationRequest> {
        self.requests.lock().unwrap().last().cloned()
    }

    pub fn request_count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }

    pub fn image_requests(&self) -> Vec<ImageRequest> {
        self.image_requests.lock().unwrap().clone()
    }

    fn next(&self) -> Result<Scripted, ProviderError> {
        self.script
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| ProviderError::Other("no scripted response".into()))
    }
}

#[async_trait]
impl AiProvider for RecordingProvider {
    async fn generate(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationResponse, ProviderError> {
        self.requests.lock().unwrap().push(request);
        match self.next()? {
            Scripted::Text { text, usage } => Ok(GenerationResponse { text, usage }),
            Scripted::Stream { deltas, usage, .. } => Ok(GenerationResponse {
                text: deltas.concat(),
                usage,
            }),
            Scripted::Error(e) => Err(e),
            Scripted::Image { .. } => {
                Err(ProviderError::Other("scripted image for a text call".into()))
            }
        }
    }

    async fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> Result<EventStream, ProviderError> {
        self.requests.lock().unwrap().push(request);
        match self.next()? {
            Scripted::Stream {
                deltas,
                usage,
                stall,
            } => {
                let mut events: Vec<StreamEvent> =
                    deltas.iter().cloned().map(StreamEvent::Delta).collect();
                if !stall {
                    events.push(StreamEvent::Full(deltas.concat()));
                    events.push(StreamEvent::Usage(usage));
                }
                let base = stream::iter(events);
                if stall {
                    // Yield the deltas, then hang so the idle timeout fires.
                    Ok(base.chain(stream::pending()).boxed())
                } else {
                    Ok(base.boxed())
                }
            }
            Scripted::Text { text, usage } => {
                let events = vec![
                    StreamEvent::Delta(text.clone()),
                    StreamEvent::Full(text),
                    StreamEvent::Usage(usage),
                ];
                Ok(stream::iter(events).boxed())
            }
            Scripted::Error(e) => Err(e),
            Scripted::Image { .. } => {
                Err(ProviderError::Other("scripted image for a stream call".into()))
            }
        }
    }

    async fn generate_image(
        &self,
        request: ImageRequest,
    ) -> Result<ImageResponse, ProviderError> {
        self.image_requests.lock().unwrap().push(request);
        match self.next()? {
            Scripted::Image { bytes, mime, usage } => Ok(ImageResponse { bytes, mime, usage }),
            Scripted::Error(e) => Err(e),
            _ => Err(ProviderError::Other("no scripted image response".into())),
        }
    }
}
