//! Settings — generic app-state spine: a small example preference (`theme`)
//! plus the report-uploads and update-checks toggles.
//!
//! The feature is entirely command-driven; no background workers, no event
//! emitters. See [`commands`] for the wire surface a downstream app extends.

pub mod commands;
