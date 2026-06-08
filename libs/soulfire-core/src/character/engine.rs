//! The character builder and NPC extraction (`CHAR-6`..`CHAR-12`).

use std::sync::Arc;

use serde::Deserialize;

use crate::model::ai_model::AiModel;
use crate::model::character::{
    Character, CharacterBuilderMessage, CharacterBuilderRole, CharacterBuilderSession,
    CharacterBuilderSnapshot, InitialMessage,
};
use crate::model::ids::{
    AdventureId, CharacterBuilderMessageId, CharacterBuilderSnapshotId, CharacterId,
};
use crate::model::metric::MetricLabel;
use crate::model::metric::UsageMetric;
use crate::model::strings::{
    CharacterContext, CharacterDescription, CharacterName, CharacterPrompt, CharacterSubtitle,
    ChatTitle, InitialMessageText,
};

use crate::ai::fence::parse_lenient;
use crate::ai::registry::resolve_model;
use crate::ai::service::AiService;
use crate::ai::types::{
    GenerationConfig, GenerationRequest, JsonMode, PromptMessage, ReasoningEffort, Usage,
};
use crate::chat::ChatEngine;
use crate::clock::Clock;
use crate::error::{CoreError, CoreResult};
use crate::store::Store;

use super::prompts;

/// Builder generation temperature (`CHAR` design note).
pub const BUILDER_TEMPERATURE: f64 = 0.8;
const BUILDER_MAX_TOKENS: u32 = 6000;
/// Persona-extraction temperature (`CHAR` design note).
pub const EXTRACTION_PROFILE_TEMPERATURE: f64 = 0.7;
const EXTRACTION_PROFILE_MAX_TOKENS: u32 = 8000;
const EXTRACTION_STATE_MAX_TOKENS: u32 = 2000;

/// The structured result of a builder turn (`CHAR-7`). Null fields mean
/// "leave unchanged".
#[derive(Debug, Clone, Deserialize)]
pub struct BuilderResult {
    pub assistant_message: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub initial_message: Option<String>,
}

impl BuilderResult {
    fn changes_character(&self) -> bool {
        self.name.is_some()
            || self.subtitle.is_some()
            || self.description.is_some()
            || self.prompt.is_some()
            || self.initial_message.is_some()
    }
}

/// Drives the character builder and NPC extraction.
#[derive(Clone)]
pub struct CharacterEngine {
    store: Arc<Store>,
    ai: AiService,
    clock: Arc<dyn Clock>,
}

impl CharacterEngine {
    pub fn new(store: Arc<Store>, ai: AiService, clock: Arc<dyn Clock>) -> Self {
        CharacterEngine { store, ai, clock }
    }

    /// Send a builder message; the assistant replies and may revise the
    /// character. A snapshot is captured before applying changes (`CHAR-7`,
    /// `CHAR-8`).
    pub async fn builder_send(
        &self,
        character_id: &CharacterId,
        user_message: &str,
    ) -> CoreResult<BuilderResult> {
        let mut character = self.load(character_id)?;
        let mut session = self
            .store
            .character_builder_session(character_id)?
            .unwrap_or_else(|| CharacterBuilderSession {
                character_id: character_id.clone(),
                ..Default::default()
            });

        let recent = recent_builder_text(&session);
        session.push_message(self.builder_message(CharacterBuilderRole::User, user_message));

        let model = resolve_model(
            None,
            self.store.app_profile()?.default_ai_model,
            AiModel::default_chat_narrative(),
        );
        let request = GenerationRequest {
            model,
            instructions: Some(prompts::builder_instructions()),
            messages: vec![PromptMessage::user(prompts::builder_input(
                character.name.as_str(),
                character.subtitle.as_str(),
                character.description.as_str(),
                character.prompt.as_str(),
                character.initial_message.text().as_str(),
                &recent,
                user_message,
            ))],
            config: GenerationConfig {
                max_output_tokens: Some(BUILDER_MAX_TOKENS),
                temperature: Some(BUILDER_TEMPERATURE),
                reasoning_effort: Some(ReasoningEffort::Medium),
                json: Some(JsonMode::Json),
                ..Default::default()
            },
        };
        let response = self.ai.generate(request).await?;
        self.meter(
            MetricLabel::CharacterBuilder,
            model,
            response.usage,
            Some(character_id),
        )?;

        let result: BuilderResult = parse_lenient(&response.text)
            .map_err(|e| CoreError::Serialization(format!("builder response: {e}")))?;

        session.push_message(
            self.builder_message(CharacterBuilderRole::Assistant, &result.assistant_message),
        );

        // Snapshot the prior state before applying changes (CHAR-8).
        if result.changes_character() {
            session.push_snapshot(self.snapshot_of(&character));
            self.apply_changes(&mut character, &result);
            character.clamp_creativity();
            character.updated_at = self.clock.now();
            self.store.save_character(&character)?;
        }
        self.store.save_character_builder_session(&session)?;
        Ok(result)
    }

