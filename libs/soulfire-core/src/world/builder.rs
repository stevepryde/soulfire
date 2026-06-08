//! The conversational world builder (`WORLD-20`, `WORLD-21`).

use std::sync::Arc;

use serde::Deserialize;

use crate::model::ai_model::AiModel;
use crate::model::ids::{WorldBlueprintId, WorldBuilderMessageId, WorldBuilderSnapshotId};
use crate::model::metric::{MetricLabel, UsageMetric};
use crate::model::strings::{WorldDescription, WorldPrompt, WorldTitle};
use crate::model::world::{
    WorldBlueprint, WorldBuilderMessage, WorldBuilderRole, WorldBuilderSession,
    WorldBuilderSnapshot,
};

use crate::ai::fence::parse_lenient;
use crate::ai::registry::resolve_model;
use crate::ai::service::AiService;
use crate::ai::types::{GenerationConfig, GenerationRequest, JsonMode, PromptMessage, Usage};
use crate::clock::Clock;
use crate::error::{CoreError, CoreResult};
use crate::store::Store;

const BUILDER_TEMPERATURE: f64 = 0.8;
const BUILDER_MAX_TOKENS: u32 = 6000;

/// The structured result of a world-builder turn (`WORLD-21`). Null = unchanged.
#[derive(Debug, Clone, Deserialize)]
pub struct WorldBuilderResult {
    pub assistant_message: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub world_prompt: Option<String>,
}

impl WorldBuilderResult {
    fn changes_blueprint(&self) -> bool {
        self.title.is_some() || self.description.is_some() || self.world_prompt.is_some()
    }
}

/// Drives the conversational world builder.
#[derive(Clone)]
pub struct WorldBuilderEngine {
    store: Arc<Store>,
    ai: AiService,
    clock: Arc<dyn Clock>,
}

impl WorldBuilderEngine {
    pub fn new(store: Arc<Store>, ai: AiService, clock: Arc<dyn Clock>) -> Self {
        WorldBuilderEngine { store, ai, clock }
    }

    /// Send a builder message; the assistant replies and may revise the world
    /// blueprint. A snapshot is captured before applying changes (`WORLD-21`).
    pub async fn builder_send(
        &self,
        blueprint_id: &WorldBlueprintId,
        user_message: &str,
    ) -> CoreResult<WorldBuilderResult> {
        let mut blueprint = self
            .store
            .blueprint(blueprint_id)?
            .ok_or_else(|| CoreError::NotFound(blueprint_id.to_string()))?;
        let mut session = self
            .store
            .world_builder_session(blueprint_id)?
            .unwrap_or_else(|| WorldBuilderSession {
                blueprint_id: blueprint_id.clone(),
                ..Default::default()
            });

        let recent = recent_text(&session);
        session.push_message(self.message(WorldBuilderRole::User, user_message));

        let model = resolve_model(
            None,
            self.store.app_profile()?.default_ai_model,
            AiModel::default_chat_narrative(),
        );
        let request = GenerationRequest {
            model,
            instructions: Some(builder_instructions()),
            messages: vec![PromptMessage::user(builder_input(
                blueprint.title.as_str(),
                blueprint.description.as_str(),
                blueprint.world_prompt.as_str(),
                &recent,
                user_message,
            ))],
            config: GenerationConfig {
                max_output_tokens: Some(BUILDER_MAX_TOKENS),
                temperature: Some(BUILDER_TEMPERATURE),
                json: Some(JsonMode::Json),
                ..Default::default()
            },
        };
        let response = self.ai.generate(request).await?;
        self.meter(model, response.usage, blueprint_id)?;

        let result: WorldBuilderResult = parse_lenient(&response.text)
            .map_err(|e| CoreError::Serialization(format!("world builder response: {e}")))?;
        session.push_message(self.message(WorldBuilderRole::Assistant, &result.assistant_message));

        if result.changes_blueprint() {
            session.push_snapshot(self.snapshot_of(&blueprint));
            if let Some(t) = &result.title {
                blueprint.title = WorldTitle::coerce(t);
            }
            if let Some(d) = &result.description {
                blueprint.description = WorldDescription::coerce(d);
            }
            if let Some(p) = &result.world_prompt {
                blueprint.world_prompt = WorldPrompt::coerce(p);
            }
            blueprint.updated_at = self.clock.now();
            self.store.save_blueprint(&blueprint)?;
        }
        self.store.save_world_builder_session(&session)?;
        Ok(result)
    }

