//! The encrypted local store (`SEC`, `DATA` persistence).

pub mod crypto;
pub mod db;
pub mod repo;
pub mod schema;

pub use db::{Store, StorePaths};
pub use repo::images::{ImageOwnerKind, StoredImage};
