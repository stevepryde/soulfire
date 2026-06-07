//! The encrypted store lifecycle: initialize, unlock, lock, re-key, and
//! device-remembered unlock (`SEC-1`..`SEC-8`).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{CoreError, CoreResult};
use crate::keychain::Keychain;

use super::crypto::{DerivedKey, StoreMeta, sqlcipher_key_literal};
use super::schema;

/// File name of the encrypted database within the data directory.
pub const DB_FILE: &str = "soulfire.db";
/// File name of the plaintext key-derivation sidecar (`SEC-4`).
pub const META_FILE: &str = "soulfire.meta.json";
/// Keychain item name for the device-remembered store key (`SEC-7`).
pub const KEYCHAIN_KEY_NAME: &str = "soulfire-store-key";

/// Filesystem locations for a store's database and sidecar.
#[derive(Debug, Clone)]
pub struct StorePaths {
    pub data_dir: PathBuf,
    pub db: PathBuf,
    pub meta: PathBuf,
}

impl StorePaths {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        StorePaths {
            db: data_dir.join(DB_FILE),
            meta: data_dir.join(META_FILE),
            data_dir,
        }
    }

    /// Whether a store has been initialized at this location (sidecar present).
    pub fn is_initialized(&self) -> bool {
        self.meta.exists()
    }
}

/// An unlocked, open encrypted store. Holds the derived key in memory (zeroized on
/// drop) for re-key and device-remember operations (`SEC-8`, `SEC-7`).
pub struct Store {
    conn: Mutex<Connection>,
    key: DerivedKey,
    meta: StoreMeta,
    paths: StorePaths,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never render the key or connection; just the location.
        f.debug_struct("Store")
            .field("data_dir", &self.paths.data_dir)
            .finish_non_exhaustive()
    }
}

impl Store {
    /// Whether a store exists at `data_dir`.
    pub fn is_initialized(data_dir: impl AsRef<Path>) -> bool {
        StorePaths::new(data_dir).is_initialized()
    }

    /// First-run setup (`SEC-4`, `DATA-25`): choose a master password, create the
    /// encrypted database, install the schema, and seed the singleton rows.
    /// Fails if a store already exists.
    pub fn initialize(data_dir: impl AsRef<Path>, password: &str) -> CoreResult<Store> {
        let paths = StorePaths::new(&data_dir);
        if paths.is_initialized() {
            return Err(CoreError::AlreadyInitialized);
        }
        std::fs::create_dir_all(&paths.data_dir)
            .map_err(|e| CoreError::Store(format!("cannot create data dir: {e}")))?;
        // A stale db with no sidecar would be unopenable; start clean.
        let _ = std::fs::remove_file(&paths.db);

        let (meta, key) = StoreMeta::create(password)?;
        let conn = open_keyed(&paths.db, &key)?;
        schema::migrate(&conn)?;

        let store = Store {
            conn: Mutex::new(conn),
            key,
            meta,
            paths,
        };
        store.seed_singletons()?;
        // Write the sidecar only after the DB is fully built, so a crash mid-init
        // leaves "not initialized" rather than a half-built store.
        store.write_meta()?;
        Ok(store)
    }

    /// Unlock an existing store with the master password (`SEC-5`, `SEC-6`).
    pub fn unlock(data_dir: impl AsRef<Path>, password: &str) -> CoreResult<Store> {
        let paths = StorePaths::new(&data_dir);
        let meta = Self::read_meta(&paths)?;
        let key = meta.unlock(password)?; // WrongPassword if it does not verify
        let conn = open_keyed(&paths.db, &key)?;
        schema::migrate(&conn)?; // forward-migrate an older store (PKG-4)
        Ok(Store {
            conn: Mutex::new(conn),
            key,
            meta,
            paths,
        })
    }

    /// Unlock using a raw key (the device-remembered path, `SEC-7`).
    fn unlock_with_key(paths: StorePaths, key: DerivedKey) -> CoreResult<Store> {
        let meta = Self::read_meta(&paths)?;
        let conn = open_keyed(&paths.db, &key)?;
        schema::migrate(&conn)?;
        Ok(Store {
            conn: Mutex::new(conn),
            key,
            meta,
            paths,
        })
    }

    /// Attempt to unlock from the device keychain (`SEC-7`). Returns `Ok(None)`
    /// when no remembered key is present (falling back to the password prompt).
    pub fn unlock_from_keychain(
        data_dir: impl AsRef<Path>,
        keychain: &dyn Keychain,
    ) -> CoreResult<Option<Store>> {
        let paths = StorePaths::new(&data_dir);
        if !paths.is_initialized() {
            return Err(CoreError::NotInitialized);
        }
        let Some(bytes) = keychain.get(KEYCHAIN_KEY_NAME)? else {
            return Ok(None);
        };
        let key = key_from_bytes(&bytes)?;
        Ok(Some(Self::unlock_with_key(paths, key)?))
    }

    /// Store the current key in the device keychain so future launches unlock
    /// without a prompt (`SEC-7`).
    pub fn remember_on_device(&self, keychain: &dyn Keychain) -> CoreResult<()> {
        keychain.set(KEYCHAIN_KEY_NAME, self.key.as_ref())
    }

    /// Remove any device-remembered key (`SEC-7`).
    pub fn forget_device(keychain: &dyn Keychain) -> CoreResult<()> {
        keychain.delete(KEYCHAIN_KEY_NAME)
    }

