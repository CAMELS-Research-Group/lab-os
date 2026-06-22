//! Tauri command surface for storage-level maintenance operations.
//!
//! Commands:
//! - [`clear_session_data`]: deletes the learner's on-disk practice history
//!   (the `sessions` table and its dependent `upload_queue` rows) while
//!   leaving settings and install identity intact.
//! - [`get_session_history`]: returns a compact summary of all persisted
//!   sessions, ordered oldest-first, for display in the Progress screen.
//! - [`get_phoneme_trends`]: aggregates per-phoneme flag-rate series across
//!   sessions and classifies each as improving/worsening/flat for the Progress screen.
//!
//! [`clear_session_data`] is deliberately *not* coupled to the frontend "Reset
//! app / UI state" action — that one only clears Zustand/localStorage. The two
//! clears are independent: a learner can wipe their history without re-running
//! first-run, or reset the UI without losing their history.

use std::sync::Mutex;

use crate::shared::error::AppError;
use crate::shared::types::{
    PhonemeAttempts, PhonemeTrend, SessionId, SessionSummary, TrendDirection,
};
use crate::storage::Connection;

pub use crate::AppState;

const DB_LOCK_POISONED_MSG: &str = "storage db lock poisoned";

// ---------------------------------------------------------------------------
// get_session_history
// ---------------------------------------------------------------------------

/// IPC return wrapper so the wire shape is `{ "sessions": [...] }`.
#[derive(serde::Serialize)]
pub struct SessionHistory {
    pub sessions: Vec<SessionSummary>,
}

/// Returns the full session history as a compact list of [`SessionSummary`]
/// rows, oldest-first. The Progress screen reverses the list for display.
///
/// Errors in a single row's `phoneme_attempts_json` (malformed JSON) are
/// swallowed — that row is returned with `flagged_count = 0` and
/// `highest_error_phoneme = None` rather than aborting the whole call.
///
/// `SessionSummary` carries `flagged_count` + `highest_error_phoneme` as part
/// of its established wire contract (`shared::types::SessionSummary`, mirrored
/// in `ipc/types.ts`). The current Progress screen only renders date +
/// duration; these aggregates are computed here so the planned per-sound
/// trends view ("coming in a future update" on Progress) can surface them with
/// no further Rust change. Forward-shaped, not dead code.
#[tauri::command]
pub async fn get_session_history(
    state: tauri::State<'_, AppState>,
) -> Result<SessionHistory, AppError> {
    let conn = lock_conn(&state.db)?;
    get_session_history_impl(&conn)
}

/// Business-logic layer; exercised directly by tests without a Tauri runtime.
pub(crate) fn get_session_history_impl(conn: &Connection) -> Result<SessionHistory, AppError> {
    let inner = conn.as_inner();
    let mut stmt = inner
        .prepare(
            "SELECT session_id, ended_at, duration_seconds, phoneme_attempts_json \
             FROM sessions ORDER BY started_at ASC",
        )
        .map_err(crate::storage::StorageError::from)?;

    let sessions: Vec<SessionSummary> = stmt
        .query_map([], |row| {
            let session_id: String = row.get(0)?;
            let ended_at: String = row.get(1)?;
            let duration_raw: i64 = row.get(2)?;
            let phoneme_json: String = row.get(3)?;
            Ok((session_id, ended_at, duration_raw, phoneme_json))
        })
        .map_err(crate::storage::StorageError::from)?
        .filter_map(|r| r.ok())
        .map(|(session_id, ended_at, duration_raw, phoneme_json)| {
            let (flagged_count, highest_error_phoneme) =
                aggregate_phoneme_attempts(&phoneme_json);
            SessionSummary {
                session_id: SessionId(session_id),
                ended_at,
                duration_seconds: duration_raw as f64,
                flagged_count,
                highest_error_phoneme,
            }
        })
        .collect();

    Ok(SessionHistory { sessions })
}

