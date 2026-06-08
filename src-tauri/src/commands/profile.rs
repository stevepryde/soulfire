use soulfire_core::model::profile::{AppProfile, PlayerProfile};
use soulfire_core::store::Store;
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

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
