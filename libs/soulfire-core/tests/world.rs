//! World turn-engine integration tests (TEST-13): start, a streamed turn with
//! diff state update, non-fatal state-update failure, the single-flight lock +
//! stale-heal, and the /gm stage/accept/reject flow.

use std::str::FromStr;
use std::sync::Arc;

use soulfire_core::ai::types::{JsonMode, ReasoningEffort};
use soulfire_core::model::ai_model::{AiModel, AiVendor};
use soulfire_core::model::strings::{WorldPrompt, WorldTitle};
use soulfire_core::model::world::{
    AdventureMessageType, GmProposalStatus, StoryStatus, WorldBlueprint,
};
use soulfire_core::secret::Secret;

use soulfire_core::ai::fake::{RecordingProvider, Scripted};
use soulfire_core::ai::provider::ApiKeySource;
use soulfire_core::ai::service::AiService;
use soulfire_core::ai::types::ProviderError;
use soulfire_core::clock::{Clock, MockClock};
use soulfire_core::error::CoreError;
use soulfire_core::store::Store;
use soulfire_core::world::{TurnOutcome, WorldBuilderEngine, WorldEngine};

struct Keys;
impl ApiKeySource for Keys {
    fn api_key(&self, _v: AiVendor) -> Option<Secret<String>> {
        Some(Secret::new("sk".to_string()))
    }
}

struct H {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    provider: Arc<RecordingProvider>,
    clock: Arc<MockClock>,
    engine: WorldEngine,
}

fn harness() -> H {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::initialize(dir.path(), "pw").unwrap());
    let provider = Arc::new(RecordingProvider::new());
    let ai = AiService::new(provider.clone(), Arc::new(Keys));
    let clock = Arc::new(MockClock::at_epoch());
    let engine = WorldEngine::new(store.clone(), ai, clock.clone() as Arc<dyn Clock>);
    H {
        _dir: dir,
        store,
        provider,
        clock,
        engine,
    }
}

fn blueprint() -> WorldBlueprint {
    WorldBlueprint::builder()
        .title(WorldTitle::from_str("Beneath Verath").unwrap())
        .world_prompt(WorldPrompt::from_str("A sunken city of secrets.").unwrap())
        .build()
}

async fn start(h: &H) -> soulfire_core::model::world::Adventure {
    h.provider.push(Scripted::text(
        "You awaken in the dark depths of Verath.",
        80,
        20,
    ));
    h.provider.push(Scripted::text(
        r#"{"player":{"name":"Diver"},"current_situation":{"location":"flooded hall","time":"night","day":1}}"#,
        60,
        40,
    ));
    let bp = blueprint();
    h.store.save_blueprint(&bp).unwrap();
    h.engine.start_adventure(&bp, |_| {}).await.unwrap()
}

#[tokio::test]
async fn start_produces_intro_and_initial_state() {
    // AC-WORLD-a: intro narration message + non-empty adventure_state.
    let h = harness();
    let adv = start(&h).await;
    let msgs = h.store.adventure_messages(&adv.adventure_id).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].message_type, AdventureMessageType::Narration);
    assert!(adv.adventure_state.as_str().contains("flooded hall"));
    assert_eq!(adv.story_status, StoryStatus::Ongoing);
}

