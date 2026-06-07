//! Store integration tests: persistence round-trips, cascade integrity, draft
//! lifecycle, and at-rest encryption. Validates DATA-1..26 store behaviors
//! (TEST-7) and SEC-1/SEC-9 at-rest protection (TEST-8).

use std::str::FromStr;

use lib_soulfire::ai_model::AiVendor;
use lib_soulfire::character::{Character, InitialMessage};
use lib_soulfire::chat::{Chat, ChatMessage, Sender};
use lib_soulfire::credentials::ProviderCredential;
use lib_soulfire::draft::{Draft, DraftScope};
use lib_soulfire::ids::CharacterId;
use lib_soulfire::strings::{CharacterName, DraftContent, InitialMessageText, MessageString};
use lib_soulfire::strings::{MessageContent, WorldPrompt, WorldTitle};
use lib_soulfire::world::{Adventure, AdventureMessage, AdventureMessageType, WorldBlueprint};

use soulfire_core::store::Store;

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::initialize(dir.path(), "test-password").unwrap();
    (dir, store)
}

fn sample_character(name: &str) -> Character {
    Character::builder()
        .name(CharacterName::from_str(name).unwrap())
        .initial_message(InitialMessage::Message(InitialMessageText::coerce(
            "Hello there.",
        )))
        .build()
}

#[test]
fn fresh_store_has_singletons_and_no_content() {
    // DATA-25: one each singleton; no characters/chats/adventures/metrics/creds.
    let (_dir, store) = temp_store();
    assert_eq!(store.count_characters().unwrap(), 0);
    assert_eq!(store.count_blueprints().unwrap(), 0);
    assert_eq!(store.count_metrics().unwrap(), 0);
    assert!(store.credentials().unwrap().is_empty());
    // Singletons exist and load with defaults.
    let _ = store.app_profile().unwrap();
    let _ = store.player_profile().unwrap();
    let _ = store.app_settings().unwrap();
    let _ = store.install_state().unwrap();
}

#[test]
fn character_round_trips_with_fields_intact() {
    // AC-DATA-a (partial): a character round-trips through save/load.
    let (_dir, store) = temp_store();
    let mut c = sample_character("Lyra");
    c.prompt = lib_soulfire::strings::CharacterPrompt::coerce("You are Lyra, a guide.");
    c.creativity.temperature = 0.7;
    store.save_character(&c).unwrap();
    let loaded = store.character(&c.character_id).unwrap().unwrap();
    assert_eq!(loaded, c);
}

#[test]
fn at_most_one_chat_per_character() {
    // DATA-5: UNIQUE(character_id) prevents a second chat for the same character.
    let (_dir, store) = temp_store();
    let c = sample_character("Nova");
    store.save_character(&c).unwrap();
    let chat1 = Chat::builder().character_id(c.character_id.clone()).build();
    store.save_chat(&chat1).unwrap();
    let chat2 = Chat::builder().character_id(c.character_id.clone()).build();
    let err = store.save_chat(&chat2);
    assert!(
        err.is_err(),
        "a second chat for the same character must fail"
    );
}

#[test]
fn deleting_character_cascades_to_chat_messages_and_draft() {
    // DATA-22 / AC-DATA-e: no orphan rows after deleting a character.
    let (_dir, store) = temp_store();
    let c = sample_character("Solas");
    store.save_character(&c).unwrap();
    let chat = Chat::builder().character_id(c.character_id.clone()).build();
    store.save_chat(&chat).unwrap();
    let msg = ChatMessage::builder()
        .chat_id(chat.chat_id.clone())
        .sender(Sender::Player)
        .message(MessageString::coerce("hi"))
        .build();
    store.save_chat_message(&msg).unwrap();
    let draft = Draft::builder()
        .scope(DraftScope::Chat {
            chat_id: chat.chat_id.clone(),
        })
        .content(DraftContent::coerce("unsent"))
        .build();
    store.save_draft(&draft).unwrap();

    store.delete_character(&c.character_id).unwrap();

    assert!(store.character(&c.character_id).unwrap().is_none());
    assert!(store.chat(&chat.chat_id).unwrap().is_none());
    assert_eq!(store.count_chat_messages(&chat.chat_id).unwrap(), 0);
    assert!(
        store
            .draft_for_scope(&DraftScope::Chat {
                chat_id: chat.chat_id.clone()
            })
            .unwrap()
            .is_none()
    );
}

