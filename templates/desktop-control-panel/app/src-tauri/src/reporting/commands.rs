//! Tauri command surface for the reporting feature.
//!
//! [`submit_feedback`] persists app-level free-form feedback from the "Give
//! Feedback" modal to the local `feedback` table (migration 003). This is the
//! global, non-session-scoped feedback entry point.
//!
//! Privacy posture (CLAUDE.md): V1 is fully on-device. This command writes to
//! local SQLite and transmits nothing. A future opt-in egress path would read
//! from the `feedback` table — that change requires an FRD amendment, not a
//! code-level decision, since it introduces network egress.

use std::sync::Mutex;

use chrono::Utc;
use rusqlite::params;
use serde::Deserialize;

use crate::shared::error::AppError;
use crate::storage::Connection;

pub use crate::AppState;

const DB_LOCK_POISONED_MSG: &str = "reporting db lock poisoned";

/// Args for [`submit_feedback`]. Both fields are optional; an entirely empty
/// submission is accepted and stored as a row with NULL note + NULL rating
/// (the comment is optional — #121).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SubmitFeedbackArgs {
    /// Optional 1..=5 star rating.
    pub rating: Option<i64>,
    /// Optional free-form suggestion text.
    pub note: Option<String>,
}

#[tauri::command]
pub async fn submit_feedback(
    args: SubmitFeedbackArgs,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = lock_conn(&state.db)?;
    submit_feedback_impl(&mut conn, args.rating, args.note.as_deref())
}

fn lock_conn(db: &Mutex<Connection>) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
    db.lock()
        .map_err(|_| AppError::InvalidState(DB_LOCK_POISONED_MSG.into()))
}

/// Inserts one feedback row. A note that is empty or whitespace-only is stored
/// as NULL. An entirely empty submission (no note, no rating) is still
/// persisted — the "Give Feedback" comment is optional (#121) and the act of
/// sending is itself a (weak) signal; V1 has no rating UI to require.
pub(crate) fn submit_feedback_impl(
    conn: &mut Connection,
    rating: Option<i64>,
    note: Option<&str>,
) -> Result<(), AppError> {
    let trimmed = note.map(str::trim).filter(|s| !s.is_empty());

    let now = Utc::now().to_rfc3339();
    conn.as_inner_mut()
        .execute(
            "INSERT INTO feedback (rating, note, submitted_at) VALUES (?1, ?2, ?3)",
            params![rating, trimmed, now],
        )
        .map_err(crate::storage::StorageError::from)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ias.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    fn row_count(conn: &Connection) -> i64 {
        conn.as_inner()
            .query_row("SELECT COUNT(*) FROM feedback", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn submit_note_only_persists_row() {
        let (_dir, mut conn) = open_tmp();
        submit_feedback_impl(&mut conn, None, Some("the mic button is hard to find")).unwrap();
        assert_eq!(row_count(&conn), 1);
        let (rating, note): (Option<i64>, Option<String>) = conn
            .as_inner()
            .query_row("SELECT rating, note FROM feedback", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(rating, None);
        assert_eq!(note.as_deref(), Some("the mic button is hard to find"));
    }

    #[test]
    fn submit_rating_only_persists_row() {
        let (_dir, mut conn) = open_tmp();
        submit_feedback_impl(&mut conn, Some(4), None).unwrap();
        assert_eq!(row_count(&conn), 1);
    }

    #[test]
    fn submit_trims_note_and_treats_blank_as_null() {
        let (_dir, mut conn) = open_tmp();
        // Whitespace-only note with no rating → stored as a NULL-note row.
        submit_feedback_impl(&mut conn, None, Some("   ")).unwrap();
        assert_eq!(row_count(&conn), 1);
        let blank_note: Option<String> = conn
            .as_inner()
            .query_row("SELECT note FROM feedback", [], |r| r.get(0))
            .unwrap();
        assert_eq!(blank_note, None);

        // Note with surrounding whitespace is trimmed on store.
        submit_feedback_impl(&mut conn, None, Some("  helpful  ")).unwrap();
        let note: Option<String> = conn
            .as_inner()
            .query_row(
                "SELECT note FROM feedback ORDER BY rowid DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(note.as_deref(), Some("helpful"));
    }

    #[test]
    fn submit_empty_submission_persists_null_row() {
        let (_dir, mut conn) = open_tmp();
        // The comment is optional (#121): an empty send still records a row.
        submit_feedback_impl(&mut conn, None, None).unwrap();
        assert_eq!(row_count(&conn), 1);
        let (rating, note): (Option<i64>, Option<String>) = conn
            .as_inner()
            .query_row("SELECT rating, note FROM feedback", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(rating, None);
        assert_eq!(note, None);
    }
}
