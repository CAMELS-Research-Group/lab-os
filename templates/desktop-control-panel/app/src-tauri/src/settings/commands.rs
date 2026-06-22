//! Tauri command surface for the settings feature.
//!
//! Spec: ADD §3.6 (settings commands), FRD §10 (Settings).
//!
//! Four commands, all serializing through the canonical [`Settings`] /
//! [`DifficultyLevel`] types defined in [`crate::shared::types`]:
//!
//! - [`get_settings`] — read the singleton + derive `report_uploads_enabled`
//!   from `install_identity.consent_revoked_at`.
//! - [`set_l1`] — upsert L1 + regional variety.
//! - [`set_difficulty`] — upsert the named difficulty level.
//! - [`set_report_uploads_enabled`] — toggle the report-upload posture.
//!   Toggle-off (`false`) aliases consent revocation (FRD F-SET-3); toggle-on
//!   after a prior revocation is explicitly out of V1 scope.
//!
//! # CL-8 deferral
//!
//! CL-8 (identity — consent + install state) has not landed yet, so
//! `identity::commands::revoke_consent` does not exist. The CL-9 plan calls for
//! `set_report_uploads_enabled(false)` to alias that command. Until CL-8 lands,
//! we perform the consent-revocation write inline here against
//! `install_identity.consent_revoked_at`. The `TODO(CL-8)` markers below mark
//! every site that must be re-pointed at the identity module once it exists.
//!
//! The install_identity row is normally created during CL-8's first-run
//! consent-acceptance flow. Until then, fresh installs have no row at all —
//! [`get_settings`] treats "no row" as `report_uploads_enabled = true` (the
//! default posture pre-consent) and [`set_report_uploads_enabled(false)`]
//! treats "no row" as a no-op (nothing to revoke yet).

use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;

use crate::shared::error::AppError;
use crate::shared::types::{DifficultyLevel, Settings};
use crate::storage::Connection;

// ---------------------------------------------------------------------------
// State held by Tauri's `manage()`. Defined in `lib.rs` so the same struct can
// be extended by later feature modules (CL-13 session lifecycle, CL-19
// evaluation orchestrator); imported here for the command bodies.
// ---------------------------------------------------------------------------

pub use crate::AppState;

// ---------------------------------------------------------------------------
// Command argument structs
// ---------------------------------------------------------------------------

/// Args for [`set_l1`]. `variety` is optional; passing `None` clears it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetL1Args {
    pub l1: String,
    pub variety: Option<String>,
}

/// Args for [`set_difficulty`]. `level` is one of `gentle | standard | strict`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetDifficultyArgs {
    pub level: DifficultyLevel,
}

/// Args for [`set_report_uploads_enabled`]. `enabled = false` aliases
/// consent revocation. `enabled = true` after a prior revocation returns an
/// `AppError::InvalidState` — V1 has no consent-restore path.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetReportUploadsEnabledArgs {
    pub enabled: bool,
}

/// Args for [`set_update_checks_enabled`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetUpdateChecksEnabledArgs {
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// Documented error message for the V1-out-of-scope toggle-on path. Pulled out
// as a constant so tests can match on it exactly.
// ---------------------------------------------------------------------------

pub(crate) const TOGGLE_ON_AFTER_REVOCATION_MSG: &str =
    "toggle-on after revocation is out of V1 scope; \
     uninstall and reinstall to grant consent again";

const DB_LOCK_POISONED_MSG: &str = "settings db lock poisoned";

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_settings(
    state: tauri::State<'_, AppState>,
) -> Result<Settings, AppError> {
    let conn = lock_conn(&state.db)?;
    get_settings_impl(&conn)
}

#[tauri::command]
pub async fn set_l1(
    args: SetL1Args,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = lock_conn(&state.db)?;
    set_l1_impl(&mut conn, &args.l1, args.variety.as_deref())
}

#[tauri::command]
pub async fn set_difficulty(
    args: SetDifficultyArgs,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = lock_conn(&state.db)?;
    set_difficulty_impl(&mut conn, args.level)
}

#[tauri::command]
pub async fn set_report_uploads_enabled(
    args: SetReportUploadsEnabledArgs,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = lock_conn(&state.db)?;
    set_report_uploads_enabled_impl(&mut conn, args.enabled)
}

