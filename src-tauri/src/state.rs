use std::sync::{Arc, Mutex};

use soulfire_core::error::CoreResult;
use soulfire_core::store::Store;

use crate::error::CommandError;

#[derive(Default)]
pub struct AppState {
    store: Mutex<Option<Arc<Store>>>,
}

impl AppState {
    pub fn set_store(&self, store: Store) {
        *self.store.lock().unwrap() = Some(Arc::new(store));
    }

    pub fn clear_store(&self) {
        *self.store.lock().unwrap() = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.store.lock().unwrap().is_some()
    }

    pub fn schema_version(&self) -> Result<Option<u32>, CommandError> {
        let store = self.store.lock().unwrap().clone();
        match store {
            Some(store) => Ok(Some(store.schema_version()?)),
            None => Ok(None),
        }
    }

    pub fn with_store<R>(
        &self,
        f: impl FnOnce(&Store) -> CoreResult<R>,
    ) -> Result<R, CommandError> {
        let store = self.store.lock().unwrap().clone();
        let store = store.ok_or(CommandError::Locked)?;
        Ok(f(&store)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_tracks_locked_and_unlocked_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::initialize(dir.path(), "pw").unwrap();
        let state = AppState::default();

        assert!(!state.is_unlocked());
        assert_eq!(state.schema_version().unwrap(), None);
        assert!(matches!(
            state.with_store(|store| store.schema_version()),
            Err(CommandError::Locked)
        ));

        state.set_store(store);
        assert!(state.is_unlocked());
        assert!(state.schema_version().unwrap().is_some());

        state.clear_store();
        assert!(!state.is_unlocked());
    }
}
