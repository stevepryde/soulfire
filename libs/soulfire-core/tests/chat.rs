//! Chat engine integration tests (TEST-11): opening messages, streaming a reply,
//! reaction extraction, the rolling summary cadence, and the character-state
//! updater — all against the recording fake provider and a mock clock.

use std::str::FromStr;
use std::sync::Arc;

use soulfire_core::model::ai_model::AiVendor;
use soulfire_core::model::character::{Character, InitialMessage};
use soulfire_core::model::chat::AI_REACTOR;
use soulfire_core::model::strings::{
    CharacterContext, CharacterName, CharacterPrompt, InitialMessageText,
};
use soulfire_core::secret::Secret;

use soulfire_core::ai::fake::{RecordingProvider, Scripted};
use soulfire_core::ai::provider::ApiKeySource;
use soulfire_core::ai::service::AiService;
use soulfire_core::ai::types::ProviderError;
use soulfire_core::chat::ChatEngine;
use soulfire_core::clock::{Clock, MockClock};
use soulfire_core::store::Store;

struct Keys;
impl ApiKeySource for Keys {
    fn api_key(&self, _vendor: AiVendor) -> Option<Secret<String>> {
        Some(Secret::new("sk-test".to_string()))
    }
}

struct Harness {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    provider: Arc<RecordingProvider>,
    clock: Arc<MockClock>,
    engine: ChatEngine,
}

fn harness() -> Harness {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::initialize(dir.path(), "pw").unwrap());
    let provider = Arc::new(RecordingProvider::new());
    let ai = AiService::new(provider.clone(), Arc::new(Keys));
    let clock = Arc::new(MockClock::at_epoch());
    let engine = ChatEngine::new(store.clone(), ai, clock.clone() as Arc<dyn Clock>);
    Harness {
        _dir: dir,
        store,
        provider,
        clock,
        engine,
    }
}

fn character(initial: InitialMessage) -> Character {
    Character::builder()
        .name(CharacterName::from_str("Lyra").unwrap())
        .prompt(CharacterPrompt::coerce("You are Lyra, a calm guide."))
        .initial_message(initial)
        .build()
}

#[tokio::test]
async fn message_initial_is_verbatim_with_no_model_call() {
    // AC-CHAT-a: a Message-initial shows the verbatim opening with no model call.
    let h = harness();
    let c = character(InitialMessage::Message(InitialMessageText::coerce(
        "Hello, traveler.",
    )));
    h.store.save_character(&c).unwrap();

    let chat = h.engine.open_chat(&c.character_id).await.unwrap();
    let msgs = h.store.chat_messages(&chat.chat_id).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].message.as_str(), "Hello, traveler.");
    assert_eq!(h.provider.request_count(), 0);

    // Reopening returns the same chat (CHAT-1).
    let again = h.engine.open_chat(&c.character_id).await.unwrap();
    assert_eq!(again.chat_id, chat.chat_id);
}

#[tokio::test]
async fn prompt_initial_generates_opening() {
    // AC-CHAT-a: a Prompt-initial generates an opening via a model call.
    let h = harness();
    let c = character(InitialMessage::Prompt(InitialMessageText::coerce(
        "Greet the player warmly.",
    )));
    h.store.save_character(&c).unwrap();
    h.provider
        .push(Scripted::text("Well met, traveler. I am Lyra.", 50, 8));

    let chat = h.engine.open_chat(&c.character_id).await.unwrap();
    let msgs = h.store.chat_messages(&chat.chat_id).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].message.as_str(), "Well met, traveler. I am Lyra.");
    assert_eq!(h.provider.request_count(), 1);
    // The persona instructions were sent as the cacheable prefix (AI-4).
    let req = h.provider.last_request().unwrap();
    assert!(req.instructions.unwrap().contains("You are Lyra"));
}

#[tokio::test]
async fn send_streams_reply_finalizes_and_meters() {
    // AC-CHAT-b: sent message persists, reply streams, finalized reply persists.
    let h = harness();
    let c = character(InitialMessage::Message(InitialMessageText::coerce("Hi.")));
    h.store.save_character(&c).unwrap();
    let chat = h.engine.open_chat(&c.character_id).await.unwrap();
    // reply stream, then a title generation call (first exchange, CHAT-3).
    h.provider.push(Scripted::stream(
        vec!["Hello ", "there, ", "friend."],
        120,
        6,
    ));
    h.provider.push(Scripted::text("A Warm Greeting", 20, 3));

    let mut streamed = String::new();
    let outcome = h
        .engine
        .send_message(&chat.chat_id, "Hello!", |d| streamed.push_str(d))
        .await
        .unwrap();

    assert_eq!(streamed, "Hello there, friend.");
    assert_eq!(outcome.reply.message.as_str(), "Hello there, friend.");
    assert_eq!(outcome.reply.token_count, 6);

    // Persisted: opening + player + reply.
    let msgs = h.store.chat_messages(&chat.chat_id).unwrap();
    assert_eq!(msgs.len(), 3);

    // A metric was recorded for the chat message (AI-15).
    let metrics = h.store.metrics_for_chat(&chat.chat_id).unwrap();
    assert!(!metrics.is_empty());

    // last_chatted_at advanced (CHAT-4).
    let updated = h.store.character(&c.character_id).unwrap().unwrap();
    assert!(updated.last_chatted_at.is_some());

    // Title generated from the first exchange (CHAT-3).
    let reloaded = h.store.chat(&chat.chat_id).unwrap().unwrap();
    assert_eq!(reloaded.title.as_str(), "A Warm Greeting");
}

