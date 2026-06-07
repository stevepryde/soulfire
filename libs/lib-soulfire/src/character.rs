//! The Character entity, its initial-message variants, and the character-builder
//! session (`DATA-1`..`DATA-4`, `DATA-14`).

use serde::{Deserialize, Serialize};

use sp_core::datetime::SpDateTime;

use crate::ai_model::AiModel;
use crate::ids::{
    AdventureId, CharacterBuilderMessageId, CharacterBuilderSnapshotId, CharacterId,
    WorldBlueprintId,
};
use crate::images::{CharacterImage, ImageTransform, StoredImageRef};
use crate::strings::{
    CharacterContext, CharacterDescription, CharacterName, CharacterPrompt, CharacterSubtitle,
    InitialMessageText,
};

/// Default creativity controls (`DATA-1`).
pub const DEFAULT_MAX_TOKENS: u32 = 2000;
pub const DEFAULT_TEMPERATURE: f64 = 1.0;
pub const DEFAULT_TOP_P: f64 = 0.95;
pub const DEFAULT_TOP_K: u32 = 3;

/// Creativity-control clamp ranges applied on save (`DATA-1`).
pub const MAX_TOKENS_RANGE: (u32, u32) = (500, 5000);
pub const TEMPERATURE_RANGE: (f64, f64) = (0.0, 2.0);
pub const TOP_P_RANGE: (f64, f64) = (0.0, 1.0);
pub const TOP_K_RANGE: (u32, u32) = (1, 200);

fn default_max_tokens() -> u32 {
    DEFAULT_MAX_TOKENS
}
fn default_temperature() -> f64 {
    DEFAULT_TEMPERATURE
}
fn default_top_p() -> f64 {
    DEFAULT_TOP_P
}
fn default_top_k() -> u32 {
    DEFAULT_TOP_K
}
fn default_version() -> u32 {
    1
}

/// The opening message a character delivers when its chat is first opened
/// (`DATA-2`). Each variant wraps a prompt/message string (0, 16000).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitialMessage {
    /// A fixed opening line sent verbatim (no model call, `CHAT-2`).
    Message(InitialMessageText),
    /// A seed the AI uses to generate its opening line (`CHAT-2`).
    Prompt(InitialMessageText),
}

impl InitialMessage {
    /// The wrapped text, regardless of variant.
    pub fn text(&self) -> &InitialMessageText {
        match self {
            InitialMessage::Message(t) | InitialMessage::Prompt(t) => t,
        }
    }

    pub fn is_prompt(&self) -> bool {
        matches!(self, InitialMessage::Prompt(_))
    }
}

impl Default for InitialMessage {
    fn default() -> Self {
        InitialMessage::Message(InitialMessageText::default())
    }
}

/// Creativity / sampling controls for a character's generations (`DATA-1`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, bon::Builder)]
pub struct CreativityControls {
    #[builder(default = default_max_tokens())]
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[builder(default = default_temperature())]
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[builder(default = default_top_p())]
    #[serde(default = "default_top_p")]
    pub top_p: f64,
    #[builder(default = default_top_k())]
    #[serde(default = "default_top_k")]
    pub top_k: u32,
}

impl Default for CreativityControls {
    fn default() -> Self {
        CreativityControls {
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
            top_p: DEFAULT_TOP_P,
            top_k: DEFAULT_TOP_K,
        }
    }
}

impl CreativityControls {
    /// Clamp every control into its valid range (`DATA-1`, applied on save).
    pub fn clamped(self) -> Self {
        CreativityControls {
            max_tokens: self.max_tokens.clamp(MAX_TOKENS_RANGE.0, MAX_TOKENS_RANGE.1),
            temperature: self.temperature.clamp(TEMPERATURE_RANGE.0, TEMPERATURE_RANGE.1),
            top_p: self.top_p.clamp(TOP_P_RANGE.0, TOP_P_RANGE.1),
            top_k: self.top_k.clamp(TOP_K_RANGE.0, TOP_K_RANGE.1),
        }
    }
}

/// A character: a persona the player chats with 1:1 (`DATA-1`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bon::Builder)]
pub struct Character {
    #[builder(default)]
    pub character_id: CharacterId,
    /// Persisted schema version for forward migration (`DATA`).
    #[builder(default = default_version())]
    #[serde(default = "default_version")]
    pub version: u32,
    #[builder(default)]
    #[serde(default)]
    pub created_at: SpDateTime,
    #[builder(default)]
    #[serde(default)]
    pub updated_at: SpDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_chatted_at: Option<SpDateTime>,

    // Profile (DATA-1)
    pub name: CharacterName,
    #[builder(default)]
    #[serde(default)]
    pub subtitle: CharacterSubtitle,
    #[builder(default)]
    #[serde(default)]
    pub description: CharacterDescription,

    /// Emoji/illustration avatar selection (`DATA-20`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<CharacterImage>,
    /// A generated/uploaded portrait stored in the encrypted store (`IMG-4`); when
    /// present it takes precedence over `image` (`IMG-8`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait: Option<StoredImageRef>,
    #[builder(default)]
    #[serde(default)]
    pub image_transform: ImageTransform,

    /// The editable system prompt — the primary authored persona (`DATA-1`).
    #[builder(default)]
    #[serde(default)]
    pub prompt: CharacterPrompt,
    pub initial_message: InitialMessage,

    #[builder(default)]
    #[serde(default)]
    pub creativity: CreativityControls,
    /// The model chosen for this character's chat, if any (`AI-9`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_model: Option<AiModel>,

    // AI-internal private fields (DATA-3); never shown in the standard editor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_context: Option<CharacterContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_state: Option<CharacterContext>,

