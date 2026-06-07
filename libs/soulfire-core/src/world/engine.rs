//! The adventure turn engine (`WORLD-3`, `WORLD-5`, `WORLD-6`, `WORLD-11`,
//! `WORLD-15`..`WORLD-19`).

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use lib_soulfire::ai_model::AiModel;
use lib_soulfire::ids::AdventureId;
use lib_soulfire::metric::{MetricLabel, UsageMetric};
use lib_soulfire::strings::{
    AdventureState, MessageContent, RecentSummary, SignificantEvents, StorySummary, WorldPrompt,
};
use lib_soulfire::world::{
    Adventure, AdventureMessage, AdventureMessageType, AdventureReadyStatus, GmChangeTarget,
    GmDiffEntry, GmProposal, GmProposalStatus, StoryStatus, WorldBlueprint,
};

use crate::ai::collect_streamed;
use crate::ai::registry::resolve_model;
use crate::ai::service::AiService;
use crate::ai::types::{GenerationConfig, GenerationRequest, JsonMode, Usage};
use crate::clock::Clock;
use crate::error::{CoreError, CoreResult};
use crate::store::Store;

use super::input::{TurnInput, parse_turn_input};
use super::memory::{
    self, SignificantEvent, next_id_from_events, parse_significant_events,
    serialize_significant_events,
};
use super::prompts;
use super::response::{self, GmIntent};
use super::state_patch::{PatchResult, apply_patches};

/// Diff updates accumulated before forcing a full reconciliation (`WORLD-11`).
pub const FULL_STATE_UPDATE_THRESHOLD: u32 = 15;
/// Live-state size that triggers a compaction directive (`WORLD-11`).
pub const LARGE_STATE_COMPACTION_THRESHOLD: usize = 10_000;
/// Stale-lock expiry so a crashed turn self-heals (`WORLD-5`).
pub const READY_STATUS_EXPIRY_SECS: i64 = 90;
/// Temperature for state-update calls (`WORLD-11`).
pub const STATE_UPDATE_TEMPERATURE: f64 = 0.15;
/// Temperature for narration (`WORLD` design notes).
pub const NARRATION_TEMPERATURE: f64 = 0.9;
const NARRATION_MAX_TOKENS: u32 = 2048;
const FULL_STATE_MAX_TOKENS: u32 = 24_576;
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// The result of a turn (`WORLD-6`, `WORLD-16`).
#[derive(Debug, Clone)]
pub enum TurnOutcome {
    /// A normal narration turn.
    Narration {
        message: AdventureMessage,
        story_status: StoryStatus,
        /// The state-update phase failed but the narration was kept (`WORLD-5`).
        state_update_failed: bool,
    },
    /// An answer-only `/gm` response (no state change).
    GmAnswer { message: AdventureMessage },
    /// A staged `/gm` change proposal awaiting Accept/Reject (`WORLD-17`).
    GmProposal {
        message: AdventureMessage,
        proposal: GmProposal,
    },
    /// A warning (empty/unknown command, `WORLD-15`).
    Warning(String),
}

/// The adventure turn engine.
#[derive(Clone)]
pub struct WorldEngine {
    store: Arc<Store>,
    ai: AiService,
    clock: Arc<dyn Clock>,
}

impl WorldEngine {
    pub fn new(store: Arc<Store>, ai: AiService, clock: Arc<dyn Clock>) -> Self {
        WorldEngine { store, ai, clock }
    }

