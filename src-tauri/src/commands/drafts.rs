use std::str::FromStr;

use soulfire_core::model::draft::{Draft, DraftScope};
use soulfire_core::model::ids::{AdventureId, ChatId};
use soulfire_core::model::strings::DraftContent;
use tauri::State;

use crate::commands::common::DeleteResult;
use crate::error::CommandError;
use crate::state::AppState;

fn draft_content(content: String) -> Result<DraftContent, CommandError> {
    DraftContent::from_str(&content).map_err(|err| CommandError::InvalidInput(err.to_string()))
}

#[tauri::command]
pub async fn get_chat_draft(
    chat_id: ChatId,
    state: State<'_, AppState>,
) -> Result<Option<Draft>, CommandError> {
    state
        .with_store(move |store| {
            store.draft_for_scope(&DraftScope::Chat {
                chat_id: chat_id.clone(),
            })
        })
        .await
}

#[tauri::command]
pub async fn save_chat_draft(
    chat_id: ChatId,
    content: String,
    state: State<'_, AppState>,
) -> Result<Draft, CommandError> {
    let content = draft_content(content)?;
    state
        .with_store(move |store| {
            let draft = Draft::builder()
                .scope(DraftScope::Chat { chat_id })
                .content(content)
                .build();
            store.save_draft(&draft)?;
            Ok(draft)
        })
        .await
}

#[tauri::command]
pub async fn clear_chat_draft(
    chat_id: ChatId,
    state: State<'_, AppState>,
) -> Result<DeleteResult, CommandError> {
    state
        .with_store(move |store| {
            let scope = DraftScope::Chat { chat_id };
            let deleted = store.draft_for_scope(&scope)?.is_some();
            if deleted {
                store.delete_draft_for_scope(&scope)?;
            }
            Ok(DeleteResult { deleted })
        })
        .await
}

#[tauri::command]
pub async fn get_adventure_draft(
    adventure_id: AdventureId,
    state: State<'_, AppState>,
) -> Result<Option<Draft>, CommandError> {
    state
        .with_store(move |store| {
            store.draft_for_scope(&DraftScope::Adventure {
                adventure_id: adventure_id.clone(),
            })
        })
        .await
}

#[tauri::command]
pub async fn save_adventure_draft(
    adventure_id: AdventureId,
    content: String,
    state: State<'_, AppState>,
) -> Result<Draft, CommandError> {
    let content = draft_content(content)?;
    state
        .with_store(move |store| {
            let draft = Draft::builder()
                .scope(DraftScope::Adventure { adventure_id })
                .content(content)
                .build();
            store.save_draft(&draft)?;
            Ok(draft)
        })
        .await
}

#[tauri::command]
pub async fn clear_adventure_draft(
    adventure_id: AdventureId,
    state: State<'_, AppState>,
) -> Result<DeleteResult, CommandError> {
    state
        .with_store(move |store| {
            let scope = DraftScope::Adventure { adventure_id };
            let deleted = store.draft_for_scope(&scope)?.is_some();
            if deleted {
                store.delete_draft_for_scope(&scope)?;
            }
            Ok(DeleteResult { deleted })
        })
        .await
}
