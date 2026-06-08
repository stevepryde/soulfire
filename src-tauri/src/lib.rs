mod commands;
mod error;
mod events;
mod state;

pub use events::{BRIDGE_EVENT, BridgeEvent, TaskKind, TaskStatus, emit_bridge_event};
pub use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::store_exists,
            commands::setup_store,
            commands::unlock_store,
            commands::lock_store,
            commands::store_status,
            commands::get_app_profile,
            commands::save_app_profile,
            commands::get_player_profile,
            commands::save_player_profile,
            commands::get_app_settings,
            commands::save_app_settings,
            commands::get_openai_credential_status,
            commands::save_openai_credential,
            commands::delete_openai_credential,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Soulfire");
}
