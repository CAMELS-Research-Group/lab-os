//! IAS client storage feature — SQLite persistence + migrations.
//!
//! Per ADD §3.8 this is the single point through which every feature module
//! that persists state opens connections. The schema (`install_identity`,
//! `settings`, `sessions`, `upload_queue`) is defined in
//! `migrations/001_init.sql` and brought up to date by [`run_migrations`].
//!
//! All connections returned by [`Connection::new`] have the project-standard
//! PRAGMAs applied: `journal_mode = WAL`, `foreign_keys = ON`,
//! `synchronous = NORMAL`. The schema is also migrated as part of `new`, so
//! callers never see a connection on a stale schema.

pub mod commands;
mod connection;
mod error;
mod migrations;

pub use connection::Connection;
pub use error::StorageError;
pub use migrations::run as run_migrations;