    /// Undo the most recent builder change, restoring the prior snapshot
    /// (`CHAR-8`). Returns `false` if there was nothing to undo.
    pub fn builder_undo(&self, character_id: &CharacterId) -> CoreResult<bool> {
        let Some(mut session) = self.store.character_builder_session(character_id)? else {
            return Ok(false);
        };
        let Some(snapshot) = session.pop_snapshot() else {
            return Ok(false);
        };
        let mut character = self.load(character_id)?;
        character.name = snapshot.name;
        character.subtitle = snapshot.subtitle;
        character.description = snapshot.description;
        character.prompt = snapshot.prompt;
        character.initial_message = snapshot.initial_message;
        character.updated_at = self.clock.now();
        self.store.save_character(&character)?;

        session.push_message(
            self.builder_message(CharacterBuilderRole::Assistant, "Reverted the last change."),
        );
        self.store.save_character_builder_session(&session)?;
        Ok(true)
    }

    /// Extract an NPC from an adventure into a standalone chat character
    /// (`CHAR-10`..`CHAR-12`). Generates the persona profile and initial state
    /// first; a failure creates no partial character (`CHAR-12`).
    pub async fn extract_npc(
        &self,
        adventure_id: &AdventureId,
        npc_name: &str,
    ) -> CoreResult<Character> {
        let adventure = self
            .store
            .adventure(adventure_id)?
            .ok_or_else(|| CoreError::NotFound(adventure_id.to_string()))?;
        let model = resolve_model(
            None,
            self.store.app_profile()?.default_ai_model,
            AiModel::default_chat_narrative(),
        );

        // Persona profile (stable traits → extracted_context).
        let persona_req = GenerationRequest {
            model,
            instructions: Some(prompts::extraction_system_prompt(
                adventure.world_prompt.as_str(),
                adventure.adventure_state.as_str(),
                adventure.story_summary.as_str(),
                npc_name,
            )),
            messages: vec![PromptMessage::user(format!(
                "Extract the character \"{npc_name}\" into a complete profile."
            ))],
            config: GenerationConfig {
                max_output_tokens: Some(EXTRACTION_PROFILE_MAX_TOKENS),
                temperature: Some(EXTRACTION_PROFILE_TEMPERATURE),
                top_p: Some(0.95),
                top_k: Some(3),
                ..Default::default()
            },
        };
        let persona = self.ai.generate(persona_req).await?;
        self.meter(MetricLabel::NpcExtraction, model, persona.usage, None)?;

        // Initial dynamic state (current emotion/relationship → character_state).
        let state_model = AiModel::default_state_utility();
        let state_req = GenerationRequest {
            model: state_model,
            instructions: Some(prompts::initial_state_system_prompt(
                npc_name,
                &persona.text,
                adventure.story_summary.as_str(),
            )),
            messages: vec![PromptMessage::user(
                "Write this character's current dynamic state.",
            )],
            config: GenerationConfig {
                max_output_tokens: Some(EXTRACTION_STATE_MAX_TOKENS),
                temperature: Some(EXTRACTION_PROFILE_TEMPERATURE),
                top_p: Some(0.95),
                top_k: Some(3),
                ..Default::default()
            },
        };
        let state = self.ai.generate(state_req).await?;
        self.meter(MetricLabel::NpcExtraction, state_model, state.usage, None)?;

        // Build the character with its recorded origin (CHAR-11) — only after both
        // calls succeed, so a failure leaves nothing partial (CHAR-12).
        let now = self.clock.now();
        let mut character = Character::builder()
            .name(CharacterName::coerce(npc_name))
            .created_at(now)
            .updated_at(now)
            .prompt(CharacterPrompt::coerce(&format!(
                "You are {npc_name}, brought to life from a shared adventure. Speak as yourself, drawing on everything you lived through together."
            )))
            .initial_message(InitialMessage::Prompt(InitialMessageText::coerce(
                "Greet the player warmly, as someone reconnecting after the events of your shared story.",
            )))
            .extracted_context(CharacterContext::coerce(persona.text.trim()))
            .character_state(CharacterContext::coerce(state.text.trim()))
            .source_blueprint_id(adventure.blueprint_id.clone())
            .source_adventure_id(adventure_id.clone())
            .source_npc_name(npc_name.to_string())
            .build();
        self.store.save_character(&character)?;

        // Create and open the chat with a generated opening and a title (CHAR-11).
        let chat_engine = ChatEngine::new(self.store.clone(), self.ai.clone(), self.clock.clone());
        let mut chat = chat_engine.open_chat(&character.character_id).await?;
        if chat.title.as_str().is_empty() {
            chat.title = ChatTitle::coerce(npc_name);
            self.store.save_chat(&chat)?;
        }

        character.last_chatted_at = Some(self.clock.now());
        self.store.save_character(&character)?;
        Ok(character)
    }

