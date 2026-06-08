//! World blueprint, adventure, adventure-message, GM-proposal, and world-builder
//! persistence with cascade deletes (`DATA-8`..`DATA-15`, `DATA-22`).

use crate::model::ids::{AdventureId, GmProposalId, WorldBlueprintId};
use crate::model::world::{
    Adventure, AdventureMessage, GmProposal, GmProposalStatus, WorldBlueprint, WorldBuilderSession,
};
use rusqlite::params;

use crate::error::CoreResult;
use crate::store::Store;

use super::{count, select_many, select_one, to_data};

impl Store {
    // ----- Blueprints (DATA-8) -----

    pub fn save_blueprint(&self, bp: &WorldBlueprint) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO world_blueprints (blueprint_id, title, created_at, updated_at, data)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(blueprint_id) DO UPDATE SET
                   title = excluded.title,
                   created_at = excluded.created_at,
                   updated_at = excluded.updated_at,
                   data = excluded.data",
                params![
                    bp.blueprint_id.to_string(),
                    bp.title.to_string(),
                    bp.created_at.to_string(),
                    bp.updated_at.to_string(),
                    to_data(bp)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn blueprint(&self, id: &WorldBlueprintId) -> CoreResult<Option<WorldBlueprint>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM world_blueprints WHERE blueprint_id = ?1",
                params![id.to_string()],
            )
        })
    }

    /// List world blueprints, most-recently-updated first, with optional
    /// case-insensitive title search. Keyset paging: pass the last blueprint of
    /// the previous page as `after` to seek the next page via the
    /// `(updated_at, blueprint_id)` index, ordered by `updated_at` then the
    /// unique `blueprint_id` tiebreaker (`UI-22`).
    pub fn list_blueprints(
        &self,
        search: Option<&str>,
        after: Option<&WorldBlueprint>,
        limit: u32,
    ) -> CoreResult<Vec<WorldBlueprint>> {
        self.with_conn(|conn| {
            let like = search.map(|s| format!("%{}%", s.to_lowercase()));
            let (cup, cid): (Option<String>, Option<String>) = match after {
                Some(b) => (
                    Some(b.updated_at.to_string()),
                    Some(b.blueprint_id.to_string()),
                ),
                None => (None, None),
            };
            select_many(
                conn,
                "SELECT data FROM world_blueprints
                 WHERE (?1 IS NULL OR lower(title) LIKE ?1)
                   AND (
                     ?3 IS NULL
                     OR updated_at < ?2
                     OR (updated_at = ?2 AND blueprint_id < ?3)
                   )
                 ORDER BY updated_at DESC, blueprint_id DESC
                 LIMIT ?4",
                params![like, cup, cid, limit],
            )
        })
    }

    pub fn count_blueprints(&self) -> CoreResult<i64> {
        self.with_conn(|conn| count(conn, "SELECT count(*) FROM world_blueprints", []))
    }

    /// Delete a blueprint and cascade: its adventures (and their messages,
    /// proposals, drafts), its builder session, and its cover (`DATA-22`).
    pub fn delete_blueprint(&self, id: &WorldBlueprintId) -> CoreResult<()> {
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            let id_s = id.to_string();
            tx.execute(
                "DELETE FROM adventure_messages WHERE adventure_id IN
                   (SELECT adventure_id FROM adventures WHERE blueprint_id = ?1)",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM gm_proposals WHERE adventure_id IN
                   (SELECT adventure_id FROM adventures WHERE blueprint_id = ?1)",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM drafts WHERE scope_key IN
                   (SELECT 'adventure:' || adventure_id FROM adventures WHERE blueprint_id = ?1)",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM adventures WHERE blueprint_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM world_builder_sessions WHERE blueprint_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM images WHERE owner_kind = 'world' AND owner_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM world_blueprints WHERE blueprint_id = ?1",
                params![id_s],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    // ----- Adventures (DATA-10) -----

    pub fn save_adventure(&self, adv: &Adventure) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO adventures
                   (adventure_id, blueprint_id, story_status, has_completed, created_at, updated_at, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(adventure_id) DO UPDATE SET
                   blueprint_id = excluded.blueprint_id,
                   story_status = excluded.story_status,
                   has_completed = excluded.has_completed,
                   created_at = excluded.created_at,
                   updated_at = excluded.updated_at,
                   data = excluded.data",
                params![
                    adv.adventure_id.to_string(),
                    adv.blueprint_id.to_string(),
                    adv.story_status.to_string(),
                    adv.has_completed as i64,
                    adv.created_at.to_string(),
                    adv.updated_at.to_string(),
                    to_data(adv)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn adventure(&self, id: &AdventureId) -> CoreResult<Option<Adventure>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM adventures WHERE adventure_id = ?1",
                params![id.to_string()],
            )
        })
    }

    /// Adventures most-recently-updated first (the Adventures tab, `UI-8`).
    /// Keyset paging: pass the last adventure of the previous page as `after` to
    /// seek the next page via the `(updated_at, adventure_id)` index, ordered by
    /// `updated_at` then the unique `adventure_id` tiebreaker (`UI-22`).
    pub fn list_adventures(
        &self,
        after: Option<&Adventure>,
        limit: u32,
    ) -> CoreResult<Vec<Adventure>> {
        self.with_conn(|conn| {
            let (cup, cid): (Option<String>, Option<String>) = match after {
                Some(a) => (
                    Some(a.updated_at.to_string()),
                    Some(a.adventure_id.to_string()),
                ),
                None => (None, None),
            };
            select_many(
                conn,
                "SELECT data FROM adventures
                 WHERE (
                   ?2 IS NULL
                   OR updated_at < ?1
                   OR (updated_at = ?1 AND adventure_id < ?2)
                 )
                 ORDER BY updated_at DESC, adventure_id DESC
                 LIMIT ?3",
                params![cup, cid, limit],
            )
        })
    }

    /// In-progress (ongoing) adventures for the "continue where you left off"
    /// surface (`WORLD-19`, `ONB-6`).
    pub fn in_progress_adventures(&self, limit: u32) -> CoreResult<Vec<Adventure>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM adventures WHERE has_completed = 0
                 ORDER BY updated_at DESC LIMIT ?1",
                params![limit],
            )
        })
    }

    pub fn adventures_for_blueprint(
        &self,
        blueprint_id: &WorldBlueprintId,
    ) -> CoreResult<Vec<Adventure>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM adventures WHERE blueprint_id = ?1 ORDER BY updated_at DESC",
                params![blueprint_id.to_string()],
            )
        })
    }

    /// Delete an adventure and its messages and pending proposals and draft
    /// (`DATA-22`).
    pub fn delete_adventure(&self, id: &AdventureId) -> CoreResult<()> {
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            let id_s = id.to_string();
            tx.execute(
                "DELETE FROM adventure_messages WHERE adventure_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM gm_proposals WHERE adventure_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM drafts WHERE scope_key = ?1",
                params![format!("adventure:{id_s}")],
            )?;
            tx.execute(
                "DELETE FROM adventures WHERE adventure_id = ?1",
                params![id_s],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    // ----- Adventure messages (DATA-12) -----

    pub fn save_adventure_message(&self, message: &AdventureMessage) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO adventure_messages (message_id, adventure_id, created_at, data)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(message_id) DO UPDATE SET
                   adventure_id = excluded.adventure_id,
                   created_at = excluded.created_at,
                   data = excluded.data",
                params![
                    message.message_id.to_string(),
                    message.adventure_id.to_string(),
                    message.created_at.to_string(),
                    to_data(message)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn adventure_messages(
        &self,
        adventure_id: &AdventureId,
    ) -> CoreResult<Vec<AdventureMessage>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM adventure_messages WHERE adventure_id = ?1
                 ORDER BY rowid ASC",
                params![adventure_id.to_string()],
            )
        })
    }

    /// The most recent `limit` turn-log entries in chronological order.
    pub fn recent_adventure_messages(
        &self,
        adventure_id: &AdventureId,
        limit: u32,
    ) -> CoreResult<Vec<AdventureMessage>> {
        self.with_conn(|conn| {
            let mut newest_first: Vec<AdventureMessage> = select_many(
                conn,
                "SELECT data FROM adventure_messages WHERE adventure_id = ?1
                 ORDER BY rowid DESC LIMIT ?2",
                params![adventure_id.to_string(), limit],
            )?;
            newest_first.reverse();
            Ok(newest_first)
        })
    }

    // ----- GM proposals (DATA-13) -----

    pub fn save_gm_proposal(&self, proposal: &GmProposal) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO gm_proposals
                   (proposal_id, adventure_id, response_message_id, status, data)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(proposal_id) DO UPDATE SET
                   adventure_id = excluded.adventure_id,
                   response_message_id = excluded.response_message_id,
                   status = excluded.status,
                   data = excluded.data",
                params![
                    proposal.proposal_id.to_string(),
                    proposal.adventure_id.to_string(),
                    proposal.response_message_id.to_string(),
                    proposal.status.to_string(),
                    to_data(proposal)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn gm_proposal(&self, id: &GmProposalId) -> CoreResult<Option<GmProposal>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM gm_proposals WHERE proposal_id = ?1",
                params![id.to_string()],
            )
        })
    }

    /// Pending (not yet accepted/rejected) proposals for an adventure (`WORLD-17`).
    pub fn pending_gm_proposals(&self, adventure_id: &AdventureId) -> CoreResult<Vec<GmProposal>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM gm_proposals WHERE adventure_id = ?1 AND status = ?2",
                params![
                    adventure_id.to_string(),
                    GmProposalStatus::Pending.to_string()
                ],
            )
        })
    }

    // ----- World builder session (DATA-15) -----

    pub fn world_builder_session(
        &self,
        blueprint_id: &WorldBlueprintId,
    ) -> CoreResult<Option<WorldBuilderSession>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM world_builder_sessions WHERE blueprint_id = ?1",
                params![blueprint_id.to_string()],
            )
        })
    }

    pub fn save_world_builder_session(&self, session: &WorldBuilderSession) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO world_builder_sessions (blueprint_id, data) VALUES (?1, ?2)
                 ON CONFLICT(blueprint_id) DO UPDATE SET data = excluded.data",
                params![session.blueprint_id.to_string(), to_data(session)?],
            )?;
            Ok(())
        })
    }
}