    // Origin when extracted from a world (DATA-4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_blueprint_id: Option<WorldBlueprintId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_adventure_id: Option<AdventureId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_npc_name: Option<String>,
}

impl Character {
    /// True when the character has both a persona profile and a dynamic state —
    /// i.e. a world-extracted character that runs the background state updater
    /// (`CHAT-12`).
    pub fn is_world_extracted(&self) -> bool {
        self.extracted_context.is_some() && self.character_state.is_some()
    }

    /// Clamp creativity controls into range (`DATA-1`); call before persisting.
    pub fn clamp_creativity(&mut self) {
        self.creativity = self.creativity.clamped();
    }
}

// ===== Character builder session (DATA-14) =====

/// Cap on the builder message log; oldest dropped (`DATA-14`).
pub const BUILDER_MESSAGE_CAP: usize = 50;
/// Cap on the builder snapshot stack; oldest dropped (`DATA-14`).
pub const BUILDER_SNAPSHOT_CAP: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterBuilderRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterBuilderMessage {
    #[serde(default)]
    pub message_id: CharacterBuilderMessageId,
    pub role: CharacterBuilderRole,
    pub content: String,
    #[serde(default)]
    pub created_at: SpDateTime,
}

/// A captured snapshot of editable character fields backing builder undo
/// (`DATA-14`, `CHAR-8`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterBuilderSnapshot {
    #[serde(default)]
    pub snapshot_id: CharacterBuilderSnapshotId,
    pub name: CharacterName,
    pub subtitle: CharacterSubtitle,
    pub description: CharacterDescription,
    pub prompt: CharacterPrompt,
    pub initial_message: InitialMessage,
    #[serde(default)]
    pub captured_at: SpDateTime,
}

/// A character-builder session keyed to one character (`DATA-14`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CharacterBuilderSession {
    pub character_id: CharacterId,
    #[serde(default)]
    pub messages: Vec<CharacterBuilderMessage>,
    #[serde(default)]
    pub snapshots: Vec<CharacterBuilderSnapshot>,
}

impl CharacterBuilderSession {
    /// Append a message, dropping the oldest beyond the cap (`DATA-14`).
    pub fn push_message(&mut self, message: CharacterBuilderMessage) {
        self.messages.push(message);
        let overflow = self.messages.len().saturating_sub(BUILDER_MESSAGE_CAP);
        if overflow > 0 {
            self.messages.drain(0..overflow);
        }
    }

    /// Push a snapshot, skipping a duplicate of the last, dropping the oldest
    /// beyond the cap (`DATA-14`).
    pub fn push_snapshot(&mut self, snapshot: CharacterBuilderSnapshot) {
        if let Some(last) = self.snapshots.last() {
            if last.name == snapshot.name
                && last.subtitle == snapshot.subtitle
                && last.description == snapshot.description
                && last.prompt == snapshot.prompt
                && last.initial_message == snapshot.initial_message
            {
                return; // duplicate-of-last skipped
            }
        }
        self.snapshots.push(snapshot);
        let overflow = self.snapshots.len().saturating_sub(BUILDER_SNAPSHOT_CAP);
        if overflow > 0 {
            self.snapshots.drain(0..overflow);
        }
    }

    /// Pop and return the most recent snapshot for undo (`CHAR-8`).
    pub fn pop_snapshot(&mut self) -> Option<CharacterBuilderSnapshot> {
        self.snapshots.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn snap(prompt: &str) -> CharacterBuilderSnapshot {
        CharacterBuilderSnapshot {
            snapshot_id: CharacterBuilderSnapshotId::new(),
            name: CharacterName::from_str("N").unwrap(),
            subtitle: CharacterSubtitle::default(),
            description: CharacterDescription::default(),
            prompt: CharacterPrompt::coerce(prompt),
            initial_message: InitialMessage::default(),
            captured_at: SpDateTime::now(),
        }
    }

    #[test]
    fn creativity_clamps_to_ranges() {
        let wild = CreativityControls {
            max_tokens: 10,
            temperature: 9.0,
            top_p: 5.0,
            top_k: 9999,
        }
        .clamped();
        assert_eq!(wild.max_tokens, 500);
        assert_eq!(wild.temperature, 2.0);
        assert_eq!(wild.top_p, 1.0);
        assert_eq!(wild.top_k, 200);
    }

    #[test]
    fn defaults_match_spec() {
        let d = CreativityControls::default();
        assert_eq!(d.max_tokens, 2000);
        assert_eq!(d.temperature, 1.0);
        assert_eq!(d.top_p, 0.95);
        assert_eq!(d.top_k, 3);
    }

    #[test]
    fn builder_snapshot_cap_drops_oldest_and_dedups_last() {
        let mut s = CharacterBuilderSession::default();
        for i in 0..12 {
            s.push_snapshot(snap(&format!("p{i}")));
        }
        assert_eq!(s.snapshots.len(), BUILDER_SNAPSHOT_CAP);
        // duplicate of last is skipped
        let last_prompt = s.snapshots.last().unwrap().prompt.clone();
        s.push_snapshot(snap(last_prompt.as_str()));
        assert_eq!(s.snapshots.len(), BUILDER_SNAPSHOT_CAP);
    }

    #[test]
    fn world_extracted_requires_both_private_fields() {
        let mut c = Character::builder()
            .name(CharacterName::from_str("Test").unwrap())
            .initial_message(InitialMessage::default())
            .build();
        assert!(!c.is_world_extracted());
        c.extracted_context = Some(CharacterContext::coerce("persona"));
        assert!(!c.is_world_extracted());
        c.character_state = Some(CharacterContext::coerce("state"));
        assert!(c.is_world_extracted());
    }
}