#[tokio::test]
async fn turn_echoes_action_streams_narration_and_applies_diff() {
    // AC-WORLD-b / d: a turn echoes the action, streams+persists narration, then
    // applies a diff state update.
    let h = harness();
    let adv = start(&h).await;
    // narration stream, then a diff state-update response.
    h.provider
        .push(Scripted::stream(vec!["You ", "wade ", "forward."], 100, 10));
    h.provider.push(Scripted::text(
        r#"{"patches":[{"path":"current_situation.time","value":"dawn"}],"new_recent_events":["Waded into the hall at dawn"],"story_status":"ongoing"}"#,
        50,
        20,
    ));

    let mut streamed = String::new();
    let outcome = h
        .engine
        .take_turn(&adv.adventure_id, "wade forward", |d| streamed.push_str(d))
        .await
        .unwrap();

    assert_eq!(streamed, "You wade forward.");
    match outcome {
        TurnOutcome::Narration {
            state_update_failed,
            ..
        } => assert!(!state_update_failed),
        other => panic!("expected narration, got {other:?}"),
    }
    // One user_action + one narration appended (plus the intro).
    let msgs = h.store.adventure_messages(&adv.adventure_id).unwrap();
    let types: Vec<_> = msgs.iter().map(|m| m.message_type).collect();
    assert_eq!(
        types,
        vec![
            AdventureMessageType::Narration, // intro
            AdventureMessageType::UserAction,
            AdventureMessageType::Narration,
        ]
    );
    // The diff applied: time advanced to dawn; lock released.
    let reloaded = h.store.adventure(&adv.adventure_id).unwrap().unwrap();
    assert!(reloaded.adventure_state.as_str().contains("dawn"));
    assert!(
        reloaded
            .recent_summary
            .as_str()
            .contains("Waded into the hall")
    );
    assert_eq!(
        reloaded.ready_status,
        soulfire_core::model::world::AdventureReadyStatus::Ready
    );
    assert_eq!(reloaded.diff_action_count, 1);

    // TEST-10 / OG parity: start + turn calls preserve the OG per-call model
    // settings and token budgets.
    let requests = h.provider.requests();
    assert_eq!(requests.len(), 4); // intro, initial state, narration, diff update

    let intro = &requests[0];
    assert_eq!(intro.model, AiModel::Gpt5_1);
    assert_eq!(intro.config.max_output_tokens, Some(2048));
    assert_eq!(intro.config.temperature, Some(0.9));
    assert_eq!(intro.config.reasoning_effort, None);
    assert!(intro.config.json.is_none());
    assert!(intro.config.cache_hint);
    assert!(
        intro
            .instructions
            .as_ref()
            .unwrap()
            .contains("Write an engaging introduction paragraph")
    );

    let initial_state = &requests[1];
    assert_eq!(initial_state.model, AiModel::Gpt5_1);
    assert_eq!(initial_state.config.max_output_tokens, Some(16_384));
    assert_eq!(initial_state.config.temperature, Some(0.0));
    assert!(matches!(
        initial_state.config.json.as_ref(),
        Some(JsonMode::Json)
    ));
    assert!(!initial_state.config.cache_hint);

    let narration = &requests[2];
    assert_eq!(narration.model, AiModel::Gpt5_1);
    assert_eq!(narration.config.max_output_tokens, Some(2048));
    assert_eq!(narration.config.temperature, Some(0.9));
    assert_eq!(narration.config.reasoning_effort, None);
    assert!(narration.config.cache_hint);
    assert!(
        narration
            .instructions
            .as_ref()
            .unwrap()
            .contains("# World Blueprint")
    );

    let diff = &requests[3];
    assert_eq!(diff.model, AiModel::Gpt5_1);
    assert_eq!(diff.config.max_output_tokens, Some(4096));
    assert_eq!(diff.config.temperature, Some(0.15));
    assert!(matches!(diff.config.json.as_ref(), Some(JsonMode::Json)));
    assert!(diff.config.cache_hint);
}

#[tokio::test]
async fn state_update_failure_keeps_narration_and_state_unchanged() {
    // AC-WORLD-b: forcing a state-update failure preserves the narration and
    // leaves state unchanged.
    let h = harness();
    let adv = start(&h).await;
    let state_before = h
        .store
        .adventure(&adv.adventure_id)
        .unwrap()
        .unwrap()
        .adventure_state
        .to_string();

    h.provider
        .push(Scripted::stream(vec!["Something happens."], 30, 5));
    // Non-transient error on the state-update call -> fails fast, non-fatal.
    h.provider
        .push(Scripted::Error(ProviderError::RateLimited("429".into())));

    let outcome = h
        .engine
        .take_turn(&adv.adventure_id, "look", |_| {})
        .await
        .unwrap();
    match outcome {
        TurnOutcome::Narration {
            state_update_failed,
            ..
        } => assert!(state_update_failed),
        other => panic!("expected narration, got {other:?}"),
    }
    let reloaded = h.store.adventure(&adv.adventure_id).unwrap().unwrap();
    // Narration kept; state unchanged; lock released.
    assert_eq!(reloaded.adventure_state.to_string(), state_before);
    assert_eq!(
        reloaded.ready_status,
        soulfire_core::model::world::AdventureReadyStatus::Ready
    );
    let msgs = h.store.adventure_messages(&adv.adventure_id).unwrap();
    assert_eq!(msgs.last().unwrap().content.as_str(), "Something happens.");
}