    /// Start an adventure from a blueprint (`WORLD-3`, `WORLD-4`).
    pub async fn start_adventure<F: FnMut(&str)>(
        &self,
        blueprint: &WorldBlueprint,
        mut _on_delta: F,
    ) -> CoreResult<Adventure> {
        let model = resolve_model(
            None,
            self.store.app_profile()?.default_ai_model,
            AiModel::default_chat_narrative(),
        );
        let player = self.store.player_profile()?;
        let section = prompts::player_profile_section(
            player.player_name.as_str(),
            player.player_attributes.as_str(),
            player.prompt_extension.as_ref().map(|p| p.as_str()),
        );

        // Intro narrative (WORLD-3).
        let intro_req = GenerationRequest {
            model,
            instructions: Some(prompts::intro_narrative_instructions(
                blueprint.world_prompt.as_str(),
            )),
            messages: vec![crate::ai::types::PromptMessage::developer(section.clone())],
            config: GenerationConfig {
                max_output_tokens: Some(NARRATION_MAX_TOKENS),
                temperature: Some(NARRATION_TEMPERATURE),
                cache_hint: true,
                ..Default::default()
            },
        };
        let intro = self.ai.generate(intro_req).await?;
        self.meter(
            MetricLabel::AdventureAction,
            model,
            intro.usage,
            None,
            Some(&blueprint.blueprint_id),
        )?;

        // Initial state (WORLD-3): only what the player would know.
        let state_req = GenerationRequest {
            model,
            instructions: Some(prompts::initial_state_instructions(
                blueprint.world_prompt.as_str(),
            )),
            messages: vec![
                crate::ai::types::PromptMessage::developer(section),
                crate::ai::types::PromptMessage::developer(format!(
                    "# Introduction Narrative: {}",
                    intro.text
                )),
            ],
            config: GenerationConfig {
                max_output_tokens: Some(FULL_STATE_MAX_TOKENS),
                temperature: Some(STATE_UPDATE_TEMPERATURE),
                json: Some(JsonMode::Json),
                ..Default::default()
            },
        };
        let state_resp = self.ai.generate(state_req).await?;
        self.meter(
            MetricLabel::AdventureFullStateUpdate,
            model,
            state_resp.usage,
            None,
            Some(&blueprint.blueprint_id),
        )?;
        let initial_state = response::rescue_state_string(&state_resp.text);

        let now = self.clock.now();
        let adventure = Adventure::builder()
            .blueprint_id(blueprint.blueprint_id.clone())
            .created_at(now)
            .updated_at(now)
            .maybe_world_title(Some(blueprint.title.clone()))
            .maybe_world_description(Some(blueprint.description.clone()))
            .maybe_world_image(blueprint.image)
            .world_image_transform(blueprint.image_transform)
            .world_prompt(blueprint.world_prompt.clone())
            .maybe_player_name(
                (!player.player_name.as_str().is_empty()).then(|| player.player_name.clone()),
            )
            .maybe_player_attributes(
                (!player.player_attributes.as_str().is_empty())
                    .then(|| player.player_attributes.clone()),
            )
            .ai_model(model)
            .adventure_state(AdventureState::coerce(&initial_state))
            .previous_narrative(intro.text.clone())
            .build();
        self.store.save_adventure(&adventure)?;

        // Persist the intro as the first narration message (WORLD-4).
        let intro_msg = self.new_message(
            &adventure.adventure_id,
            AdventureMessageType::Narration,
            &intro.text,
        );
        self.store.save_adventure_message(&intro_msg)?;
        Ok(adventure)
    }

    /// Take a turn: dispatch plain actions, `/gm` requests, and warnings
    /// (`WORLD-5`, `WORLD-15`, `WORLD-16`).
    pub async fn take_turn<F: FnMut(&str)>(
        &self,
        adventure_id: &AdventureId,
        raw_input: &str,
        on_delta: F,
    ) -> CoreResult<TurnOutcome> {
        match parse_turn_input(raw_input) {
            TurnInput::Action(action) => {
                self.run_action_turn(adventure_id, &action, on_delta).await
            }
            TurnInput::GmRequest(request) => self.run_gm_request(adventure_id, &request).await,
            TurnInput::GmEmpty => Ok(TurnOutcome::Warning(
                "Add a request after /gm, e.g. `/gm skip to morning`.".to_string(),
            )),
            TurnInput::Unknown(cmd) => Ok(TurnOutcome::Warning(format!("Unknown command: {cmd}"))),
        }
    }