/// Parse `phoneme_attempts_json` and compute the two derived fields needed for
/// [`SessionSummary`]:
///
/// - `flagged_count`: sum of `roll.flagged` across all phoneme entries.
/// - `highest_error_phoneme`: the symbol with the greatest `flagged` count
///   among entries with `flagged > 0`; ties broken by lower `mean_certainty`
///   (ascending — treating `None` as +∞ so it loses ties), then by symbol
///   ascending for determinism. `None` when nothing is flagged.
///
/// On parse failure returns `(0, None)` — the row still appears in the
/// history list, just without aggregated error data.
fn aggregate_phoneme_attempts(json: &str) -> (u32, Option<String>) {
    let attempts: PhonemeAttempts = match serde_json::from_str(json) {
        Ok(a) => a,
        Err(e) => {
            log::warn!("get_session_history: bad phoneme_attempts_json — {e}");
            return (0, None);
        }
    };

    let flagged_count: u32 = attempts.0.values().map(|r| r.flagged).sum();

    // Collect only flagged entries and apply the tie-breaking sort.
    let mut flagged_entries: Vec<(&String, u32, Option<f64>)> = attempts
        .0
        .iter()
        .filter(|(_, r)| r.flagged > 0)
        .map(|(sym, r)| (sym, r.flagged, r.mean_certainty))
        .collect();

    // Sort: descending flagged, then ascending mean_certainty (None = +∞ →
    // sorts last among peers with the same flag count), then symbol ascending.
    flagged_entries.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| {
                // Ascending mean_certainty: lower certainty wins.
                // None is treated as +∞ so it sorts after any Some value.
                // `partial_cmp` on the f64 directly is self-evidently correct
                // for any value (NaN never occurs in a valid AttemptRollup, but
                // `unwrap_or(Equal)` keeps the sort total if it ever did).
                match (a.2, b.2) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => std::cmp::Ordering::Greater, // a is +∞ → comes after b
                    (Some(_), None) => std::cmp::Ordering::Less,    // b is +∞ → a comes first
                    (Some(av), Some(bv)) => {
                        av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }
            })
            .then_with(|| a.0.cmp(b.0))
    });

    let highest = flagged_entries.first().map(|(sym, _, _)| sym.to_string());
    (flagged_count, highest)
}

// ---------------------------------------------------------------------------
// get_phoneme_trends
// ---------------------------------------------------------------------------

/// IPC return wrapper so the wire shape is `{ "per_phoneme": [...] }`.
#[derive(serde::Serialize)]
pub struct PhonemeTrends {
    pub per_phoneme: Vec<PhonemeTrend>,
}

/// Dead-band (absolute flag-rate delta) below which a trend reads as `Flat`.
const TREND_DEAD_BAND: f64 = 0.10;

/// Returns the cross-session per-phoneme trend series the Progress "trends"
/// cards depend on.
///
/// Reads every `sessions` row oldest→newest, deserializes each
/// `phoneme_attempts_json`, and aggregates per phoneme. Malformed rows are
/// skipped (same resilience as [`get_session_history`]) without failing the
/// call. The result contains **only** phonemes observed at least once
/// (`attempts_total > 0`).
#[tauri::command]
pub async fn get_phoneme_trends(
    state: tauri::State<'_, AppState>,
) -> Result<PhonemeTrends, AppError> {
    let conn = lock_conn(&state.db)?;
    get_phoneme_trends_impl(&conn)
}

/// Business-logic layer; exercised directly by tests without a Tauri runtime.
pub(crate) fn get_phoneme_trends_impl(conn: &Connection) -> Result<PhonemeTrends, AppError> {
    let inner = conn.as_inner();
    let mut stmt = inner
        .prepare("SELECT phoneme_attempts_json FROM sessions ORDER BY started_at ASC")
        .map_err(crate::storage::StorageError::from)?;

    let rows: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(crate::storage::StorageError::from)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(PhonemeTrends {
        per_phoneme: aggregate_phoneme_trends(&rows),
    })
}

/// Per-phoneme accumulator while walking the session rows oldest→newest.
#[derive(Default)]
struct TrendAccumulator {
    attempts_total: u32,
    flagged_total: u32,
    /// One entry per OBSERVED session (`occurrences > 0`), oldest→newest.
    session_flag_rate: Vec<f64>,
}

