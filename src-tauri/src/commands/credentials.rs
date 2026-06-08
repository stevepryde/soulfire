use serde::Serialize;
use soulfire_core::model::ai_model::AiVendor;
use soulfire_core::model::credentials::ProviderCredential;
use tauri::State;

use crate::commands::common::parse_api_key;
use crate::error::CommandError;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub provider: AiVendor,
    pub configured: bool,
    pub masked: Option<String>,
}

fn credential_status_for(
    provider: AiVendor,
    credential: Option<&ProviderCredential>,
) -> CredentialStatus {
    CredentialStatus {
        provider,
        configured: credential.is_some(),
        masked: credential.map(ProviderCredential::masked),
    }
}

#[tauri::command]
pub async fn get_openai_credential_status(
    state: State<'_, AppState>,
) -> Result<CredentialStatus, CommandError> {
    state
        .with_store(|store| {
            let credential = store.credential(AiVendor::OpenAI)?;
            Ok(credential_status_for(AiVendor::OpenAI, credential.as_ref()))
        })
        .await
}

#[tauri::command]
pub async fn save_openai_credential(
    api_key: String,
    state: State<'_, AppState>,
) -> Result<CredentialStatus, CommandError> {
    let api_key = parse_api_key(api_key)?;
    state
        .with_store(move |store| {
            let credential = ProviderCredential::new(AiVendor::OpenAI, api_key);
            store.save_credential(&credential)?;
            Ok(credential_status_for(AiVendor::OpenAI, Some(&credential)))
        })
        .await
}

#[tauri::command]
pub async fn delete_openai_credential(
    state: State<'_, AppState>,
) -> Result<CredentialStatus, CommandError> {
    state
        .with_store(|store| {
            store.delete_credential(AiVendor::OpenAI)?;
            Ok(credential_status_for(AiVendor::OpenAI, None))
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_status_never_contains_raw_key() {
        let credential = ProviderCredential::new(AiVendor::OpenAI, "sk-secret-123456");
        let status = credential_status_for(AiVendor::OpenAI, Some(&credential));

        assert!(status.configured);
        assert_eq!(status.provider, AiVendor::OpenAI);
        let masked = status.masked.unwrap();
        assert!(masked.ends_with("3456"));
        assert!(!masked.contains("secret"));
    }

    #[test]
    fn credential_status_handles_missing_key() {
        let status = credential_status_for(AiVendor::OpenAI, None);

        assert!(!status.configured);
        assert_eq!(status.masked, None);
    }
}
