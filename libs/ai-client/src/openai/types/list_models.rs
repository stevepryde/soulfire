use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIModelsListResponse {
    pub models: Vec<OpenAIModelInfo>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModelInfo {
    pub id: String,
    pub object: String,
    pub owned_by: String,
    pub created: u64,
}
