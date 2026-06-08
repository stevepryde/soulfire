use serde::Serialize;
use soulfire_core::model::ids::{AdventureId, WorldBlueprintId};
use soulfire_core::model::world::{Adventure, AdventureMessage, GmProposal, WorldBlueprint};
use tauri::State;

use crate::commands::common::{DeleteResult, normalize_list_limit, normalize_search};
use crate::error::CommandError;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldBlueprintListPage {
    pub items: Vec<WorldBlueprint>,
    pub next_cursor: Option<WorldBlueprintId>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdventureListPage {
    pub items: Vec<Adventure>,
    pub next_cursor: Option<AdventureId>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdventureDetail {
    pub adventure: Adventure,
    pub messages: Vec<AdventureMessage>,
    pub pending_proposals: Vec<GmProposal>,
}

#[tauri::command]
pub async fn list_world_blueprints(
    search: Option<String>,
    after_blueprint_id: Option<WorldBlueprintId>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<WorldBlueprintListPage, CommandError> {
    let search = normalize_search(search);
    let limit = normalize_list_limit(limit);
    state
        .with_store(move |store| {
            let after = match after_blueprint_id {
                Some(id) => Some(
                    store
                        .blueprint(&id)?
                        .ok_or_else(|| soulfire_core::CoreError::NotFound(id.to_string()))?,
                ),
                None => None,
            };
            let mut items = store.list_blueprints(search.as_deref(), after.as_ref(), limit + 1)?;
            let has_more = items.len() > limit as usize;
            if has_more {
                items.truncate(limit as usize);
            }
            let next_cursor = has_more
                .then(|| items.last().map(|item| item.blueprint_id.clone()))
                .flatten();
            Ok(WorldBlueprintListPage {
                items,
                next_cursor,
                has_more,
            })
        })
        .await
}

#[tauri::command]
pub async fn load_world_blueprint(
    blueprint_id: WorldBlueprintId,
    state: State<'_, AppState>,
) -> Result<WorldBlueprint, CommandError> {
    state
        .with_store(move |store| {
            store
                .blueprint(&blueprint_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(blueprint_id.to_string()))
        })
        .await
}

#[tauri::command]
pub async fn delete_world_blueprint(
    blueprint_id: WorldBlueprintId,
    state: State<'_, AppState>,
) -> Result<DeleteResult, CommandError> {
    state
        .with_store(move |store| {
            let deleted = store.blueprint(&blueprint_id)?.is_some();
            if deleted {
                store.delete_blueprint(&blueprint_id)?;
            }
            Ok(DeleteResult { deleted })
        })
        .await
}

#[tauri::command]
pub async fn list_adventures(
    after_adventure_id: Option<AdventureId>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<AdventureListPage, CommandError> {
    let limit = normalize_list_limit(limit);
    state
        .with_store(move |store| {
            let after = match after_adventure_id {
                Some(id) => Some(
                    store
                        .adventure(&id)?
                        .ok_or_else(|| soulfire_core::CoreError::NotFound(id.to_string()))?,
                ),
                None => None,
            };
            let mut items = store.list_adventures(after.as_ref(), limit + 1)?;
            let has_more = items.len() > limit as usize;
            if has_more {
                items.truncate(limit as usize);
            }
            let next_cursor = has_more
                .then(|| items.last().map(|item| item.adventure_id.clone()))
                .flatten();
            Ok(AdventureListPage {
                items,
                next_cursor,
                has_more,
            })
        })
        .await
}

#[tauri::command]
pub async fn list_in_progress_adventures(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<Adventure>, CommandError> {
    let limit = normalize_list_limit(limit);
    state
        .with_store(move |store| store.in_progress_adventures(limit))
        .await
}

#[tauri::command]
pub async fn load_adventure(
    adventure_id: AdventureId,
    state: State<'_, AppState>,
) -> Result<AdventureDetail, CommandError> {
    state
        .with_store(move |store| {
            let adventure = store
                .adventure(&adventure_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(adventure_id.to_string()))?;
            let messages = store.adventure_messages(&adventure_id)?;
            let pending_proposals = store.pending_gm_proposals(&adventure_id)?;
            Ok(AdventureDetail {
                adventure,
                messages,
                pending_proposals,
            })
        })
        .await
}

#[tauri::command]
pub async fn delete_adventure(
    adventure_id: AdventureId,
    state: State<'_, AppState>,
) -> Result<DeleteResult, CommandError> {
    state
        .with_store(move |store| {
            let deleted = store.adventure(&adventure_id)?.is_some();
            if deleted {
                store.delete_adventure(&adventure_id)?;
            }
            Ok(DeleteResult { deleted })
        })
        .await
}
