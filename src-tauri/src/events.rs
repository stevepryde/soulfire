use serde::{Deserialize, Serialize};
use soulfire_core::model::character::Character;
use soulfire_core::model::chat::{Chat, ChatMessage};
use soulfire_core::model::ids::{
    AdventureId, AdventureMessageId, CharacterId, ChatId, MessageId, WorldBlueprintId,
};
use soulfire_core::model::images::StoredImageRef;
use soulfire_core::model::world::{Adventure, AdventureMessage, AdventureReadyStatus, GmProposal};
use tauri::{AppHandle, Emitter, Runtime};

pub const BRIDGE_EVENT: &str = "soulfire://event";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    ChatReply,
    ChatSummary,
    CharacterStateUpdate,
    CharacterBuilder,
    NpcExtraction,
    AdventureStart,
    AdventureTurn,
    AdventureCommand,
    WorldBuilder,
    ImageGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Started,
    Streaming,
    Persisting,
    Complete,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum BridgeEvent {
    TaskStatus {
        task: TaskKind,
        status: TaskStatus,
        entity_id: Option<String>,
    },
    ChatMessageAiStart {
        chat_id: ChatId,
    },
    ChatMessageChunk {
        chat_id: ChatId,
        message_id: MessageId,
        chunk: String,
    },
    ChatMessageComplete {
        message: ChatMessage,
    },
    ChatMessageReactions {
        chat_id: ChatId,
        message_id: MessageId,
        message: ChatMessage,
    },
    AdventureUserActionEcho {
        message: AdventureMessage,
    },
    AdventureCommandEcho {
        message: AdventureMessage,
    },
    AdventureNarrationChunk {
        adventure_id: AdventureId,
        message_id: AdventureMessageId,
        chunk: String,
    },
    AdventureNarrationComplete {
        adventure: Adventure,
        narration_message: AdventureMessage,
    },
    AdventureCommandComplete {
        adventure: Adventure,
        response_message: AdventureMessage,
    },
    AdventureReadyStatus {
        adventure_id: AdventureId,
        status: AdventureReadyStatus,
    },
    GmProposalReady {
        proposal: GmProposal,
    },
    CharacterReady {
        character: Character,
        chat: Chat,
    },
    CharacterImageReady {
        character_id: CharacterId,
        portrait: Option<StoredImageRef>,
    },
    WorldImageReady {
        blueprint_id: WorldBlueprintId,
        cover: Option<StoredImageRef>,
    },
    Error {
        task: Option<TaskKind>,
        entity_id: Option<String>,
        message: String,
    },
}

pub fn emit_bridge_event<R: Runtime>(
    app: &AppHandle<R>,
    event: BridgeEvent,
) -> Result<(), tauri::Error> {
    app.emit(BRIDGE_EVENT, event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_names_and_fields_are_stable_for_react() {
        let event = BridgeEvent::AdventureReadyStatus {
            adventure_id: AdventureId::new(),
            status: AdventureReadyStatus::UpdatingNarrative,
        };
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["type"], "adventure_ready_status");
        assert!(json.get("adventureId").is_some());
        assert_eq!(json["status"], "updating_narrative");
    }

    #[test]
    fn task_status_serializes_as_small_enums() {
        let event = BridgeEvent::TaskStatus {
            task: TaskKind::ChatReply,
            status: TaskStatus::Streaming,
            entity_id: Some("chat_11111111-1111-4111-8111-111111111111".to_string()),
        };
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["type"], "task_status");
        assert_eq!(json["task"], "chat_reply");
        assert_eq!(json["status"], "streaming");
        assert_eq!(
            json["entityId"],
            "chat_11111111-1111-4111-8111-111111111111"
        );
    }
}
