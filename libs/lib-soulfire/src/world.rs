//! Worlds: blueprints, adventures, turn-log messages, GM proposals, and the
//! world-builder session (`DATA-8`..`DATA-15`).

use serde::{Deserialize, Serialize};

use sp_core::datetime::SpDateTime;

use crate::ai_model::AiModel;
use crate::ids::{
    AdventureId, AdventureMessageId, GmProposalId, WorldBlueprintId, WorldBuilderMessageId,
    WorldBuilderSnapshotId,
};
use crate::images::{ImageTransform, StoredImageRef, WorldImage};
use crate::strings::{
    AdventureState, MessageContent, PlayerAttributes, PlayerName, RecentSummary, SignificantEvents,
    StorySummary, WorldDescription, WorldPrompt, WorldTitle,
};

fn one() -> u32 {
    1
}

/// The outcome status of an adventure's story (`DATA-10`).
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum StoryStatus {
    #[default]
    Ongoing,
    Success,
    Failure,
}

impl StoryStatus {
    /// True once the story has reached a terminal outcome.
    pub fn is_terminal(&self) -> bool {
        !matches!(self, StoryStatus::Ongoing)
    }
}

/// The kind of an adventure turn-log entry (`DATA-12`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AdventureMessageType {
    /// Game-master prose.
    Narration,
    /// A player action.
    UserAction,
    /// An out-of-band player→GM request.
    GameMasterRequest,
    /// An out-of-band GM→player response.
    GameMasterResponse,
}

/// Per-adventure turn-engine status used for the single-flight lock (`WORLD-5`).
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AdventureReadyStatus {
    #[default]
    Ready,
    UpdatingNarrative,
    UpdatingState,
    UpdatingCommand,
}

// ===== Blueprint (DATA-8) =====

/// An authored, reusable world template (`DATA-8`). One blueprint spawns many
/// independent adventures (`WORLD-1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct WorldBlueprint {
    #[builder(default)]
    pub blueprint_id: WorldBlueprintId,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,
    #[builder(default)]
    #[serde(default)]
    pub updated_at: SpDateTime,
    pub title: WorldTitle,
    /// Shown to the player; not sent to the AI (`DATA-8`).
    #[builder(default)]
    #[serde(default)]
    pub description: WorldDescription,
    /// The full freeform authored world sent to the AI (`DATA-8`, `DATA-9`).
    pub world_prompt: WorldPrompt,
    /// Emoji cover selection (`DATA-20`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<WorldImage>,
    /// A generated/uploaded cover stored in the encrypted store (`IMG-4`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<StoredImageRef>,
    /// 16:6 cover framing (`IMG-7`).
    #[builder(default)]
    #[serde(default)]
    pub image_transform: ImageTransform,
}

// ===== Adventure (DATA-10, DATA-11) =====

/// One playthrough of one blueprint (`DATA-10`), carrying a denormalized world
/// snapshot for display, a private prompt copy, live state, and memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct Adventure {
    #[builder(default)]
    pub adventure_id: AdventureId,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,
    #[builder(default)]
    #[serde(default)]
    pub updated_at: SpDateTime,
    pub blueprint_id: WorldBlueprintId,

    // Denormalized world snapshot for display (DATA-10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_title: Option<WorldTitle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_description: Option<WorldDescription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_image: Option<WorldImage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_cover: Option<StoredImageRef>,
    #[builder(default)]
    #[serde(default)]
    pub world_image_transform: ImageTransform,

    /// A private per-adventure copy of the blueprint prompt at start, so `/gm`
    /// retcons affect only this adventure (`DATA-10`, `WORLD-17`).
    pub world_prompt: WorldPrompt,

    // Player identity snapshotted at start (DATA-10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_name: Option<PlayerName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_attributes: Option<PlayerAttributes>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_model: Option<AiModel>,

    // Live state & memory (DATA-11).
    #[builder(default)]
    #[serde(default)]
    pub adventure_state: AdventureState,
    #[builder(default)]
    #[serde(default)]
    pub recent_summary: RecentSummary,
    #[builder(default)]
    #[serde(default)]
    pub significant_events: SignificantEvents,
    #[builder(default)]
    #[serde(default)]
    pub story_summary: StorySummary,

    #[builder(default)]
    #[serde(default)]
    pub story_status: StoryStatus,
    /// Sticky once the story becomes non-ongoing (`DATA-10`, `WORLD-6`).
    #[builder(default)]
    #[serde(default)]
    pub has_completed: bool,

    // Turn-engine bookkeeping (DATA-10).
    #[builder(default)]
    #[serde(default)]
    pub ready_status: AdventureReadyStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_status_updated_at: Option<SpDateTime>,
    #[builder(default)]
    #[serde(default)]
    pub diff_action_count: u32,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub next_significant_event_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_narrative: Option<String>,
}

/// One entry in an adventure's turn log (`DATA-12`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct AdventureMessage {
    #[builder(default)]
    pub message_id: AdventureMessageId,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    pub adventure_id: AdventureId,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,
    pub message_type: AdventureMessageType,
    pub content: MessageContent,
}

// ===== GM proposal (DATA-13) =====

/// Which side of a `/gm` change a proposal targets (`DATA-13`, `WORLD-16`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum GmChangeTarget {
    AdventureState,
    WorldBlueprint,
}

/// One human-readable diff entry in a staged proposal (`DATA-13`, `WORLD-17`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GmDiffEntry {
    pub target: GmChangeTarget,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

