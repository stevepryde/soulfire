use soulfire_core::model::settings::AppSettings;
use soulfire_core::store::Store;
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

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