    /// Re-key the store to a new master password such that the old password no
    /// longer unlocks it (`SEC-8`). Updates the sidecar and, if a device-
    /// remembered key exists, refreshes it.
    pub fn change_master_password(
        &mut self,
        new_password: &str,
        keychain: Option<&dyn Keychain>,
    ) -> CoreResult<()> {
        let (new_meta, new_key) = StoreMeta::rekey(new_password)?;
        {
            let conn = self.conn.lock().unwrap();
            conn.execute_batch(&format!(
                "PRAGMA rekey = \"{}\";",
                sqlcipher_key_literal(&new_key)
            ))?;
        }
        self.key = new_key;
        self.meta = new_meta;
        self.write_meta()?;

        // Refresh a device-remembered secret so it keeps working post-change.
        if let Some(kc) = keychain {
            if kc.get(KEYCHAIN_KEY_NAME)?.is_some() {
                kc.set(KEYCHAIN_KEY_NAME, self.key.as_ref())?;
            }
        }
        Ok(())
    }

    /// Run a closure with the open connection under the store lock. All store and
    /// repository access goes through here, serializing DB access for the
    /// single-process app.
    pub(crate) fn with_conn<R>(
        &self,
        f: impl FnOnce(&Connection) -> CoreResult<R>,
    ) -> CoreResult<R> {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }

    /// The key-derivation sidecar for this store (no secrets).
    pub fn meta(&self) -> &StoreMeta {
        &self.meta
    }

    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }

    fn read_meta(paths: &StorePaths) -> CoreResult<StoreMeta> {
        if !paths.is_initialized() {
            return Err(CoreError::NotInitialized);
        }
        let json = std::fs::read_to_string(&paths.meta)
            .map_err(|e| CoreError::Store(format!("cannot read sidecar: {e}")))?;
        StoreMeta::from_json(&json)
    }

    fn write_meta(&self) -> CoreResult<()> {
        let json = self.meta.to_json()?;
        std::fs::write(&self.paths.meta, json)
            .map_err(|e| CoreError::Store(format!("cannot write sidecar: {e}")))
    }
}

/// Open a connection at `path` and apply the SQLCipher raw key (`SEC-1`,
/// `SEC-2`). The key pragma must run before any other statement.
fn open_keyed(path: &Path, key: &DerivedKey) -> CoreResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(&format!("PRAGMA key = \"{}\";", sqlcipher_key_literal(key)))?;
    // Touch the schema to confirm the key actually decrypts the file. A wrong key
    // (or a corrupt/plaintext file) fails here rather than later (`SEC-6`).
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|_| CoreError::WrongPassword)?;
    Ok(conn)
}

fn key_from_bytes(bytes: &[u8]) -> CoreResult<DerivedKey> {
    use super::crypto::KEY_LEN;
    if bytes.len() != KEY_LEN {
        return Err(CoreError::Crypto("remembered key has wrong length".into()));
    }
    let mut key = zeroize::Zeroizing::new([0u8; KEY_LEN]);
    key.as_mut().copy_from_slice(bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keychain::InMemoryKeychain;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn initialize_then_unlock_with_correct_password() {
        let dir = tmp();
        {
            let _store = Store::initialize(dir.path(), "pw-123").unwrap();
            assert!(Store::is_initialized(dir.path()));
        }
        let _store = Store::unlock(dir.path(), "pw-123").unwrap();
    }

    #[test]
    fn wrong_password_is_rejected_and_store_stays_intact() {
        let dir = tmp();
        Store::initialize(dir.path(), "right").unwrap();
        let err = Store::unlock(dir.path(), "wrong").unwrap_err();
        assert!(matches!(err, CoreError::WrongPassword));
        // The right password still works afterward (store intact, SEC-6).
        Store::unlock(dir.path(), "right").unwrap();
    }

    #[test]
    fn double_initialize_fails() {
        let dir = tmp();
        Store::initialize(dir.path(), "pw").unwrap();
        let err = Store::initialize(dir.path(), "pw").unwrap_err();
        assert!(matches!(err, CoreError::AlreadyInitialized));
    }

    #[test]
    fn change_password_rejects_old_and_accepts_new() {
        let dir = tmp();
        {
            let mut store = Store::initialize(dir.path(), "old-pw").unwrap();
            store.change_master_password("new-pw", None).unwrap();
        }
        assert!(matches!(
            Store::unlock(dir.path(), "old-pw").unwrap_err(),
            CoreError::WrongPassword
        ));
        Store::unlock(dir.path(), "new-pw").unwrap();
    }

    #[test]
    fn device_remembered_unlock_round_trips_and_clears() {
        let dir = tmp();
        let kc = InMemoryKeychain::new();
        {
            let store = Store::initialize(dir.path(), "pw").unwrap();
            store.remember_on_device(&kc).unwrap();
        }
        // Subsequent launch unlocks without a password (SEC-7).
        let unlocked = Store::unlock_from_keychain(dir.path(), &kc).unwrap();
        assert!(unlocked.is_some());
        // Disabling removes the secret; next attempt falls back (None).
        Store::forget_device(&kc).unwrap();
        assert!(Store::unlock_from_keychain(dir.path(), &kc).unwrap().is_none());
    }

    #[test]
    fn remembered_key_survives_password_change() {
        let dir = tmp();
        let kc = InMemoryKeychain::new();
        {
            let mut store = Store::initialize(dir.path(), "old").unwrap();
            store.remember_on_device(&kc).unwrap();
            store.change_master_password("new", Some(&kc)).unwrap();
        }
        // The remembered unlock still opens the re-keyed store (SEC-8).
        assert!(Store::unlock_from_keychain(dir.path(), &kc).unwrap().is_some());
    }
}
