//! Encrypted image-byte persistence (`IMG-4`, `SEC-3`). Bytes live inside the
//! encrypted database, so they are protected at rest with no extra key handling.

use rusqlite::{OptionalExtension, params};

use crate::error::CoreResult;
use crate::store::Store;

/// The kind of entity an image belongs to (the `owner_kind` column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOwnerKind {
    Character,
    World,
    Profile,
}

impl ImageOwnerKind {
    fn as_str(self) -> &'static str {
        match self {
            ImageOwnerKind::Character => "character",
            ImageOwnerKind::World => "world",
            ImageOwnerKind::Profile => "profile",
        }
    }
}

/// A stored image: its MIME type, cache-bust version, and raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredImage {
    pub mime: String,
    pub version: u32,
    pub bytes: Vec<u8>,
}

impl Store {
    /// Store (or replace) an entity's image bytes at `version` (`IMG-4`).
    pub fn put_image(
        &self,
        kind: ImageOwnerKind,
        owner_id: &str,
        mime: &str,
        version: u32,
        bytes: &[u8],
    ) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO images (owner_kind, owner_id, mime, version, bytes)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(owner_kind, owner_id) DO UPDATE SET
                   mime = excluded.mime,
                   version = excluded.version,
                   bytes = excluded.bytes",
                params![kind.as_str(), owner_id, mime, version, bytes],
            )?;
            Ok(())
        })
    }

    /// Load an entity's stored image, if any (`IMG-8` precedence).
    pub fn image(&self, kind: ImageOwnerKind, owner_id: &str) -> CoreResult<Option<StoredImage>> {
        self.with_conn(|conn| {
            let row = conn
                .query_row(
                    "SELECT mime, version, bytes FROM images
                     WHERE owner_kind = ?1 AND owner_id = ?2",
                    params![kind.as_str(), owner_id],
                    |r| {
                        Ok(StoredImage {
                            mime: r.get(0)?,
                            version: r.get::<_, i64>(1)? as u32,
                            bytes: r.get(2)?,
                        })
                    },
                )
                .optional()?;
            Ok(row)
        })
    }

    /// Remove an entity's stored image (clear back to emoji, `IMG-3`).
    pub fn delete_image(&self, kind: ImageOwnerKind, owner_id: &str) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute(
                "DELETE FROM images WHERE owner_kind = ?1 AND owner_id = ?2",
                params![kind.as_str(), owner_id],
            )?;
            Ok(())
        })
    }
}
