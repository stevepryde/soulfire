use serde::Serialize;
use soulfire_core::chat::SendProgress;
use soulfire_core::model::chat::{Chat, ChatMessage};
use soulfire_core::model::ids::{CharacterId, ChatId};
use tauri::{AppHandle, Runtime, State};

use crate::commands::common::{DeleteResult, emit_error, emit_event, parse_message_text};
use crate::error::CommandError;
use crate::events::{BridgeEvent, TaskKind, TaskStatus, emit_bridge_event};
use crate::services;
use crate::state::AppState;

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

#[tauri::command]
pub async fn load_chat(
    chat_id: ChatId,
    state: State<'_, AppState>,
) -> Result<ChatThread, CommandError> {
    let store = state.store_handle()?;
    let chat_for_load = chat_id.clone();
    let chat = store
        .run(move |store| {
            store
                .chat(&chat_for_load)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(chat_for_load.to_string()))
        })
        .await?;
    let messages = store
        .run(move |store| store.chat_messages(&chat_id))
        .await?;

    Ok(ChatThread { chat, messages })
}

#[tauri::command]
pub async fn delete_chat(
    chat_id: ChatId,
    state: State<'_, AppState>,
) -> Result<DeleteResult, CommandError> {
    state
        .with_store(move |store| {
            let deleted = store.chat(&chat_id)?.is_some();
            if deleted {
                store.delete_chat(&chat_id)?;
            }
            Ok(DeleteResult { deleted })
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
