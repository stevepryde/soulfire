mod commands;
mod error;
mod events;
mod services;
mod state;

pub use events::{BRIDGE_EVENT, BridgeEvent, TaskKind, TaskStatus, emit_bridge_event};
pub use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::lifecycle::store_exists,
            commands::lifecycle::setup_store,
            commands::lifecycle::unlock_store,
            commands::lifecycle::lock_store,
            commands::lifecycle::store_status,
            commands::profile::get_app_profile,
            commands::profile::save_app_profile,
            commands::profile::get_player_profile,
            commands::profile::save_player_profile,
            commands::settings::get_app_settings,
            commands::settings::save_app_settings,
            commands::credentials::get_openai_credential_status,
            commands::credentials::save_openai_credential,
            commands::credentials::delete_openai_credential,
            commands::chat::open_character_chat,
            commands::chat::send_chat_message,
            commands::adventure::start_adventure,
            commands::adventure::take_adventure_turn,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Soulfire");
}