    /// Undo the most recent builder change (`WORLD-21`). `false` if nothing to undo.
    pub fn builder_undo(&self, blueprint_id: &WorldBlueprintId) -> CoreResult<bool> {
        let Some(mut session) = self.store.world_builder_session(blueprint_id)? else {
            return Ok(false);
        };
        let Some(snapshot) = session.pop_snapshot() else {
            return Ok(false);
        };
        let mut blueprint = self
            .store
            .blueprint(blueprint_id)?
            .ok_or_else(|| CoreError::NotFound(blueprint_id.to_string()))?;
        blueprint.title = snapshot.title;
        blueprint.description = snapshot.description;
        blueprint.world_prompt = snapshot.world_prompt;
        blueprint.updated_at = self.clock.now();
        self.store.save_blueprint(&blueprint)?;
        session
            .push_message(self.message(WorldBuilderRole::Assistant, "Reverted the last change."));
        self.store.save_world_builder_session(&session)?;
        Ok(true)
    }

    fn snapshot_of(&self, bp: &WorldBlueprint) -> WorldBuilderSnapshot {
        WorldBuilderSnapshot {
            snapshot_id: WorldBuilderSnapshotId::new(),
            title: bp.title.clone(),
            description: bp.description.clone(),
            world_prompt: bp.world_prompt.clone(),
            captured_at: self.clock.now(),
        }
    }

    fn message(&self, role: WorldBuilderRole, content: &str) -> WorldBuilderMessage {
        WorldBuilderMessage {
            message_id: WorldBuilderMessageId::new(),
            role,
            content: content.to_string(),
            created_at: self.clock.now(),
        }
    }

    fn meter(
        &self,
        model: AiModel,
        usage: Usage,
        blueprint_id: &WorldBlueprintId,
    ) -> CoreResult<()> {
        let metric = UsageMetric::builder()
            .created_at(self.clock.now())
            .label(MetricLabel::WorldBuilder)
            .blueprint_id(blueprint_id.clone())
            .input_tokens(usage.input_tokens)
            .output_tokens(usage.output_tokens)
            .maybe_cached_input_tokens(usage.cached_input_tokens)
            .ai_model(model)
            .build();
        self.store.save_metric(&metric)
    }
}

fn recent_text(session: &WorldBuilderSession) -> String {
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
                WorldBuilderRole::User => "User",
                WorldBuilderRole::Assistant => "Assistant",
            };
            format!("{who}: {}", m.content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The collaborative world-builder instructions (`WORLD-20`), verbatim (`PROD-7`).
pub fn builder_instructions() -> String {
    r#"You are Soulfire's collaborative world builder.
Help the user create richer interactive adventure worlds. You may answer questions, ask focused follow-up questions, or directly revise the world blueprint.

When the user asks for a one-shot, generate a complete, play-ready world prompt.
Otherwise, scaffold thoughtfully and ask the next best question so the world can be built section by section.
World prompts should be structured, concise, and useful to an AI game master. Prefer sections such as premise, player fantasy, lore, locations, factions, NPCs, rules, quests, secrets, tone, and adventure hooks.
Soulfire is a private adult roleplay app. World prompts may include mature themes, explicit erotic language, dark romance, kink, violence, and intense emotional material when requested. Preserve the user's intent, tone, and explicit terminology instead of sanitizing, euphemizing, moralizing, adding generic safety disclaimers, or inserting artificial consent-gating rules. Only narrow content that would violate provider rules or the user's own requested constraints.
Keep title under 200 characters, description under 1000 characters, and world_prompt under 50000 characters.

Return only JSON with this exact shape:
{
  "assistant_message": "Brief conversational response to show in chat.",
  "title": null or "full replacement title",
  "description": null or "full replacement description",
  "world_prompt": null or "full replacement world prompt"
}

Set a field to null when it should not change. If you change the prompt, return the complete replacement world_prompt, not a patch."#
        .to_string()
}

fn builder_input(
    title: &str,
    description: &str,
    world_prompt: &str,
    recent: &str,
    user_message: &str,
) -> String {
    format!(
        "Current world:\nTitle: {title}\nDescription: {description}\nWorld prompt:\n{world_prompt}\n\nRecent builder chat:\n{recent}\n\nLatest user message:\n{user_message}"
    )
}
