//! Chats, chat messages, senders, and reactions (`DATA-5`..`DATA-7`).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use sp_core::datetime::SpDateTime;

use crate::ai_model::AiModel;
use crate::ids::{CharacterId, ChatId, MessageId};
use crate::images::CharacterImage;
use crate::strings::{ChatTitle, MessageString, StorySummary};

/// The emoji set permitted for reactions (`DATA-6`). Any reactor emoji outside
/// this set is dropped on save.
pub const ALLOWED_EMOJIS: [&str; 8] = ["👍", "❤️", "😍", "😂", "💯", "🙏", "😢", "✨"];

/// The reactor key used for the player's own reactions (`DATA-6`).
pub const PLAYER_REACTOR: &str = "player";
/// The reactor key used for a character's (AI's) reactions (`DATA-6`, `CHAT-8`).
pub const AI_REACTOR: &str = "AI";

/// True if `emoji` is in the allowed reaction set (`DATA-6`).
pub fn is_allowed_emoji(emoji: &str) -> bool {
    ALLOWED_EMOJIS.contains(&emoji)
}

/// A message author: either the player or a character (`DATA-7`). The single-user
/// model collapses Soulfire-OG's separate "me"/"user" sender kinds into one
/// player kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Sender {
    Player,
    Character {
        character_id: CharacterId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        image: Option<CharacterImage>,
    },
}

impl Sender {
    pub fn is_player(&self) -> bool {
        matches!(self, Sender::Player)
    }

    pub fn character_id(&self) -> Option<&CharacterId> {
        match self {
            Sender::Character { character_id, .. } => Some(character_id),
            Sender::Player => None,
        }
    }
}

/// An insertion-ordered map of reactor → emoji (`DATA-6`). Reactor keys are the
/// player ([`PLAYER_REACTOR`]) or a character ([`AI_REACTOR`]). Only allowed
/// emojis are retained.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Reactions(pub IndexMap<String, String>);

impl Reactions {
    pub fn new() -> Self {
        Reactions(IndexMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, reactor: &str) -> Option<&str> {
        self.0.get(reactor).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }

    /// Set a reactor's reaction to an allowed emoji. A disallowed emoji is
    /// ignored (the reaction is not recorded), preserving insertion order
    /// (`DATA-6`).
    pub fn set(&mut self, reactor: impl Into<String>, emoji: impl Into<String>) {
        let emoji = emoji.into();
        if is_allowed_emoji(&emoji) {
            self.0.insert(reactor.into(), emoji);
        }
    }

    /// Remove a reactor's reaction.
    pub fn clear_reactor(&mut self, reactor: &str) {
        self.0.shift_remove(reactor);
    }

    /// Drop any entries whose emoji is not in the allowed set, preserving the
    /// order of those retained (`DATA-6`, applied on save).
    pub fn retain_allowed(&mut self) {
        self.0.retain(|_, emoji| is_allowed_emoji(emoji));
    }
}

/// A single chat message (`DATA-6`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct ChatMessage {
    #[builder(default)]
    pub message_id: MessageId,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    pub chat_id: ChatId,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,
    pub sender: Sender,
    pub message: MessageString,
    /// Token usage recorded for this message (`DATA-6`).
    #[builder(default)]
    #[serde(default)]
    pub token_count: u32,
    #[builder(default)]
    #[serde(default)]
    pub emoji_reactions: Reactions,
}

/// A 1:1 chat between the player and one character (`DATA-5`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct Chat {
    #[builder(default)]
    pub chat_id: ChatId,
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    #[builder(default)]
    #[serde(default)]
    pub started_at: SpDateTime,
    #[builder(default)]
    #[serde(default)]
    pub updated_at: SpDateTime,
    #[builder(default)]
    #[serde(default)]
    pub title: ChatTitle,
    /// The character this chat is with (`DATA-1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<CharacterId>,
    /// The player and the character, each with a sender descriptor (`DATA-5`).
    #[builder(default)]
    #[serde(default)]
    pub participants: Vec<Sender>,
    /// The model used for this chat, if chosen (`AI-9`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_model: Option<AiModel>,
    /// Rolling conversation summary used as long-term memory (`CHAT-10`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_summary: Option<StorySummary>,
    /// Messages since the last summary regeneration (`CHAT-10`).
    #[builder(default)]
    #[serde(default)]
    pub messages_since_summary: u32,
}

fn one() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reactions_drop_disallowed_emoji_on_set() {
        let mut r = Reactions::new();
        r.set(PLAYER_REACTOR, "👍");
        r.set(AI_REACTOR, "🚀"); // not allowed -> ignored
        assert_eq!(r.get(PLAYER_REACTOR), Some("👍"));
        assert_eq!(r.get(AI_REACTOR), None);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn reactions_preserve_insertion_order() {
        let mut r = Reactions::new();
        r.set("a", "👍");
        r.set("b", "❤️");
        r.set("c", "✨");
        let keys: Vec<&String> = r.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn retain_allowed_filters_existing() {
        let mut map = IndexMap::new();
        map.insert("a".to_string(), "👍".to_string());
        map.insert("b".to_string(), "🚀".to_string()); // disallowed
        let mut r = Reactions(map);
        r.retain_allowed();
        assert_eq!(r.len(), 1);
        assert_eq!(r.get("a"), Some("👍"));
    }

    #[test]
    fn sender_serializes_with_kind_tag() {
        let json = serde_json::to_string(&Sender::Player).unwrap();
        assert_eq!(json, "{\"kind\":\"player\"}");
        let s = Sender::Character {
            character_id: CharacterId::new(),
            image: None,
        };
        assert!(
            serde_json::to_string(&s)
                .unwrap()
                .contains("\"kind\":\"character\"")
        );
    }
}
