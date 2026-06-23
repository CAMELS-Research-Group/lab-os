//! App configuration — build-time constants and runtime paths.
//!
//! # [`BuildConfig`]
//!
//! Static, compile-time-injected constants for the version string, updater
//! manifest endpoint, and updater public key. Values come from environment
//! variables read at build time via [`option_env!`] (with dev-friendly
//! fallbacks). A corresponding `cargo:rerun-if-env-changed=...` directive in
//! [`build.rs`] ensures these constants are rebuilt whenever the build
//! environment changes.
//!
//! The env vars consumed by a release build script:
//!
//! - `APP_VERSION` — client app version (defaults to `CARGO_PKG_VERSION`)
//! - `UPDATER_PUBKEY` — updater public key (minisign / TUF), if the updater
//!   path uses one
//! - `UPDATER_MANIFEST_URL` — updater manifest endpoint
//!
//! ## Dev-build behavior (and white-label default)
//!
//! Auto-update is OPTIONAL and configure-post-fork. When the env vars are unset
//! — the default for a fresh fork and for a typical local `cargo build`:
//!
//! - `UPDATER_MANIFEST_URL` uses the `example.invalid` TLD reserved by RFC 6761.
//!   It will not resolve, so the build cannot accidentally talk to a real
//!   endpoint. The update check is additionally gated behind an opt-in setting
//!   (off by default), so with no override the app never makes a network call
//!   and the updater is effectively disabled — the build is clean with NO
//!   required release-host value.
//! - `UPDATER_PUBKEY` is empty → updates disabled.
//!
//! A fork that wants auto-update sets `UPDATER_MANIFEST_URL` (and the release
//! download page — see `update::commands`) to its own host AFTER forking. See
//! the app README.
//!
//! # [`RuntimeConfig`]
//!
//! Derived at app start from a [`tauri::AppHandle`] (production) or a
//! [`PathSource`] (tests). Resolves the on-disk paths the client writes to:
//! the SQLite database and the log directory.

use std::path::PathBuf;

use crate::shared::error::AppError;

// ---------------------------------------------------------------------------
// BuildConfig
// ---------------------------------------------------------------------------

/// Build-time constants for the app.
///
/// See module docs for the env-var surface and dev-build behavior.
pub struct BuildConfig;

impl BuildConfig {
    /// Client app version. Cargo always sets `CARGO_PKG_VERSION`, so this can
    /// safely fall back to it via `env!`.
    pub const APP_VERSION: &'static str =
        match option_env!("APP_VERSION") {
            Some(v) => v,
            None => env!("CARGO_PKG_VERSION"),
        };

    /// Updater public key. Empty in dev / unconfigured → updates disabled.
    pub const UPDATER_PUBKEY: &'static str =
        match option_env!("UPDATER_PUBKEY") {
            Some(v) => v,
            None => "",
        };

    /// Updater manifest endpoint.
    ///
    /// OPTIONAL / configure-post-fork: unset → an unresolvable `.invalid`
    /// sentinel (RFC 6761), so a fresh fork builds clean and the update check
    /// reports "no update" without reaching a real host. A fork that wants
    /// auto-update overrides `UPDATER_MANIFEST_URL` with its own endpoint.
    pub const UPDATER_MANIFEST_URL: &'static str =
        match option_env!("UPDATER_MANIFEST_URL") {
            Some(v) => v,
            None => "https://example.invalid/updates.json",
        };
}

// ---------------------------------------------------------------------------
// RuntimeConfig
// ---------------------------------------------------------------------------

/// Filename beneath the OS app-data dir. Kept as a `const` so the tests can
/// refer to it by name rather than re-spelling the literal.
///
/// Placeholder name — rename after forking if a product-specific DB filename
/// is preferred (purely cosmetic; nothing external depends on it).
const DB_FILENAME: &str = "app.db";

/// Abstraction over the two Tauri-provided directory lookups [`RuntimeConfig`]
/// needs. Production uses the [`tauri::AppHandle`] impl; tests use a
/// [`tempfile::TempDir`]-backed fake.
///
/// The trait exists purely to make `RuntimeConfig` unit-testable without
/// constructing a real Tauri app — `tauri::AppHandle` is non-trivial to mock.
pub(crate) trait PathSource {
    /// `<data_dir>/<bundle_identifier>` on every platform.
    fn app_data_dir(&self) -> Result<PathBuf, ConfigError>;
    /// Platform-specific log directory (see `tauri::path::PathResolver::app_log_dir`).
    fn app_log_dir(&self) -> Result<PathBuf, ConfigError>;
}

/// Errors produced while resolving runtime paths. Converts into
/// [`AppError::Config`] when bubbled out of a Tauri command.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required path lookup (app-data or log directory) failed. The
    /// `which` field names the lookup; `detail` carries the underlying
    /// Tauri error rendered as a string (the concrete `tauri::Error` type
    /// is not part of this module's public surface — and `thiserror`
    /// treats a `source` field as an `Error` source, which `String` is not).
    #[error("path lookup failed for {which}: {detail}")]
    PathLookup {
        which: &'static str,
        detail: String,
    },
}

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::Config(err.to_string())
    }
}

/// Runtime paths derived from the OS app-data directory.
///
/// Layout under `<app_data_dir>`:
///
/// - `app.db` — SQLite database.
///
/// The log directory is whatever Tauri's `app_log_dir()` returns — on
/// Windows that's `%LOCALAPPDATA%\<bundle_identifier>\logs`; on macOS it's
/// `~/Library/Logs/<bundle_identifier>`; on Linux it's
/// `~/.local/share/<bundle_identifier>/logs`.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub db_path: PathBuf,
    pub log_dir: PathBuf,
}

