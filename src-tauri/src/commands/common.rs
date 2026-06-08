use std::path::PathBuf;

use serde::Serialize;
use soulfire_core::store::AsyncStore;
use tauri::{AppHandle, Runtime};

use crate::error::CommandError;
use crate::events::{BridgeEvent, TaskKind, TaskStatus, emit_bridge_event};
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreStatus {
    pub initialized: bool,
    pub unlocked: bool,
    pub schema_version: Option<u32>,
}

pub(super) fn parse_data_dir(path: String) -> Result<PathBuf, CommandError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidInput(
            "data_dir is required".to_string(),
        ));
    }
    Ok(PathBuf::from(trimmed))
}

pub(super) fn parse_master_password(password: String) -> Result<String, CommandError> {
    if password.is_empty() {
        return Err(CommandError::InvalidInput(
            "master_password is required".to_string(),
        ));
    }
    Ok(password)
}

pub(super) fn parse_api_key(api_key: String) -> Result<String, CommandError> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidInput(
            "api_key is required".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

pub(super) fn parse_message_text(text: String) -> Result<String, CommandError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidInput(
            "message is required".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

pub(super) fn emit_event<R: Runtime>(
    app: &AppHandle<R>,
    event: BridgeEvent,
) -> Result<(), CommandError> {
    emit_bridge_event(app, event)
        .map_err(|err| CommandError::Core(format!("failed to emit bridge event: {err}")))
}

pub(super) fn emit_error<R: Runtime>(
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

pub(super) async fn status_for(
    path: &PathBuf,
    state: &AppState,
) -> Result<StoreStatus, CommandError> {
    Ok(StoreStatus {
        initialized: AsyncStore::is_initialized(path).await?,
        unlocked: state.is_unlocked(),
        schema_version: state.schema_version().await?,
    })
}
