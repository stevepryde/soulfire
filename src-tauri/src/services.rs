use std::sync::Arc;

use soulfire_core::ai::{AiService, ApiKeySource, OpenAiProvider};
use soulfire_core::chat::ChatEngine;
use soulfire_core::clock::SystemClock;
use soulfire_core::model::ai_model::AiVendor;
use soulfire_core::secret::Secret;
use soulfire_core::store::{AsyncStore, Store};

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
    let store = store.inner();
    let keys: Arc<dyn ApiKeySource> = Arc::new(StoreKeySource {
        store: Arc::clone(&store),
    });
    let provider = Arc::new(OpenAiProvider::new(Arc::clone(&keys)));
    let ai = AiService::new(provider, keys);

    ChatEngine::new(store, ai, Arc::new(SystemClock))
}