#[tokio::test]
async fn trailing_emoji_becomes_ai_reaction_on_player_message() {
    // AC-CHAT-c: a reply ending in an allowed emoji shows clean text and records
    // that emoji as the character's reaction to the player's message.
    let h = harness();
    let c = character(InitialMessage::Message(InitialMessageText::coerce("Hi.")));
    h.store.save_character(&c).unwrap();
    let chat = h.engine.open_chat(&c.character_id).await.unwrap();
    h.provider
        .push(Scripted::text("I'm so happy to see you ❤️", 100, 7));
    h.provider.push(Scripted::text("Joyful Reunion", 10, 2)); // title

    let outcome = h
        .engine
        .send_message(&chat.chat_id, "I missed you", |_| {})
        .await
        .unwrap();

    assert_eq!(outcome.reply.message.as_str(), "I'm so happy to see you");
    let player = h
        .store
        .chat_message(&outcome.player_message.message_id)
        .unwrap()
        .unwrap();
    assert_eq!(player.emoji_reactions.get(AI_REACTOR), Some("❤️"));
}

#[tokio::test]
async fn summary_regenerates_and_failure_preserves_prior() {
    // AC-CHAT-e: summary updates and resets counter; a failed summary preserves
    // the old summary.
    let h = harness();
    let c = character(InitialMessage::Message(InitialMessageText::coerce("Hi.")));
    h.store.save_character(&c).unwrap();
    let mut chat = h.engine.open_chat(&c.character_id).await.unwrap();
    chat.chat_summary = Some(soulfire_core::model::strings::StorySummary::coerce(
        "Old summary.",
    ));
    chat.messages_since_summary = 20;
    h.store.save_chat(&chat).unwrap();

    // Successful summary regen.
    h.provider
        .push(Scripted::text("They greeted each other warmly.", 200, 12));
    h.engine.generate_summary(&chat.chat_id).await.unwrap();
    let after = h.store.chat(&chat.chat_id).unwrap().unwrap();
    assert_eq!(
        after.chat_summary.unwrap().as_str(),
        "They greeted each other warmly."
    );
    assert_eq!(after.messages_since_summary, 0);

    // A failed summary pass preserves the prior summary.
    h.provider
        .push(Scripted::Error(ProviderError::RateLimited("429".into())));
    let err = h.engine.generate_summary(&chat.chat_id).await;
    assert!(err.is_err());
    let preserved = h.store.chat(&chat.chat_id).unwrap().unwrap();
    assert_eq!(
        preserved.chat_summary.unwrap().as_str(),
        "They greeted each other warmly."
    );
}

#[tokio::test]
async fn character_state_update_evolves_state_and_failure_preserves() {
    // AC-CHAT-f (single-pass): the dynamic state changes after a state update; a
    // failure leaves the prior state intact.
    let h = harness();
    let mut c = character(InitialMessage::Message(InitialMessageText::coerce("Hi.")));
    c.extracted_context = Some(CharacterContext::coerce("Lyra: a steadfast guide."));
    c.character_state = Some(CharacterContext::coerce("Calm and watchful."));
    h.store.save_character(&c).unwrap();
    let chat = h.engine.open_chat(&c.character_id).await.unwrap();
    let _ = chat;

    h.provider.push(Scripted::text(
        "You feel a growing warmth toward the player.",
        150,
        40,
    ));
    h.engine
        .run_character_state_update(&c.character_id)
        .await
        .unwrap();
    let updated = h.store.character(&c.character_id).unwrap().unwrap();
    assert_eq!(
        updated.character_state.unwrap().as_str(),
        "You feel a growing warmth toward the player."
    );

    // A failed update leaves the prior state intact (CHAT-13). Use a
    // non-transient error so it fails fast without retry/backoff.
    h.provider
        .push(Scripted::Error(ProviderError::RateLimited("429".into())));
    assert!(
        h.engine
            .run_character_state_update(&c.character_id)
            .await
            .is_err()
    );
    let preserved = h.store.character(&c.character_id).unwrap().unwrap();
    assert_eq!(
        preserved.character_state.unwrap().as_str(),
        "You feel a growing warmth toward the player."
    );
}
