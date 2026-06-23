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
    /// - `foreign_keys = ON` — enforces any FKs a downstream app's domain
    ///   migrations add (the spine schema itself has none).
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
    use tempfile::TempDir;

    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
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
            vec!["install_identity".to_string(), "settings".to_string(),]
        );
    }

    #[test]
    fn connection_new_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");

        let conn = Connection::new(&path).unwrap();
        assert_eq!(migrations::current_version(conn.as_inner()).unwrap(), 2);
        drop(conn);

        let conn = Connection::new(&path).unwrap();
        assert_eq!(migrations::current_version(conn.as_inner()).unwrap(), 2);
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
                "INSERT INTO settings (id, theme) VALUES (1, 'dark')",
                [],
            )
            .unwrap();
            tx.commit().unwrap();
        }
        let theme: String = conn
            .as_inner()
            .query_row("SELECT theme FROM settings WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(theme, "dark");
    }

    #[test]
    fn transaction_drop_rolls_back() {
        let (_dir, mut conn) = open_tmp();
        {
            let tx = conn.transaction().unwrap();
            tx.execute(
                "INSERT INTO settings (id, theme) VALUES (1, 'light')",
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
    // Settings singleton constraint
    // -----------------------------------------------------------------

    #[test]
    fn settings_singleton_constraint() {
        let (_dir, conn) = open_tmp();
        let err = conn
            .as_inner()
            .execute(
                "INSERT INTO settings (id, theme) VALUES (2, 'system')",
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
}
