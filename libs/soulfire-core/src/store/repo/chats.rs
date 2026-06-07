//! Chat and chat-message persistence (`DATA-5`, `DATA-6`, `DATA-22`).

use lib_soulfire::chat::{Chat, ChatMessage};
use lib_soulfire::ids::{ChatId, MessageId};
use rusqlite::params;

use crate::error::CoreResult;
use crate::store::Store;

use super::{select_many, select_one, to_data};

impl Store {
    /// Insert or update a chat (`DATA-5`). The `UNIQUE(character_id)` constraint
    /// enforces at most one chat per character.
    pub fn save_chat(&self, chat: &Chat) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO chats (chat_id, character_id, updated_at, data)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(chat_id) DO UPDATE SET
                   character_id = excluded.character_id,
                   updated_at = excluded.updated_at,
                   data = excluded.data",
                params![
                    chat.chat_id.to_string(),
                    chat.character_id.as_ref().map(|c| c.to_string()),
                    chat.updated_at.to_string(),
                    to_data(chat)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn chat(&self, id: &ChatId) -> CoreResult<Option<Chat>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM chats WHERE chat_id = ?1",
                params![id.to_string()],
            )
        })
    }

    /// Delete a chat: removes its messages and draft but keeps the character
    /// (`DATA-22`).
    pub fn delete_chat(&self, id: &ChatId) -> CoreResult<()> {
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            let id_s = id.to_string();
            tx.execute(
                "DELETE FROM chat_messages WHERE chat_id = ?1",
                params![id_s],
            )?;
            tx.execute(
                "DELETE FROM drafts WHERE scope_key = ?1",
                params![format!("chat:{id_s}")],
            )?;
            tx.execute("DELETE FROM chats WHERE chat_id = ?1", params![id_s])?;
            tx.commit()?;
            Ok(())
        })
    }

    // ----- Messages (DATA-6) -----

    pub fn save_chat_message(&self, message: &ChatMessage) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO chat_messages (message_id, chat_id, created_at, data)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(message_id) DO UPDATE SET
                   chat_id = excluded.chat_id,
                   created_at = excluded.created_at,
                   data = excluded.data",
                params![
                    message.message_id.to_string(),
                    message.chat_id.to_string(),
                    message.created_at.to_string(),
                    to_data(message)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn chat_message(&self, id: &MessageId) -> CoreResult<Option<ChatMessage>> {
        self.with_conn(|conn| {
            select_one(
                conn,
                "SELECT data FROM chat_messages WHERE message_id = ?1",
                params![id.to_string()],
            )
        })
    }

    /// All messages for a chat in chronological order (`DATA-6`).
    pub fn chat_messages(&self, chat_id: &ChatId) -> CoreResult<Vec<ChatMessage>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM chat_messages WHERE chat_id = ?1 ORDER BY rowid ASC",
                params![chat_id.to_string()],
            )
        })
    }

    /// The most recent `limit` messages for a chat, returned in chronological
    /// order (oldest→newest), for bounded prompt history (`CHAT-5`).
    pub fn recent_chat_messages(
        &self,
        chat_id: &ChatId,
        limit: u32,
    ) -> CoreResult<Vec<ChatMessage>> {
        self.with_conn(|conn| {
            let mut newest_first: Vec<ChatMessage> = select_many(
                conn,
                "SELECT data FROM chat_messages WHERE chat_id = ?1
                 ORDER BY rowid DESC LIMIT ?2",
                params![chat_id.to_string(), limit],
            )?;
            newest_first.reverse();
            Ok(newest_first)
        })
    }

    pub fn count_chat_messages(&self, chat_id: &ChatId) -> CoreResult<i64> {
        self.with_conn(|conn| {
            super::count(
                conn,
                "SELECT count(*) FROM chat_messages WHERE chat_id = ?1",
                params![chat_id.to_string()],
            )
        })
    }
}
