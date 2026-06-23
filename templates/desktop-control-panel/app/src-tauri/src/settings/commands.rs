//! Tauri command surface for the settings feature.
//!
//! Generic app-state spine: the settings singleton carries a small example
//! preference set (`theme`) plus two spine toggles. Four commands, all
//! serializing through the local [`Settings`] wire type:
//!
//! - [`get_settings`] — read the singleton + derive `report_uploads_enabled`
//!   from `install_identity.consent_revoked_at`.
//! - [`set_theme`] — upsert the generic `theme` preference. The example
//!   preference a downstream app replaces / extends with its own.
//! - [`set_report_uploads_enabled`] — toggle the telemetry/consent posture.
//!   Toggle-off (`false`) aliases consent revocation; toggle-on after a prior
//!   revocation is explicitly out of scope (there is no consent-restore path).
//! - [`set_update_checks_enabled`] — toggle the updater opt-in (spine: backs
//!   the update-checks egress, off by default).
//!
//! The `install_identity` row is normally created during the first-run consent
//! flow; until then a fresh install has no row at all — [`get_settings`] treats
//! "no row" as `report_uploads_enabled = true` (the pre-consent default) and
//! [`set_report_uploads_enabled(false)`] treats "no row" as a no-op.

use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::shared::error::AppError;
use crate::storage::Connection;

pub use crate::AppState;

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

/// Generic theme preference — the example preference demonstrating the get/set
/// round-trip. Serialized as `"system" | "light" | "dark"`, matching the
/// `settings.theme` CHECK constraint.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// User-mutable settings persisted in the singleton `settings` row.
///
/// `theme` is the generic example preference. `report_uploads_enabled` is a
/// telemetry/consent posture derived from `install_identity.consent_revoked_at`
/// (not a stored settings column). `update_checks_enabled` gates the updater
/// egress and is stored directly.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Settings {
    pub theme: Theme,
    pub report_uploads_enabled: bool,
    pub update_checks_enabled: bool,
}

// ---------------------------------------------------------------------------
// Command argument structs
// ---------------------------------------------------------------------------

/// Args for [`set_theme`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetThemeArgs {
    pub theme: Theme,
}

/// Args for [`set_report_uploads_enabled`]. `enabled = false` aliases consent
/// revocation. `enabled = true` after a prior revocation returns an
/// `AppError::InvalidState` — there is no consent-restore path.
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
// Documented error message for the out-of-scope toggle-on path. Pulled out as
// a constant so tests can match on it exactly.
// ---------------------------------------------------------------------------

pub(crate) const TOGGLE_ON_AFTER_REVOCATION_MSG: &str =
    "toggle-on after revocation is out of scope; \
     uninstall and reinstall to grant consent again";

const DB_LOCK_POISONED_MSG: &str = "settings db lock poisoned";

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, AppError> {
    let conn = lock_conn(&state.db)?;
    get_settings_impl(&conn)
}

#[tauri::command]
pub async fn set_theme(
    args: SetThemeArgs,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut conn = lock_conn(&state.db)?;
    set_theme_impl(&mut conn, args.theme)
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
// Locking helper — converts `Mutex` poisoning into a typed AppError rather than
// panicking, keeping the Tauri-command surface infallible-of-panic.
// ---------------------------------------------------------------------------

fn lock_conn(db: &Mutex<Connection>) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
    db.lock()
        .map_err(|_| AppError::InvalidState(DB_LOCK_POISONED_MSG.into()))
}

// ---------------------------------------------------------------------------
// Business-logic layer. Free functions on `&Connection` / `&mut Connection` so
// tests can exercise them without spinning up a Tauri runtime.
// ---------------------------------------------------------------------------

/// Reads the singleton settings row + derives `report_uploads_enabled` from the
/// install_identity row's `consent_revoked_at`.
///
/// Defaults when nothing has been persisted yet:
///
/// - No settings row → `theme = System`, `update_checks_enabled = false`.
/// - No install_identity row → `report_uploads_enabled = true` (the
///   pre-consent posture; the row is created by the consent flow).
pub(crate) fn get_settings_impl(conn: &Connection) -> Result<Settings, AppError> {
    let inner = conn.as_inner();

    // Read settings singleton (id = 1). Absent → defaults.
    let settings_row: Option<(String, bool)> = inner
        .query_row(
            "SELECT theme, update_checks_enabled FROM settings WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?;

    let (theme, update_checks_enabled) = match settings_row {
        Some((theme_str, update_checks)) => (parse_theme(&theme_str)?, update_checks),
        None => (Theme::System, false),
    };

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
        theme,
        report_uploads_enabled,
        update_checks_enabled,
    })
}

