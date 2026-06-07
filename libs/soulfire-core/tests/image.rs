//! Image engine tests (TEST-14): async generation, version bump, failure keeps
//! the prior image, and at-rest encryption of stored bytes.

use std::str::FromStr;
use std::sync::Arc;

use lib_soulfire::ai_model::AiVendor;
use lib_soulfire::character::{Character, InitialMessage};
use lib_soulfire::strings::{CharacterName, InitialMessageText};
use sp_core::secret::Secret;

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
