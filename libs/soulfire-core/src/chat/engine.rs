//! The chat engine: opening a chat, streaming replies, reactions, the rolling
//! summary, and the background character-state updater (`CHAT-1`..`CHAT-14`).

use std::sync::Arc;
use std::time::Duration;

use crate::model::ai_model::AiModel;
use crate::model::character::{Character, InitialMessage};
use crate::model::chat::{AI_REACTOR, Chat, ChatMessage, Sender};
use crate::model::ids::{CharacterId, ChatId};
use crate::model::metric::{MetricLabel, UsageMetric};
use crate::model::strings::{CharacterContext, ChatTitle, MessageString, StorySummary};

use crate::ai::collect_streamed;
use crate::ai::registry::resolve_model;
use crate::ai::service::AiService;
use crate::ai::types::{GenerationConfig, GenerationRequest, PromptMessage, Usage};
use crate::clock::Clock;
use crate::error::{CoreError, CoreResult};
use crate::prompt::{CharacterPromptInput, build_character_prompt};
use crate::store::Store;

use super::coalesce::{Coalescer, Decision};
use super::history::to_history_messages;
use super::prompts;
use super::sanitise::sanitise_reply;

/// Bound on the prompt history sent to the model (`CHAT-5`).
pub const MAX_HISTORY_MESSAGES: u32 = 20;
/// Messages between rolling-summary regenerations (`CHAT-10`).
pub const SUMMARY_INTERVAL: u32 = 20;
/// Window of recent messages folded into a summary (`CHAT-10`).
pub const SUMMARY_WINDOW: u32 = 40;
/// Window of recent messages used for a character-state update (`CHAT-12`).
pub const STATE_UPDATE_WINDOW: u32 = 10;
/// Floor on generated output tokens so replies are not truncated (`CHAT-7`).
pub const MIN_OUTPUT_TOKENS: u32 = 2000;
/// Idle timeout for a streamed reply (`CHAT-6`).
pub const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
/// Temperature for the character-state updater (`CHAT-12`).
pub const STATE_UPDATE_TEMPERATURE: f64 = 0.3;

/// The result of sending a player message (`CHAT-4`).
#[derive(Debug, Clone)]
pub struct SendOutcome {
    pub player_message: ChatMessage,
    pub reply: ChatMessage,
    /// The rolling summary is due — the caller should run [`ChatEngine::generate_summary`]
    /// as a background pass (`CHAT-10`, `CHAT-11`).
    pub summary_due: bool,
    /// A character-state update is due — the caller should queue it (`CHAT-12`).
    pub state_update_due: bool,
}

/// Orchestrates character chat over the store, AI service, and clock.
#[derive(Clone)]
pub struct ChatEngine {
    store: Arc<Store>,
    ai: AiService,
    clock: Arc<dyn Clock>,
    coalescer: Arc<Coalescer>,
}

impl ChatEngine {
    pub fn new(store: Arc<Store>, ai: AiService, clock: Arc<dyn Clock>) -> Self {
        ChatEngine {
            store,
            ai,
            clock,
            coalescer: Arc::new(Coalescer::new()),
        }
    }

    /// Open the chat for a character, creating it (with an opening message) if it
    /// does not exist (`CHAT-1`, `CHAT-2`). Reopening returns the existing chat.
    pub async fn open_chat(&self, character_id: &CharacterId) -> CoreResult<Chat> {
        if let Some(existing) = self.store.chat_id_for_character(character_id)? {
            return self
                .store
                .chat(&existing)?
                .ok_or_else(|| CoreError::NotFound(existing.to_string()));
        }
        let character = self.load_character(character_id)?;
        let now = self.clock.now();
        let chat = Chat {
            chat_id: ChatId::new(),
            version: 1,
            started_at: now,
            updated_at: now,
            title: ChatTitle::default(),
            character_id: Some(character_id.clone()),
            participants: vec![
                Sender::Player,
                Sender::Character {
                    character_id: character_id.clone(),
                    image: character.image,
                },
            ],
            ai_model: None,
            chat_summary: None,
            messages_since_summary: 0,
        };
        self.store.save_chat(&chat)?;

        // Deliver the opening message (CHAT-2).
        let opening_text = match &character.initial_message {
            InitialMessage::Message(text) => text.to_string(),
            InitialMessage::Prompt(seed) => {
                self.generate_opening(&chat, &character, seed.as_str())
                    .await?
            }
        };
        let opening = self.new_character_message(&chat, &character, &opening_text, 0);
        self.store.save_chat_message(&opening)?;
        Ok(chat)
    }