#[tokio::test]
async fn forced_full_state_update_uses_og_request_config() {
    // TEST-10 / WORLD-13: after enough diff updates, the engine skips the diff
    // path and asks for a full state replacement with the OG full budget.
    let h = harness();
    let mut adv = start(&h).await;
    adv.diff_action_count = soulfire_core::world::engine::FULL_STATE_UPDATE_THRESHOLD;
    h.store.save_adventure(&adv).unwrap();

    h.provider
        .push(Scripted::stream(vec!["The hall opens."], 100, 10));
    h.provider.push(Scripted::text(
        "{\"updated_state\":{\"player\":{\"name\":\"Diver\"},\"current_situation\":{\"location\":\"open hall\",\"time\":\"dawn\",\"day\":2}},\"recent_events\":[\"Opened the hall\"],\"story_summary\":\"## Rolling Story\\nThe hall opened.\",\"story_status\":\"ongoing\"}",
        70,
        35,
    ));

    let outcome = h
        .engine
        .take_turn(&adv.adventure_id, "open the hall", |_| {})
        .await
        .unwrap();
    assert!(matches!(outcome, TurnOutcome::Narration { .. }));

    let reloaded = h.store.adventure(&adv.adventure_id).unwrap().unwrap();
    assert_eq!(reloaded.diff_action_count, 0);
    assert!(reloaded.adventure_state.as_str().contains("open hall"));

    let requests = h.provider.requests();
    assert_eq!(requests.len(), 4); // start intro/state + narration/full update
    let full = &requests[3];
    assert_eq!(full.model, AiModel::Gpt5_1);
    assert_eq!(full.config.max_output_tokens, Some(24_576));
    assert_eq!(full.config.temperature, Some(0.15));
    assert!(matches!(full.config.json.as_ref(), Some(JsonMode::Json)));
    assert!(full.config.cache_hint);
    assert!(
        full.instructions
            .as_ref()
            .unwrap()
            .contains("output the updated adventure state")
    );
    assert!(
        full.messages
            .iter()
            .any(|m| m.content.contains("open the hall"))
    );
}

