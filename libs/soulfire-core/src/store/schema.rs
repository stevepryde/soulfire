//! Database schema and migrations.
//!
//! One table per entity (`DATA` design note). Each table carries the columns
//! needed for indexed queries plus a `data` JSON column holding the full
//! serialized record, so records port with full fidelity and evolve via their
//! per-record `version` (`DATA`, `PKG-4`). Timestamps are stored as the
//! RFC 3339 string form of `SpDateTime`, which sorts chronologically.

use rusqlite::Connection;

use crate::error::CoreResult;

/// The current schema version. Bumped when a migration is added (`PKG-4`).
pub const SCHEMA_VERSION: u32 = 1;

/// Apply schema migrations up to [`SCHEMA_VERSION`], idempotently. Uses
/// SQLite's `user_version` pragma to track the applied version so a newer app
/// migrates an older store forward transparently (`PKG-4`).
pub fn migrate(conn: &Connection) -> CoreResult<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let current: u32 =
        conn.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))? as u32;

    if current < 1 {
        conn.execute_batch(SCHEMA_V1)?;
    }

    if current != SCHEMA_VERSION {
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }
    Ok(())
}

/// Schema version 1: the full launch schema.
const SCHEMA_V1: &str = r#"
-- Singletons (DATA-16, DATA-17, DATA-18, DATA-24). The CHECK pins one row.
CREATE TABLE app_profile   (id INTEGER PRIMARY KEY CHECK (id = 0), data TEXT NOT NULL);
CREATE TABLE player_profile(id INTEGER PRIMARY KEY CHECK (id = 0), data TEXT NOT NULL);
CREATE TABLE app_settings  (id INTEGER PRIMARY KEY CHECK (id = 0), data TEXT NOT NULL);
CREATE TABLE install_state (id INTEGER PRIMARY KEY CHECK (id = 0), data TEXT NOT NULL);

-- Provider credentials (DATA-19). Encrypted with the whole store (SEC-1).
CREATE TABLE credentials (
    provider TEXT PRIMARY KEY,
    data     TEXT NOT NULL
);

-- Characters (DATA-1).
CREATE TABLE characters (
    character_id    TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    last_chatted_at TEXT,
    data            TEXT NOT NULL
);
CREATE INDEX idx_characters_last_chatted ON characters(last_chatted_at);

-- Chats (DATA-5). UNIQUE(character_id) => at most one chat per character.
CREATE TABLE chats (
    chat_id      TEXT PRIMARY KEY,
    character_id TEXT UNIQUE,
    updated_at   TEXT NOT NULL,
    data         TEXT NOT NULL
);

-- Chat messages (DATA-6).
CREATE TABLE chat_messages (
    message_id TEXT PRIMARY KEY,
    chat_id    TEXT NOT NULL,
    created_at TEXT NOT NULL,
    data       TEXT NOT NULL
);
CREATE INDEX idx_chat_messages_chat ON chat_messages(chat_id, created_at);

-- Character builder sessions (DATA-14), keyed to one character.
CREATE TABLE character_builder_sessions (
    character_id TEXT PRIMARY KEY,
    data         TEXT NOT NULL
);

-- World blueprints (DATA-8).
CREATE TABLE world_blueprints (
    blueprint_id TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    data         TEXT NOT NULL
);

-- Adventures (DATA-10).
CREATE TABLE adventures (
    adventure_id  TEXT PRIMARY KEY,
    blueprint_id  TEXT NOT NULL,
    story_status  TEXT NOT NULL,
    has_completed INTEGER NOT NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    data          TEXT NOT NULL
);
CREATE INDEX idx_adventures_blueprint ON adventures(blueprint_id);
CREATE INDEX idx_adventures_updated   ON adventures(updated_at);

-- Adventure turn-log messages (DATA-12).
CREATE TABLE adventure_messages (
    message_id   TEXT PRIMARY KEY,
    adventure_id TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    data         TEXT NOT NULL
);
CREATE INDEX idx_adventure_messages_adv ON adventure_messages(adventure_id, created_at);

-- Staged GM proposals (DATA-13).
CREATE TABLE gm_proposals (
    proposal_id         TEXT PRIMARY KEY,
    adventure_id        TEXT NOT NULL,
    response_message_id TEXT NOT NULL,
    status              TEXT NOT NULL,
    data                TEXT NOT NULL
);
CREATE INDEX idx_gm_proposals_adv ON gm_proposals(adventure_id);

-- World builder sessions (DATA-15), keyed to one blueprint.
CREATE TABLE world_builder_sessions (
    blueprint_id TEXT PRIMARY KEY,
    data         TEXT NOT NULL
);

-- Composer drafts (DATA-26). UNIQUE(scope_key) => at most one per scope.
CREATE TABLE drafts (
    draft_id  TEXT PRIMARY KEY,
    scope_key TEXT NOT NULL UNIQUE,
    data      TEXT NOT NULL
);

-- Usage metrics (DATA-20a). Columns mirror the aggregation dimensions (STAT-4).
CREATE TABLE usage_metrics (
    metric_id           TEXT PRIMARY KEY,
    created_at          TEXT NOT NULL,
    label               TEXT NOT NULL,
    ai_model            TEXT NOT NULL,
    chat_id             TEXT,
    adventure_id        TEXT,
    blueprint_id        TEXT,
    character_id        TEXT,
    input_tokens        INTEGER NOT NULL,
    cached_input_tokens INTEGER,
    output_tokens       INTEGER NOT NULL,
    data                TEXT NOT NULL
);
CREATE INDEX idx_metrics_created   ON usage_metrics(created_at);
CREATE INDEX idx_metrics_model     ON usage_metrics(ai_model);
CREATE INDEX idx_metrics_label     ON usage_metrics(label);
CREATE INDEX idx_metrics_chat      ON usage_metrics(chat_id);
CREATE INDEX idx_metrics_adventure ON usage_metrics(adventure_id);

-- Generated/uploaded image bytes (DATA, IMG-4, SEC-3). Stored inside the
-- encrypted database so bytes are protected at rest with no extra key handling.
CREATE TABLE images (
    owner_kind TEXT NOT NULL,
    owner_id   TEXT NOT NULL,
    mime       TEXT NOT NULL,
    version    INTEGER NOT NULL,
    bytes      BLOB NOT NULL,
    PRIMARY KEY (owner_kind, owner_id)
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_creates_tables_and_sets_version() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let v: u32 = conn
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap() as u32;
        assert_eq!(v, SCHEMA_VERSION);

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(count >= 16, "expected all entity tables, got {count}");
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // second run must not error or duplicate
    }
}
