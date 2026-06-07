//! Entity repositories over the encrypted connection.
//!
//! Each entity persists as a row carrying its indexed columns plus a `data` JSON
//! blob holding the full serialized record. These helpers centralize the
//! JSON-column read/write so per-entity methods stay small.

pub mod characters;
pub mod chats;
pub mod drafts;
pub mod images;
pub mod metrics;
pub mod singletons;
pub mod worlds;

use rusqlite::{Connection, Row};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::CoreResult;

/// Serialize an entity to its JSON `data` column value.
pub(crate) fn to_data<T: Serialize>(value: &T) -> CoreResult<String> {
    Ok(serde_json::to_string(value)?)
}

/// Deserialize an entity from a row's `data` column (assumed to be the named or
/// last column selected as `data`).
pub(crate) fn from_data<T: DeserializeOwned>(row: &Row, idx: usize) -> rusqlite::Result<T> {
    let json: String = row.get(idx)?;
    serde_json::from_str(&json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Select a single entity by a query returning one `data` column.
pub(crate) fn select_one<T: DeserializeOwned>(
    conn: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> CoreResult<Option<T>> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params)?;
    match rows.next()? {
        Some(row) => Ok(Some(from_data(row, 0)?)),
        None => Ok(None),
    }
}

/// Select many entities by a query returning one `data` column per row.
pub(crate) fn select_many<T: DeserializeOwned>(
    conn: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> CoreResult<Vec<T>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params, |row| from_data::<T>(row, 0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Count rows matching a query (the query must select a single count column).
pub(crate) fn count(conn: &Connection, sql: &str, params: impl rusqlite::Params) -> CoreResult<i64> {
    Ok(conn.query_row(sql, params, |r| r.get(0))?)
}
