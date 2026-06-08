use std::sync::Mutex;

use soulfire_core::error::CoreResult;
use soulfire_core::store::{AsyncStore, Store};

use crate::error::CommandError;

#[derive(Default)]
pub struct AppState {
    store: Mutex<Option<AsyncStore>>,
}

impl AppState {
    pub fn set_store(&self, store: AsyncStore) {
        *self.store.lock().unwrap() = Some(store);
    }

    pub fn clear_store(&self) {
        *self.store.lock().unwrap() = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.store.lock().unwrap().is_some()
    }

    pub fn store_handle(&self) -> Result<AsyncStore, CommandError> {
        self.store
            .lock()
            .unwrap()
            .clone()
            .ok_or(CommandError::Locked)
    }

    pub async fn schema_version(&self) -> Result<Option<u32>, CommandError> {
        let store = self.store.lock().unwrap().clone();
        match store {
            Some(store) => Ok(Some(store.run(Store::schema_version).await?)),
            None => Ok(None),
        }
    }

    pub async fn with_store<R: Send + 'static>(
        &self,
        f: impl FnOnce(&Store) -> CoreResult<R> + Send + 'static,
    ) -> Result<R, CommandError> {
        let store = self.store.lock().unwrap().clone();
        let store = store.ok_or(CommandError::Locked)?;
        Ok(store.run(f).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soulfire_core::model::settings::{AppSettings, ColorTheme, ContentToggles};

    #[tokio::test(flavor = "current_thread")]
    async fn state_tracks_locked_and_unlocked_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = AsyncStore::initialize(dir.path(), "pw").await.unwrap();
        let state = AppState::default();

        assert!(!state.is_unlocked());
        assert_eq!(state.schema_version().await.unwrap(), None);
        assert!(matches!(
            state.with_store(Store::schema_version).await,
            Err(CommandError::Locked)
        ));

        state.set_store(store);
        assert!(state.is_unlocked());
        assert!(state.schema_version().await.unwrap().is_some());

        state.clear_store();
        assert!(!state.is_unlocked());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn state_persists_async_store_work_across_lock_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let store = AsyncStore::initialize(dir.path(), "pw").await.unwrap();
        let state = AppState::default();
        state.set_store(store);

        let settings = AppSettings {
            color_theme: ColorTheme::Teal,
            content_toggles: ContentToggles {
                adult_content: true,
            },
            ..AppSettings::default()
        };
        state
            .with_store(move |store| {
                store.save_app_settings(&settings)?;
                Ok(())
            })
            .await
            .unwrap();

        state.clear_store();
        assert!(matches!(
            state.with_store(Store::app_settings).await,
            Err(CommandError::Locked)
        ));

        state.set_store(AsyncStore::unlock(dir.path(), "pw").await.unwrap());
        let restored = state.with_store(Store::app_settings).await.unwrap();
        assert_eq!(restored.color_theme, ColorTheme::Teal);
        assert!(restored.content_toggles.adult_content);
    }
}