/// Upserts the generic `theme` preference on the singleton row, preserving
/// `update_checks_enabled`.
pub(crate) fn set_theme_impl(conn: &mut Connection, theme: Theme) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();
    let theme_str = theme_to_str(theme);

    inner
        .execute(
            "INSERT INTO settings (id, theme, schema_version) \
             VALUES (1, ?1, 1) \
             ON CONFLICT(id) DO UPDATE SET theme = excluded.theme",
            params![theme_str],
        )
        .map_err(crate::storage::StorageError::from)?;

    Ok(())
}

/// Toggle the report-upload / telemetry posture.
///
/// - `enabled = false` writes `install_identity.consent_revoked_at = now()` if
///   not already revoked. If the install_identity row does not exist yet, this
///   is a no-op — there is no consent to revoke.
/// - `enabled = true` after a prior revocation returns
///   [`AppError::InvalidState`] with [`TOGGLE_ON_AFTER_REVOCATION_MSG`].
///   `enabled = true` when not previously revoked is a no-op.
pub(crate) fn set_report_uploads_enabled_impl(
    conn: &mut Connection,
    enabled: bool,
) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();

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
        if already_revoked {
            return Err(AppError::InvalidState(
                TOGGLE_ON_AFTER_REVOCATION_MSG.to_string(),
            ));
        }
        Ok(())
    } else {
        if already_revoked || consent_revoked_at.is_none() {
            return Ok(());
        }

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
/// preserving `theme`.
///
/// `true` opts the user in to periodic network egress for update checks;
/// `false` opts them out. Off by default (column `DEFAULT 0`); the user must
/// explicitly enable it. Toggling on and off is always valid.
pub(crate) fn set_update_checks_enabled_impl(
    conn: &mut Connection,
    enabled: bool,
) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();

    // Read current theme so the INSERT branch (first call on a fresh DB)
    // preserves the schema default and the ON CONFLICT branch touches only
    // update_checks_enabled.
    let current_theme: String = inner
        .query_row(
            "SELECT theme FROM settings WHERE id = 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(crate::storage::StorageError::from)?
        .unwrap_or_else(|| "system".to_string());

    let enabled_int: i32 = if enabled { 1 } else { 0 };

    inner
        .execute(
            "INSERT INTO settings (id, theme, update_checks_enabled, schema_version) \
             VALUES (1, ?1, ?2, 1) \
             ON CONFLICT(id) DO UPDATE SET \
                 update_checks_enabled = excluded.update_checks_enabled",
            params![current_theme, enabled_int],
        )
        .map_err(crate::storage::StorageError::from)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Theme <-> SQL column helpers. The SQL column is a `TEXT` with a
// `CHECK (theme IN ('system', 'light', 'dark'))` constraint, so the only valid
// string forms are the snake_case names.
// ---------------------------------------------------------------------------

fn theme_to_str(theme: Theme) -> &'static str {
    match theme {
        Theme::System => "system",
        Theme::Light => "light",
        Theme::Dark => "dark",
    }
}

fn parse_theme(s: &str) -> Result<Theme, AppError> {
    match s {
        "system" => Ok(Theme::System),
        "light" => Ok(Theme::Light),
        "dark" => Ok(Theme::Dark),
        other => Err(AppError::InvalidState(format!(
            "settings.theme has invalid value {other:?}; expected one of \
             system | light | dark (schema CHECK should have prevented this)"
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

    /// Open a fresh DB under a TempDir. Returns the TempDir so the caller holds
    /// the lifetime (drop = deletion).
    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    /// Insert a synthetic install_identity row to stand in for what the
    /// consent-accept flow populates during first-run. Tests for the
    /// consent-revoked code paths need this because no row exists by default.
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
        assert_eq!(settings.theme, Theme::System);
        // No install_identity row yet → default to true (pre-consent posture).
        assert!(settings.report_uploads_enabled);
        // update_checks_enabled defaults to false (opt-in, off by default).
        assert!(!settings.update_checks_enabled);
    }

    // --- set_theme round-trip (the generic example preference) ------------

    #[test]
    fn set_theme_round_trip_dark() {
        let (_dir, mut conn) = open_tmp();
        set_theme_impl(&mut conn, Theme::Dark).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.theme, Theme::Dark);
    }

    #[test]
    fn set_theme_round_trip_light() {
        let (_dir, mut conn) = open_tmp();
        set_theme_impl(&mut conn, Theme::Light).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.theme, Theme::Light);
    }

    #[test]
    fn set_theme_overwrites_existing() {
        let (_dir, mut conn) = open_tmp();
        set_theme_impl(&mut conn, Theme::Dark).unwrap();
        set_theme_impl(&mut conn, Theme::System).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.theme, Theme::System);
    }

    // --- theme parsing ----------------------------------------------------

    #[test]
    fn parse_theme_accepts_all_three() {
        assert_eq!(parse_theme("system").unwrap(), Theme::System);
        assert_eq!(parse_theme("light").unwrap(), Theme::Light);
        assert_eq!(parse_theme("dark").unwrap(), Theme::Dark);
    }

    #[test]
    fn parse_theme_rejects_unknown_value() {
        let err = parse_theme("neon").unwrap_err();
        match err {
            AppError::InvalidState(msg) => assert!(msg.contains("neon")),
            other => panic!("expected AppError::InvalidState, got {other:?}"),
        }
    }

    #[test]
    fn settings_theme_check_constraint_rejects_invalid_value() {
        let (_dir, conn) = open_tmp();
        let err = conn
            .as_inner()
            .execute("INSERT INTO settings (id, theme) VALUES (1, 'neon')", [])
            .unwrap_err();
        match err {
            rusqlite::Error::SqliteFailure(e, _) => {
                assert_eq!(
                    e.extended_code,
                    rusqlite::ffi::SQLITE_CONSTRAINT_CHECK,
                    "expected SQLITE_CONSTRAINT_CHECK, got extended_code={}",
                    e.extended_code
                );
            }
            other => panic!("expected SqliteFailure(CHECK), got {other:?}"),
        }
    }

    // --- toggle-off path -------------------------------------------------

    #[test]
    fn toggle_off_sets_consent_revoked_at_and_flips_report_uploads_enabled() {
        let (_dir, mut conn) = open_tmp();
        insert_synthetic_identity(&conn, None);

        assert!(get_settings_impl(&conn).unwrap().report_uploads_enabled);

        set_report_uploads_enabled_impl(&mut conn, false).unwrap();

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

    #[test]
    fn toggle_off_with_no_install_identity_row_is_no_op() {
        let (_dir, mut conn) = open_tmp();
        set_report_uploads_enabled_impl(&mut conn, false).unwrap();
        let count: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM install_identity", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
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
        set_report_uploads_enabled_impl(&mut conn, true).unwrap();
        assert!(get_settings_impl(&conn).unwrap().report_uploads_enabled);
    }

    #[test]
    fn toggle_on_with_no_install_identity_row_is_no_op() {
        let (_dir, mut conn) = open_tmp();
        set_report_uploads_enabled_impl(&mut conn, true).unwrap();
        let count: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM install_identity", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
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
        let (_dir, conn) = open_tmp();
        let settings = get_settings_impl(&conn).unwrap();
        assert!(!settings.update_checks_enabled);
    }

    /// set_theme preserves update_checks_enabled (its ON CONFLICT clause names
    /// only `theme`).
    #[test]
    fn set_update_checks_enabled_preserved_across_set_theme() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        set_theme_impl(&mut conn, Theme::Dark).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.theme, Theme::Dark);
        assert!(
            settings.update_checks_enabled,
            "set_theme must preserve update_checks_enabled"
        );
    }

    /// set_update_checks_enabled preserves the theme preference.
    #[test]
    fn set_theme_preserved_across_set_update_checks_enabled() {
        let (_dir, mut conn) = open_tmp();
        set_theme_impl(&mut conn, Theme::Light).unwrap();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        let settings = get_settings_impl(&conn).unwrap();
        assert_eq!(settings.theme, Theme::Light);
        assert!(settings.update_checks_enabled);
    }
}
