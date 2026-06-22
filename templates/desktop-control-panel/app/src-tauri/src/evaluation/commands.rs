//! Tauri command surface for the evaluation feature.
//!
//! Spec: ADD §3.6 (evaluation commands), planning task CL-19.
//!
//! V1 surfaces one command from this module:
//!
//! - [`get_evaluation_result`] — late-arriving listeners (the React Results
//!   screen on a fresh window, replay path post-`eval:done` race) fetch the
//!   persisted [`EvaluationResult`] from the `sessions` table.
//!
//! The evaluation pipeline itself is fire-and-forget from
//! `recording::commands::end_session` via
//! [`crate::evaluation::orchestrator::run_evaluation`]; that path is not a
//! Tauri command — the command boundary is the `end_session` call, the
//! evaluation orchestrator runs asynchronously and communicates via the
//! `eval:progress` / `eval:done` / `eval:error` event surface.

use std::sync::Mutex;

use crate::evaluation::orchestrator::get_evaluation_result_impl;
use crate::shared::error::AppError;
use crate::shared::types::EvaluationResult;
use crate::storage::Connection;
use crate::AppState;

const DB_LOCK_POISONED_MSG: &str = "evaluation db lock poisoned";

fn lock_conn(db: &Mutex<Connection>) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
    db.lock()
        .map_err(|_| AppError::InvalidState(DB_LOCK_POISONED_MSG.into()))
}

/// Late-arrival fetch of a persisted evaluation result. Returns
/// `Ok(Some(result))` for a known `session_id`, `Ok(None)` for an unknown
/// one — explicitly NOT an error.
#[tauri::command]
pub async fn get_evaluation_result(
    session_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Option<EvaluationResult>, AppError> {
    let conn = lock_conn(&state.db)?;
    get_evaluation_result_impl(&conn, &session_id)
}
