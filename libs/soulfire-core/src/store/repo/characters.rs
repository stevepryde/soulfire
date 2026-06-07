//! Character persistence, the character-builder session, and character cascade
//! deletes (`DATA-1`, `DATA-14`, `DATA-22`).

use lib_soulfire::character::{Character, CharacterBuilderSession};
use lib_soulfire::ids::{CharacterId, ChatId};
use rusqlite::{OptionalExtension, params};

use crate::error::CoreResult;
use crate::store::Store;

use super::{select_many, select_one, to_data};

impl Store {
    /// Insert or update a character (`DATA-1`).
    pub fn save_character(&self, character: &Character) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO characters
                   (character_id, name, created_at, updated_at, last_chatted_at, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(character_id) DO UPDATE SET
                   name = excluded.name,
                   created_at = excluded.created_at,
                   updated_at = excluded.updated_at,
                   last_chatted_at = excluded.last_chatted_at,
                   data = excluded.data",
                params![
                    character.character_id.to_string(),
                    character.name.to_string(),
                    character.created_at.to_string(),
                    character.updated_at.to_string(),
                    character.last_chatted_at.map(|t| t.to_string()),
                    to_data(character)?,
                ],
            )?;
            Ok(())
        })
    }

    /// Load a character by id (`DATA-1`).
    pub fn character(&self, id: &CharacterId) -> CoreResult<Option<Character>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM characters WHERE character_id = ?1",
                params![id.to_string()],
            )
        })
    }

    /// List characters most-recently-chatted first, then newest, with optional
    /// case-insensitive name search (`CHAR-13`).
    ///
    /// Keyset (cursor) paging: pass the last character of the previous page as
    /// `after` to fetch the next page; the query seeks past that row rather than
    /// re-scanning the prefix, so each page costs `O(limit)` (`UI-22`). The order
    /// is a stable total order — `last_chatted_at` (non-null first, descending),
    /// then `created_at` descending, then `character_id` as the unique tiebreaker
    /// — which the cursor predicate mirrors exactly.
    pub fn list_characters(
        &self,
        search: Option<&str>,
        after: Option<&Character>,
        limit: u32,
    ) -> CoreResult<Vec<Character>> {
        self.with_conn(|conn| {
            let like = search.map(|s| format!("%{}%", s.to_lowercase()));
            // Cursor anchor columns (NULL-typed bindings when no cursor yet). The
            // `cg` group flag matches `last_chatted_at IS NULL` (0 = chatted).
            let (cg, clca, ccrt, cid): (
                Option<i64>,
                Option<String>,
                Option<String>,
                Option<String>,
            ) = match after {
                Some(c) => (
                    Some(c.last_chatted_at.is_none() as i64),
                    c.last_chatted_at.map(|t| t.to_string()),
                    Some(c.created_at.to_string()),
                    Some(c.character_id.to_string()),
                ),
                None => (None, None, None, None),
            };
            // `?5 IS NULL` (cid) means "no cursor" -> first page, no keyset filter.
            // Otherwise admit only rows strictly after the cursor in the order
            // above. `IS` compares NULLs as equal so the null-chatted group is
            // handled without special-casing.
            let sql = "SELECT data FROM characters
                       WHERE (?1 IS NULL OR lower(name) LIKE ?1)
                         AND (
                           ?5 IS NULL
                           OR (last_chatted_at IS NULL) > ?2
                           OR ((last_chatted_at IS NULL) = ?2 AND (
                                 last_chatted_at < ?3
                              OR (last_chatted_at IS ?3 AND created_at < ?4)
                              OR (last_chatted_at IS ?3 AND created_at = ?4 AND character_id < ?5)
                           ))
                         )
                       ORDER BY last_chatted_at IS NULL, last_chatted_at DESC,
                                created_at DESC, character_id DESC
                       LIMIT ?6";
            select_many(conn, sql, params![like, cg, clca, ccrt, cid, limit])
        })
    }

    /// Total number of characters (for empty-state / pagination).
    pub fn count_characters(&self) -> CoreResult<i64> {
        self.with_conn(|conn| super::count(conn, "SELECT count(*) FROM characters", []))
    }

    /// Delete a character and everything that depends on it: its chat, that
    /// chat's messages and draft, and its builder session (`DATA-22`). Also drops
    /// any stored portrait. No orphan rows remain.
    pub fn delete_character(&self, id: &CharacterId) -> CoreResult<()> {
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            let id_s = id.to_string();

            // Find the character's chat (if any) and cascade through it.
            let chat_id: Option<String> = tx
                .query_row(
                    "SELECT chat_id FROM chats WHERE character_id = ?1",
                    params![id_s],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(chat_id) = chat_id {
                tx.execute(
                    "DELETE FROM chat_messages WHERE chat_id = ?1",
                    params![chat_id],
                )?;
                tx.execute(
                    "DELETE FROM drafts WHERE scope_key = ?1",
                    params![format!("chat:{chat_id}")],
                )?;
                tx.execute("DELETE FROM chats WHERE chat_id = ?1", params![chat_id])?;
            }

            tx.execute(
                "DELETE FROM character_builder_sessions WHERE character_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM images WHERE owner_kind = 'character' AND owner_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM characters WHERE character_id = ?1",
                params![id_s],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    // ----- Character builder session (DATA-14) -----

    /// The builder session for a character, if one has been started (`DATA-14`).
    pub fn character_builder_session(
        &self,
        character_id: &CharacterId,
    ) -> CoreResult<Option<CharacterBuilderSession>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM character_builder_sessions WHERE character_id = ?1",
                params![character_id.to_string()],
            )
        })
    }

    /// Insert or update a character builder session (`DATA-14`).
    pub fn save_character_builder_session(
        &self,
        session: &CharacterBuilderSession,
    ) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO character_builder_sessions (character_id, data) VALUES (?1, ?2)
                 ON CONFLICT(character_id) DO UPDATE SET data = excluded.data",
                params![session.character_id.to_string(), to_data(session)?],
            )?;
            Ok(())
        })
    }

    /// Whether a chat already exists for a character (`DATA-5`: at most one).
    pub fn chat_id_for_character(&self, character_id: &CharacterId) -> CoreResult<Option<ChatId>> {
        self.with_conn(|conn| {
            let id: Option<String> = conn
                .query_row(
                    "SELECT chat_id FROM chats WHERE character_id = ?1",
                    params![character_id.to_string()],
                    |r| r.get(0),
                )
                .optional()?;
            Ok(id.and_then(|s| s.parse().ok()))
        })
    }
}
