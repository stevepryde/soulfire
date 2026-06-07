//! Composer-draft persistence (`DATA-26`). At most one draft per scope.

use lib_soulfire::draft::{Draft, DraftScope};
use rusqlite::params;

use crate::error::CoreResult;
use crate::store::Store;

use super::{select_one, to_data};

impl Store {
    /// Save a draft, replacing any prior draft for the same scope (`DATA-26`).
    pub fn save_draft(&self, draft: &Draft) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO drafts (draft_id, scope_key, data) VALUES (?1, ?2, ?3)
                 ON CONFLICT(scope_key) DO UPDATE SET
                   draft_id = excluded.draft_id,
                   data = excluded.data",
                params![
                    draft.draft_id.to_string(),
                    draft.scope.key(),
                    to_data(draft)?,
                ],
            )?;
            Ok(())
        })
    }

    /// The draft for a scope, if any (`DATA-26`; restored on reopening).
    pub fn draft_for_scope(&self, scope: &DraftScope) -> CoreResult<Option<Draft>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM drafts WHERE scope_key = ?1",
                params![scope.key()],
            )
        })
    }

    /// Delete the draft for a scope (on submit or parent deletion, `DATA-26`).
    pub fn delete_draft_for_scope(&self, scope: &DraftScope) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM drafts WHERE scope_key = ?1",
                params![scope.key()],
            )?;
            Ok(())
        })
    }
}
