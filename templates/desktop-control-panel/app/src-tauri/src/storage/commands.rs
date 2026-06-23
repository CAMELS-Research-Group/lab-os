//! Tauri command surface for storage-level maintenance operations.
//!
//! Commands:
//! - [`clear_app_data`]: resets the generalized app-state spine to defaults —
//!   deletes the `settings` singleton row (preferences revert to their schema
//!   defaults on next read/write) while leaving the `install_identity` row
//!   intact, so per-install identity + consent state survive the clear.
//!
//! [`clear_app_data`] is deliberately *not* coupled to a frontend "Reset app /
//! UI state" action — that one only clears local UI state. The two clears are
//! independent: a user can wipe their persisted preferences without losing
//! their install identity.

use std::sync::Mutex;

use crate::shared::error::AppError;
use crate::storage::Connection;

pub use crate::AppState;

const DB_LOCK_POISONED_MSG: &str = "storage db lock poisoned";

// ---------------------------------------------------------------------------
// clear_app_data
// ---------------------------------------------------------------------------

/// Resets the persisted app-state to defaults.
///
/// Deletes the `settings` singleton row; `install_identity` is untouched, so
/// the per-install id + consent state survive the clear. Preferences revert to
/// their schema defaults the next time the row is (re)created by a setter.
///
/// Returns the number of `settings` rows deleted (0 or 1) so the caller can
/// confirm the reset (and so a test can assert it).
#[tauri::command]
pub async fn clear_app_data(state: tauri::State<'_, AppState>) -> Result<u64, AppError> {
    let mut conn = lock_conn(&state.db)?;
    clear_app_data_impl(&mut conn)
}

fn lock_conn(db: &Mutex<Connection>) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
    db.lock()
        .map_err(|_| AppError::InvalidState(DB_LOCK_POISONED_MSG.into()))
}

/// Business-logic layer, exercised directly by tests without a Tauri runtime.
///
/// The delete runs inside a transaction so a mid-clear failure rolls back
/// atomically.
pub(crate) fn clear_app_data_impl(conn: &mut Connection) -> Result<u64, AppError> {
    let tx = conn.transaction()?;
    let settings_deleted = tx
        .execute("DELETE FROM settings", [])
        .map_err(crate::storage::StorageError::from)?;
    tx.commit().map_err(crate::storage::StorageError::from)?;
    Ok(settings_deleted as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.as_inner()
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn clear_app_data_removes_settings_row() {
        let (_dir, mut conn) = open_tmp();
        conn.as_inner()
            .execute("INSERT INTO settings (id, theme) VALUES (1, 'dark')", [])
            .unwrap();

        let deleted = clear_app_data_impl(&mut conn).unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(count(&conn, "settings"), 0);
    }

    #[test]
    fn clear_app_data_preserves_install_identity() {
        let (_dir, mut conn) = open_tmp();
        conn.as_inner()
            .execute("INSERT INTO settings (id, theme) VALUES (1, 'light')", [])
            .unwrap();
        conn.as_inner()
            .execute(
                "INSERT INTO install_identity \
                 (id, uuid, consent_granted_at, consent_revoked_at, registered_at, schema_version) \
                 VALUES (1, 'u', '2026-06-01T00:00:00Z', NULL, '2026-06-01T00:00:00Z', 1)",
                [],
            )
            .unwrap();

        clear_app_data_impl(&mut conn).unwrap();

        assert_eq!(count(&conn, "settings"), 0);
        assert_eq!(
            count(&conn, "install_identity"),
            1,
            "identity must survive the clear"
        );
        let uuid: String = conn
            .as_inner()
            .query_row("SELECT uuid FROM install_identity WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(uuid, "u");
    }

    #[test]
    fn clear_app_data_on_empty_db_is_noop() {
        let (_dir, mut conn) = open_tmp();
        let deleted = clear_app_data_impl(&mut conn).unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(count(&conn, "settings"), 0);
    }
}
