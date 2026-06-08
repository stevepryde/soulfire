use std::path::PathBuf;

use serde::Serialize;
use soulfire_core::model::ai_model::AiVendor;
use soulfire_core::model::credentials::ProviderCredential;
use soulfire_core::model::profile::{AppProfile, PlayerProfile};
use soulfire_core::model::settings::AppSettings;
use soulfire_core::store::{AsyncStore, Store};
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreStatus {
    pub initialized: bool,
    pub unlocked: bool,
    pub schema_version: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub provider: AiVendor,
    pub configured: bool,
    pub masked: Option<String>,
}

fn parse_data_dir(path: String) -> Result<PathBuf, CommandError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidInput(
            "data_dir is required".to_string(),
        ));
    }
    Ok(PathBuf::from(trimmed))
}

fn parse_master_password(password: String) -> Result<String, CommandError> {
    if password.is_empty() {
        return Err(CommandError::InvalidInput(
            "master_password is required".to_string(),
        ));
    }
    Ok(password)
}

fn parse_api_key(api_key: String) -> Result<String, CommandError> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidInput(
            "api_key is required".to_string(),
        ));
    }
    Ok(trimmed.to_string())
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

async fn status_for(path: &PathBuf, state: &AppState) -> Result<StoreStatus, CommandError> {
    Ok(StoreStatus {
        initialized: AsyncStore::is_initialized(path).await?,
        unlocked: state.is_unlocked(),
        schema_version: state.schema_version().await?,
    })
}

#[tauri::command]
pub async fn store_exists(data_dir: String) -> Result<bool, CommandError> {
    Ok(AsyncStore::is_initialized(parse_data_dir(data_dir)?).await?)
}

#[tauri::command]
pub async fn setup_store(
    data_dir: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<StoreStatus, CommandError> {
    let path = parse_data_dir(data_dir)?;
    let password = parse_master_password(master_password)?;
    let store = AsyncStore::initialize(&path, password).await?;
    state.set_store(store);
    status_for(&path, &state).await
}

#[tauri::command]
pub async fn unlock_store(
    data_dir: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<StoreStatus, CommandError> {
    let path = parse_data_dir(data_dir)?;
    let password = parse_master_password(master_password)?;
    let store = AsyncStore::unlock(&path, password).await?;
    state.set_store(store);
    status_for(&path, &state).await
}

#[tauri::command]
pub async fn lock_store(
    data_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<StoreStatus, CommandError> {
    state.clear_store();
    let initialized = match data_dir {
        Some(path) => AsyncStore::is_initialized(parse_data_dir(path)?).await?,
        None => false,
    };
    Ok(StoreStatus {
        initialized,
        unlocked: false,
        schema_version: None,
    })
}

#[tauri::command]
pub async fn store_status(
    data_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<StoreStatus, CommandError> {
    let initialized = match data_dir {
        Some(path) => AsyncStore::is_initialized(parse_data_dir(path)?).await?,
        None => false,
    };
    Ok(StoreStatus {
        initialized,
        unlocked: state.is_unlocked(),
        schema_version: state.schema_version().await?,
    })
}

#[tauri::command]
pub async fn get_app_profile(state: State<'_, AppState>) -> Result<AppProfile, CommandError> {
    state.with_store(Store::app_profile).await
}

#[tauri::command]
pub async fn save_app_profile(
    profile: AppProfile,
    state: State<'_, AppState>,
) -> Result<AppProfile, CommandError> {
    state
        .with_store(move |store| {
            store.save_app_profile(&profile)?;
            Ok(profile)
        })
        .await
}

#[tauri::command]
pub async fn get_player_profile(state: State<'_, AppState>) -> Result<PlayerProfile, CommandError> {
    state.with_store(Store::player_profile).await
}

#[tauri::command]
pub async fn save_player_profile(
    profile: PlayerProfile,
    state: State<'_, AppState>,
) -> Result<PlayerProfile, CommandError> {
    state
        .with_store(move |store| {
            store.save_player_profile(&profile)?;
            Ok(profile)
        })
        .await
}

#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    state.with_store(Store::app_settings).await
}

#[tauri::command]
pub async fn save_app_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<AppSettings, CommandError> {
    state
        .with_store(move |store| {
            store.save_app_settings(&settings)?;
            Ok(settings)
        })
        .await
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
