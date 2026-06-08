use serde::Serialize;
use soulfire_core::model::ids::{CharacterId, WorldBlueprintId};
use soulfire_core::model::images::StoredImageRef;
use soulfire_core::store::ImageOwnerKind;
use tauri::{AppHandle, Runtime, State};

use crate::commands::common::{emit_error, emit_event};
use crate::error::CommandError;
use crate::events::{BridgeEvent, TaskKind, TaskStatus};
use crate::services;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredImageBytes {
    pub mime: String,
    pub version: u32,
    pub bytes: Vec<u8>,
}

fn parse_image_upload(mime: String, bytes: Vec<u8>) -> Result<(String, Vec<u8>), CommandError> {
    let mime = mime.trim().to_string();
    if mime.is_empty() {
        return Err(CommandError::InvalidInput("mime is required".to_string()));
    }
    if bytes.is_empty() {
        return Err(CommandError::InvalidInput(
            "image bytes are required".to_string(),
        ));
    }
    Ok((mime, bytes))
}

#[tauri::command]
pub async fn get_character_portrait_bytes(
    character_id: CharacterId,
    state: State<'_, AppState>,
) -> Result<Option<StoredImageBytes>, CommandError> {
    let store = state.store_handle()?;
    let owner_id = character_id.to_string();
    let image = store
        .run(move |store| store.image(ImageOwnerKind::Character, &owner_id))
        .await?;
    Ok(image.map(|image| StoredImageBytes {
        mime: image.mime,
        version: image.version,
        bytes: image.bytes,
    }))
}

#[tauri::command]
pub async fn get_world_cover_bytes(
    blueprint_id: WorldBlueprintId,
    state: State<'_, AppState>,
) -> Result<Option<StoredImageBytes>, CommandError> {
    let store = state.store_handle()?;
    let owner_id = blueprint_id.to_string();
    let image = store
        .run(move |store| store.image(ImageOwnerKind::World, &owner_id))
        .await?;
    Ok(image.map(|image| StoredImageBytes {
        mime: image.mime,
        version: image.version,
        bytes: image.bytes,
    }))
}

#[tauri::command]
pub async fn generate_character_portrait<R: Runtime>(
    character_id: CharacterId,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<StoredImageRef, CommandError> {
    let entity_id = Some(character_id.to_string());
    let store = state.store_handle()?;
    let engine = services::image_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::ImageGeneration,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;

    let portrait = match engine.generate_character_portrait(&character_id).await {
        Ok(portrait) => portrait,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, TaskKind::ImageGeneration, entity_id, message.clone());
            return Err(err.into());
        }
    };

    emit_event(
        &app,
        BridgeEvent::CharacterImageReady {
            character_id,
            portrait: Some(portrait.clone()),
        },
    )?;
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::ImageGeneration,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(portrait)
}

#[tauri::command]
pub async fn generate_world_cover<R: Runtime>(
    blueprint_id: WorldBlueprintId,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<StoredImageRef, CommandError> {
    let entity_id = Some(blueprint_id.to_string());
    let store = state.store_handle()?;
    let engine = services::image_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::ImageGeneration,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;

    let cover = match engine.generate_world_cover(&blueprint_id).await {
        Ok(cover) => cover,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, TaskKind::ImageGeneration, entity_id, message.clone());
            return Err(err.into());
        }
    };

    emit_event(
        &app,
        BridgeEvent::WorldImageReady {
            blueprint_id,
            cover: Some(cover.clone()),
        },
    )?;
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::ImageGeneration,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(cover)
}

#[tauri::command]
pub async fn set_character_portrait_bytes<R: Runtime>(
    character_id: CharacterId,
    mime: String,
    bytes: Vec<u8>,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<StoredImageRef, CommandError> {
    let (mime, bytes) = parse_image_upload(mime, bytes)?;
    let store = state.store_handle()?;
    let engine = services::image_engine(&store);
    let portrait = engine.set_character_portrait_bytes(&character_id, &mime, &bytes)?;
    emit_event(
        &app,
        BridgeEvent::CharacterImageReady {
            character_id,
            portrait: Some(portrait.clone()),
        },
    )?;
    Ok(portrait)
}

#[tauri::command]
pub async fn set_world_cover_bytes<R: Runtime>(
    blueprint_id: WorldBlueprintId,
    mime: String,
    bytes: Vec<u8>,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<StoredImageRef, CommandError> {
    let (mime, bytes) = parse_image_upload(mime, bytes)?;
    let store = state.store_handle()?;
    let engine = services::image_engine(&store);
    let cover = engine.set_world_cover_bytes(&blueprint_id, &mime, &bytes)?;
    emit_event(
        &app,
        BridgeEvent::WorldImageReady {
            blueprint_id,
            cover: Some(cover.clone()),
        },
    )?;
    Ok(cover)
}

#[tauri::command]
pub async fn clear_character_portrait<R: Runtime>(
    character_id: CharacterId,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let store = state.store_handle()?;
    let engine = services::image_engine(&store);
    engine.clear_character_portrait(&character_id)?;
    emit_event(
        &app,
        BridgeEvent::CharacterImageReady {
            character_id,
            portrait: None,
        },
    )?;
    Ok(())
}

#[tauri::command]
pub async fn clear_world_cover<R: Runtime>(
    blueprint_id: WorldBlueprintId,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let store = state.store_handle()?;
    let engine = services::image_engine(&store);
    engine.clear_world_cover(&blueprint_id)?;
    emit_event(
        &app,
        BridgeEvent::WorldImageReady {
            blueprint_id,
            cover: None,
        },
    )?;
    Ok(())
}