#[tauri::command]
pub async fn set_update_checks_enabled(
    args: SetUpdateChecksEnabledArgs,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = lock_conn(&state.db)?;
    set_update_checks_enabled_impl(&mut conn, args.enabled)
}

// ---------------------------------------------------------------------------
// Locking helper — converts `Mutex` poisoning into a typed AppError rather
// than panicking. Poisoning should never happen in practice (none of these
// callsites panic while holding the lock), but propagating it keeps the
// Tauri-command surface infallible-of-panic.
// ---------------------------------------------------------------------------

fn lock_conn(db: &Mutex<Connection>) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
    db.lock()
        .map_err(|_| AppError::InvalidState(DB_LOCK_POISONED_MSG.into()))
}

// ---------------------------------------------------------------------------
// Business-logic layer. Free functions on `&Connection` / `&mut Connection`
// so tests can exercise them without spinning up a Tauri runtime. Each maps
// 1:1 with its `#[tauri::command]` wrapper above.
// ---------------------------------------------------------------------------

/// Reads the singleton settings row + derives `report_uploads_enabled` from
/// the install_identity row's `consent_revoked_at`.
///
/// Defaults when nothing has been persisted yet:
///
/// - No settings row → `l1 = ""`, `regional_variety = None`,
///   `difficulty = Gentle`.
/// - No install_identity row → `report_uploads_enabled = true`.
///   The row is created by CL-8's consent flow; pre-consent installs see the
///   pre-revocation default.
pub(crate) fn get_settings_impl(conn: &Connection) -> Result<Settings, AppError> {
    let inner = conn.as_inner();

    // Read settings singleton (id = 1). Absent → defaults.
    let settings_row: Option<(String, Option<String>, String, bool)> = inner
        .query_row(
            "SELECT l1, regional_variety, difficulty, update_checks_enabled \
             FROM settings WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?;

    let (l1, regional_variety, difficulty, update_checks_enabled) = match settings_row {
        Some((l1, variety, diff_str, update_checks)) => {
            let difficulty = parse_difficulty(&diff_str)?;
            (l1, variety, difficulty, update_checks)
        }
        None => (String::new(), None, DifficultyLevel::Gentle, false),
    };

    // TODO(CL-8): once consent flow lands, this assumes the row was created
    //             during accept_consent. For now, treat "no row" as not-revoked.
    let consent_revoked_at: Option<Option<String>> = inner
        .query_row(
            "SELECT consent_revoked_at FROM install_identity WHERE id = 1",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?;

    let report_uploads_enabled = match consent_revoked_at {
        // No install_identity row yet — treat as not revoked.
        None => true,
        // Row exists, NULL revoked_at — not revoked.
        Some(None) => true,
        // Row exists, NOT NULL revoked_at — revoked.
        Some(Some(_)) => false,
    };

    Ok(Settings {
        l1,
        regional_variety,
        difficulty,
        report_uploads_enabled,
        update_checks_enabled,
    })
}

/// Upserts L1 + regional variety on the singleton row. Difficulty is left at
/// its existing value (or the schema default of `'gentle'` on first insert).
pub(crate) fn set_l1_impl(
    conn: &mut Connection,
    l1: &str,
    variety: Option<&str>,
) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();

    // Read-modify-write: read current difficulty (or default), then UPSERT.
    // The singleton CHECK(id=1) constraint means we always operate on id=1.
    let current_difficulty: String = inner
        .query_row(
            "SELECT difficulty FROM settings WHERE id = 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?
        .unwrap_or_else(|| "gentle".to_string());

    inner
        .execute(
            "INSERT INTO settings (id, l1, regional_variety, difficulty, schema_version) \
             VALUES (1, ?1, ?2, ?3, 1) \
             ON CONFLICT(id) DO UPDATE SET \
                 l1 = excluded.l1, \
                 regional_variety = excluded.regional_variety",
            params![l1, variety, current_difficulty],
        )
        .map_err(crate::storage::StorageError::from)?;

    Ok(())
}

/// Upserts the named difficulty level. L1 + regional_variety are preserved.
pub(crate) fn set_difficulty_impl(
    conn: &mut Connection,
    level: DifficultyLevel,
) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();
    let level_str = difficulty_to_str(&level);

    // Read-modify-write on l1 + regional_variety so the upsert preserves them.
    let current: Option<(String, Option<String>)> = inner
        .query_row(
            "SELECT l1, regional_variety FROM settings WHERE id = 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?;

    let (l1, variety) = current.unwrap_or_else(|| (String::new(), None));

    inner
        .execute(
            "INSERT INTO settings (id, l1, regional_variety, difficulty, schema_version) \
             VALUES (1, ?1, ?2, ?3, 1) \
             ON CONFLICT(id) DO UPDATE SET \
                 difficulty = excluded.difficulty",
            params![l1, variety, level_str],
        )
        .map_err(crate::storage::StorageError::from)?;

    Ok(())
}

/// Toggle the report-upload posture.
///
/// - `enabled = false` writes `install_identity.consent_revoked_at = now()` if
///   not already revoked. If the install_identity row does not exist yet
///   (CL-8 hasn't created it), this is a no-op — there is no consent to
///   revoke.
/// - `enabled = true` after a prior revocation returns
///   [`AppError::InvalidState`] with [`TOGGLE_ON_AFTER_REVOCATION_MSG`].
///   `enabled = true` when not previously revoked is a no-op.
pub(crate) fn set_report_uploads_enabled_impl(
    conn: &mut Connection,
    enabled: bool,
) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();

    // Read current consent_revoked_at. None = no row exists.
    let consent_revoked_at: Option<Option<String>> = inner
        .query_row(
            "SELECT consent_revoked_at FROM install_identity WHERE id = 1",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?;

    let already_revoked = matches!(consent_revoked_at, Some(Some(_)));

    if enabled {
        // Toggle-on. Only an error if the user was previously revoked.
        if already_revoked {
            return Err(AppError::InvalidState(
                TOGGLE_ON_AFTER_REVOCATION_MSG.to_string(),
            ));
        }
        // Not revoked → nothing to do.
        Ok(())
    } else {
        // Toggle-off. If already revoked or no row exists, no-op.
        if already_revoked || consent_revoked_at.is_none() {
            return Ok(());
        }

        // TODO(CL-8): replace the inline revoke with a call to
        //             `identity::commands::revoke_consent` once CL-8 lands.
        let now = Utc::now().to_rfc3339();
        inner
            .execute(
                "UPDATE install_identity SET consent_revoked_at = ?1 WHERE id = 1",
                params![now],
            )
            .map_err(crate::storage::StorageError::from)?;

        Ok(())
    }
}

/// Upserts the `update_checks_enabled` column on the singleton settings row,
/// preserving `l1`, `regional_variety`, and `difficulty`.
///
/// `true` opts the user in to periodic network egress for update checks;
/// `false` opts them out. This is the inverse of the report-uploads toggle:
/// the default is off (column `DEFAULT 0`), and the user must explicitly
/// enable it via the Settings screen. There is no consent-revocation
/// complication — toggling this on and off is always valid.
pub(crate) fn set_update_checks_enabled_impl(
    conn: &mut Connection,
    enabled: bool,
) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();

    // Read current l1 + regional_variety + difficulty so the upsert's INSERT
    // branch (first call on a fresh DB) preserves the schema defaults and the
    // ON CONFLICT branch touches only update_checks_enabled.
    let current: Option<(String, Option<String>, String)> = inner
        .query_row(
            "SELECT l1, regional_variety, difficulty FROM settings WHERE id = 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?)),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?;

    let (l1, variety, difficulty) = current.unwrap_or_else(|| (String::new(), None, "gentle".to_string()));
    let enabled_int: i32 = if enabled { 1 } else { 0 };

    inner
        .execute(
            "INSERT INTO settings (id, l1, regional_variety, difficulty, update_checks_enabled, schema_version) \
             VALUES (1, ?1, ?2, ?3, ?4, 1) \
             ON CONFLICT(id) DO UPDATE SET \
                 update_checks_enabled = excluded.update_checks_enabled",
            params![l1, variety, difficulty, enabled_int],
        )
        .map_err(crate::storage::StorageError::from)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// DifficultyLevel <-> SQL column helpers. The SQL column is a `TEXT` with a
// `CHECK (difficulty IN ('gentle', 'standard', 'strict'))` constraint, so the
// only valid string forms are the snake_case names.
// ---------------------------------------------------------------------------

fn difficulty_to_str(level: &DifficultyLevel) -> &'static str {
    match level {
        DifficultyLevel::Gentle => "gentle",
        DifficultyLevel::Standard => "standard",
        DifficultyLevel::Strict => "strict",
    }
}

fn parse_difficulty(s: &str) -> Result<DifficultyLevel, AppError> {
    match s {
        "gentle" => Ok(DifficultyLevel::Gentle),
        "standard" => Ok(DifficultyLevel::Standard),
        "strict" => Ok(DifficultyLevel::Strict),
        other => Err(AppError::InvalidState(format!(
            "settings.difficulty has invalid value {other:?}; expected one of \
             gentle | standard | strict (schema CHECK should have prevented this)"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use tempfile::TempDir;

    /// Open a fresh DB under a TempDir. Returns the TempDir so the caller
    /// holds the lifetime (drop = deletion).
    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ias.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    /// Insert a synthetic install_identity row to stand in for what CL-8's
    /// `accept_consent` will populate during first-run. Tests for the
    /// consent-revoked code paths need this because no row exists by default
    /// pre-CL-8.
    ///
    /// TODO(CL-8): once CL-8 lands, prefer driving this via the real
    ///             `accept_consent` command rather than a raw INSERT.
    fn insert_synthetic_identity(conn: &Connection, consent_revoked_at: Option<&str>) {
        conn.as_inner()
            .execute(
                "INSERT INTO install_identity \
                 (id, uuid, consent_granted_at, consent_revoked_at, registered_at, schema_version) \
                 VALUES (1, ?1, ?2, ?3, ?2, 1)",
                params![
                    "11111111-2222-3333-4444-555555555555",
                    "2026-06-01T00:00:00Z",
                    consent_revoked_at,
                ],
            )
            .unwrap();
    }

    // --- get_settings on a fresh DB --------------------------------------

    #[test]
    fn get_settings_fresh_db_returns_defaults() {
        let (_dir, conn) = open_tmp();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "");
        assert_eq!(settings.regional_variety, None);
        assert_eq!(settings.difficulty, DifficultyLevel::Gentle);
        // No install_identity row yet → default to true (pre-consent posture).
        assert!(settings.report_uploads_enabled);
        // update_checks_enabled defaults to false (opt-in, off by default).
        assert!(!settings.update_checks_enabled);
    }

    // --- set_l1 round-trip ------------------------------------------------

    #[test]
    fn set_l1_round_trip_with_variety() {
        let (_dir, mut conn) = open_tmp();
        set_l1_impl(&mut conn, "spa", Some("Caribbean")).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "spa");
        assert_eq!(settings.regional_variety, Some("Caribbean".to_string()));
        // Difficulty should default to gentle on first insert.
        assert_eq!(settings.difficulty, DifficultyLevel::Gentle);
    }

    #[test]
    fn set_l1_round_trip_without_variety() {
        let (_dir, mut conn) = open_tmp();
        set_l1_impl(&mut conn, "cmn", None).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "cmn");
        assert_eq!(settings.regional_variety, None);
    }

    #[test]
    fn set_l1_overwrites_existing_l1_and_variety() {
        let (_dir, mut conn) = open_tmp();
        set_l1_impl(&mut conn, "spa", Some("Caribbean")).unwrap();
        set_l1_impl(&mut conn, "por", None).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "por");
        assert_eq!(settings.regional_variety, None);
    }

    // --- set_difficulty round-trip ---------------------------------------

    #[test]
    fn set_difficulty_round_trip_standard() {
        let (_dir, mut conn) = open_tmp();
        set_difficulty_impl(&mut conn, DifficultyLevel::Standard).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.difficulty, DifficultyLevel::Standard);
    }

    #[test]
    fn set_difficulty_round_trip_strict() {
        let (_dir, mut conn) = open_tmp();
        set_difficulty_impl(&mut conn, DifficultyLevel::Strict).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.difficulty, DifficultyLevel::Strict);
    }

    #[test]
    fn set_difficulty_then_back_to_gentle() {
        let (_dir, mut conn) = open_tmp();
        set_difficulty_impl(&mut conn, DifficultyLevel::Strict).unwrap();
        set_difficulty_impl(&mut conn, DifficultyLevel::Gentle).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.difficulty, DifficultyLevel::Gentle);
    }

    // --- partial updates preserve other fields ---------------------------

    #[test]
    fn set_l1_then_set_difficulty_preserves_both() {
        let (_dir, mut conn) = open_tmp();
        set_l1_impl(&mut conn, "kor", Some("Seoul")).unwrap();
        set_difficulty_impl(&mut conn, DifficultyLevel::Standard).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "kor");
        assert_eq!(settings.regional_variety, Some("Seoul".to_string()));
        assert_eq!(settings.difficulty, DifficultyLevel::Standard);
    }

    #[test]
    fn set_difficulty_then_set_l1_preserves_both() {
        let (_dir, mut conn) = open_tmp();
        set_difficulty_impl(&mut conn, DifficultyLevel::Strict).unwrap();
        set_l1_impl(&mut conn, "ara", None).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "ara");
        assert_eq!(settings.regional_variety, None);
        assert_eq!(settings.difficulty, DifficultyLevel::Strict);
    }

    // --- toggle-off path -------------------------------------------------

    #[test]
    fn toggle_off_sets_consent_revoked_at_and_flips_report_uploads_enabled() {
        let (_dir, mut conn) = open_tmp();
        // CL-8 stand-in: insert install_identity row with NULL consent_revoked_at.
        insert_synthetic_identity(&conn, None);

        // Pre: report_uploads_enabled should be true.
        assert!(get_settings_impl(&conn).unwrap().report_uploads_enabled);

        set_report_uploads_enabled_impl(&mut conn, false).unwrap();

        // Post: consent_revoked_at should now be non-NULL, and the derived
        // report_uploads_enabled should read false.
        let revoked_at: Option<String> = conn
            .as_inner()
            .query_row(
                "SELECT consent_revoked_at FROM install_identity WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            revoked_at.is_some(),
            "consent_revoked_at should be non-NULL after toggle-off"
        );
        assert!(!get_settings_impl(&conn).unwrap().report_uploads_enabled);
    }

    #[test]
    fn toggle_off_is_idempotent_when_already_revoked() {
        let (_dir, mut conn) = open_tmp();
        insert_synthetic_identity(&conn, Some("2026-05-30T12:00:00Z"));

        // Already revoked — toggle-off should no-op and not modify the
        // existing revoked_at timestamp.
        set_report_uploads_enabled_impl(&mut conn, false).unwrap();

        let revoked_at: Option<String> = conn
            .as_inner()
            .query_row(
                "SELECT consent_revoked_at FROM install_identity WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(revoked_at.as_deref(), Some("2026-05-30T12:00:00Z"));
    }

    /// Toggle-off when no install_identity row exists yet. Per the CL-8
    /// deferral note in this module's docstring, this is a transient state
    /// (rows are normally created during the consent-accept flow). The
    /// command is a no-op rather than an error — there is no consent to
    /// revoke, and surfacing an error here would block frontend code that
    /// reasonably calls `set_report_uploads_enabled(false)` defensively on
    /// any settings-screen mount.
    #[test]
    fn toggle_off_with_no_install_identity_row_is_no_op() {
        let (_dir, mut conn) = open_tmp();
        // No insert_synthetic_identity call — install_identity is empty.
        set_report_uploads_enabled_impl(&mut conn, false).unwrap();
        // Still no row.
        let count: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM install_identity", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        // get_settings still returns the pre-consent default.
        assert!(get_settings_impl(&conn).unwrap().report_uploads_enabled);
    }

    // --- toggle-on after revocation --------------------------------------

    #[test]
    fn toggle_on_after_revocation_returns_invalid_state_error() {
        let (_dir, mut conn) = open_tmp();
        insert_synthetic_identity(&conn, Some("2026-05-30T12:00:00Z"));

        let err = set_report_uploads_enabled_impl(&mut conn, true).unwrap_err();
        match err {
            AppError::InvalidState(msg) => {
                assert_eq!(msg, TOGGLE_ON_AFTER_REVOCATION_MSG);
            }
            other => panic!("expected AppError::InvalidState, got {other:?}"),
        }

        // The revoked_at timestamp should remain unchanged.
        let revoked_at: Option<String> = conn
            .as_inner()
            .query_row(
                "SELECT consent_revoked_at FROM install_identity WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(revoked_at.as_deref(), Some("2026-05-30T12:00:00Z"));
    }

    #[test]
    fn toggle_on_when_not_revoked_is_no_op() {
        let (_dir, mut conn) = open_tmp();
        insert_synthetic_identity(&conn, None);
        // Should not error; should not modify state.
        set_report_uploads_enabled_impl(&mut conn, true).unwrap();
        assert!(get_settings_impl(&conn).unwrap().report_uploads_enabled);
    }

    #[test]
    fn toggle_on_with_no_install_identity_row_is_no_op() {
        let (_dir, mut conn) = open_tmp();
        // No row → no revocation → no error.
        set_report_uploads_enabled_impl(&mut conn, true).unwrap();
        let count: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM install_identity", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    // --- difficulty parsing -----------------------------------------------

    #[test]
    fn parse_difficulty_accepts_all_three_levels() {
        assert_eq!(parse_difficulty("gentle").unwrap(), DifficultyLevel::Gentle);
        assert_eq!(
            parse_difficulty("standard").unwrap(),
            DifficultyLevel::Standard
        );
        assert_eq!(parse_difficulty("strict").unwrap(), DifficultyLevel::Strict);
    }

    #[test]
    fn parse_difficulty_rejects_unknown_value() {
        let err = parse_difficulty("extreme").unwrap_err();
        match err {
            AppError::InvalidState(msg) => assert!(msg.contains("extreme")),
            other => panic!("expected AppError::InvalidState, got {other:?}"),
        }
    }

    // --- set_update_checks_enabled round-trip ----------------------------

    #[test]
    fn set_update_checks_enabled_round_trip_true() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert!(settings.update_checks_enabled);
    }

    #[test]
    fn set_update_checks_enabled_round_trip_false_after_true() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        set_update_checks_enabled_impl(&mut conn, false).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert!(!settings.update_checks_enabled);
    }

    #[test]
    fn set_update_checks_enabled_defaults_to_false_on_fresh_db() {
        // A database migrated from v1 to v2 has no settings row at all;
        // get_settings_impl should report false (the column DEFAULT 0 is
        // correct for the no-row path, and the code-level default also
        // matches).
        let (_dir, conn) = open_tmp();
        let settings = get_settings_impl(&conn).unwrap();
        assert!(!settings.update_checks_enabled);
    }

    /// Demonstrates the realistic path: other setters preserve update_checks_enabled.
    #[test]
    fn set_update_checks_enabled_preserved_across_set_l1() {
        let (_dir, mut conn) = open_tmp();
        // Enable update checks first.
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        // Then change L1 — must not clobber update_checks_enabled.
        set_l1_impl(&mut conn, "spa", Some("Iberian")).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "spa");
        assert_eq!(settings.regional_variety, Some("Iberian".to_string()));
        assert!(settings.update_checks_enabled, "set_l1 must preserve update_checks_enabled");
    }

    /// Demonstrates the realistic path: set_difficulty preserves update_checks_enabled.
    #[test]
    fn set_update_checks_enabled_preserved_across_set_difficulty() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        set_difficulty_impl(&mut conn, DifficultyLevel::Standard).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.difficulty, DifficultyLevel::Standard);
        assert!(settings.update_checks_enabled, "set_difficulty must preserve update_checks_enabled");
    }

    /// set_l1 and set_difficulty preserve update_checks_enabled because their
    /// ON CONFLICT DO UPDATE clauses only name their own columns.
    /// Additional cross-setter preservation test: full chain.
    #[test]
    fn update_checks_enabled_preserved_through_full_settings_chain() {
        let (_dir, mut conn) = open_tmp();
        // Enable update checks.
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        // Modify L1 then difficulty.
        set_l1_impl(&mut conn, "kor", None).unwrap();
        set_difficulty_impl(&mut conn, DifficultyLevel::Strict).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.l1, "kor");
        assert_eq!(settings.difficulty, DifficultyLevel::Strict);
        assert!(settings.update_checks_enabled, "update_checks_enabled must survive the full chain");
    }
}
