mod commands;
mod error;
mod state;

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
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Soulfire");
}