impl RuntimeConfig {
    /// Resolve runtime paths from a Tauri [`AppHandle`].
    ///
    /// This is the production entry point. Tests should use
    /// [`RuntimeConfig::from_path_source`] with a fake.
    pub fn from_app_handle(app: &tauri::AppHandle) -> Result<Self, ConfigError> {
        Self::from_path_source(&TauriPathSource { app })
    }

    /// Resolve runtime paths from any [`PathSource`]. Used by both the
    /// production [`from_app_handle`] path and the unit tests.
    pub(crate) fn from_path_source<S: PathSource>(source: &S) -> Result<Self, ConfigError> {
        let data_root = source.app_data_dir()?;
        let log_dir = source.app_log_dir()?;

        Ok(Self {
            db_path: data_root.join(DB_FILENAME),
            log_dir,
        })
    }
}

/// `PathSource` impl that defers to `tauri::Manager::path()`.
struct TauriPathSource<'a> {
    app: &'a tauri::AppHandle,
}

impl<'a> PathSource for TauriPathSource<'a> {
    fn app_data_dir(&self) -> Result<PathBuf, ConfigError> {
        use tauri::Manager as _;
        self.app
            .path()
            .app_data_dir()
            .map_err(|e| ConfigError::PathLookup {
                which: "app_data_dir",
                detail: e.to_string(),
            })
    }

    fn app_log_dir(&self) -> Result<PathBuf, ConfigError> {
        use tauri::Manager as _;
        self.app
            .path()
            .app_log_dir()
            .map_err(|e| ConfigError::PathLookup {
                which: "app_log_dir",
                detail: e.to_string(),
            })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ---- BuildConfig ------------------------------------------------------

    #[test]
    fn build_config_constants_are_accessible() {
        // Smoke test: the surviving constants compile + link and are present in
        // a dev build. Production-env-injected values will be covered when the
        // release-build automation lands.
        assert!(!BuildConfig::APP_VERSION.is_empty());
        // UPDATER_PUBKEY is intentionally empty in dev / unconfigured — assert
        // it's the empty string rather than non-empty.
        assert_eq!(BuildConfig::UPDATER_PUBKEY, "");
        assert!(!BuildConfig::UPDATER_MANIFEST_URL.is_empty());
    }

    #[test]
    fn build_config_dev_updater_url_uses_invalid_tld() {
        // The dev / unconfigured fallback must use the `.invalid` TLD (RFC 6761)
        // so a fresh fork cannot accidentally reach a real endpoint and the
        // updater is effectively disabled with no required host value.
        assert!(
            BuildConfig::UPDATER_MANIFEST_URL.contains("example.invalid"),
            "UPDATER_MANIFEST_URL dev fallback should use example.invalid TLD"
        );
    }

    // ---- RuntimeConfig ----------------------------------------------------

    /// `PathSource` backed by a `TempDir`. The data dir and log dir are two
    /// separate subdirectories of the temp root, mirroring the production
    /// shape where they are platform-distinct paths.
    struct FakePathSource {
        data_dir: PathBuf,
        log_dir: PathBuf,
    }

    impl FakePathSource {
        fn under(root: &TempDir) -> Self {
            let data_dir = root.path().join("data");
            let log_dir = root.path().join("logs");
            std::fs::create_dir_all(&data_dir).unwrap();
            std::fs::create_dir_all(&log_dir).unwrap();
            Self { data_dir, log_dir }
        }
    }

    impl PathSource for FakePathSource {
        fn app_data_dir(&self) -> Result<PathBuf, ConfigError> {
            Ok(self.data_dir.clone())
        }
        fn app_log_dir(&self) -> Result<PathBuf, ConfigError> {
            Ok(self.log_dir.clone())
        }
    }

    /// `PathSource` that always errors. Used to verify error propagation.
    struct FailingPathSource;

    impl PathSource for FailingPathSource {
        fn app_data_dir(&self) -> Result<PathBuf, ConfigError> {
            Err(ConfigError::PathLookup {
                which: "app_data_dir",
                detail: "synthetic failure".into(),
            })
        }
        fn app_log_dir(&self) -> Result<PathBuf, ConfigError> {
            Err(ConfigError::PathLookup {
                which: "app_log_dir",
                detail: "synthetic failure".into(),
            })
        }
    }

    #[test]
    fn runtime_config_db_path_is_under_data_dir() {
        let root = TempDir::new().unwrap();
        let src = FakePathSource::under(&root);
        let cfg = RuntimeConfig::from_path_source(&src).unwrap();

        assert!(
            cfg.db_path.starts_with(&src.data_dir),
            "db_path {:?} should be under data_dir {:?}",
            cfg.db_path,
            src.data_dir
        );
        assert_eq!(cfg.db_path.file_name().unwrap(), DB_FILENAME);
    }

    #[test]
    fn runtime_config_log_dir_matches_source() {
        let root = TempDir::new().unwrap();
        let src = FakePathSource::under(&root);
        let cfg = RuntimeConfig::from_path_source(&src).unwrap();

        assert_eq!(cfg.log_dir, src.log_dir);
    }

    #[test]
    fn runtime_config_propagates_path_source_errors() {
        let err = RuntimeConfig::from_path_source(&FailingPathSource).unwrap_err();
        match err {
            ConfigError::PathLookup { which, .. } => {
                assert_eq!(which, "app_data_dir");
            }
        }
    }

    #[test]
    fn config_error_converts_into_app_error_config_variant() {
        let err: AppError = ConfigError::PathLookup {
            which: "app_data_dir",
            detail: "x".into(),
        }
        .into();
        match err {
            AppError::Config(msg) => {
                assert!(msg.contains("app_data_dir"));
                assert!(msg.contains("x"));
            }
            other => panic!("expected AppError::Config, got {:?}", other),
        }
    }
}
