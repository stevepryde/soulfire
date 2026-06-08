//! Async store facade for app-shell command handlers.
//!
//! SQLCipher/rusqlite access is synchronous, so UI-facing async callers should
//! enter the store through this wrapper. Each operation runs on Tokio's blocking
//! pool, keeping Tauri async commands free to keep the webview responsive
//! (`UI-24`, `PROD-17`).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{CoreError, CoreResult};

use super::Store;

/// Cloneable async handle around an unlocked encrypted store.
#[derive(Clone, Debug)]
pub struct AsyncStore {
    inner: Arc<Store>,
}

impl AsyncStore {
    /// First-run setup on a blocking worker, returning a UI-safe async handle.
    pub async fn initialize(
        data_dir: impl AsRef<Path>,
        password: impl Into<String>,
    ) -> CoreResult<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        let password = password.into();
        let store = spawn_store_task(move || Store::initialize(data_dir, &password)).await?;
        Ok(Self::from_store(store))
    }

    /// Unlock an existing store on a blocking worker.
    pub async fn unlock(
        data_dir: impl AsRef<Path>,
        password: impl Into<String>,
    ) -> CoreResult<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        let password = password.into();
        let store = spawn_store_task(move || Store::unlock(data_dir, &password)).await?;
        Ok(Self::from_store(store))
    }

    /// Wrap an already-open store.
    pub fn from_store(store: Store) -> Self {
        AsyncStore {
            inner: Arc::new(store),
        }
    }

    /// Shared access for orchestration layers that need to compose core engines.
    pub fn inner(&self) -> Arc<Store> {
        Arc::clone(&self.inner)
    }

    /// Run synchronous store/repository work on Tokio's blocking pool.
    ///
    /// Tauri command handlers should prefer this method over calling synchronous
    /// store methods directly. The closure may call any `Store` repository
    /// method and should return only DTO/model values that are safe to send back
    /// to the async caller.
    pub async fn run<R>(
        &self,
        operation: impl FnOnce(&Store) -> CoreResult<R> + Send + 'static,
    ) -> CoreResult<R>
    where
        R: Send + 'static,
    {
        let store = Arc::clone(&self.inner);
        spawn_store_task(move || operation(&store)).await
    }
}

async fn spawn_store_task<R>(
    operation: impl FnOnce() -> CoreResult<R> + Send + 'static,
) -> CoreResult<R>
where
    R: Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(join_error)?
}

fn join_error(err: tokio::task::JoinError) -> CoreError {
    CoreError::Store(format!("blocking store task failed: {err}"))
}

#[allow(dead_code)]
fn _assert_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AsyncStore>();
    assert_send_sync::<PathBuf>();
}
