use std::path::PathBuf;

use serde::Serialize;
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
