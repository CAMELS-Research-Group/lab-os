//! Connection wrapper applying project-standard PRAGMAs (WAL journaling,
//! foreign-key enforcement, NORMAL synchronous) and running any pending
//! migrations on open.

use std::path::Path;

use rusqlite::{Connection as RusqliteConnection, Transaction};

use crate::storage::error::StorageError;
use crate::storage::migrations;

/// Opaque wrapper over [`rusqlite::Connection`]. Constructed via
/// [`Connection::new`] which guarantees the project-standard PRAGMAs are
/// applied and the schema is up-to-date before the handle is returned.
pub struct Connection {
    inner: RusqliteConnection,
}

impl Connection {
    /// Opens (creating if missing) the SQLite database at `db_path`, applies
    /// the project-standard PRAGMAs, and runs any pending migrations.
    ///
    /// PRAGMAs:
    ///
    /// - `journal_mode = WAL` — better concurrency for the read-heavy
    ///   reporting paths.
    /// - `foreign_keys = ON` — enforces the `upload_queue.session_id` FK.
    /// - `synchronous = NORMAL` — durability/perf tradeoff appropriate for
    ///   WAL-mode user databases.
    ///
    /// PRAGMAs are applied before migrations so that `foreign_keys = ON`
    /// is in force when future migrations restructure FK-bearing tables.
    pub fn new(db_path: &Path) -> Result<Self, StorageError> {
        // sqlite's `open` does not create missing parent dirs; on first launch
        // the OS app-data dir for our bundle identifier does not exist yet, so
        // open fails with SQLITE_CANTOPEN until we materialise it ourselves.
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::InvalidState(format!(
                    "create db parent dir {}: {e}",
                    parent.display()
                ))
            })?;
        }

        let mut inner = RusqliteConnection::open(db_path)?;

        // `journal_mode` is a query-returning pragma (it echoes the new mode
        // back). `pragma_update` discards the returned row, which is what we
        // want here — we only care that the mode is set.
        inner.pragma_update(None, "journal_mode", "WAL")?;
        inner.pragma_update(None, "foreign_keys", "ON")?;
        inner.pragma_update(None, "synchronous", "NORMAL")?;

        migrations::run(&mut inner)?;
        Ok(Self { inner })
    }

    /// Begins a deferred transaction. Drop without calling `commit` rolls
    /// back; this is the standard `rusqlite::Transaction` shape and is
    /// exposed directly so callers retain the implicit rollback semantics.
    pub fn transaction(&mut self) -> Result<Transaction<'_>, StorageError> {
        Ok(self.inner.transaction()?)
    }

    /// Borrow the raw `rusqlite::Connection`. Feature modules (identity,
    /// settings, sessions, upload-queue) use this to prepare statements and
    /// run queries directly; centralising every helper here is premature.
    /// Allowed-dead for now because the feature modules consuming it (CL-7,
    /// CL-8, CL-13, CL-19) have not landed yet — tests in this module do
    /// exercise it.
    #[allow(dead_code)]
    pub(crate) fn as_inner(&self) -> &RusqliteConnection {
        &self.inner
    }

    /// Mutable counterpart to [`Connection::as_inner`].
    #[allow(dead_code)]
    pub(crate) fn as_inner_mut(&mut self) -> &mut RusqliteConnection {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations;
    use rusqlite::params;
    use tempfile::TempDir;

    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ias.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    /// Insert helper: writes a session row with the minimum NOT NULL columns
    /// satisfied by anonymous synthetic values (per repo data-protection
    /// rule). `session_id` parameterised so callers can disambiguate.
    fn insert_session(conn: &RusqliteConnection, session_id: &str) {
        conn.execute(
            "INSERT INTO sessions (
                session_id, started_at, ended_at, duration_seconds,
                l1_at_session, regional_variety_at_session, phoneme_attempts_json,
                difficulty_level, difficulty_thresholds_json, threshold_table_version,
                reattempt_counts_json, cumulative_session_count,
                app_version, model_version, os_family, os_major
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7,
                ?8, ?9, ?10,
                ?11, ?12,
                ?13, ?14, ?15, ?16
            )",
            params![
                session_id,
                "2026-06-01T00:00:00Z",
                "2026-06-01T00:05:00Z",
                300_i64,
                "es",
                None::<&str>,
                "[]",
                "gentle",
                r#"{"/r/":0.5}"#,
                3_i64,
                "{}",
                1_i64,
                "0.1.0",
                "model-v1",
                "linux",
                "6",
            ],
        )
        .unwrap();
    }

    // -----------------------------------------------------------------
    // Schema + open semantics
    // -----------------------------------------------------------------

    #[test]
    fn connection_new_creates_expected_tables() {
        let (_dir, conn) = open_tmp();
        let mut stmt = conn
            .as_inner()
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' \
                 AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(
            names,
            vec![
                "feedback".to_string(),
                "install_identity".to_string(),
                "sessions".to_string(),
                "settings".to_string(),
                "upload_queue".to_string(),
            ]
        );
    }

    #[test]
    fn connection_new_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ias.db");

        let conn = Connection::new(&path).unwrap();
        assert_eq!(migrations::current_version(conn.as_inner()).unwrap(), 3);
        drop(conn);

        let conn = Connection::new(&path).unwrap();
        assert_eq!(migrations::current_version(conn.as_inner()).unwrap(), 3);
    }

    #[test]
    fn connection_applies_project_pragmas() {
        let (_dir, conn) = open_tmp();
        let inner = conn.as_inner();

        let journal: String = inner
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal.to_lowercase(), "wal");

        let fks: i32 = inner
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fks, 1);

        let sync: i32 = inner
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .unwrap();
        // synchronous=NORMAL is integer 1
        assert_eq!(sync, 1);
    }

    // -----------------------------------------------------------------
    // Transaction semantics
    // -----------------------------------------------------------------

    #[test]
    fn transaction_commit_persists_changes() {
        let (_dir, mut conn) = open_tmp();
        {
            let tx = conn.transaction().unwrap();
            tx.execute(
                "INSERT INTO settings (id, l1, difficulty) VALUES (1, 'es', 'standard')",
                [],
            )
            .unwrap();
            tx.commit().unwrap();
        }
        let difficulty: String = conn
            .as_inner()
            .query_row(
                "SELECT difficulty FROM settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(difficulty, "standard");
    }

    #[test]
    fn transaction_drop_rolls_back() {
        let (_dir, mut conn) = open_tmp();
        {
            let tx = conn.transaction().unwrap();
            tx.execute(
                "INSERT INTO settings (id, l1, difficulty) VALUES (1, 'es', 'strict')",
                [],
            )
            .unwrap();
            // No commit — drop on scope exit rolls back.
        }
        let count: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    // -----------------------------------------------------------------
    // V0.4 session columns + constraints
    // -----------------------------------------------------------------

    #[test]
    fn session_row_roundtrip_with_v04_fields() {
        let (_dir, conn) = open_tmp();
        insert_session(conn.as_inner(), "test-session-1");

        let (level, thresholds, version): (String, String, i32) = conn
            .as_inner()
            .query_row(
                "SELECT difficulty_level, difficulty_thresholds_json, threshold_table_version \
                 FROM sessions WHERE session_id = ?1",
                params!["test-session-1"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(level, "gentle");
        assert_eq!(thresholds, r#"{"/r/":0.5}"#);
        assert_eq!(version, 3);
    }

    #[test]
    fn settings_difficulty_check_constraint_rejects_invalid_level() {
        let (_dir, conn) = open_tmp();
        let err = conn
            .as_inner()
            .execute(
                "INSERT INTO settings (id, l1, difficulty) VALUES (1, '', 'extreme')",
                [],
            )
            .unwrap_err();
        match err {
            rusqlite::Error::SqliteFailure(e, _) => {
                assert_eq!(
                    e.extended_code,
                    rusqlite::ffi::SQLITE_CONSTRAINT_CHECK,
                    "expected SQLITE_CONSTRAINT_CHECK (275), got extended_code={}",
                    e.extended_code
                );
            }
            other => panic!("expected SqliteFailure(CHECK), got {other:?}"),
        }
    }

    #[test]
    fn settings_singleton_constraint() {
        let (_dir, conn) = open_tmp();
        let err = conn
            .as_inner()
            .execute(
                "INSERT INTO settings (id, l1, difficulty) VALUES (2, '', 'gentle')",
                [],
            )
            .unwrap_err();
        match err {
            rusqlite::Error::SqliteFailure(e, _) => {
                assert_eq!(
                    e.extended_code,
                    rusqlite::ffi::SQLITE_CONSTRAINT_CHECK,
                    "expected SQLITE_CONSTRAINT_CHECK (275), got extended_code={}",
                    e.extended_code
                );
            }
            other => panic!("expected SqliteFailure(CHECK), got {other:?}"),
        }
    }

    #[test]
    fn upload_queue_fk_cascades_on_session_delete() {
        let (_dir, conn) = open_tmp();
        insert_session(conn.as_inner(), "test-session-cascade");
        conn.as_inner()
            .execute(
                "INSERT INTO upload_queue (
                    session_id, payload_kind, payload_json, queued_at, attempt_count
                 ) VALUES (?1, 'session_report', '{}', '2026-06-01T00:00:00Z', 0)",
                params!["test-session-cascade"],
            )
            .unwrap();

        let before: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM upload_queue", [], |row| row.get(0))
            .unwrap();
        assert_eq!(before, 1);

        conn.as_inner()
            .execute(
                "DELETE FROM sessions WHERE session_id = ?1",
                params!["test-session-cascade"],
            )
            .unwrap();

        let after: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM upload_queue", [], |row| row.get(0))
            .unwrap();
        assert_eq!(after, 0, "FK ON DELETE CASCADE should have removed the queue row");
    }
}
