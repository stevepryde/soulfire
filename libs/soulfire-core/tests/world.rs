//! World turn-engine integration tests (TEST-13): start, a streamed turn with
//! diff state update, non-fatal state-update failure, the single-flight lock +
//! stale-heal, and the /gm stage/accept/reject flow.

use std::str::FromStr;
use std::sync::Arc;

use soulfire_core::model::ai_model::AiVendor;
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
use soulfire_core::world::{TurnOutcome, WorldEngine};

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
