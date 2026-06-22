//! Errors produced by the storage layer.
//!
//! Per ADD §3.10, each feature module owns its own `thiserror`-derived error
//! enum and bubbles up into [`crate::shared::error::AppError`] via `#[from]`.
//!
//! Variants:
//!
//! - [`StorageError::Sqlite`] — catch-all wrap of `rusqlite::Error` covering
//!   connection errors, query errors, and constraint violations. The migration
//!   runner currently relies on this variant when a migration's SQL fails, so
//!   diagnostics surface as `sqlite error: …` until a future migration opts
//!   into [`StorageError::MigrationFailed`] for richer reporting.
//! - [`StorageError::MigrationFailed`] — reserved for future migrations that
//!   want to annotate failure with the migration version. The current
//!   `migrations::run` impl uses `?` to bubble [`StorageError::Sqlite`]; this
//!   variant exists so a later migration may wrap with version context
//!   without an enum churn.
//! - [`StorageError::InvalidState`] — runtime sanity checks (e.g., opening a
//!   database whose `user_version` is newer than this binary knows about).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("migration {version} failed: {source}")]
    MigrationFailed {
        version: i32,
        source: rusqlite::Error,
    },

    #[error("invalid state: {0}")]
    InvalidState(String),
}
