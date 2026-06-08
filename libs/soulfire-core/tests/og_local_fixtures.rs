//! OG-to-local data fixture proof (TEST-10, TEST-12).
//!
//! The fixture uses Soulfire-OG's surviving feature concepts and stable ID
//! prefixes, after applying the local-only removals from the roadmap: no user
//! ownership, auth/session, billing, publication/moderation, ratings, or remote
//! image storage fields. This proves the adapted local records round-trip through
//! serde and the encrypted SQLite store.

use serde::Deserialize;

use soulfire_core::model::character::{Character, CharacterBuilderSession};
use soulfire_core::model::chat::{Chat, ChatMessage};
use soulfire_core::model::draft::{Draft, DraftScope};
use soulfire_core::model::metric::UsageMetric;
use soulfire_core::model::profile::{AppProfile, PlayerProfile};
use soulfire_core::model::settings::AppSettings;
use soulfire_core::model::world::{
    Adventure, AdventureMessage, GmProposal, WorldBlueprint, WorldBuilderSession,
};
use soulfire_core::store::Store;

#[derive(Debug, Deserialize)]
struct Fixture {
    characters: Vec<Character>,
    chats: Vec<Chat>,
    chat_messages: Vec<ChatMessage>,
    blueprints: Vec<WorldBlueprint>,
    adventures: Vec<Adventure>,
    adventure_messages: Vec<AdventureMessage>,
    gm_proposals: Vec<GmProposal>,
    character_builder_sessions: Vec<CharacterBuilderSession>,
    world_builder_sessions: Vec<WorldBuilderSession>,
    usage_metrics: Vec<UsageMetric>,
    app_profile: AppProfile,
    player_profile: PlayerProfile,
    app_settings: AppSettings,
    drafts: Vec<Draft>,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("fixtures/og_local_models.json")).unwrap()
}

#[test]
fn og_local_model_fixture_round_trips_through_serde() {
    let f = fixture();

    assert_eq!(f.characters.len(), 1);
    assert_eq!(f.chats.len(), 1);
    assert_eq!(f.chat_messages.len(), 2);
    assert_eq!(f.blueprints.len(), 1);
    assert_eq!(f.adventures.len(), 1);
    assert_eq!(f.adventure_messages.len(), 2);
    assert_eq!(f.gm_proposals.len(), 1);
    assert_eq!(f.character_builder_sessions.len(), 1);
    assert_eq!(f.world_builder_sessions.len(), 1);
    assert_eq!(f.usage_metrics.len(), 1);
    assert_eq!(f.drafts.len(), 2);

    let json = serde_json::to_string(&f.characters[0]).unwrap();
    let back: Character = serde_json::from_str(&json).unwrap();
    assert_eq!(back, f.characters[0]);
    assert!(back.is_world_extracted());

    let json = serde_json::to_string(&f.adventures[0]).unwrap();
    let back: Adventure = serde_json::from_str(&json).unwrap();
    assert_eq!(back, f.adventures[0]);
    assert_eq!(back.diff_action_count, 7);

    let json = serde_json::to_string(&f.gm_proposals[0]).unwrap();
    let back: GmProposal = serde_json::from_str(&json).unwrap();
    assert_eq!(back.changes.len(), 2);
    assert_eq!(back, f.gm_proposals[0]);
}

#[test]
fn og_local_model_fixture_persists_in_encrypted_store() {
    let f = fixture();
    let dir = tempfile::tempdir().unwrap();
    let store = Store::initialize(dir.path(), "pw").unwrap();

    for character in &f.characters {
        store.save_character(character).unwrap();
    }
    for blueprint in &f.blueprints {
        store.save_blueprint(blueprint).unwrap();
    }
    for chat in &f.chats {
        store.save_chat(chat).unwrap();
    }
    for message in &f.chat_messages {
        store.save_chat_message(message).unwrap();
    }
    for adventure in &f.adventures {
        store.save_adventure(adventure).unwrap();
    }
    for message in &f.adventure_messages {
        store.save_adventure_message(message).unwrap();
    }
    for proposal in &f.gm_proposals {
        store.save_gm_proposal(proposal).unwrap();
    }
    for session in &f.character_builder_sessions {
        store.save_character_builder_session(session).unwrap();
    }
    for session in &f.world_builder_sessions {
        store.save_world_builder_session(session).unwrap();
    }
    for metric in &f.usage_metrics {
        store.save_metric(metric).unwrap();
    }
    for draft in &f.drafts {
        store.save_draft(draft).unwrap();
    }
    store.save_app_profile(&f.app_profile).unwrap();
    store.save_player_profile(&f.player_profile).unwrap();
    store.save_app_settings(&f.app_settings).unwrap();

    assert_eq!(
        store.character(&f.characters[0].character_id).unwrap(),
        Some(f.characters[0].clone())
    );
    assert_eq!(
        store.blueprint(&f.blueprints[0].blueprint_id).unwrap(),
        Some(f.blueprints[0].clone())
    );
    assert_eq!(
        store.chat(&f.chats[0].chat_id).unwrap(),
        Some(f.chats[0].clone())
    );
    assert_eq!(
        store.chat_messages(&f.chats[0].chat_id).unwrap(),
        f.chat_messages
    );
    assert_eq!(
        store.adventure(&f.adventures[0].adventure_id).unwrap(),
        Some(f.adventures[0].clone())
    );
    assert_eq!(
        store
            .adventure_messages(&f.adventures[0].adventure_id)
            .unwrap(),
        f.adventure_messages
    );
    assert_eq!(
        store.gm_proposal(&f.gm_proposals[0].proposal_id).unwrap(),
        Some(f.gm_proposals[0].clone())
    );
    assert_eq!(
        store
            .character_builder_session(&f.characters[0].character_id)
            .unwrap(),
        Some(f.character_builder_sessions[0].clone())
    );
    assert_eq!(
        store
            .world_builder_session(&f.blueprints[0].blueprint_id)
            .unwrap(),
        Some(f.world_builder_sessions[0].clone())
    );
    assert_eq!(store.all_metrics().unwrap(), f.usage_metrics);
    assert_eq!(store.app_profile().unwrap(), f.app_profile);
    assert_eq!(store.player_profile().unwrap(), f.player_profile);
    assert_eq!(store.app_settings().unwrap(), f.app_settings);

    for draft in &f.drafts {
        assert_eq!(
            store.draft_for_scope(&draft.scope).unwrap(),
            Some(draft.clone())
        );
    }

    assert!(matches!(f.drafts[0].scope, DraftScope::Chat { .. }));
    assert!(matches!(f.drafts[1].scope, DraftScope::Adventure { .. }));
}