/// Aggregate a sequence of `phoneme_attempts_json` strings (already ordered
/// oldest→newest) into one [`PhonemeTrend`] per phoneme observed at least once.
///
/// Malformed JSON rows are logged and skipped (they contribute nothing). A
/// session in which a phoneme has `occurrences == 0` — or is absent entirely —
/// is NOT counted as an observed session for that phoneme and adds no entry to
/// its `session_flag_rate` series (the series is observed-only, no gaps).
fn aggregate_phoneme_trends(rows: &[String]) -> Vec<PhonemeTrend> {
    use std::collections::BTreeMap;

    // BTreeMap → deterministic (symbol-ascending) output order.
    let mut acc: BTreeMap<String, TrendAccumulator> = BTreeMap::new();

    for json in rows {
        let attempts: PhonemeAttempts = match serde_json::from_str(json) {
            Ok(a) => a,
            Err(e) => {
                log::warn!("get_phoneme_trends: bad phoneme_attempts_json — {e}");
                continue;
            }
        };
        for (sym, roll) in attempts.0.iter() {
            if roll.occurrences == 0 {
                continue; // absent-this-session → not observed, no series entry.
            }
            let entry = acc.entry(sym.clone()).or_default();
            entry.attempts_total += roll.occurrences;
            entry.flagged_total += roll.flagged;
            entry
                .session_flag_rate
                .push(f64::from(roll.flagged) / f64::from(roll.occurrences));
        }
    }

    acc.into_iter()
        .filter(|(_, a)| a.attempts_total > 0)
        .map(|(phoneme, a)| {
            let example_word = crate::evaluation::feedback::lookup_articulation(&phoneme)
                .map(|e| e.example_word.to_string())
                .unwrap_or_default();
            let trend_direction = trend_direction(&a.session_flag_rate);
            PhonemeTrend {
                phoneme,
                example_word,
                attempts_total: a.attempts_total,
                flagged_total: a.flagged_total,
                trend_direction,
                sessions_observed: a.session_flag_rate.len() as u32,
                session_flag_rate: a.session_flag_rate,
            }
        })
        .collect()
}

/// Classify the direction of an observed-only flag-rate series.
///
/// `< 2` observed sessions → `Flat` (the frontend renders the "not enough
/// practice yet" state off `sessions_observed`). Otherwise split into earlier
/// and later halves — for odd `N`, the middle element joins the LATER half
/// (earlier = first `N/2`, later = remaining `N - N/2`) — and compare means
/// against a `0.10` absolute dead-band: a later mean meaningfully lower →
/// `Improving`, meaningfully higher → `Worsening`, else `Flat`.
fn trend_direction(series: &[f64]) -> TrendDirection {
    let n = series.len();
    // Mirror of `MIN_OBSERVED = 2` in the Progress screen (src/screens/
    // Progress.tsx): the frontend only ranks a phoneme at >= this many observed
    // sessions, so a future edit to either "2 observed sessions" threshold must
    // update the other to stay consistent.
    if n < 2 {
        return TrendDirection::Flat;
    }
    let split = n / 2; // earlier half size; middle (odd N) lands in later half.
    let earlier = &series[..split];
    let later = &series[split..];
    let mean = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    let earlier_mean = mean(earlier);
    let later_mean = mean(later);

    if later_mean < earlier_mean - TREND_DEAD_BAND {
        TrendDirection::Improving
    } else if later_mean > earlier_mean + TREND_DEAD_BAND {
        TrendDirection::Worsening
    } else {
        TrendDirection::Flat
    }
}

/// Deletes all session/history rows from the database.
///
/// Removes every row from `upload_queue` and `sessions`. Settings and install
/// identity are untouched, so the learner's preferences (L1, difficulty,
/// update-check opt-in) and consent state survive the clear.
///
/// Returns the number of `sessions` rows deleted so the caller can confirm the
/// wipe (and so a test can assert it).
#[tauri::command]
pub async fn clear_session_data(
    state: tauri::State<'_, AppState>,
) -> Result<u64, AppError> {
    let mut conn = lock_conn(&state.db)?;
    clear_session_data_impl(&mut conn)
}

fn lock_conn(db: &Mutex<Connection>) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
    db.lock()
        .map_err(|_| AppError::InvalidState(DB_LOCK_POISONED_MSG.into()))
}