    /// A normal player action turn (`WORLD-5`, `WORLD-6`).
    async fn run_action_turn<F: FnMut(&str)>(
        &self,
        adventure_id: &AdventureId,
        action: &str,
        on_delta: F,
    ) -> CoreResult<TurnOutcome> {
        let mut adventure = self.load(adventure_id)?;
        self.claim_lock(&mut adventure, AdventureReadyStatus::UpdatingNarrative)?;

        // (a) Persist the player's action immediately.
        let user_msg = self.new_message(adventure_id, AdventureMessageType::UserAction, action);
        self.store.save_adventure_message(&user_msg)?;

        // (c) Narration, streamed (WORLD-5).
        let model = adventure
            .ai_model
            .unwrap_or_else(AiModel::default_chat_narrative);
        let adult = self.store.app_settings()?.content_toggles.adult_content;
        let player_ext = self
            .store
            .player_profile()?
            .prompt_extension
            .map(|p| p.to_string());
        let narration_req = GenerationRequest {
            model,
            instructions: Some(prompts::narrative_instructions(
                adventure.world_prompt.as_str(),
                player_ext.as_deref(),
                adult,
            )),
            messages: prompts::narrative_input(
                adventure.significant_events.as_str(),
                adventure.adventure_state.as_str(),
                adventure.story_summary.as_str(),
                adventure.recent_summary.as_str(),
                adventure.previous_narrative.as_deref().unwrap_or(""),
                action,
            ),
            config: GenerationConfig {
                max_output_tokens: Some(NARRATION_MAX_TOKENS),
                temperature: Some(NARRATION_TEMPERATURE),
                cache_hint: true,
                ..Default::default()
            },
        };
        let stream = self.ai.generate_stream(narration_req).await?;
        let narration = match collect_streamed(stream, STREAM_IDLE_TIMEOUT, on_delta).await {
            Ok(r) => r,
            Err(e) => {
                // Release the lock so a failed turn self-heals.
                adventure.ready_status = AdventureReadyStatus::Ready;
                self.store.save_adventure(&adventure)?;
                return Err(e.into());
            }
        };

        // Commit the narration before the state update (WORLD-5).
        let narration_msg = self.new_message(
            adventure_id,
            AdventureMessageType::Narration,
            &narration.text,
        );
        self.store.save_adventure_message(&narration_msg)?;
        self.meter(
            MetricLabel::AdventureAction,
            model,
            narration.usage,
            Some(adventure_id),
            None,
        )?;
        adventure.previous_narrative = Some(narration.text.clone());
        adventure.updated_at = self.clock.now();
        self.store.save_adventure(&adventure)?;

        // (d) State-update phase — non-fatal (WORLD-5).
        adventure.ready_status = AdventureReadyStatus::UpdatingState;
        self.store.save_adventure(&adventure)?;
        let state_update_failed = self
            .reconcile_state(&mut adventure, action, &narration.text, adult, model)
            .await
            .is_err();

        let story_status = adventure.story_status;
        if story_status.is_terminal() {
            adventure.has_completed = true;
        }
        adventure.ready_status = AdventureReadyStatus::Ready;
        adventure.updated_at = self.clock.now();
        self.store.save_adventure(&adventure)?;

        Ok(TurnOutcome::Narration {
            message: narration_msg,
            story_status,
            state_update_failed,
        })
    }

