use serde::Serialize;
use soulfire_core::character::BuilderResult;
use soulfire_core::model::character::{Character, CharacterBuilderSession};
use soulfire_core::model::chat::Chat;
use soulfire_core::model::ids::{AdventureId, CharacterId, WorldBlueprintId};
use soulfire_core::model::world::{WorldBlueprint, WorldBuilderSession};
use soulfire_core::store::AsyncStore;
use soulfire_core::world::WorldBuilderResult;
use tauri::{AppHandle, Runtime, State};

use crate::commands::common::{emit_error, emit_event, parse_message_text};
use crate::error::CommandError;
use crate::events::{BridgeEvent, TaskKind, TaskStatus};
use crate::services;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBuilderState {
    pub character: Character,
    pub session: CharacterBuilderSession,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBuilderTurnResult {
    pub result: BuilderResult,
    pub character: Character,
    pub session: CharacterBuilderSession,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBuilderUndoResult {
    pub undone: bool,
    pub character: Character,
    pub session: CharacterBuilderSession,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NpcExtractionResult {
    pub character: Character,
    pub chat: Chat,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldBuilderState {
    pub blueprint: WorldBlueprint,
    pub session: WorldBuilderSession,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldBuilderTurnResult {
    pub result: WorldBuilderResult,
    pub blueprint: WorldBlueprint,
    pub session: WorldBuilderSession,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldBuilderUndoResult {
    pub undone: bool,
    pub blueprint: WorldBlueprint,
    pub session: WorldBuilderSession,
}

async fn character_builder_state(
    store: &AsyncStore,
    character_id: CharacterId,
) -> Result<CharacterBuilderState, CommandError> {
    Ok(store
        .run(move |store| {
            let character = store
                .character(&character_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(character_id.to_string()))?;
            let session = store
                .character_builder_session(&character_id)?
                .unwrap_or_else(|| CharacterBuilderSession {
                    character_id,
                    ..Default::default()
                });
            Ok(CharacterBuilderState { character, session })
        })
        .await?)
}

async fn world_builder_state(
    store: &AsyncStore,
    blueprint_id: WorldBlueprintId,
) -> Result<WorldBuilderState, CommandError> {
    Ok(store
        .run(move |store| {
            let blueprint = store
                .blueprint(&blueprint_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(blueprint_id.to_string()))?;
            let session = store
                .world_builder_session(&blueprint_id)?
                .unwrap_or_else(|| WorldBuilderSession {
                    blueprint_id,
                    ..Default::default()
                });
            Ok(WorldBuilderState { blueprint, session })
        })
        .await?)
}

#[tauri::command]
pub async fn get_character_builder_state(
    character_id: CharacterId,
    state: State<'_, AppState>,
) -> Result<CharacterBuilderState, CommandError> {
    let store = state.store_handle()?;
    character_builder_state(&store, character_id).await
}

#[tauri::command]
pub async fn send_character_builder_message<R: Runtime>(
    character_id: CharacterId,
    message: String,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<CharacterBuilderTurnResult, CommandError> {
    let message = parse_message_text(message)?;
    let entity_id = Some(character_id.to_string());
    let store = state.store_handle()?;
    let engine = services::character_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::CharacterBuilder,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;
    let result = match engine.builder_send(&character_id, &message).await {
        Ok(result) => result,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, TaskKind::CharacterBuilder, entity_id, message.clone());
            return Err(err.into());
        }
    };
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::CharacterBuilder,
            status: TaskStatus::Persisting,
            entity_id: entity_id.clone(),
        },
    )?;
    let state = character_builder_state(&store, character_id).await?;
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::CharacterBuilder,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(CharacterBuilderTurnResult {
        result,
        character: state.character,
        session: state.session,
    })
}

#[tauri::command]
pub async fn undo_character_builder(
    character_id: CharacterId,
    state: State<'_, AppState>,
) -> Result<CharacterBuilderUndoResult, CommandError> {
    let store = state.store_handle()?;
    let engine = services::character_engine(&store);
    let undone = engine.builder_undo(&character_id)?;
    let state = character_builder_state(&store, character_id).await?;
    Ok(CharacterBuilderUndoResult {
        undone,
        character: state.character,
        session: state.session,
    })
}

#[tauri::command]
pub async fn extract_npc<R: Runtime>(
    adventure_id: AdventureId,
    npc_name: String,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<NpcExtractionResult, CommandError> {
    let npc_name = parse_message_text(npc_name)?;
    let entity_id = Some(adventure_id.to_string());
    let store = state.store_handle()?;
    let engine = services::character_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::NpcExtraction,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;
    let character = match engine.extract_npc(&adventure_id, &npc_name).await {
        Ok(character) => character,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, TaskKind::NpcExtraction, entity_id, message.clone());
            return Err(err.into());
        }
    };
    let character_id = character.character_id.clone();
    let chat = store
        .run(move |store| {
            let chat_id = store
                .chat_id_for_character(&character_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(character_id.to_string()))?;
            store
                .chat(&chat_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(chat_id.to_string()))
        })
        .await?;
    emit_event(
        &app,
        BridgeEvent::CharacterReady {
            character: character.clone(),
            chat: chat.clone(),
        },
    )?;
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::NpcExtraction,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(NpcExtractionResult { character, chat })
}

#[tauri::command]
pub async fn get_world_builder_state(
    blueprint_id: WorldBlueprintId,
    state: State<'_, AppState>,
) -> Result<WorldBuilderState, CommandError> {
    let store = state.store_handle()?;
    world_builder_state(&store, blueprint_id).await
}

#[tauri::command]
pub async fn send_world_builder_message<R: Runtime>(
    blueprint_id: WorldBlueprintId,
    message: String,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<WorldBuilderTurnResult, CommandError> {
    let message = parse_message_text(message)?;
    let entity_id = Some(blueprint_id.to_string());
    let store = state.store_handle()?;
    let engine = services::world_builder_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::WorldBuilder,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;
    let result = match engine.builder_send(&blueprint_id, &message).await {
        Ok(result) => result,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, TaskKind::WorldBuilder, entity_id, message.clone());
            return Err(err.into());
        }
    };
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::WorldBuilder,
            status: TaskStatus::Persisting,
            entity_id: entity_id.clone(),
        },
    )?;
    let state = world_builder_state(&store, blueprint_id).await?;
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::WorldBuilder,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(WorldBuilderTurnResult {
        result,
        blueprint: state.blueprint,
        session: state.session,
    })
}

#[tauri::command]
pub async fn undo_world_builder(
    blueprint_id: WorldBlueprintId,
    state: State<'_, AppState>,
) -> Result<WorldBuilderUndoResult, CommandError> {
    let store = state.store_handle()?;
    let engine = services::world_builder_engine(&store);
    let undone = engine.builder_undo(&blueprint_id)?;
    let state = world_builder_state(&store, blueprint_id).await?;
    Ok(WorldBuilderUndoResult {
        undone,
        blueprint: state.blueprint,
        session: state.session,
    })
}