/// Lifecycle of a staged `/gm` change proposal (`DATA-13`, `WORLD-17`).
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum GmProposalStatus {
    #[default]
    Pending,
    Accepted,
    Rejected,
}

/// A staged out-of-band game-master change, awaiting Accept/Reject (`DATA-13`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct GmProposal {
    #[builder(default)]
    pub proposal_id: GmProposalId,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    pub adventure_id: AdventureId,
    /// The `game_master_response` message that proposed the change (`DATA-13`).
    pub response_message_id: AdventureMessageId,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,

    /// Proposed replacement adventure-state, if the proposal changes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_adventure_state: Option<AdventureState>,
    /// Proposed replacement blueprint prompt (the adventure's private copy only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_world_prompt: Option<WorldPrompt>,
    /// Computed memory updates that accompany the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_recent_summary: Option<RecentSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_significant_events: Option<SignificantEvents>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_story_summary: Option<StorySummary>,

    /// Human-readable change summary (`DATA-13`, `WORLD-17`).
    #[builder(default)]
    #[serde(default)]
    pub changes: Vec<GmDiffEntry>,
    #[builder(default)]
    #[serde(default)]
    pub status: GmProposalStatus,
}

// ===== World builder session (DATA-15) =====

/// Cap on the world-builder message log (`DATA-15`).
pub const WORLD_BUILDER_MESSAGE_CAP: usize = 50;
/// Cap on the world-builder snapshot stack (`DATA-15`).
pub const WORLD_BUILDER_SNAPSHOT_CAP: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldBuilderRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldBuilderMessage {
    #[serde(default)]
    pub message_id: WorldBuilderMessageId,
    pub role: WorldBuilderRole,
    pub content: String,
    #[serde(default)]
    pub created_at: SpDateTime,
}

/// A captured snapshot of editable blueprint fields backing builder undo
/// (`DATA-15`, `WORLD-21`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldBuilderSnapshot {
    #[serde(default)]
    pub snapshot_id: WorldBuilderSnapshotId,
    pub title: WorldTitle,
    pub description: WorldDescription,
    pub world_prompt: WorldPrompt,
    #[serde(default)]
    pub captured_at: SpDateTime,
}

/// A world-builder session keyed to one blueprint (`DATA-15`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorldBuilderSession {
    pub blueprint_id: WorldBlueprintId,
    #[serde(default)]
    pub messages: Vec<WorldBuilderMessage>,
    #[serde(default)]
    pub snapshots: Vec<WorldBuilderSnapshot>,
}

impl WorldBuilderSession {
    pub fn push_message(&mut self, message: WorldBuilderMessage) {
        self.messages.push(message);
        let overflow = self.messages.len().saturating_sub(WORLD_BUILDER_MESSAGE_CAP);
        if overflow > 0 {
            self.messages.drain(0..overflow);
        }
    }

    pub fn push_snapshot(&mut self, snapshot: WorldBuilderSnapshot) {
        if let Some(last) = self.snapshots.last() {
            if last.title == snapshot.title
                && last.description == snapshot.description
                && last.world_prompt == snapshot.world_prompt
            {
                return; // duplicate-of-last skipped
            }
        }
        self.snapshots.push(snapshot);
        let overflow = self
            .snapshots
            .len()
            .saturating_sub(WORLD_BUILDER_SNAPSHOT_CAP);
        if overflow > 0 {
            self.snapshots.drain(0..overflow);
        }
    }

    pub fn pop_snapshot(&mut self) -> Option<WorldBuilderSnapshot> {
        self.snapshots.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn story_status_terminal() {
        assert!(!StoryStatus::Ongoing.is_terminal());
        assert!(StoryStatus::Success.is_terminal());
        assert!(StoryStatus::Failure.is_terminal());
    }

    #[test]
    fn message_type_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&AdventureMessageType::GameMasterRequest).unwrap(),
            "\"game_master_request\""
        );
    }

    #[test]
    fn adventure_defaults_are_sane() {
        let adv = Adventure::builder()
            .blueprint_id(WorldBlueprintId::new())
            .world_prompt(WorldPrompt::from_str("A dark wood.").unwrap())
            .build();
        assert_eq!(adv.story_status, StoryStatus::Ongoing);
        assert!(!adv.has_completed);
        assert_eq!(adv.ready_status, AdventureReadyStatus::Ready);
        assert_eq!(adv.diff_action_count, 0);
        assert_eq!(adv.next_significant_event_id, 1);
    }

    #[test]
    fn world_builder_snapshot_cap_and_dedup() {
        let mut s = WorldBuilderSession::default();
        let mk = |p: &str| WorldBuilderSnapshot {
            snapshot_id: WorldBuilderSnapshotId::new(),
            title: WorldTitle::from_str("T").unwrap(),
            description: WorldDescription::default(),
            world_prompt: WorldPrompt::coerce(p),
            captured_at: SpDateTime::now(),
        };
        for i in 0..12 {
            s.push_snapshot(mk(&format!("p{i}")));
        }
        assert_eq!(s.snapshots.len(), WORLD_BUILDER_SNAPSHOT_CAP);
        let last = s.snapshots.last().unwrap().world_prompt.clone();
        s.push_snapshot(mk(last.as_str()));
        assert_eq!(s.snapshots.len(), WORLD_BUILDER_SNAPSHOT_CAP);
    }
}