    /// Reconcile the live state and memory after a turn (`WORLD-11`..`WORLD-13`).
    /// Returns `Err` on a non-fatal failure (the caller keeps the narration).
    async fn reconcile_state(
        &self,
        adventure: &mut Adventure,
        action: &str,
        narrative: &str,
        adult: bool,
        model: AiModel,
    ) -> CoreResult<()> {
        let compaction =
            if adventure.adventure_state.as_str().len() > LARGE_STATE_COMPACTION_THRESHOLD {
                prompts::COMPACTION_PROMPT
            } else {
                ""
            };
        let force_full = adventure.diff_action_count >= FULL_STATE_UPDATE_THRESHOLD;

        let input = prompts::state_update_input(
            adventure.adventure_state.as_str(),
            adventure.recent_summary.as_str(),
            adventure.significant_events.as_str(),
            adventure.story_summary.as_str(),
            action,
            narrative,
        );

        // Try diff first unless a full pass is forced (WORLD-11).
        if !force_full {
            let instructions = prompts::diff_state_update_instructions(
                adventure.world_prompt.as_str(),
                compaction,
                adult,
            );
            if let Ok(resp) = self
                .state_call(
                    model,
                    instructions,
                    input.clone(),
                    MetricLabel::AdventureDiffStateUpdate,
                    adventure,
                )
                .await
            {
                if let Ok(diff) = response::parse_diff_update(&resp.text) {
                    let current: Value = serde_json::from_str(adventure.adventure_state.as_str())
                        .unwrap_or(Value::Object(Default::default()));
                    if let PatchResult::Success(new_state) = apply_patches(&current, &diff.patches)
                    {
                        self.apply_diff_memory(adventure, &new_state, &diff);
                        adventure.diff_action_count += 1;
                        return Ok(());
                    }
                }
                // Diff failed to parse or validate → fall through to full.
            }
        }

        // Full reconciliation (forced, or diff fallback) (WORLD-11, WORLD-13).
        let instructions = prompts::full_state_update_instructions(
            adventure.world_prompt.as_str(),
            compaction,
            adult,
        );
        let resp = self
            .state_call(
                model,
                instructions,
                input,
                MetricLabel::AdventureFullStateUpdate,
                adventure,
            )
            .await?;
        let next_id = next_id_from_events(&parse_significant_events(
            adventure.significant_events.as_str(),
        ));
        let full = response::parse_full_update(&resp.text, next_id)?;
        self.apply_full_memory(adventure, full);
        adventure.diff_action_count = 0;
        Ok(())
    }

    /// Apply a diff update's memory + new state (`WORLD-12`), with the no-wipe
    /// guard (`WORLD-10`).
    fn apply_diff_memory(
        &self,
        adventure: &mut Adventure,
        new_state: &Value,
        diff: &response::DiffUpdate,
    ) {
        adventure.adventure_state =
            AdventureState::coerce(&serde_json::to_string(new_state).unwrap_or_default());

        // Recent events: prepend, cap 20.
        let existing = memory::parse_recent_events(adventure.recent_summary.as_str());
        let merged = memory::merge_recent_events(&existing, &diff.new_recent_events);
        self.set_recent(adventure, merged, &existing);

        // Significant events: apply add/update/remove, prune.
        let existing_sig = parse_significant_events(adventure.significant_events.as_str());
        let next_id = next_id_from_events(&existing_sig);
        let (mut updated, _next) = memory::apply_significant_event_updates(
            &existing_sig,
            &diff.significant_event_updates,
            next_id,
        );
        let prune_next = next_id_from_events(&updated);
        memory::prune_significant_events(&mut updated, prune_next);
        self.set_significant(adventure, updated, &existing_sig);

        if let Some(summary) = &diff.story_summary {
            if !summary.trim().is_empty() {
                adventure.story_summary = StorySummary::coerce(summary);
            }
        }
        if let Some(status) = diff.story_status {
            adventure.story_status = status;
        }
    }

    /// Apply a full update's memory + state (`WORLD-13`), with the no-wipe guard.
    fn apply_full_memory(&self, adventure: &mut Adventure, full: response::FullUpdate) {
        adventure.adventure_state = AdventureState::coerce(&full.updated_state);
        let prior_recent = memory::parse_recent_events(adventure.recent_summary.as_str());
        self.set_recent(adventure, full.recent_events, &prior_recent);
        let prior_sig = parse_significant_events(adventure.significant_events.as_str());
        let mut sig = full.significant_events;
        let prune_next = next_id_from_events(&sig);
        memory::prune_significant_events(&mut sig, prune_next);
        self.set_significant(adventure, sig, &prior_sig);
        if let Some(summary) = full.story_summary {
            if !summary.trim().is_empty() {
                adventure.story_summary = StorySummary::coerce(&summary);
            }
        }
        if let Some(status) = full.story_status {
            adventure.story_status = status;
        }
    }

