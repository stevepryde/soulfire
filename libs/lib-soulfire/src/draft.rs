//! Unsent composer drafts (`DATA-26`). Local UI state, never sent to the AI until
//! the user submits; at most one draft per scope; deleted with their parent.

use serde::{Deserialize, Serialize};

use sp_core::datetime::SpDateTime;

use crate::ids::{AdventureId, ChatId, DraftId};
use crate::strings::DraftContent;

/// What a draft belongs to (`DATA-26`). At most one draft exists per scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "scope")]
pub enum DraftScope {
    Chat { chat_id: ChatId },
    Adventure { adventure_id: AdventureId },
}

impl DraftScope {
    /// A stable key identifying this scope (used for the per-scope uniqueness
    /// constraint, `DATA-26`).
    pub fn key(&self) -> String {
        match self {
            DraftScope::Chat { chat_id } => format!("chat:{chat_id}"),
            DraftScope::Adventure { adventure_id } => format!("adventure:{adventure_id}"),
        }
    }
}

/// An unsent composer draft for a chat or adventure (`DATA-26`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct Draft {
    #[builder(default)]
    pub draft_id: DraftId,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,
    #[builder(default)]
    #[serde(default)]
    pub updated_at: SpDateTime,
    #[serde(flatten)]
    pub scope: DraftScope,
    #[builder(default)]
    #[serde(default)]
    pub content: DraftContent,
}

fn one() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_keys_are_distinct_per_kind() {
        let chat = DraftScope::Chat {
            chat_id: ChatId::new(),
        };
        let adv = DraftScope::Adventure {
            adventure_id: AdventureId::new(),
        };
        assert!(chat.key().starts_with("chat:"));
        assert!(adv.key().starts_with("adventure:"));
    }

    #[test]
    fn draft_round_trips_with_flattened_scope() {
        let d = Draft::builder()
            .scope(DraftScope::Chat {
                chat_id: ChatId::new(),
            })
            .content(DraftContent::coerce("half-written message"))
            .build();
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains("\"scope\":\"chat\""));
        let back: Draft = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }
}