    // ----- internals -----

    fn apply_changes(&self, character: &mut Character, result: &BuilderResult) {
        if let Some(name) = &result.name {
            character.name = CharacterName::coerce(name);
        }
        if let Some(subtitle) = &result.subtitle {
            character.subtitle = CharacterSubtitle::coerce(subtitle);
        }
        if let Some(description) = &result.description {
            character.description = CharacterDescription::coerce(description);
        }
        if let Some(prompt) = &result.prompt {
            character.prompt = CharacterPrompt::coerce(prompt);
        }
        if let Some(initial) = &result.initial_message {
            character.initial_message =
                InitialMessage::Message(InitialMessageText::coerce(initial));
        }
    }

    fn snapshot_of(&self, character: &Character) -> CharacterBuilderSnapshot {
        CharacterBuilderSnapshot {
            snapshot_id: CharacterBuilderSnapshotId::new(),
            name: character.name.clone(),
            subtitle: character.subtitle.clone(),
            description: character.description.clone(),
            prompt: character.prompt.clone(),
            initial_message: character.initial_message.clone(),
            captured_at: self.clock.now(),
        }
    }

    fn builder_message(
        &self,
        role: CharacterBuilderRole,
        content: &str,
    ) -> CharacterBuilderMessage {
        CharacterBuilderMessage {
            message_id: CharacterBuilderMessageId::new(),
            role,
            content: content.to_string(),
            created_at: self.clock.now(),
        }
    }

    fn load(&self, id: &CharacterId) -> CoreResult<Character> {
        self.store
            .character(id)?
            .ok_or_else(|| CoreError::NotFound(id.to_string()))
    }

    fn meter(
        &self,
        label: MetricLabel,
        model: AiModel,
        usage: Usage,
        character_id: Option<&CharacterId>,
    ) -> CoreResult<()> {
        let metric = UsageMetric::builder()
            .created_at(self.clock.now())
            .label(label)
            .maybe_character_id(character_id.cloned())
            .input_tokens(usage.input_tokens)
            .output_tokens(usage.output_tokens)
            .maybe_cached_input_tokens(usage.cached_input_tokens)
            .ai_model(model)
            .build();
        self.store.save_metric(&metric)
    }
}

fn recent_builder_text(session: &CharacterBuilderSession) -> String {
    session
        .messages
        .iter()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|m| {
            let who = match m.role {
                CharacterBuilderRole::User => "User",
                CharacterBuilderRole::Assistant => "Assistant",
            };
            format!("{who}: {}", m.content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
