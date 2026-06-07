//! The core error type shared across the engine.

/// Result alias for core operations.
pub type CoreResult<T> = Result<T, CoreError>;

/// Errors surfaced by the store and engines.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The store is locked; unlock with the master password first (`SEC-5`).
    #[error("store is locked")]
    Locked,

    /// The supplied master password was incorrect (`SEC-6`).
    #[error("incorrect master password")]
    WrongPassword,

    /// The store has already been initialized (first-run setup ran already).
    #[error("store already initialized")]
    AlreadyInitialized,

    /// The store has not been initialized yet (no master password set).
    #[error("store not initialized")]
    NotInitialized,

    /// A requested entity does not exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// Input failed validation (length bounds, required fields, etc.).
    #[error("invalid input: {0}")]
    Validation(String),

    /// A cryptographic operation failed (key derivation, verifier, rekey).
    #[error("crypto error: {0}")]
    Crypto(String),

    /// A persistence error from the underlying database.
    #[error("store error: {0}")]
    Store(String),

    /// A (de)serialization error for a persisted record.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// The OS secure credential store was unavailable or failed (`SEC-7`).
    #[error("keychain error: {0}")]
    Keychain(String),

    /// An AI provider error surfaced to the user (`AI-12`).
    #[error(transparent)]
    Provider(#[from] crate::ai::types::ProviderError),

    /// Any other error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        CoreError::Store(e.to_string())
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Serialization(e.to_string())
    }
}