    /// Set recent events unless that would wipe existing memory (`WORLD-10`).
    fn set_recent(&self, adventure: &mut Adventure, new: Vec<String>, prior: &[String]) {
        if new.is_empty() && !prior.is_empty() {
            return; // no-wipe guard
        }
        adventure.recent_summary = RecentSummary::coerce(&memory::serialize_recent_events(&new));
    }

    /// Set significant events unless that would wipe existing memory (`WORLD-10`).
    fn set_significant(
        &self,
        adventure: &mut Adventure,
        new: Vec<SignificantEvent>,
        prior: &[SignificantEvent],
    ) {
        if new.is_empty() && !prior.is_empty() {
            return; // no-wipe guard
        }
        adventure.next_significant_event_id = next_id_from_events(&new);
        adventure.significant_events =
            SignificantEvents::coerce(&serialize_significant_events(&new));
    }

    async fn state_call(
        &self,
        model: AiModel,
        instructions: String,
        input: Vec<crate::ai::types::PromptMessage>,
        label: MetricLabel,
        adventure: &Adventure,
    ) -> CoreResult<crate::ai::types::GenerationResponse> {
        let req = GenerationRequest {
            model,
            instructions: Some(instructions),
            messages: input,
            config: GenerationConfig {
                max_output_tokens: Some(FULL_STATE_MAX_TOKENS),
                temperature: Some(STATE_UPDATE_TEMPERATURE),
                json: Some(JsonMode::Json),
                cache_hint: true,
                ..Default::default()
            },
        };
        let resp = self.ai.generate(req).await?;
        self.meter(
            label,
            model,
            resp.usage,
            Some(&adventure.adventure_id),
            None,
        )?;
        Ok(resp)
    }

    // ----- /gm flow (WORLD-16, WORLD-17) -----