    /// Send a player message and stream the character's reply (`CHAT-4`..`CHAT-9`).
    /// `on_delta` receives each streamed text delta for live rendering.
    pub async fn send_message<F>(
        &self,
        chat_id: &ChatId,
        text: &str,
        on_delta: F,
    ) -> CoreResult<SendOutcome>
    where
        F: FnMut(&str),
    {
        let mut chat = self
            .store
            .chat(chat_id)?
            .ok_or_else(|| CoreError::NotFound(chat_id.to_string()))?;
        let character_id = chat
            .character_id
            .clone()
            .ok_or_else(|| CoreError::Validation("chat has no character".into()))?;
        let mut character = self.load_character(&character_id)?;

        // (a) Persist and show the player message immediately (CHAT-4).
        let now = self.clock.now();
        let player_message = ChatMessage::builder()
            .chat_id(chat_id.clone())
            .created_at(now)
            .sender(Sender::Player)
            .message(MessageString::coerce(text))
            .build();
        self.store.save_chat_message(&player_message)?;

        // Resolve and persist the chat model (AI-9).
        let model = resolve_model(
            chat.ai_model,
            self.store.app_profile()?.default_ai_model,
            AiModel::default_chat_narrative(),
        );
        if chat.ai_model.is_none() {
            chat.ai_model = Some(model);
        }

        // Build the prompt: persona instructions + bounded history (CHAT-5).
        let input = self.character_prompt_input(&character)?;
        let assembled = build_character_prompt(&input.as_ref());
        let history = self
            .store
            .recent_chat_messages(chat_id, MAX_HISTORY_MESSAGES)?;
        let messages = to_history_messages(&history);

        let request = GenerationRequest {
            model,
            instructions: Some(assembled.instructions()),
            messages,
            config: GenerationConfig {
                max_output_tokens: Some(character.creativity.max_tokens.max(MIN_OUTPUT_TOKENS)),
                temperature: Some(character.creativity.temperature),
                top_p: Some(character.creativity.top_p),
                top_k: Some(character.creativity.top_k),
                reasoning_effort: None,
                json: None,
                cache_hint: true,
            },
        };

        // (c) Stream the reply with the idle timeout (CHAT-4, CHAT-6).
        let stream = self.ai.generate_stream(request).await?;
        let response = collect_streamed(stream, STREAM_IDLE_TIMEOUT, on_delta).await?;

        // (d) Post-process: extract a trailing reaction, normalize lists (CHAT-8).
        let (clean, reaction) = sanitise_reply(&response.text);
        let reply_time = self.clock.now();
        let reply = self.new_character_message_at(
            &chat,
            &character,
            &clean,
            response.usage.output_tokens as u32,
            reply_time,
        );
        self.store.save_chat_message(&reply)?;

        // Record the character's reaction to the player's message (CHAT-8).
        if let Some(emoji) = reaction {
            let mut pm = player_message.clone();
            pm.emoji_reactions.set(AI_REACTOR, emoji);
            self.store.save_chat_message(&pm)?;
        }

        // Metering (AI-15).
        self.meter(
            MetricLabel::ChatMessage,
            model,
            response.usage,
            Some(chat_id),
            Some(&character_id),
        )?;

        // Advance timestamps (CHAT-4).
        character.last_chatted_at = Some(reply_time);
        character.updated_at = reply_time;
        self.store.save_character(&character)?;
        chat.updated_at = reply_time;
        chat.messages_since_summary += 2;

        // Auto-title from the first exchange (CHAT-3).
        if chat.title.as_str().is_empty() {
            if let Ok(title) = self.generate_title(&chat, text).await {
                chat.title = title;
            }
        }

        let summary_due = chat.messages_since_summary >= SUMMARY_INTERVAL;
        self.store.save_chat(&chat)?;

        Ok(SendOutcome {
            player_message,
            reply,
            summary_due,
            state_update_due: character.is_world_extracted(),
        })
    }

    /// Regenerate the rolling conversation summary from the recent window and
    /// reset the counter (`CHAT-10`, `CHAT-11`). A failure leaves the prior
    /// summary and conversation intact.
    pub async fn generate_summary(&self, chat_id: &ChatId) -> CoreResult<()> {
        let mut chat = self
            .store
            .chat(chat_id)?
            .ok_or_else(|| CoreError::NotFound(chat_id.to_string()))?;
        let messages = self.store.recent_chat_messages(chat_id, SUMMARY_WINDOW)?;
        let (player_name, character_name) = self.participant_names(&chat)?;
        let conversation = prompts::conversation_text(&messages, &player_name, &character_name);
        let prompt = prompts::summary_prompt(
            chat.chat_summary.as_ref().map(|s| s.as_str()),
            &conversation,
        );

        let model = chat.ai_model.unwrap_or_else(AiModel::default_state_utility);
        let request = GenerationRequest {
            model,
            instructions: None,
            messages: vec![PromptMessage::developer(prompt)],
            config: GenerationConfig::default(),
        };
        let response = self.ai.generate(request).await?; // Err leaves summary intact
        self.meter(
            MetricLabel::ChatSummary,
            model,
            response.usage,
            Some(chat_id),
            None,
        )?;

        chat.chat_summary = Some(StorySummary::coerce(&response.text));
        chat.messages_since_summary = 0;
        self.store.save_chat(&chat)?;
        Ok(())
    }