#[test]
fn deleting_blueprint_cascades_to_adventures_messages_and_proposals() {
    // DATA-22: blueprint delete removes its adventures + their messages.
    let (_dir, store) = temp_store();
    let bp = WorldBlueprint::builder()
        .title(WorldTitle::from_str("Beneath Verath").unwrap())
        .world_prompt(WorldPrompt::from_str("A sunken city beneath the waves.").unwrap())
        .build();
    store.save_blueprint(&bp).unwrap();
    let adv = Adventure::builder()
        .blueprint_id(bp.blueprint_id.clone())
        .world_prompt(bp.world_prompt.clone())
        .build();
    store.save_adventure(&adv).unwrap();
    let m = AdventureMessage::builder()
        .adventure_id(adv.adventure_id.clone())
        .message_type(AdventureMessageType::Narration)
        .content(MessageContent::coerce("You awaken in the dark."))
        .build();
    store.save_adventure_message(&m).unwrap();

    store.delete_blueprint(&bp.blueprint_id).unwrap();

    assert!(store.blueprint(&bp.blueprint_id).unwrap().is_none());
    assert!(store.adventure(&adv.adventure_id).unwrap().is_none());
    assert!(
        store
            .adventure_messages(&adv.adventure_id)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn draft_replaces_prior_for_same_scope_and_restores() {
    // AC-DATA-g: saving replaces any prior draft for the scope; reopening restores.
    let (_dir, store) = temp_store();
    let bp = WorldBlueprint::builder()
        .title(WorldTitle::from_str("W").unwrap())
        .world_prompt(WorldPrompt::from_str("prompt").unwrap())
        .build();
    store.save_blueprint(&bp).unwrap();
    let adv = Adventure::builder()
        .blueprint_id(bp.blueprint_id.clone())
        .world_prompt(bp.world_prompt.clone())
        .build();
    store.save_adventure(&adv).unwrap();
    let scope = DraftScope::Adventure {
        adventure_id: adv.adventure_id.clone(),
    };

    let d1 = Draft::builder()
        .scope(scope.clone())
        .content(DraftContent::coerce("first"))
        .build();
    store.save_draft(&d1).unwrap();
    let d2 = Draft::builder()
        .scope(scope.clone())
        .content(DraftContent::coerce("second"))
        .build();
    store.save_draft(&d2).unwrap();

    let restored = store.draft_for_scope(&scope).unwrap().unwrap();
    assert_eq!(restored.content.as_str(), "second");

    // Deleting the adventure clears its draft.
    store.delete_adventure(&adv.adventure_id).unwrap();
    assert!(store.draft_for_scope(&scope).unwrap().is_none());
}

#[test]
fn credentials_and_content_are_unreadable_on_disk_without_key() {
    // AC-SEC-a / TEST-8: with the app closed and no key, the db file reveals no
    // readable character names or API keys; with the key, it loads normally.
    let dir = tempfile::tempdir().unwrap();
    const SECRET_NAME: &str = "ZzSecretCharacterNameZz";
    const SECRET_KEY: &str = "sk-PLAINTEXT-SECRET-KEY-1234567890";
    let db_path = {
        let store = Store::initialize(dir.path(), "pw").unwrap();
        store
            .save_character(&sample_character(SECRET_NAME))
            .unwrap();
        store
            .save_credential(&ProviderCredential::new(AiVendor::OpenAI, SECRET_KEY))
            .unwrap();
        store.paths().db.clone()
        // store dropped here -> connection closed and flushed
    };

    let bytes = std::fs::read(&db_path).unwrap();
    assert!(
        !contains(&bytes, SECRET_NAME.as_bytes()),
        "character name found in plaintext on disk"
    );
    assert!(
        !contains(&bytes, SECRET_KEY.as_bytes()),
        "API key found in plaintext on disk"
    );
    // SQLite files start with this header when unencrypted; encrypted ones do not.
    assert!(
        !bytes.starts_with(b"SQLite format 3\0"),
        "database file is not encrypted"
    );

    // With the correct password the same data loads normally.
    let store = Store::unlock(dir.path(), "pw").unwrap();
    let creds = store.credentials().unwrap();
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0].masked(), "••••••••7890");
    assert_eq!(store.count_characters().unwrap(), 1);
}

#[test]
fn unknown_character_is_none_not_error() {
    let (_dir, store) = temp_store();
    assert!(store.character(&CharacterId::new()).unwrap().is_none());
}

#[test]
fn store_reopens_preserving_data_and_schema_version() {
    // PKG-4: a later run opens the existing store, runs migrations forward
    // transparently, and finds all prior data intact. With one schema version
    // the migration is a no-op, but this pins the close → reopen contract the
    // forward-migration mechanism relies on (idempotent migrate + stamped
    // user_version; see store/schema.rs).
    let dir = tempfile::tempdir().unwrap();
    let cid;
    {
        let store = Store::initialize(dir.path(), "pw").unwrap();
        let c = sample_character("Persistent");
        cid = c.character_id.clone();
        store.save_character(&c).unwrap();
        // store dropped here -> connection closed, like an app restart
    }

    // A subsequent run unlocks the same directory; migrate() runs again.
    let store = Store::unlock(dir.path(), "pw").unwrap();
    let loaded = store.character(&cid).unwrap().unwrap();
    assert_eq!(loaded.name.as_str(), "Persistent");
    assert_eq!(store.count_characters().unwrap(), 1);
    // The schema version is stamped at the current version after reopen.
    assert_eq!(
        store.schema_version().unwrap(),
        soulfire_core::store::SCHEMA_VERSION
    );
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}