#[tokio::test]
async fn lock_refuses_concurrent_turn_then_self_heals() {
    // AC-WORLD-b: a second action mid-turn is refused; the stale lock self-heals.
    let h = harness();
    let mut adv = start(&h).await;
    // Manually mark a turn in progress.
    adv.ready_status = soulfire_core::model::world::AdventureReadyStatus::UpdatingNarrative;
    adv.ready_status_updated_at = Some(h.clock.now());
    h.store.save_adventure(&adv).unwrap();

    let err = h
        .engine
        .take_turn(&adv.adventure_id, "act", |_| {})
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::TurnInProgress));

    // After the stale-lock expiry, the turn proceeds (self-heal).
    h.clock.advance_secs(91);
    h.provider.push(Scripted::stream(vec!["Healed."], 10, 2));
    h.provider.push(Scripted::text(r#"{"patches":[]}"#, 5, 2));
    let outcome = h
        .engine
        .take_turn(&adv.adventure_id, "act", |_| {})
        .await
        .unwrap();
    assert!(matches!(outcome, TurnOutcome::Narration { .. }));
}

#[tokio::test]
async fn unknown_and_empty_commands_warn() {
    // WORLD-15.
    let h = harness();
    let adv = start(&h).await;
    assert!(matches!(
        h.engine
            .take_turn(&adv.adventure_id, "/gm", |_| {})
            .await
            .unwrap(),
        TurnOutcome::Warning(_)
    ));
    assert!(matches!(
        h.engine
            .take_turn(&adv.adventure_id, "/fly", |_| {})
            .await
            .unwrap(),
        TurnOutcome::Warning(_)
    ));
}

#[tokio::test]
async fn world_builder_applies_changes_and_uses_og_request_config() {
    // AC-WORLD-g / TEST-10: a world-builder turn can revise editable fields and
    // its structured request keeps the OG 0.8 / 9000 / medium-reasoning config.
    let h = harness();
    let bp = blueprint();
    h.store.save_blueprint(&bp).unwrap();
    h.provider.push(Scripted::text(
        r#"{"assistant_message":"Made it stranger.","description":"A city under black glass.","world_prompt":"A sunken city sealed beneath obsidian glass."}"#,
        90,
        40,
    ));

    let builder = WorldBuilderEngine::new(
        h.store.clone(),
        AiService::new(h.provider.clone(), Arc::new(Keys)),
        h.clock.clone() as Arc<dyn Clock>,
    );
    let result = builder
        .builder_send(&bp.blueprint_id, "make it stranger")
        .await
        .unwrap();

    assert_eq!(result.assistant_message, "Made it stranger.");
    let updated = h.store.blueprint(&bp.blueprint_id).unwrap().unwrap();
    assert_eq!(updated.description.as_str(), "A city under black glass.");
    assert!(updated.world_prompt.as_str().contains("obsidian glass"));

    let session = h
        .store
        .world_builder_session(&bp.blueprint_id)
        .unwrap()
        .unwrap();
    assert_eq!(session.snapshots.len(), 1);

    let req = h.provider.last_request().unwrap();
    assert_eq!(req.model, AiModel::Gpt5_1);
    assert_eq!(req.config.max_output_tokens, Some(9000));
    assert_eq!(req.config.temperature, Some(0.8));
    assert_eq!(req.config.reasoning_effort, Some(ReasoningEffort::Medium));
    assert!(matches!(req.config.json.as_ref(), Some(JsonMode::Json)));
    assert_eq!(req.messages.len(), 1);
    assert!(req.messages[0].content.contains("Current world:"));
    assert!(
        req.messages[0]
            .content
            .contains("Latest user message:\nmake it stranger")
    );
    let instructions = req.instructions.unwrap();
    assert!(instructions.contains("collaborative world builder"));
    assert!(instructions.contains("Return only JSON with this exact shape"));
}

#[tokio::test]
async fn gm_change_is_staged_and_accept_applies_reject_does_not() {
    // AC-WORLD-e: /gm yields a staged proposal; Accept applies it, Reject doesn't.
    let h = harness();
    let adv = start(&h).await;
    // classify -> adventure_state; proposal with a new state.
    h.provider
        .push(Scripted::text(r#"{"intent":"adventure_state"}"#, 10, 2));
    h.provider.push(Scripted::text(
        r#"{"response":"Skipped to morning.","updated_adventure_state":{"current_situation":{"time":"morning","day":2}}}"#,
        40,
        20,
    ));

    let outcome = h
        .engine
        .take_turn(&adv.adventure_id, "/gm skip to morning", |_| {})
        .await
        .unwrap();
    let proposal = match outcome {
        TurnOutcome::GmProposal { proposal, .. } => proposal,
        other => panic!("expected proposal, got {other:?}"),
    };
    assert_eq!(proposal.status, GmProposalStatus::Pending);
    assert!(!proposal.changes.is_empty()); // a readable diff was computed

    // TEST-10 / OG parity: /gm uses the utility model for classification and the
    // mini model with low reasoning for change proposals.
    let requests = h.provider.requests();
    assert_eq!(requests.len(), 4); // start intro/state + classify/proposal
    let classify = &requests[2];
    assert_eq!(classify.model, AiModel::Gpt5_4Nano);
    assert_eq!(classify.config.max_output_tokens, Some(128));
    assert_eq!(classify.config.temperature, Some(0.0));
    assert_eq!(classify.config.reasoning_effort, None);
    assert!(matches!(
        classify.config.json.as_ref(),
        Some(JsonMode::Json)
    ));

    let gm_proposal = &requests[3];
    assert_eq!(gm_proposal.model, AiModel::Gpt5_4Mini);
    assert_eq!(gm_proposal.config.max_output_tokens, Some(24_576));
    assert_eq!(gm_proposal.config.temperature, Some(0.2));
    assert_eq!(
        gm_proposal.config.reasoning_effort,
        Some(ReasoningEffort::Low)
    );
    assert!(matches!(
        gm_proposal.config.json.as_ref(),
        Some(JsonMode::Json)
    ));
    assert!(gm_proposal.config.cache_hint);

    // Reject changes nothing.
    let before = h
        .store
        .adventure(&adv.adventure_id)
        .unwrap()
        .unwrap()
        .adventure_state
        .to_string();
    // (use a clone of the proposal id for a separate reject scenario)
    h.engine
        .reject_proposal(&proposal.proposal_id)
        .await
        .unwrap();
    let after_reject = h
        .store
        .adventure(&adv.adventure_id)
        .unwrap()
        .unwrap()
        .adventure_state
        .to_string();
    assert_eq!(before, after_reject);
    assert_eq!(
        h.store
            .gm_proposal(&proposal.proposal_id)
            .unwrap()
            .unwrap()
            .status,
        GmProposalStatus::Rejected
    );

    // Accept applies the proposed state.
    h.engine
        .accept_proposal(&proposal.proposal_id)
        .await
        .unwrap();
    let accepted = h.store.adventure(&adv.adventure_id).unwrap().unwrap();
    assert!(accepted.adventure_state.as_str().contains("morning"));
}
