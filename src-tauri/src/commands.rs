use std::path::PathBuf;

use serde::Serialize;
use soulfire_core::chat::SendProgress;
use soulfire_core::model::ai_model::AiVendor;
use soulfire_core::model::chat::{Chat, ChatMessage};
use soulfire_core::model::credentials::ProviderCredential;
use soulfire_core::model::ids::{CharacterId, ChatId};
use soulfire_core::model::profile::{AppProfile, PlayerProfile};
use soulfire_core::model::settings::AppSettings;
use soulfire_core::store::{AsyncStore, Store};
use tauri::{AppHandle, Runtime, State};

use crate::error::CommandError;
use crate::events::{BridgeEvent, TaskKind, TaskStatus, emit_bridge_event};
use crate::services;
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

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatThread {
    pub chat: Chat,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSendResult {
    pub chat: Chat,
    pub player_message: ChatMessage,
    pub reply: ChatMessage,
    pub summary_due: bool,
    pub state_update_due: bool,
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

fn parse_message_text(text: String) -> Result<String, CommandError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidInput(
            "message is required".to_string(),
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

fn emit_event<R: Runtime>(app: &AppHandle<R>, event: BridgeEvent) -> Result<(), CommandError> {
    emit_bridge_event(app, event)
        .map_err(|err| CommandError::Core(format!("failed to emit bridge event: {err}")))
}

fn emit_error<R: Runtime>(
    app: &AppHandle<R>,
    task: TaskKind,
    entity_id: Option<String>,
    message: String,
) {
    let _ = emit_bridge_event(
        app,
        BridgeEvent::TaskStatus {
            task,
            status: TaskStatus::Failed,
            entity_id: entity_id.clone(),
        },
    );
    let _ = emit_bridge_event(
        app,
        BridgeEvent::Error {
            task: Some(task),
            entity_id,
            message,
        },
    );
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

#[tauri::command]
pub async fn open_character_chat(
    character_id: CharacterId,
    state: State<'_, AppState>,
) -> Result<ChatThread, CommandError> {
    let store = state.store_handle()?;
    let engine = services::chat_engine(&store);
    let chat = engine.open_chat(&character_id).await?;
    let chat_id = chat.chat_id.clone();
    let messages = store
        .run(move |store| store.chat_messages(&chat_id))
        .await?;

    Ok(ChatThread { chat, messages })
}

#[tauri::command]
pub async fn send_chat_message<R: Runtime>(
    chat_id: ChatId,
    message: String,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<ChatSendResult, CommandError> {
    let message = parse_message_text(message)?;
    let entity_id = Some(chat_id.to_string());
    let store = state.store_handle()?;
    let engine = services::chat_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::ChatReply,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;
    emit_event(
        &app,
        BridgeEvent::ChatMessageAiStart {
            chat_id: chat_id.clone(),
        },
    )?;

    let chunk_app = app.clone();
    let chunk_chat_id = chat_id.clone();
    let progress_app = app.clone();
    let outcome = match engine
        .send_message_observed(
            &chat_id,
            &message,
            move |delta| {
                let _ = emit_bridge_event(
                    &chunk_app,
                    BridgeEvent::ChatMessageChunk {
                        chat_id: chunk_chat_id.clone(),
                        chunk: delta.to_string(),
                    },
                );
            },
            move |progress| match progress {
                SendProgress::PlayerMessage(message) => {
                    let _ = emit_bridge_event(
                        &progress_app,
                        BridgeEvent::ChatMessageCreated { message },
                    );
                }
            },
        )
        .await
    {
        Ok(outcome) => outcome,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, TaskKind::ChatReply, entity_id, message.clone());
            return Err(err.into());
        }
    };

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::ChatReply,
            status: TaskStatus::Persisting,
            entity_id: entity_id.clone(),
        },
    )?;
    emit_event(
        &app,
        BridgeEvent::ChatMessageComplete {
            message: outcome.reply.clone(),
        },
    )?;

    let player_message_id = outcome.player_message.message_id.clone();
    let maybe_reacted_player = store
        .run(move |store| store.chat_message(&player_message_id))
        .await?;
    if let Some(message) = maybe_reacted_player {
        if message.emoji_reactions != outcome.player_message.emoji_reactions {
            emit_event(
                &app,
                BridgeEvent::ChatMessageReactions {
                    chat_id: chat_id.clone(),
                    message_id: message.message_id.clone(),
                    message,
                },
            )?;
        }
    }

    let chat_for_load = chat_id.clone();
    let chat = store
        .run(move |store| {
            store
                .chat(&chat_for_load)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(chat_for_load.to_string()))
        })
        .await?;

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::ChatReply,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(ChatSendResult {
        chat,
        player_message: outcome.player_message,
        reply: outcome.reply,
        summary_due: outcome.summary_due,
        state_update_due: outcome.state_update_due,
    })
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