/// Business-logic layer, exercised directly by tests without a Tauri runtime.
///
/// Both deletes run inside a single transaction so a mid-clear failure rolls
/// back atomically — the learner never ends up with sessions gone but their
/// upload queue still referencing them (the `upload_queue.session_id` FK has
/// `ON DELETE CASCADE`, so deleting `upload_queue` first is belt-and-braces).
pub(crate) fn clear_session_data_impl(conn: &mut Connection) -> Result<u64, AppError> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM upload_queue", [])
        .map_err(crate::storage::StorageError::from)?;
    let sessions_deleted = tx
        .execute("DELETE FROM sessions", [])
        .map_err(crate::storage::StorageError::from)?;
    tx.commit().map_err(crate::storage::StorageError::from)?;
    Ok(sessions_deleted as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use tempfile::TempDir;

    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ias.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    /// Insert helper with a fixed `started_at` and `phoneme_attempts_json = "[]"`.
    fn insert_session(conn: &Connection, session_id: &str) {
        insert_session_full(conn, session_id, "2026-06-01T00:00:00Z", "[]");
    }

    /// Insert helper with caller-controlled `started_at` and `phoneme_attempts_json`.
    fn insert_session_full(
        conn: &Connection,
        session_id: &str,
        started_at: &str,
        phoneme_attempts_json: &str,
    ) {
        conn.as_inner()
            .execute(
                "INSERT INTO sessions (
                    session_id, started_at, ended_at, duration_seconds,
                    l1_at_session, regional_variety_at_session, phoneme_attempts_json,
                    difficulty_level, difficulty_thresholds_json, threshold_table_version,
                    reattempt_counts_json, cumulative_session_count,
                    app_version, model_version, os_family, os_major
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
                )",
                params![
                    session_id,
                    started_at,
                    "2026-06-01T00:05:00Z",
                    300_i64,
                    "es",
                    None::<&str>,
                    phoneme_attempts_json,
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

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.as_inner()
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn clear_session_data_removes_sessions_and_queue() {
        let (_dir, mut conn) = open_tmp();
        insert_session(&conn, "s1");
        insert_session(&conn, "s2");
        conn.as_inner()
            .execute(
                "INSERT INTO upload_queue (
                    session_id, payload_kind, payload_json, queued_at, attempt_count
                 ) VALUES (?1, 'session_report', '{}', '2026-06-01T00:00:00Z', 0)",
                params!["s1"],
            )
            .unwrap();

        let deleted = clear_session_data_impl(&mut conn).unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(count(&conn, "sessions"), 0);
        assert_eq!(count(&conn, "upload_queue"), 0);
    }

    #[test]
    fn clear_session_data_preserves_settings_and_identity() {
        let (_dir, mut conn) = open_tmp();
        conn.as_inner()
            .execute(
                "INSERT INTO settings (id, l1, difficulty) VALUES (1, 'spa', 'strict')",
                [],
            )
            .unwrap();
        conn.as_inner()
            .execute(
                "INSERT INTO install_identity \
                 (id, uuid, consent_granted_at, consent_revoked_at, registered_at, schema_version) \
                 VALUES (1, 'u', '2026-06-01T00:00:00Z', NULL, '2026-06-01T00:00:00Z', 1)",
                [],
            )
            .unwrap();
        insert_session(&conn, "s1");

        clear_session_data_impl(&mut conn).unwrap();

        assert_eq!(count(&conn, "sessions"), 0);
        assert_eq!(count(&conn, "settings"), 1, "settings must survive the clear");
        assert_eq!(count(&conn, "install_identity"), 1, "identity must survive the clear");
        let l1: String = conn
            .as_inner()
            .query_row("SELECT l1 FROM settings WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(l1, "spa");
    }

    #[test]
    fn clear_session_data_on_empty_db_is_noop() {
        let (_dir, mut conn) = open_tmp();
        let deleted = clear_session_data_impl(&mut conn).unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(count(&conn, "sessions"), 0);
    }

    // -----------------------------------------------------------------------
    // get_session_history tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_session_history_empty_db_returns_zero_sessions() {
        let (_dir, conn) = open_tmp();
        let history = get_session_history_impl(&conn).unwrap();
        assert!(
            history.sessions.is_empty(),
            "expected 0 sessions on empty DB, got {}",
            history.sessions.len()
        );
    }

    #[test]
    fn get_session_history_ordering_and_aggregation() {
        let (_dir, conn) = open_tmp();

        // Session A — older, has flagged phonemes.
        let pa_json = r#"{"θ":{"occurrences":3,"flagged":2,"mean_certainty":0.4},"w":{"occurrences":2,"flagged":1,"mean_certainty":0.6}}"#;
        insert_session_full(&conn, "session-a", "2026-06-01T10:00:00Z", pa_json);

        // Session B — newer, no flagged phonemes.
        let pb_json = r#"{"θ":{"occurrences":3,"flagged":0,"mean_certainty":0.9}}"#;
        insert_session_full(&conn, "session-b", "2026-06-01T11:00:00Z", pb_json);

        let history = get_session_history_impl(&conn).unwrap();
        assert_eq!(history.sessions.len(), 2, "expected 2 sessions");

        // Oldest-first: A then B.
        assert_eq!(history.sessions[0].session_id, SessionId("session-a".to_string()));
        assert_eq!(history.sessions[1].session_id, SessionId("session-b".to_string()));

        // Session A aggregation: flagged_count = 2 + 1 = 3, highest = "θ" (2 flags > "w"'s 1).
        assert_eq!(history.sessions[0].flagged_count, 3);
        assert_eq!(
            history.sessions[0].highest_error_phoneme,
            Some("θ".to_string())
        );

        // Session B: nothing flagged.
        assert_eq!(history.sessions[1].flagged_count, 0);
        assert_eq!(history.sessions[1].highest_error_phoneme, None);
    }

    #[test]
    fn get_session_history_tie_breaking_by_certainty_then_symbol() {
        let (_dir, conn) = open_tmp();

        // Two phonemes tied on flagged count; lower mean_certainty wins.
        // "z" has certainty 0.3 (lower) → should be highest_error_phoneme.
        let pa_json = r#"{"a":{"occurrences":2,"flagged":2,"mean_certainty":0.7},"z":{"occurrences":2,"flagged":2,"mean_certainty":0.3}}"#;
        insert_session_full(&conn, "session-tie", "2026-06-01T10:00:00Z", pa_json);

        let history = get_session_history_impl(&conn).unwrap();
        assert_eq!(history.sessions.len(), 1);
        assert_eq!(history.sessions[0].flagged_count, 4);
        assert_eq!(
            history.sessions[0].highest_error_phoneme,
            Some("z".to_string()),
            "lower certainty (0.3) should win the tie over 0.7"
        );
    }

    #[test]
    fn get_session_history_bad_phoneme_json_defaults_gracefully() {
        let (_dir, conn) = open_tmp();
        // Intentionally malformed JSON — should not abort the whole call.
        insert_session_full(&conn, "bad-json-session", "2026-06-01T10:00:00Z", "NOT_JSON");

        let history = get_session_history_impl(&conn).unwrap();
        assert_eq!(history.sessions.len(), 1);
        assert_eq!(history.sessions[0].flagged_count, 0);
        assert_eq!(history.sessions[0].highest_error_phoneme, None);
    }

    // -----------------------------------------------------------------------
    // get_phoneme_trends tests
    // -----------------------------------------------------------------------

    /// Find the trend for a phoneme in the result, or panic.
    fn trend_for<'a>(trends: &'a PhonemeTrends, phoneme: &str) -> &'a PhonemeTrend {
        trends
            .per_phoneme
            .iter()
            .find(|t| t.phoneme == phoneme)
            .unwrap_or_else(|| panic!("expected a trend for {phoneme}"))
    }

    #[test]
    fn get_phoneme_trends_empty_db_returns_nothing() {
        let (_dir, conn) = open_tmp();
        let trends = get_phoneme_trends_impl(&conn).unwrap();
        assert!(trends.per_phoneme.is_empty());
    }

    #[test]
    fn get_phoneme_trends_improving() {
        let (_dir, conn) = open_tmp();
        // Flag rate falls 1.0 → 0.0 across 4 sessions: earlier mean 0.875, later 0.125.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":4,"flagged":4,"mean_certainty":0.2}}"#);
        insert_session_full(&conn, "s2", "2026-06-01T02:00:00Z", r#"{"θ":{"occurrences":4,"flagged":3,"mean_certainty":0.3}}"#);
        insert_session_full(&conn, "s3", "2026-06-01T03:00:00Z", r#"{"θ":{"occurrences":4,"flagged":1,"mean_certainty":0.6}}"#);
        insert_session_full(&conn, "s4", "2026-06-01T04:00:00Z", r#"{"θ":{"occurrences":4,"flagged":0,"mean_certainty":0.9}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&trends, "θ");
        assert_eq!(t.attempts_total, 16);
        assert_eq!(t.flagged_total, 8);
        assert_eq!(t.sessions_observed, 4);
        assert_eq!(t.session_flag_rate, vec![1.0, 0.75, 0.25, 0.0]);
        assert_eq!(t.trend_direction, TrendDirection::Improving);
    }

    #[test]
    fn get_phoneme_trends_worsening() {
        let (_dir, conn) = open_tmp();
        // Flag rate climbs 0.0 → 1.0: later half mean well above earlier half.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":4,"flagged":0,"mean_certainty":0.9}}"#);
        insert_session_full(&conn, "s2", "2026-06-01T02:00:00Z", r#"{"θ":{"occurrences":4,"flagged":1,"mean_certainty":0.6}}"#);
        insert_session_full(&conn, "s3", "2026-06-01T03:00:00Z", r#"{"θ":{"occurrences":4,"flagged":3,"mean_certainty":0.3}}"#);
        insert_session_full(&conn, "s4", "2026-06-01T04:00:00Z", r#"{"θ":{"occurrences":4,"flagged":4,"mean_certainty":0.2}}"#);

        let t = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&t, "θ");
        assert_eq!(t.session_flag_rate, vec![0.0, 0.25, 0.75, 1.0]);
        assert_eq!(t.trend_direction, TrendDirection::Worsening);
    }

    #[test]
    fn get_phoneme_trends_odd_series_middle_joins_later_half() {
        let (_dir, conn) = open_tmp();
        // 3 sessions, flag rate 1.0 → 0.5 → 0.0. split = n/2 = 1, so the middle
        // element (0.5) joins the LATER half: earlier = [1.0] mean 1.0,
        // later = [0.5, 0.0] mean 0.25 → Improving.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":2,"flagged":2,"mean_certainty":0.2}}"#);
        insert_session_full(&conn, "s2", "2026-06-01T02:00:00Z", r#"{"θ":{"occurrences":2,"flagged":1,"mean_certainty":0.5}}"#);
        insert_session_full(&conn, "s3", "2026-06-01T03:00:00Z", r#"{"θ":{"occurrences":2,"flagged":0,"mean_certainty":0.9}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&trends, "θ");
        assert_eq!(t.sessions_observed, 3);
        assert_eq!(t.session_flag_rate, vec![1.0, 0.5, 0.0]);
        assert_eq!(t.trend_direction, TrendDirection::Improving);
    }

    #[test]
    fn get_phoneme_trends_two_session_boundary() {
        let (_dir, conn) = open_tmp();
        // N=2: split = 1, earlier = [1.0], later = [0.0] → Improving.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":2,"flagged":2,"mean_certainty":0.2}}"#);
        insert_session_full(&conn, "s2", "2026-06-01T02:00:00Z", r#"{"θ":{"occurrences":2,"flagged":0,"mean_certainty":0.9}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&trends, "θ");
        assert_eq!(t.session_flag_rate, vec![1.0, 0.0]);
        assert_eq!(t.trend_direction, TrendDirection::Improving);
    }

    #[test]
    fn get_phoneme_trends_flat_within_dead_band() {
        let (_dir, conn) = open_tmp();
        // Flag rate hovers; earlier/later means differ by < 0.10.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":4,"flagged":2,"mean_certainty":0.5}}"#);
        insert_session_full(&conn, "s2", "2026-06-01T02:00:00Z", r#"{"θ":{"occurrences":4,"flagged":2,"mean_certainty":0.5}}"#);
        insert_session_full(&conn, "s3", "2026-06-01T03:00:00Z", r#"{"θ":{"occurrences":4,"flagged":2,"mean_certainty":0.5}}"#);
        insert_session_full(&conn, "s4", "2026-06-01T04:00:00Z", r#"{"θ":{"occurrences":4,"flagged":2,"mean_certainty":0.5}}"#);

        let t = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&t, "θ");
        assert_eq!(t.session_flag_rate, vec![0.5, 0.5, 0.5, 0.5]);
        assert_eq!(t.trend_direction, TrendDirection::Flat);
    }

    #[test]
    fn get_phoneme_trends_single_session_is_flat() {
        let (_dir, conn) = open_tmp();
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":4,"flagged":4,"mean_certainty":0.2}}"#);

        let t = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&t, "θ");
        assert_eq!(t.sessions_observed, 1, "one observed session");
        assert_eq!(t.session_flag_rate, vec![1.0]);
        assert_eq!(
            t.trend_direction,
            TrendDirection::Flat,
            "< 2 observed sessions → flat"
        );
    }

    #[test]
    fn get_phoneme_trends_absent_sessions_excluded_from_series() {
        let (_dir, conn) = open_tmp();
        // θ occurs in s1 and s3 only; absent in s2 (key missing) and
        // zero-occurrence in s4. Its observed series must have exactly 2 entries.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":2,"flagged":2,"mean_certainty":0.2}}"#);
        insert_session_full(&conn, "s2", "2026-06-01T02:00:00Z", r#"{"w":{"occurrences":3,"flagged":1,"mean_certainty":0.5}}"#);
        insert_session_full(&conn, "s3", "2026-06-01T03:00:00Z", r#"{"θ":{"occurrences":2,"flagged":0,"mean_certainty":0.9}}"#);
        insert_session_full(&conn, "s4", "2026-06-01T04:00:00Z", r#"{"θ":{"occurrences":0,"flagged":0,"mean_certainty":null}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&trends, "θ");
        assert_eq!(t.sessions_observed, 2, "only s1 and s3 observed θ");
        assert_eq!(t.session_flag_rate.len(), t.sessions_observed as usize);
        assert_eq!(t.session_flag_rate, vec![1.0, 0.0], "oldest→newest, no gaps");
        assert_eq!(t.attempts_total, 4);
        assert_eq!(t.flagged_total, 2);

        // w only appeared once (s2) → present but flat.
        let w = trend_for(&trends, "w");
        assert_eq!(w.sessions_observed, 1);
        assert_eq!(w.trend_direction, TrendDirection::Flat);
    }

    #[test]
    fn get_phoneme_trends_example_word_lookup() {
        let (_dir, conn) = open_tmp();
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":2,"flagged":1,"mean_certainty":0.4}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&trends, "θ");
        // θ is in the V1 articulation table → non-empty example word.
        assert!(!t.example_word.is_empty(), "θ should resolve an example word");
    }

    #[test]
    fn get_phoneme_trends_unknown_phoneme_empty_example_word() {
        let (_dir, conn) = open_tmp();
        // "ZZZ" is not in the 13-entry articulation table.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"ZZZ":{"occurrences":2,"flagged":1,"mean_certainty":0.4}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&trends, "ZZZ");
        assert_eq!(t.example_word, "", "unknown phoneme falls back to empty word");
        assert_eq!(t.attempts_total, 2, "still carries real attempt data");
    }

    #[test]
    fn get_phoneme_trends_skips_malformed_json_row() {
        let (_dir, conn) = open_tmp();
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":4,"flagged":4,"mean_certainty":0.2}}"#);
        insert_session_full(&conn, "s2", "2026-06-01T02:00:00Z", "NOT_JSON");
        insert_session_full(&conn, "s3", "2026-06-01T03:00:00Z", r#"{"θ":{"occurrences":4,"flagged":0,"mean_certainty":0.9}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        let t = trend_for(&trends, "θ");
        // Malformed s2 contributes nothing; θ observed in s1 + s3 only.
        assert_eq!(t.sessions_observed, 2);
        assert_eq!(t.session_flag_rate, vec![1.0, 0.0]);
        assert_eq!(t.attempts_total, 8);
        assert_eq!(t.flagged_total, 4);
    }

    #[test]
    fn get_phoneme_trends_excludes_zero_attempt_phonemes() {
        let (_dir, conn) = open_tmp();
        // θ only ever has occurrences == 0 → never observed → excluded entirely.
        insert_session_full(&conn, "s1", "2026-06-01T01:00:00Z", r#"{"θ":{"occurrences":0,"flagged":0,"mean_certainty":null}}"#);

        let trends = get_phoneme_trends_impl(&conn).unwrap();
        assert!(
            trends.per_phoneme.iter().all(|t| t.phoneme != "θ"),
            "phoneme with attempts_total == 0 must be excluded"
        );
    }
}
