//! Image engine tests (TEST-14): async generation, version bump, failure keeps
//! the prior image, and at-rest encryption of stored bytes.

use std::str::FromStr;
use std::sync::Arc;

use soulfire_core::model::ai_model::{AiModel, AiVendor};
use soulfire_core::model::character::{Character, InitialMessage};
use soulfire_core::model::strings::{
    CharacterDescription, CharacterName, InitialMessageText, WorldDescription, WorldPrompt,
    WorldTitle,
};
use soulfire_core::model::world::WorldBlueprint;
use soulfire_core::secret::Secret;

use soulfire_core::ai::fake::{RecordingProvider, Scripted};
use soulfire_core::ai::provider::ApiKeySource;
use soulfire_core::ai::service::AiService;
use soulfire_core::ai::types::{ProviderError, Usage};
use soulfire_core::clock::{Clock, MockClock};
use soulfire_core::image::ImageEngine;
use soulfire_core::store::{ImageOwnerKind, Store};

struct Keys;
impl ApiKeySource for Keys {
    fn api_key(&self, _v: AiVendor) -> Option<Secret<String>> {
        Some(Secret::new("sk".to_string()))
    }
}

fn img(bytes: &[u8]) -> Scripted {
    Scripted::Image {
        bytes: bytes.to_vec(),
        mime: "image/png".to_string(),
        usage: Usage {
            input_tokens: 5,
            output_tokens: 1,
            cached_input_tokens: None,
        },
    }
}

