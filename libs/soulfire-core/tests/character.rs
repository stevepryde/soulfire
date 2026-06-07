//! Character builder + NPC extraction tests (TEST-12).

use std::str::FromStr;
use std::sync::Arc;

use lib_soulfire::ai_model::AiVendor;
use lib_soulfire::character::{Character, InitialMessage};
use lib_soulfire::strings::{
    CharacterName, CharacterPrompt, InitialMessageText, WorldPrompt, WorldTitle,
};
use lib_soulfire::world::{Adventure, WorldBlueprint};
use sp_core::secret::Secret;

use soulfire_core::ai::fake::{RecordingProvider, Scripted};
use soulfire_core::ai::provider::ApiKeySource;
use soulfire_core::ai::service::AiService;
use soulfire_core::ai::types::ProviderError;
use soulfire_core::character::CharacterEngine;
use soulfire_core::clock::{Clock, MockClock};
use soulfire_core::store::Store;

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
    engine: CharacterEngine,
}

fn harness() -> H {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::initialize(dir.path(), "pw").unwrap());
    let provider = Arc::new(RecordingProvider::new());
    let ai = AiService::new(provider.clone(), Arc::new(Keys));
    let clock = Arc::new(MockClock::at_epoch()) as Arc<dyn Clock>;
    let engine = CharacterEngine::new(store.clone(), ai, clock);
    H { _dir: dir, store, provider, engine }
}

fn base_character() -> Character {
    Character::builder()
        .name(CharacterName::from_str("Draft").unwrap())
        .prompt(CharacterPrompt::coerce("initial prompt"))
        .initial_message(InitialMessage::Message(InitialMessageText::coerce("hi")))
        .build()
}

#[tokio::test]
async fn builder_applies_changes_and_pushes_snapshot_then_undo_restores() {
    // AC-CHAR-b: a builder turn that changes the prompt applies it and pushes a
    // snapshot; Undo restores; Undo disabled with no snapshots.
    let h = harness();
    let c = base_character();
    h.store.save_character(&c).unwrap();

    // Undo with no snapshots yet → false.
    assert!(!h.engine.builder_undo(&c.character_id).unwrap());

    h.provider.push(Scripted::text(
        r#"{"assistant_message": "Made her bolder.", "prompt": "You are Lyra, bold and sharp-tongued."}"#,
        50,
        20,
    ));
    let result = h.engine.builder_send(&c.character_id, "make her bolder").await.unwrap();
    assert_eq!(result.assistant_message, "Made her bolder.");

    let updated = h.store.character(&c.character_id).unwrap().unwrap();
    assert_eq!(updated.prompt.as_str(), "You are Lyra, bold and sharp-tongued.");

    // A snapshot was captured.
    let session = h.store.character_builder_session(&c.character_id).unwrap().unwrap();
    assert_eq!(session.snapshots.len(), 1);

    // Undo restores the prior prompt.
    assert!(h.engine.builder_undo(&c.character_id).unwrap());
    let reverted = h.store.character(&c.character_id).unwrap().unwrap();
    assert_eq!(reverted.prompt.as_str(), "initial prompt");
}

#[tokio::test]
async fn builder_without_field_changes_pushes_no_snapshot() {
    let h = harness();
    let c = base_character();
    h.store.save_character(&c).unwrap();
    h.provider.push(Scripted::text(
        r#"{"assistant_message": "What tone do you want?"}"#,
        10,
        5,
    ));
    h.engine.builder_send(&c.character_id, "help").await.unwrap();
    let session = h.store.character_builder_session(&c.character_id).unwrap().unwrap();
    assert!(session.snapshots.is_empty());
}

fn seed_adventure(h: &H) -> Adventure {
    let bp = WorldBlueprint::builder()
        .title(WorldTitle::from_str("Verath").unwrap())
        .world_prompt(WorldPrompt::from_str("A drowned city.").unwrap())
        .build();
    h.store.save_blueprint(&bp).unwrap();
    let adv = Adventure::builder()
        .blueprint_id(bp.blueprint_id.clone())
        .world_prompt(bp.world_prompt.clone())
        .adventure_state(lib_soulfire::strings::AdventureState::coerce(
            r#"{"npcs":{"Mara":{"attitude":"wary"}}}"#,
        ))
        .story_summary(lib_soulfire::strings::StorySummary::coerce(
            "## Rolling Story\nMara guided the player through the depths.",
        ))
        .build();
    h.store.save_adventure(&adv).unwrap();
    adv
}

#[tokio::test]
async fn extraction_produces_character_with_context_state_origin_and_chat() {
    // AC-CHAR-c: extracting an NPC produces a character with non-empty
    // extracted_context and character_state, origin fields, and an opened chat.
    let h = harness();
    let adv = seed_adventure(&h);
    h.provider.push(Scripted::text("You are Mara, a wary guide.", 80, 60)); // persona
    h.provider.push(Scripted::text("You feel cautious but curious.", 40, 30)); // state
    h.provider.push(Scripted::text("Hello again, traveler.", 30, 10)); // opening (Prompt initial)

    let character = h.engine.extract_npc(&adv.adventure_id, "Mara").await.unwrap();

    assert_eq!(character.name.as_str(), "Mara");
    assert!(character.extracted_context.unwrap().as_str().contains("wary guide"));
    assert!(character.character_state.unwrap().as_str().contains("cautious"));
    assert_eq!(character.source_adventure_id.as_ref(), Some(&adv.adventure_id));
    assert_eq!(character.source_npc_name.as_deref(), Some("Mara"));

    // A chat was created and opened with an opening message + a title.
    let chat_id = h.store.chat_id_for_character(&character.character_id).unwrap().unwrap();
    let chat = h.store.chat(&chat_id).unwrap().unwrap();
    assert_eq!(chat.title.as_str(), "Mara");
    let msgs = h.store.chat_messages(&chat_id).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].message.as_str(), "Hello again, traveler.");
}

#[tokio::test]
async fn failed_extraction_creates_no_partial_character() {
    // AC-CHAR-d: a forced extraction failure leaves no new character or chat.
    let h = harness();
    let adv = seed_adventure(&h);
    h.provider.push(Scripted::Error(ProviderError::RateLimited("429".into()))); // persona fails

    let err = h.engine.extract_npc(&adv.adventure_id, "Mara").await;
    assert!(err.is_err());
    assert_eq!(h.store.count_characters().unwrap(), 0);
}
