use reqwest::StatusCode;

pub type AiResult<T> = std::result::Result<T, AiError>;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("invalid client: {0}")]
    InvalidClient(String),
    #[error("invalid model")]
    InvalidModel,
    #[error("missing api key")]
    MissingApiKey,
    #[error("invalid api key")]
    InvalidApiKey,
    #[error("request error: {0}")]
    Request(#[source] reqwest::Error),
    #[error("response error: {0}")]
    Response(#[source] reqwest::Error),
    #[error("External API error: [{0}] {1}")]
    ApiError(StatusCode, String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
