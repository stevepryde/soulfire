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
            commands::images::get_character_portrait_bytes,
            commands::images::get_world_cover_bytes,
            commands::images::generate_character_portrait,
            commands::images::generate_world_cover,
            commands::images::set_character_portrait_bytes,
            commands::images::set_world_cover_bytes,
            commands::images::clear_character_portrait,
            commands::images::clear_world_cover,
            commands::prompts::get_character_prompt_view,
            commands::prompts::save_character_prompt_section,
            commands::stats::get_token_stats,
            commands::stats::get_chat_token_stats,
            commands::stats::get_adventure_token_stats,
            commands::stats::clear_token_stats,
            commands::characters::save_character,
            commands::characters::list_characters,
            commands::characters::load_character,
            commands::characters::delete_character,
            commands::worlds::save_world_blueprint,
            commands::worlds::list_world_blueprints,
            commands::worlds::load_world_blueprint,
            commands::worlds::delete_world_blueprint,
            commands::worlds::list_adventures,
            commands::worlds::list_in_progress_adventures,
            commands::worlds::load_adventure,
            commands::worlds::delete_adventure,
            commands::chat::load_chat,
            commands::chat::delete_chat,
            commands::chat::open_character_chat,
            commands::chat::send_chat_message,
            commands::adventure::start_adventure,
            commands::adventure::take_adventure_turn,
            commands::adventure::accept_gm_proposal,
            commands::adventure::reject_gm_proposal,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Soulfire");
}
