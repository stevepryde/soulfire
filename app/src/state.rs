//! App-level wiring: the unlocked engine context, the store-backed API-key
//! source, and the per-platform data location (`PKG-3`).

use std::path::PathBuf;
use std::sync::Arc;

use lib_soulfire::ai_model::AiVendor;
use sp_core::secret::Secret;

use soulfire_core::ai::provider::{AiProvider, ApiKeySource};
use soulfire_core::ai::{AiService, OpenAiProvider};
use soulfire_core::character::CharacterEngine;
use soulfire_core::chat::ChatEngine;
use soulfire_core::clock::{Clock, SystemClock};
use soulfire_core::image::ImageEngine;
use soulfire_core::store::Store;
use soulfire_core::world::{WorldBuilderEngine, WorldEngine};

/// An [`ApiKeySource`] that reads provider keys from the encrypted store on
/// demand (`SEC-9`). Keys are never held beyond a request.
pub struct StoreKeys(pub Arc<Store>);

impl ApiKeySource for StoreKeys {
    fn api_key(&self, vendor: AiVendor) -> Option<Secret<String>> {
        self.0.credential(vendor).ok().flatten().map(|c| c.key)
    }
}

/// The unlocked application context: the store and the wired engines. Cloned
/// into Dioxus context so screens can drive the core (`UI` design notes).
#[derive(Clone)]
pub struct AppContext {
    pub store: Arc<Store>,
    pub chat: ChatEngine,
    pub world: WorldEngine,
    pub world_builder: WorldBuilderEngine,
    pub character: CharacterEngine,
    pub image: ImageEngine,
}

impl AppContext {
    /// Wire the engines over an unlocked store.
    pub fn new(store: Store) -> Self {
        let store = Arc::new(store);
        let keys: Arc<dyn ApiKeySource> = Arc::new(StoreKeys(store.clone()));
        let provider: Arc<dyn AiProvider> = Arc::new(OpenAiProvider::new(keys.clone()));
        let ai = AiService::new(provider, keys);
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        AppContext {
            chat: ChatEngine::new(store.clone(), ai.clone(), clock.clone()),
            world: WorldEngine::new(store.clone(), ai.clone(), clock.clone()),
            world_builder: WorldBuilderEngine::new(store.clone(), ai.clone(), clock.clone()),
            character: CharacterEngine::new(store.clone(), ai.clone(), clock.clone()),
            image: ImageEngine::new(store.clone(), ai.clone(), clock.clone()),
            store,
        }
    }

    /// Whether an OpenAI key is configured (`AI-3` gating in the UI).
    pub fn has_api_key(&self) -> bool {
        self.store
            .credential(AiVendor::OpenAI)
            .ok()
            .flatten()
            .is_some()
    }
}

/// The per-user application data directory (`PKG-3`). Falls back to a temp dir if
/// the platform dirs are unavailable.
pub fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "Soulfire", "Soulfire")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir().join("soulfire"))
}

/// Whether the store has already been initialized at the data location (`SEC-4`).
pub fn is_initialized() -> bool {
    Store::is_initialized(data_dir())
}
