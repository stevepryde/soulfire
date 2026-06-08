//! Singleton rows (`DATA-16`..`DATA-18`, `DATA-24`, `DATA-25`) and provider
//! credentials (`DATA-19`).

use crate::model::ai_model::AiVendor;
use crate::model::credentials::ProviderCredential;
use crate::model::install::InstallState;
use crate::model::profile::{AppProfile, PlayerProfile};
use crate::model::settings::AppSettings;
use rusqlite::params;

use crate::error::CoreResult;
use crate::store::Store;

use super::{select_many, select_one, to_data};

impl Store {
    /// Seed the four singleton rows for a fresh store (`DATA-25`). Idempotent via
    /// `INSERT OR IGNORE` so re-running during a forward migration is safe.
    pub(crate) fn seed_singletons(&self) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO app_profile (id, data) VALUES (0, ?1)",
                params![to_data(&AppProfile::default())?],
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO player_profile (id, data) VALUES (0, ?1)",
                params![to_data(&PlayerProfile::default())?],
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO app_settings (id, data) VALUES (0, ?1)",
                params![to_data(&AppSettings::default())?],
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO install_state (id, data) VALUES (0, ?1)",
                params![to_data(&InstallState::default())?],
            )?;
            Ok(())
        })
    }

    // ----- App profile (DATA-16) -----

    pub fn app_profile(&self) -> CoreResult<AppProfile> {
        self.with_conn(|conn| {
            Ok(
                select_one(conn, "SELECT data FROM app_profile WHERE id = 0", [])?
                    .unwrap_or_default(),
            )
        })
    }

    pub fn save_app_profile(&self, profile: &AppProfile) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE app_profile SET data = ?1 WHERE id = 0",
                params![to_data(profile)?],
            )?;
            Ok(())
        })
    }

    // ----- Player profile (DATA-17) -----

    pub fn player_profile(&self) -> CoreResult<PlayerProfile> {
        self.with_conn(|conn| {
            Ok(
                select_one(conn, "SELECT data FROM player_profile WHERE id = 0", [])?
                    .unwrap_or_default(),
            )
        })
    }

    pub fn save_player_profile(&self, profile: &PlayerProfile) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE player_profile SET data = ?1 WHERE id = 0",
                params![to_data(profile)?],
            )?;
            Ok(())
        })
    }

    // ----- App settings (DATA-18) -----

    pub fn app_settings(&self) -> CoreResult<AppSettings> {
        self.with_conn(|conn| {
            Ok(
                select_one(conn, "SELECT data FROM app_settings WHERE id = 0", [])?
                    .unwrap_or_default(),
            )
        })
    }

    pub fn save_app_settings(&self, settings: &AppSettings) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE app_settings SET data = ?1 WHERE id = 0",
                params![to_data(settings)?],
            )?;
            Ok(())
        })
    }

    // ----- Install state (DATA-24) -----

    pub fn install_state(&self) -> CoreResult<InstallState> {
        self.with_conn(|conn| {
            Ok(
                select_one(conn, "SELECT data FROM install_state WHERE id = 0", [])?
                    .unwrap_or_default(),
            )
        })
    }

    pub fn save_install_state(&self, state: &InstallState) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE install_state SET data = ?1 WHERE id = 0",
                params![to_data(state)?],
            )?;
            Ok(())
        })
    }

    // ----- Credentials (DATA-19) -----

    /// The stored credential for a provider, if any (`DATA-19`).
    pub fn credential(&self, provider: AiVendor) -> CoreResult<Option<ProviderCredential>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM credentials WHERE provider = ?1",
                params![provider.to_string()],
            )
        })
    }

    /// Store (or replace) a provider credential (`DATA-19`, `SEC-9`).
    pub fn save_credential(&self, credential: &ProviderCredential) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO credentials (provider, data) VALUES (?1, ?2)
                 ON CONFLICT(provider) DO UPDATE SET data = excluded.data",
                params![credential.provider.to_string(), to_data(credential)?],
            )?;
            Ok(())
        })
    }

    /// Remove a provider credential (`SEC-10` clear).
    pub fn delete_credential(&self, provider: AiVendor) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM credentials WHERE provider = ?1",
                params![provider.to_string()],
            )?;
            Ok(())
        })
    }

    /// All stored credentials.
    pub fn credentials(&self) -> CoreResult<Vec<ProviderCredential>> {
        self.with_conn(|conn| select_many(conn, "SELECT data FROM credentials", []))
    }
}
