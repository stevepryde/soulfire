use std::sync::Arc;

use soulfire_core::ai::{AiService, ApiKeySource, OpenAiProvider};
use soulfire_core::character::CharacterEngine;
use soulfire_core::chat::ChatEngine;
use soulfire_core::clock::SystemClock;
use soulfire_core::image::ImageEngine;
use soulfire_core::model::ai_model::AiVendor;
use soulfire_core::secret::Secret;
use soulfire_core::store::{AsyncStore, Store};
use soulfire_core::world::{WorldBuilderEngine, WorldEngine};

struct StoreKeySource {
    store: Arc<Store>,
}

impl ApiKeySource for StoreKeySource {
    fn api_key(&self, vendor: AiVendor) -> Option<Secret<String>> {
        self.store
            .credential(vendor)
            .ok()
            .flatten()
            .map(|credential| credential.key)
    }
}

pub fn chat_engine(store: &AsyncStore) -> ChatEngine {
    let (store, ai) = ai_runtime(store);
    ChatEngine::new(store, ai, Arc::new(SystemClock))
}

pub fn character_engine(store: &AsyncStore) -> CharacterEngine {
    let (store, ai) = ai_runtime(store);
    CharacterEngine::new(store, ai, Arc::new(SystemClock))
}

pub fn world_engine(store: &AsyncStore) -> WorldEngine {
    let (store, ai) = ai_runtime(store);
    WorldEngine::new(store, ai, Arc::new(SystemClock))
}

pub fn world_builder_engine(store: &AsyncStore) -> WorldBuilderEngine {
    let (store, ai) = ai_runtime(store);
    WorldBuilderEngine::new(store, ai, Arc::new(SystemClock))
}

pub fn image_engine(store: &AsyncStore) -> ImageEngine {
    let (store, ai) = ai_runtime(store);
    ImageEngine::new(store, ai, Arc::new(SystemClock))
}

fn ai_runtime(store: &AsyncStore) -> (Arc<Store>, AiService) {
    let store = store.inner();
    let keys: Arc<dyn ApiKeySource> = Arc::new(StoreKeySource {
        store: Arc::clone(&store),
    });
    let provider = Arc::new(OpenAiProvider::new(Arc::clone(&keys)));
    let ai = AiService::new(provider, keys);

    (store, ai)
}