    /// Queue a coalesced character-state update (`CHAT-12`, `CHAT-13`). Only one
    /// update runs at a time per character; concurrent requests collapse to one
    /// pending run. Returns immediately; the work runs on a spawned task.
    pub fn queue_character_state_update(self: &Arc<Self>, character_id: CharacterId) {
        let key = character_id.to_string();
        if self.coalescer.request(&key) == Decision::Coalesced {
            return; // a run is in flight; one pending run is now recorded
        }
        let engine = self.clone();
        tokio::spawn(async move {
            let key = character_id.to_string();
            loop {
                if let Err(e) = engine.run_character_state_update(&character_id).await {
                    tracing::warn!("character-state update failed: {e}");
                }
                if !engine.coalescer.finish(&key) {
                    break;
                }
            }
        });
    }

    /// Run one character-state update pass for a world-extracted character
    /// (`CHAT-12`). A failure leaves the prior `character_state` intact.
    pub async fn run_character_state_update(&self, character_id: &CharacterId) -> CoreResult<()> {
        let mut character = self.load_character(character_id)?;
        let (Some(profile), Some(state)) = (
            character.extracted_context.clone(),
            character.character_state.clone(),
        ) else {
            return Ok(()); // not a world-extracted character; nothing to do
        };
        let Some(chat_id) = self.store.chat_id_for_character(character_id)? else {
            return Ok(());
        };
        let messages = self
            .store
            .recent_chat_messages(&chat_id, STATE_UPDATE_WINDOW)?;
        let chat = self.store.chat(&chat_id)?;
        let (player_name, character_name) = match &chat {
            Some(c) => self.participant_names(c)?,
            None => ("Player".to_string(), character.name.to_string()),
        };
        let conversation = prompts::conversation_text(&messages, &player_name, &character_name);
        let prompt = prompts::state_update_prompt(
            character.name.as_str(),
            profile.as_str(),
            state.as_str(),
            &conversation,
        );

        let model = AiModel::default_state_utility();
        let request = GenerationRequest {
            model,
            instructions: None,
            messages: vec![
                PromptMessage::developer(prompt),
                PromptMessage::user(
                    "Update the character's dynamic state based on the recent conversation.",
                ),
            ],
            config: GenerationConfig {
                max_output_tokens: Some(2000),
                temperature: Some(STATE_UPDATE_TEMPERATURE),
                top_p: Some(0.95),
                top_k: Some(3),
                reasoning_effort: None,
                json: None,
                cache_hint: false,
            },
        };
        let response = self.ai.generate(request).await?; // Err leaves state intact
        self.meter(
            MetricLabel::CharacterStateUpdate,
            model,
            response.usage,
            Some(&chat_id),
            Some(character_id),
        )?;

        character.character_state = Some(CharacterContext::coerce(response.text.trim()));
        character.updated_at = self.clock.now();
        self.store.save_character(&character)?;
        Ok(())
    }

    // ----- internals -----

    async fn generate_opening(
        &self,
        chat: &Chat,
        character: &Character,
        seed: &str,
    ) -> CoreResult<String> {
        let model = resolve_model(
            chat.ai_model,
            self.store.app_profile()?.default_ai_model,
            AiModel::default_chat_narrative(),
        );
        let input = self.character_prompt_input(character)?;
        let assembled = build_character_prompt(&input.as_ref());
        let request = GenerationRequest {
            model,
            instructions: Some(assembled.instructions()),
            messages: vec![PromptMessage::user(format!(
                "Begin the conversation with your opening message. Guidance: {seed}"
            ))],
            config: GenerationConfig {
                max_output_tokens: Some(character.creativity.max_tokens.max(MIN_OUTPUT_TOKENS)),
                temperature: Some(character.creativity.temperature),
                top_p: Some(character.creativity.top_p),
                top_k: Some(character.creativity.top_k),
                reasoning_effort: None,
                json: None,
                cache_hint: true,
            },
        };
        let response = self.ai.generate(request).await?;
        if let Some(cid) = chat.character_id.as_ref() {
            self.meter(
                MetricLabel::ChatMessage,
                model,
                response.usage,
                Some(&chat.chat_id),
                Some(cid),
            )?;
        }
        let (clean, _) = sanitise_reply(&response.text);
        Ok(clean)
    }

