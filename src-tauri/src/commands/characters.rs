use serde::Serialize;
use soulfire_core::model::character::Character;
use soulfire_core::model::ids::CharacterId;
use tauri::State;

use crate::commands::common::{DeleteResult, normalize_list_limit, normalize_search};
use crate::error::CommandError;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterListPage {
    pub items: Vec<Character>,
    pub next_cursor: Option<CharacterId>,
    pub has_more: bool,
}

#[tauri::command]
pub async fn save_character(
    mut character: Character,
    state: State<'_, AppState>,
) -> Result<Character, CommandError> {
    character.clamp_creativity();
    state
        .with_store(move |store| {
            store.save_character(&character)?;
            Ok(character)
        })
        .await
}

#[tauri::command]
pub async fn list_characters(
    search: Option<String>,
    after_character_id: Option<CharacterId>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<CharacterListPage, CommandError> {
    let search = normalize_search(search);
    let limit = normalize_list_limit(limit);
    state
        .with_store(move |store| {
            let after = match after_character_id {
                Some(id) => Some(
                    store
                        .character(&id)?
                        .ok_or_else(|| soulfire_core::CoreError::NotFound(id.to_string()))?,
                ),
                None => None,
            };
            let mut items = store.list_characters(search.as_deref(), after.as_ref(), limit + 1)?;
            let has_more = items.len() > limit as usize;
            if has_more {
                items.truncate(limit as usize);
            }
            let next_cursor = has_more
                .then(|| items.last().map(|item| item.character_id.clone()))
                .flatten();
            Ok(CharacterListPage {
                items,
                next_cursor,
                has_more,
            })
        })
        .await
}

#[tauri::command]
pub async fn load_character(
    character_id: CharacterId,
    state: State<'_, AppState>,
) -> Result<Character, CommandError> {
    state
        .with_store(move |store| {
            store
                .character(&character_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(character_id.to_string()))
        })
        .await
}

#[tauri::command]
pub async fn delete_character(
    character_id: CharacterId,
    state: State<'_, AppState>,
) -> Result<DeleteResult, CommandError> {
    state
        .with_store(move |store| {
            let deleted = store.character(&character_id)?.is_some();
            if deleted {
                store.delete_character(&character_id)?;
            }
            Ok(DeleteResult { deleted })
        })
        .await
}
