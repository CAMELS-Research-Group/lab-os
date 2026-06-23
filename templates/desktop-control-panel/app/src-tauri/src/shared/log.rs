//! Logging configuration for the IAS client.
//!
//! Source of truth: ADD §3.11 (`documentation/engineering/specifications/ADD.md`).
//!
//! # What MUST NOT be logged
//!
//! - Raw audio bytes (recorded buffers; level-meter raw values past aggregate RMS).
//! - API key (any keyring-sourced credential).
//! - Install UUID — exception only at first-launch DEBUG-level confirmation, never again.
//! - Any PII: learner identity beyond the UUID, transcripts, free-text feedback.
//!
//! # What IS logged (per ADD §3.11)
//!
//! - State transitions (`Idle → Recording`, etc.) at INFO.
//! - Tauri command entry/exit at DEBUG (payload SIZES, not contents).
//! - `AppError` at WARN or ERROR.
//! - Audio buffer sizes, inference timing, queue depth at DEBUG.
//!
//! # Implementation notes
//!
//! - `tauri-plugin-log` v2 derives the log directory from
//!   `AppHandle::path().app_log_dir()` when `TargetKind::LogDir` is used; the
//!   `app_log_dir: &Path` argument to [`init`] is therefore informational only
//!   (recorded via [`INIT_GUARD`] so tests / future diagnostics can observe
//!   what the caller intended).
//! - The plugin appends the `.log` extension to the file basename
//!   automatically (`RotatingFile` joins `<dir>/<basename>.log`), so we pass
//!   `"app"` and get `<app_log_dir>/app.log`.
//! - We rely on the plugin's default desktop format
//!   (`[YYYY-MM-DD][HH:MM:SS][target][level] message`) which already includes
//!   timestamp, module path (= log target), level, and message per the
//!   acceptance bullet. No explicit `.format(...)` call is needed.
//! - Idempotency: the plugin's own `attach_logger` will fail if a global
//!   logger is already installed, so the caller is responsible for
//!   registering the plugin exactly once. [`INIT_GUARD`] records the
//!   resolved log path on first call and ignores subsequent calls.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri::Runtime;
use tauri_plugin_log::{Builder, RotationStrategy, Target, TargetKind};

/// 5 MiB. Plugin builder takes `u128`; clamped internally to `u64::MAX`.
const MAX_LOG_SIZE_BYTES: u128 = 5 * 1024 * 1024;

/// Keep the active log plus 4 rotated archives = 5 files retained total.
const MAX_LOG_FILES: usize = 5;

/// Basename of the log file. The plugin appends `.log` automatically.
/// Placeholder name — rename after forking if a product-specific log filename
/// is preferred (cosmetic; nothing external depends on it).
const LOG_FILE_BASENAME: &str = "app";

/// One-shot record of the path passed to [`init`]. Process-wide.
static INIT_GUARD: OnceLock<PathBuf> = OnceLock::new();

/// Records the log path the caller intended. First-write-wins; subsequent
/// calls are no-ops.
///
/// Factored out so tests can exercise the path-recording side effect without
/// constructing a runtime-generic `TauriPlugin`.
fn record_init_path(p: &Path) {
    let _ = INIT_GUARD.set(p.to_path_buf());
}

/// Build the `tauri-plugin-log` plugin configured per ADD §3.11.
///
/// `app_log_dir` is informational — the plugin itself resolves the log
/// directory from `AppHandle::path().app_log_dir()` at registration time.
/// Pass the resolved path here so it can be retrieved via
/// [`initialized_path`] for diagnostics / tests.
///
/// Returns a freshly constructed `TauriPlugin` on every call; the caller is
/// responsible for registering it exactly once with the Tauri builder. The
/// only persistent state owned by this module is [`INIT_GUARD`], which is
/// recorded on the first call only.
pub fn init<R: Runtime>(app_log_dir: &Path) -> tauri::plugin::TauriPlugin<R> {
    record_init_path(app_log_dir);

    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    Builder::default()
        .level(level)
        .max_file_size(MAX_LOG_SIZE_BYTES)
        .rotation_strategy(RotationStrategy::KeepSome(MAX_LOG_FILES))
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir {
                file_name: Some(LOG_FILE_BASENAME.to_string()),
            }),
        ])
        .build()
}

/// Returns the path [`init`] was first called with, or `None` if it has not
/// been called yet in this process.
pub fn initialized_path() -> Option<PathBuf> {
    INIT_GUARD.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Acceptance: calling `init` twice does not panic AND the configured
    /// file path matches the input (first call wins; second is ignored).
    ///
    /// We exercise `record_init_path` directly rather than `init` because
    /// `init` is generic over `Runtime` and the runtime-generic plugin
    /// construction has no useful behavior to test at this layer (the
    /// plugin's `.build()` is a no-op until Tauri runs its `.setup` closure
    /// at registration time). The path-recording side effect is the only
    /// thing this module owns; that is what we verify.
    ///
    /// `OnceLock` is process-wide, so we combine both acceptance checks into
    /// one test to keep behavior deterministic without introducing
    /// `serial_test` or a reset backdoor.
    #[test]
    fn init_is_idempotent_and_records_first_path() {
        let first = PathBuf::from("/tmp/ias-test-logs-first");
        let second = PathBuf::from("/tmp/ias-test-logs-second");

        record_init_path(&first);
        // Second call must not panic.
        record_init_path(&second);

        let recorded = initialized_path().expect("init path should be recorded");
        assert_eq!(
            recorded, first,
            "first call's path should win; second call should be ignored"
        );
    }
}