    async fn run_gm_request(
        &self,
        adventure_id: &AdventureId,
        request: &str,
    ) -> CoreResult<TurnOutcome> {
        let mut adventure = self.load(adventure_id)?;
        self.claim_lock(&mut adventure, AdventureReadyStatus::UpdatingCommand)?;

        let req_msg = self.new_message(
            adventure_id,
            AdventureMessageType::GameMasterRequest,
            request,
        );
        self.store.save_adventure_message(&req_msg)?;

        let model = adventure
            .ai_model
            .unwrap_or_else(AiModel::default_chat_narrative);
        let adult = self.store.app_settings()?.content_toggles.adult_content;

        // Classify (WORLD-16).
        let classify = self
            .ai
            .generate(GenerationRequest {
                model,
                instructions: Some(prompts::gm_classification_instructions()),
                messages: vec![crate::ai::types::PromptMessage::user(request.to_string())],
                config: GenerationConfig {
                    json: Some(JsonMode::Json),
                    ..Default::default()
                },
            })
            .await?;
        self.meter(
            MetricLabel::GmCommand,
            model,
            classify.usage,
            Some(adventure_id),
            None,
        )?;
        let intent = response::parse_gm_intent(&classify.text);

        let input = prompts::gm_command_input(
            adventure.adventure_state.as_str(),
            "",
            adventure.recent_summary.as_str(),
            adventure.significant_events.as_str(),
            adventure.story_summary.as_str(),
            adventure.previous_narrative.as_deref().unwrap_or(""),
            request,
        );

        if intent == GmIntent::AnswerOnly {
            let resp = self
                .ai
                .generate(GenerationRequest {
                    model,
                    instructions: Some(prompts::gm_answer_instructions(
                        adventure.world_prompt.as_str(),
                    )),
                    messages: input,
                    config: GenerationConfig {
                        json: Some(JsonMode::Json),
                        ..Default::default()
                    },
                })
                .await?;
            self.meter(
                MetricLabel::GmCommand,
                model,
                resp.usage,
                Some(adventure_id),
                None,
            )?;
            let answer = response::parse_gm_answer(&resp.text);
            let msg = self.new_message(
                adventure_id,
                AdventureMessageType::GameMasterResponse,
                &answer,
            );
            self.store.save_adventure_message(&msg)?;
            adventure.ready_status = AdventureReadyStatus::Ready;
            self.store.save_adventure(&adventure)?;
            return Ok(TurnOutcome::GmAnswer { message: msg });
        }

        // Change proposal (WORLD-16, WORLD-17): staged, not applied.
        let resp = self
            .ai
            .generate(GenerationRequest {
                model,
                instructions: Some(prompts::gm_proposal_instructions(
                    adventure.world_prompt.as_str(),
                    intent.as_str(),
                    adult,
                )),
                messages: input,
                config: GenerationConfig {
                    json: Some(JsonMode::Json),
                    ..Default::default()
                },
            })
            .await?;
        self.meter(
            MetricLabel::GmCommand,
            model,
            resp.usage,
            Some(adventure_id),
            None,
        )?;
        let proposal_data = response::parse_gm_proposal(&resp.text)?;

        let msg = self.new_message(
            adventure_id,
            AdventureMessageType::GameMasterResponse,
            &proposal_data.response,
        );
        self.store.save_adventure_message(&msg)?;

        let changes = self.compute_diff(&adventure, &proposal_data);
        let proposal = GmProposal::builder()
            .adventure_id(adventure_id.clone())
            .response_message_id(msg.message_id.clone())
            .created_at(self.clock.now())
            .maybe_proposed_adventure_state(
                proposal_data
                    .updated_adventure_state
                    .as_deref()
                    .map(AdventureState::coerce),
            )
            .maybe_proposed_world_prompt(
                proposal_data
                    .updated_world_blueprint
                    .as_deref()
                    .map(WorldPrompt::coerce),
            )
            .changes(changes)
            .build();
        self.store.save_gm_proposal(&proposal)?;

        adventure.ready_status = AdventureReadyStatus::Ready;
        self.store.save_adventure(&adventure)?;
        Ok(TurnOutcome::GmProposal {
            message: msg,
            proposal,
        })
    }

    /// Accept a staged proposal, applying it (`WORLD-17`). May overwrite the
    /// adventure's private blueprint copy only — never the source blueprint.
    pub async fn accept_proposal(
        &self,
        proposal_id: &lib_soulfire::ids::GmProposalId,
    ) -> CoreResult<()> {
        let mut proposal = self
            .store
            .gm_proposal(proposal_id)?
            .ok_or_else(|| CoreError::NotFound(proposal_id.to_string()))?;
        let mut adventure = self.load(&proposal.adventure_id)?;

        if let Some(state) = &proposal.proposed_adventure_state {
            adventure.adventure_state = state.clone();
        }
        if let Some(prompt) = &proposal.proposed_world_prompt {
            adventure.world_prompt = prompt.clone(); // private copy only (DATA-10)
        }
        if let Some(summary) = &proposal.proposed_story_summary {
            adventure.story_summary = summary.clone();
        }
        adventure.updated_at = self.clock.now();
        self.store.save_adventure(&adventure)?;

        proposal.status = GmProposalStatus::Accepted;
        self.store.save_gm_proposal(&proposal)?;
        Ok(())
    }

    /// Reject a staged proposal; nothing changes (`WORLD-17`).
    pub async fn reject_proposal(
        &self,
        proposal_id: &lib_soulfire::ids::GmProposalId,
    ) -> CoreResult<()> {
        let mut proposal = self
            .store
            .gm_proposal(proposal_id)?
            .ok_or_else(|| CoreError::NotFound(proposal_id.to_string()))?;
        proposal.status = GmProposalStatus::Rejected;
        self.store.save_gm_proposal(&proposal)?;
        Ok(())
    }