    async fn generate_title(&self, chat: &Chat, first_message: &str) -> CoreResult<ChatTitle> {
        let model = chat.ai_model.unwrap_or_else(AiModel::default_state_utility);
        let request = GenerationRequest {
            model,
            instructions: None,
            messages: vec![PromptMessage::developer(prompts::title_prompt(
                first_message,
            ))],
            config: GenerationConfig::default(),
        };
        let response = self.ai.generate(request).await?;
        self.meter(
            MetricLabel::ChatSummary,
            model,
            response.usage,
            Some(&chat.chat_id),
            None,
        )?;
        Ok(ChatTitle::coerce(response.text.trim()))
    }

    fn load_character(&self, id: &CharacterId) -> CoreResult<Character> {
        self.store
            .character(id)?
            .ok_or_else(|| CoreError::NotFound(id.to_string()))
    }

    /// Owned prompt-input strings for a character, loading any linked world.
    fn character_prompt_input(&self, character: &Character) -> CoreResult<OwnedPromptInput> {
        let mut input = OwnedPromptInput {
            character_prompt: character.prompt.to_string(),
            extracted_context: character.extracted_context.as_ref().map(|c| c.to_string()),
            character_state: character.character_state.as_ref().map(|c| c.to_string()),
            is_adventure_linked: character.source_adventure_id.is_some(),
            world_context: None,
            world_state: None,
            story_so_far: None,
            toggles: self.store.app_settings()?.content_toggles,
        };
        if let Some(bp_id) = &character.source_blueprint_id {
            if let Some(bp) = self.store.blueprint(bp_id)? {
                input.world_context = Some(bp.world_prompt.to_string());
            }
        }
        if let Some(adv_id) = &character.source_adventure_id {
            if let Some(adv) = self.store.adventure(adv_id)? {
                input.world_state = Some(adv.adventure_state.to_string());
                if !adv.story_summary.as_str().is_empty() {
                    input.story_so_far = Some(adv.story_summary.to_string());
                }
            }
        }
        Ok(input)
    }

    fn participant_names(&self, chat: &Chat) -> CoreResult<(String, String)> {
        let player = self.store.player_profile()?.player_name.to_string();
        let player = if player.is_empty() {
            "Player".to_string()
        } else {
            player
        };
        let character = match &chat.character_id {
            Some(cid) => self
                .store
                .character(cid)?
                .map(|c| c.name.to_string())
                .unwrap_or_else(|| "Character".to_string()),
            None => "Character".to_string(),
        };
        Ok((player, character))
    }

    fn new_character_message(
        &self,
        chat: &Chat,
        character: &Character,
        text: &str,
        tokens: u32,
    ) -> ChatMessage {
        self.new_character_message_at(chat, character, text, tokens, self.clock.now())
    }

    fn new_character_message_at(
        &self,
        chat: &Chat,
        character: &Character,
        text: &str,
        tokens: u32,
        at: crate::datetime::SfDateTime,
    ) -> ChatMessage {
        ChatMessage::builder()
            .chat_id(chat.chat_id.clone())
            .created_at(at)
            .sender(Sender::Character {
                character_id: character.character_id.clone(),
                image: character.image,
            })
            .message(MessageString::coerce(text))
            .token_count(tokens)
            .build()
    }

    fn meter(
        &self,
        label: MetricLabel,
        model: AiModel,
        usage: Usage,
        chat_id: Option<&ChatId>,
        character_id: Option<&CharacterId>,
    ) -> CoreResult<()> {
        let metric = UsageMetric::builder()
            .created_at(self.clock.now())
            .label(label)
            .maybe_chat_id(chat_id.cloned())
            .maybe_character_id(character_id.cloned())
            .input_tokens(usage.input_tokens)
            .output_tokens(usage.output_tokens)
            .maybe_cached_input_tokens(usage.cached_input_tokens)
            .ai_model(model)
            .build();
        self.store.save_metric(&metric) // skips zero-token records
    }
}

/// Owned strings backing a [`CharacterPromptInput`] (borrows would dangle across
/// the awaits in the engine).
struct OwnedPromptInput {
    character_prompt: String,
    extracted_context: Option<String>,
    character_state: Option<String>,
    is_adventure_linked: bool,
    world_context: Option<String>,
    world_state: Option<String>,
    story_so_far: Option<String>,
    toggles: crate::model::settings::ContentToggles,
}

impl OwnedPromptInput {
    fn as_ref(&self) -> CharacterPromptInput<'_> {
        CharacterPromptInput {
            character_prompt: &self.character_prompt,
            extracted_context: self.extracted_context.as_deref(),
            character_state: self.character_state.as_deref(),
            is_adventure_linked: self.is_adventure_linked,
            world_context: self.world_context.as_deref(),
            world_state: self.world_state.as_deref(),
            story_so_far: self.story_so_far.as_deref(),
            toggles: self.toggles,
        }
    }
}
