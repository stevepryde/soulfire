use soulfire_core::store::AsyncStore;
use tauri::State;

use crate::commands::common::{StoreStatus, parse_data_dir, parse_master_password, status_for};
use crate::error::CommandError;
use crate::state::AppState;

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
