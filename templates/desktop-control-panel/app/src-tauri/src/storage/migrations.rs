//! Idempotent forward-only migration runner driven by `PRAGMA user_version`.
//!
//! Each entry in [`MIGRATIONS`] pairs a target schema version with the SQL
//! batch that brings a database at version `target - 1` up to `target`. The
//! runner applies any entries whose version is strictly greater than the
//! current `user_version`, each wrapped in its own transaction so failures
//! roll back atomically.
//!
//! When adding a migration: append `(N, include_str!("migrations/00N_*.sql"))`
//! and ensure the SQL file is committed alongside.

use rusqlite::Connection as RusqliteConnection;

use crate::storage::error::StorageError;

const INIT_SQL: &str = include_str!("migrations/001_init.sql");
const UPDATE_CHECKS_SQL: &str = include_str!("migrations/002_update_checks.sql");

const MIGRATIONS: &[(i32, &str)] = &[(1, INIT_SQL), (2, UPDATE_CHECKS_SQL)];

/// Reads the current `PRAGMA user_version` from the connection. Returns `0`
/// for a freshly-created database that has not had any migrations applied.
pub fn current_version(conn: &RusqliteConnection) -> Result<i32, StorageError> {
    let v: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    Ok(v)
}

/// Applies any pending migrations to `conn`, returning the version the
/// database is at after the run completes. Idempotent: a re-invocation on an
/// already-migrated database is a no-op and returns the same version.
pub fn run(conn: &mut RusqliteConnection) -> Result<i32, StorageError> {
    // Single-writer assumption: this client opens the DB from one Tauri process at a time,
    // so reading `current_version` outside the migration transaction is race-free for V1.
    // If multi-process access is ever introduced, move this read inside the transaction.
    let current = current_version(conn)?;
    let mut applied = current;

    for &(version, sql) in MIGRATIONS {
        if version > current {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
            applied = version;
        }
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection as RusqliteConnection;

    #[test]
    fn current_version_starts_at_zero() {
        let conn = RusqliteConnection::open_in_memory().unwrap();
        assert_eq!(current_version(&conn).unwrap(), 0);
    }

    #[test]
    fn migrations_run_idempotent() {
        let mut conn = RusqliteConnection::open_in_memory().unwrap();
        let first = run(&mut conn).unwrap();
        assert_eq!(first, 2);
        // Second invocation should be a no-op and return the same version.
        let second = run(&mut conn).unwrap();
        assert_eq!(second, 2);
        assert_eq!(current_version(&conn).unwrap(), 2);
    }
}
