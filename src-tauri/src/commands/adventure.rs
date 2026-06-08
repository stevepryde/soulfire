use serde::Serialize;
use soulfire_core::model::ids::{AdventureId, GmProposalId, WorldBlueprintId};
use soulfire_core::model::world::{Adventure, AdventureMessage, GmProposal};
use soulfire_core::store::AsyncStore;
use soulfire_core::world::{TurnOutcome, TurnProgress};
use tauri::{AppHandle, Runtime, State};

use crate::commands::common::{emit_error, emit_event, parse_message_text};
use crate::error::CommandError;
use crate::events::{BridgeEvent, TaskKind, TaskStatus, emit_bridge_event};
use crate::services;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdventureThread {
    pub adventure: Adventure,
    pub messages: Vec<AdventureMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdventureTurnResult {
    pub adventure: Adventure,
    pub messages: Vec<AdventureMessage>,
    pub warning: Option<String>,
    pub proposal: Option<GmProposal>,
    pub state_update_failed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmProposalDecisionResult {
    pub adventure: Adventure,
    pub proposal: GmProposal,
    pub pending_proposals: Vec<GmProposal>,
}

async fn gm_proposal_decision_result(
    store: &AsyncStore,
    proposal_id: GmProposalId,
) -> Result<GmProposalDecisionResult, CommandError> {
    Ok(store
        .run(move |store| {
            let proposal = store
                .gm_proposal(&proposal_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(proposal_id.to_string()))?;
            let adventure = store.adventure(&proposal.adventure_id)?.ok_or_else(|| {
                soulfire_core::CoreError::NotFound(proposal.adventure_id.to_string())
            })?;
            let pending_proposals = store.pending_gm_proposals(&proposal.adventure_id)?;
            Ok(GmProposalDecisionResult {
                adventure,
                proposal,
                pending_proposals,
            })
        })
        .await?)
}

#[tauri::command]
pub async fn start_adventure<R: Runtime>(
    blueprint_id: WorldBlueprintId,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<AdventureThread, CommandError> {
    let entity_id = Some(blueprint_id.to_string());
    let store = state.store_handle()?;
    let engine = services::world_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::AdventureStart,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;

    let blueprint_for_load = blueprint_id.clone();
    let blueprint = store
        .run(move |store| {
            store
                .blueprint(&blueprint_for_load)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(blueprint_for_load.to_string()))
        })
        .await?;

    let adventure = match engine.start_adventure(&blueprint, |_| {}).await {
        Ok(adventure) => adventure,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, TaskKind::AdventureStart, entity_id, message.clone());
            return Err(err.into());
        }
    };

    let adventure_id = adventure.adventure_id.clone();
    let messages = store
        .run(move |store| store.adventure_messages(&adventure_id))
        .await?;

    if let Some(narration_message) = messages.last().cloned() {
        emit_event(
            &app,
            BridgeEvent::AdventureNarrationComplete {
                adventure: adventure.clone(),
                narration_message,
            },
        )?;
    }
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task: TaskKind::AdventureStart,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(AdventureThread {
        adventure,
        messages,
    })
}

#[tauri::command]
pub async fn take_adventure_turn<R: Runtime>(
    adventure_id: AdventureId,
    input: String,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<AdventureTurnResult, CommandError> {
    let input = parse_message_text(input)?;
    let task = if input.starts_with("/gm") {
        TaskKind::AdventureCommand
    } else {
        TaskKind::AdventureTurn
    };
    let entity_id = Some(adventure_id.to_string());
    let store = state.store_handle()?;
    let engine = services::world_engine(&store);

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task,
            status: TaskStatus::Started,
            entity_id: entity_id.clone(),
        },
    )?;

    let chunk_app = app.clone();
    let chunk_adventure_id = adventure_id.clone();
    let progress_app = app.clone();
    let outcome = match engine
        .take_turn_observed(
            &adventure_id,
            &input,
            move |delta| {
                let _ = emit_bridge_event(
                    &chunk_app,
                    BridgeEvent::AdventureNarrationChunk {
                        adventure_id: chunk_adventure_id.clone(),
                        chunk: delta.to_string(),
                    },
                );
            },
            move |progress| match progress {
                TurnProgress::UserAction(message) => {
                    let _ = emit_bridge_event(
                        &progress_app,
                        BridgeEvent::AdventureUserActionEcho { message },
                    );
                }
                TurnProgress::GmRequest(message) => {
                    let _ = emit_bridge_event(
                        &progress_app,
                        BridgeEvent::AdventureCommandEcho { message },
                    );
                }
            },
        )
        .await
    {
        Ok(outcome) => outcome,
        Err(err) => {
            let message = err.to_string();
            emit_error(&app, task, entity_id, message.clone());
            return Err(err.into());
        }
    };

    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task,
            status: TaskStatus::Persisting,
            entity_id: entity_id.clone(),
        },
    )?;

    let adventure_for_load = adventure_id.clone();
    let adventure = store
        .run(move |store| {
            store
                .adventure(&adventure_for_load)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(adventure_for_load.to_string()))
        })
        .await?;
    let messages_adventure_id = adventure_id.clone();
    let messages = store
        .run(move |store| store.adventure_messages(&messages_adventure_id))
        .await?;

    let mut warning = None;
    let mut proposal = None;
    let mut state_update_failed = false;

    match outcome {
        TurnOutcome::Narration {
            message,
            state_update_failed: failed,
            ..
        } => {
            state_update_failed = failed;
            emit_event(
                &app,
                BridgeEvent::AdventureNarrationComplete {
                    adventure: adventure.clone(),
                    narration_message: message,
                },
            )?;
        }
        TurnOutcome::GmAnswer { message } => {
            emit_event(
                &app,
                BridgeEvent::AdventureCommandComplete {
                    adventure: adventure.clone(),
                    response_message: message,
                },
            )?;
        }
        TurnOutcome::GmProposal {
            message,
            proposal: staged,
        } => {
            emit_event(
                &app,
                BridgeEvent::AdventureCommandComplete {
                    adventure: adventure.clone(),
                    response_message: message,
                },
            )?;
            emit_event(
                &app,
                BridgeEvent::GmProposalReady {
                    proposal: staged.clone(),
                },
            )?;
            proposal = Some(staged);
        }
        TurnOutcome::Warning(message) => {
            warning = Some(message);
        }
    }

    emit_event(
        &app,
        BridgeEvent::AdventureReadyStatus {
            adventure_id: adventure_id.clone(),
            status: adventure.ready_status,
        },
    )?;
    emit_event(
        &app,
        BridgeEvent::TaskStatus {
            task,
            status: TaskStatus::Complete,
            entity_id,
        },
    )?;

    Ok(AdventureTurnResult {
        adventure,
        messages,
        warning,
        proposal,
        state_update_failed,
    })
}

#[tauri::command]
pub async fn accept_gm_proposal(
    proposal_id: GmProposalId,
    state: State<'_, AppState>,
) -> Result<GmProposalDecisionResult, CommandError> {
    let store = state.store_handle()?;
    let engine = services::world_engine(&store);
    engine.accept_proposal(&proposal_id).await?;
    gm_proposal_decision_result(&store, proposal_id).await
}

#[tauri::command]
pub async fn reject_gm_proposal(
    proposal_id: GmProposalId,
    state: State<'_, AppState>,
) -> Result<GmProposalDecisionResult, CommandError> {
    let store = state.store_handle()?;
    let engine = services::world_engine(&store);
    engine.reject_proposal(&proposal_id).await?;
    gm_proposal_decision_result(&store, proposal_id).await
}
