//! Settings — L1, regional variety, difficulty level, report-uploads toggle.
//!
//! Per ADD §3.6 (Tauri command surface) + FRD §10 (Settings). The feature is
//! entirely command-driven for V1; no background workers, no event emitters.

pub mod commands;
