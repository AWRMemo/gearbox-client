use thiserror::Error;

/// Unified domain error type for `relay-core`.
///
/// All internal errors are converted to `String` at the Tauri / mobile
/// command boundary via `.to_string_cmd()`.
#[derive(Error, Debug)]
pub enum RelayError {
    #[error("database error: {0}")]
    DbError(#[from] rusqlite::Error),

    #[error("crypto error: {0}")]
    CryptoError(String),

    #[error("sync error: {0}")]
    SyncError(String),

    #[error("ai error: {0}")]
    AiError(String),

    #[error("vector error: {0}")]
    VectorError(String),
}

impl RelayError {
    /// Convert to a plain `String` suitable for returning across FFI / Tauri
    /// command boundaries.
    pub fn to_string_cmd(&self) -> String {
        self.to_string()
    }
}