#[tokio::test]
async fn generate_then_regenerate_bumps_version_failure_keeps_prior() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::initialize(dir.path(), "pw").unwrap());
    let provider = Arc::new(RecordingProvider::new());
    let ai = AiService::new(provider.clone(), Arc::new(Keys));
    let engine = ImageEngine::new(
        store.clone(),
        ai,
        Arc::new(MockClock::at_epoch()) as Arc<dyn Clock>,
    );

    let c = Character::builder()
        .name(CharacterName::from_str("Lyra").unwrap())
        .description(CharacterDescription::coerce("A serene lantern keeper."))
        .initial_message(InitialMessage::Message(InitialMessageText::coerce("hi")))
        .build();
    store.save_character(&c).unwrap();

    // First generation.
    provider.push(img(b"PNGDATA-ZZSECRETBYTES-v1"));
    let r1 = engine
        .generate_character_portrait(&c.character_id)
        .await
        .unwrap();
    assert_eq!(r1.version, 1);
    let requests = provider.image_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].model, AiModel::Gpt5_1);
    assert!(requests[0].prompt.contains("Lyra"));
    assert!(requests[0].prompt.contains("serene lantern keeper"));
    assert!(requests[0].prompt.contains("character portrait"));
    assert!(requests[0].prompt.contains("Head-and-shoulders"));

    let stored = store
        .image(ImageOwnerKind::Character, &c.character_id.to_string())
        .unwrap()
        .unwrap();
    assert_eq!(stored.version, 1);
    assert_eq!(
        store
            .character(&c.character_id)
            .unwrap()
            .unwrap()
            .portrait
            .unwrap()
            .version,
        1
    );

    // Regeneration bumps the version (IMG-3 / AC-IMG-b).
    provider.push(img(b"PNGDATA-v2"));
    let r2 = engine
        .generate_character_portrait(&c.character_id)
        .await
        .unwrap();
    assert_eq!(r2.version, 2);

    // A failed generation leaves the prior image in place (IMG-2 / AC-IMG-a).
    provider.push(Scripted::Error(ProviderError::RateLimited("429".into())));
    assert!(
        engine
            .generate_character_portrait(&c.character_id)
            .await
            .is_err()
    );
    assert_eq!(
        store
            .character(&c.character_id)
            .unwrap()
            .unwrap()
            .portrait
            .unwrap()
            .version,
        2
    );
    assert_eq!(
        store
            .image(ImageOwnerKind::Character, &c.character_id.to_string())
            .unwrap()
            .unwrap()
            .version,
        2
    );

    // Clearing returns to the emoji avatar (IMG-3).
    engine.clear_character_portrait(&c.character_id).unwrap();
    assert!(
        store
            .character(&c.character_id)
            .unwrap()
            .unwrap()
            .portrait
            .is_none()
    );
    assert!(
        store
            .image(ImageOwnerKind::Character, &c.character_id.to_string())
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn world_cover_request_uses_blueprint_prompt_and_default_model() {
    // TEST-10 / IMG-1: world covers use the narrative default model and a
    // cinematic cover prompt derived from the local blueprint.
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::initialize(dir.path(), "pw").unwrap());
    let provider = Arc::new(RecordingProvider::new());
    let ai = AiService::new(provider.clone(), Arc::new(Keys));
    let engine = ImageEngine::new(
        store.clone(),
        ai,
        Arc::new(MockClock::at_epoch()) as Arc<dyn Clock>,
    );

    let bp = WorldBlueprint::builder()
        .title(WorldTitle::from_str("Beneath Verath").unwrap())
        .description(WorldDescription::coerce("A drowned city of secret bells."))
        .world_prompt(
            WorldPrompt::from_str("A long world prompt that is not needed here.").unwrap(),
        )
        .build();
    store.save_blueprint(&bp).unwrap();

    provider.push(img(b"PNGDATA-WORLD"));
    let cover = engine.generate_world_cover(&bp.blueprint_id).await.unwrap();
    assert_eq!(cover.version, 1);

    let requests = provider.image_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].model, AiModel::Gpt5_1);
    assert!(requests[0].prompt.contains("Beneath Verath"));
    assert!(requests[0].prompt.contains("Wide cinematic cover art"));
    assert!(
        requests[0]
            .prompt
            .contains("A drowned city of secret bells.")
    );
    assert!(requests[0].prompt.contains("no text"));

    let stored = store
        .image(ImageOwnerKind::World, &bp.blueprint_id.to_string())
        .unwrap()
        .unwrap();
    assert_eq!(stored.version, 1);

    let uploaded = engine
        .set_world_cover_bytes(&bp.blueprint_id, "image/png", b"PNGDATA-UPLOADED")
        .unwrap();
    assert_eq!(uploaded.version, 2);
    assert_eq!(
        store
            .image(ImageOwnerKind::World, &bp.blueprint_id.to_string())
            .unwrap()
            .unwrap()
            .bytes,
        b"PNGDATA-UPLOADED"
    );
}

#[tokio::test]
async fn stored_image_bytes_are_unreadable_on_disk_without_key() {
    // AC-IMG-c / TEST-14: stored image bytes are unreadable from disk.
    let dir = tempfile::tempdir().unwrap();
    let db_path;
    {
        let store = Arc::new(Store::initialize(dir.path(), "pw").unwrap());
        let provider = Arc::new(RecordingProvider::new());
        let ai = AiService::new(provider.clone(), Arc::new(Keys));
        let engine = ImageEngine::new(
            store.clone(),
            ai,
            Arc::new(MockClock::at_epoch()) as Arc<dyn Clock>,
        );
        let c = Character::builder()
            .name(CharacterName::from_str("Nox").unwrap())
            .initial_message(InitialMessage::Message(InitialMessageText::coerce("hi")))
            .build();
        store.save_character(&c).unwrap();
        provider.push(img(b"ZZSECRETBYTES-IMAGE-PLAINTEXT"));
        engine
            .generate_character_portrait(&c.character_id)
            .await
            .unwrap();
        db_path = store.paths().db.clone();
    }
    let bytes = std::fs::read(&db_path).unwrap();
    assert!(
        !bytes
            .windows(b"ZZSECRETBYTES".len())
            .any(|w| w == b"ZZSECRETBYTES"),
        "image bytes found in plaintext on disk"
    );
}
