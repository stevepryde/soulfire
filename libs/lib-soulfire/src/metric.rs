//! Usage metering records (`DATA-20a`, `AI-15`, `STAT`).

use serde::{Deserialize, Serialize};

use sp_core::datetime::SpDateTime;

use crate::ai_model::AiModel;
use crate::ids::{AdventureId, CharacterId, ChatId, MetricId, WorldBlueprintId};

/// What a metered AI request was for (`DATA-20a`). The operation kind a usage
/// entry is grouped by in the by-operation breakdown (`STAT-4`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MetricLabel {
    /// A character chat reply (`CHAT-4`).
    ChatMessage,
    /// A rolling chat summary, also used for title generation (`CHAT-3`,`CHAT-10`).
    ChatSummary,
    /// A background character-state update (`CHAT-12`).
    CharacterStateUpdate,
    /// A character-builder turn (`CHAR-7`).
    CharacterBuilder,
    /// An NPC-extraction call (persona profile or initial state, `CHAR-10`).
    NpcExtraction,
    /// An adventure narration turn / intro (`WORLD-3`, `WORLD-5`).
    AdventureAction,
    /// A full adventure-state reconciliation / initial-state call (`WORLD-13`).
    AdventureFullStateUpdate,
    /// A diff adventure-state update (`WORLD-12`).
    AdventureDiffStateUpdate,
    /// An out-of-band `/gm` classify/answer/proposal call (`WORLD-16`).
    GmCommand,
    /// A world-builder turn (`WORLD-21`).
    WorldBuilder,
    /// An image generation (`IMG-3`).
    ImageGeneration,
}

/// One metered AI request's usage (`DATA-20a`). Entity associations are populated
/// whenever the request belongs to that entity, so totals roll up by chat,
/// adventure, world, character, model, operation, and time (`STAT-4`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct UsageMetric {
    #[builder(default)]
    pub metric_id: MetricId,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,
    pub label: MetricLabel,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adventure_id: Option<AdventureId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blueprint_id: Option<WorldBlueprintId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<CharacterId>,

    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Cached-input tokens where the provider reports them, recorded as a subset
    /// of `input_tokens` so totals are not double-counted (`STAT-3`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_input_tokens: Option<u64>,

    pub ai_model: AiModel,
}

impl UsageMetric {
    /// True when this request recorded no tokens at all; such records are not
    /// written (`DATA-20a`).
    pub fn is_zero(&self) -> bool {
        self.input_tokens == 0
            && self.output_tokens == 0
            && self.cached_input_tokens.unwrap_or(0) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_serializes_to_spec_names() {
        assert_eq!(
            serde_json::to_string(&MetricLabel::ChatMessage).unwrap(),
            "\"chat_message\""
        );
        assert_eq!(
            serde_json::to_string(&MetricLabel::AdventureDiffStateUpdate).unwrap(),
            "\"adventure_diff_state_update\""
        );
        assert_eq!(
            serde_json::to_string(&MetricLabel::ImageGeneration).unwrap(),
            "\"image_generation\""
        );
    }

    #[test]
    fn zero_token_record_is_detected() {
        let m = UsageMetric::builder()
            .label(MetricLabel::ChatMessage)
            .input_tokens(0)
            .output_tokens(0)
            .ai_model(AiModel::Gpt5_1)
            .build();
        assert!(m.is_zero());
    }
}