    // ----- internals -----

    /// Claim the single-flight turn lock, refusing if a turn is in progress and
    /// the lock is not stale (`WORLD-5`).
    fn claim_lock(
        &self,
        adventure: &mut Adventure,
        status: AdventureReadyStatus,
    ) -> CoreResult<()> {
        let now = self.clock.now();
        if adventure.ready_status != AdventureReadyStatus::Ready {
            let stale = adventure
                .ready_status_updated_at
                .map(|t| now.seconds_since(t) >= READY_STATUS_EXPIRY_SECS)
                .unwrap_or(true);
            if !stale {
                return Err(CoreError::TurnInProgress);
            }
        }
        adventure.ready_status = status;
        adventure.ready_status_updated_at = Some(now);
        self.store.save_adventure(adventure)?;
        Ok(())
    }

    fn load(&self, id: &AdventureId) -> CoreResult<Adventure> {
        self.store
            .adventure(id)?
            .ok_or_else(|| CoreError::NotFound(id.to_string()))
    }

    fn new_message(
        &self,
        adventure_id: &AdventureId,
        message_type: AdventureMessageType,
        content: &str,
    ) -> AdventureMessage {
        AdventureMessage::builder()
            .adventure_id(adventure_id.clone())
            .created_at(self.clock.now())
            .message_type(message_type)
            .content(MessageContent::coerce(content))
            .build()
    }

    /// Compute a human-readable diff for a proposal (`WORLD-17`).
    fn compute_diff(
        &self,
        adventure: &Adventure,
        proposal: &response::GmProposalResponse,
    ) -> Vec<GmDiffEntry> {
        let mut entries = Vec::new();
        if let Some(new_state) = &proposal.updated_adventure_state {
            let before: Value =
                serde_json::from_str(adventure.adventure_state.as_str()).unwrap_or(Value::Null);
            let after: Value = serde_json::from_str(new_state).unwrap_or(Value::Null);
            if let (Some(b), Some(a)) = (before.as_object(), after.as_object()) {
                let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
                keys.sort();
                keys.dedup();
                for key in keys {
                    if b.get(key) != a.get(key) {
                        entries.push(GmDiffEntry {
                            target: GmChangeTarget::AdventureState,
                            path: key.clone(),
                            before: b.get(key).map(|v| v.to_string()),
                            after: a.get(key).map(|v| v.to_string()),
                        });
                    }
                }
            } else {
                entries.push(GmDiffEntry {
                    target: GmChangeTarget::AdventureState,
                    path: "adventure_state".to_string(),
                    before: Some(adventure.adventure_state.to_string()),
                    after: Some(new_state.clone()),
                });
            }
        }
        if let Some(new_prompt) = &proposal.updated_world_blueprint {
            entries.push(GmDiffEntry {
                target: GmChangeTarget::WorldBlueprint,
                path: "world_prompt".to_string(),
                before: Some(adventure.world_prompt.to_string()),
                after: Some(new_prompt.clone()),
            });
        }
        entries
    }

    fn meter(
        &self,
        label: MetricLabel,
        model: AiModel,
        usage: Usage,
        adventure_id: Option<&AdventureId>,
        blueprint_id: Option<&lib_soulfire::ids::WorldBlueprintId>,
    ) -> CoreResult<()> {
        let metric = UsageMetric::builder()
            .created_at(self.clock.now())
            .label(label)
            .maybe_adventure_id(adventure_id.cloned())
            .maybe_blueprint_id(blueprint_id.cloned())
            .input_tokens(usage.input_tokens)
            .output_tokens(usage.output_tokens)
            .maybe_cached_input_tokens(usage.cached_input_tokens)
            .ai_model(model)
            .build();
        self.store.save_metric(&metric)
    }
}
