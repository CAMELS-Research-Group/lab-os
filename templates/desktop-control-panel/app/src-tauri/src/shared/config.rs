//! IAS client configuration — build-time constants and runtime paths.
//!
//! Spec: ADD §3.1 ("Configuration files"), TRD §4.8 (Model URL + digest pinning).
//!
//! # [`BuildConfig`]
//!
//! Static, compile-time-injected constants for the URLs, version strings, and
//! cryptographic material that pin a release build to a specific backend +
//! model + updater key set. Values come from environment variables read at
//! build time via [`option_env!`] (with dev-friendly fallbacks). A
//! corresponding `cargo:rerun-if-env-changed=...` directive in [`build.rs`]
//! ensures these constants are rebuilt whenever the build environment changes.
//!
//! The seven env vars consumed by a release build script:
//!
//! - `IAS_BACKEND_URL` — IAS backend service base URL
//! - `IAS_MODEL_URL` — ONNX model download URL
//! - `IAS_MODEL_SHA256` — hex-encoded SHA-256 of the pinned model file
//! - `IAS_MODEL_VERSION` — semver-ish identifier for the pinned model
//! - `IAS_APP_VERSION` — client app version (defaults to `CARGO_PKG_VERSION`)
//! - `IAS_UPDATER_PUBKEY` — Tauri updater public key (minisign / TUF)
//! - `IAS_UPDATER_MANIFEST_URL` — updater manifest endpoint
//!
//! ## Dev-build behavior
//!
//! When the env vars are unset (typical local `cargo build`):
//!
//! - `BACKEND_URL` points at `https://localhost:8000` (the dev sidecar).
//! - `MODEL_URL` and `UPDATER_MANIFEST_URL` use the `example.invalid` TLD
//!   reserved by RFC 6761 — they will not resolve, so the dev build cannot
//!   accidentally talk to a real production endpoint.
//! - `MODEL_SHA256` is the all-zero 64-char string. Any real `.onnx` file
//!   loaded at runtime will fail the digest check — **this is intentional**.
//!   A dev build that wants to exercise the real model-load path must set
//!   `IAS_MODEL_SHA256` explicitly.
//! - `UPDATER_PUBKEY` is empty; CL-23 will treat that as "updates disabled".
//!
//! # [`RuntimeConfig`]
//!
//! Derived at app start from a [`tauri::AppHandle`] (production) or a
//! [`PathSource`] (tests). Resolves the on-disk paths the client writes to:
//! the SQLite database, the model cache directory, and the log directory.

use std::path::PathBuf;

use crate::shared::error::AppError;

// ---------------------------------------------------------------------------
// BuildConfig
// ---------------------------------------------------------------------------

/// All-zero 64-char SHA-256 digest used as the dev-build sentinel for
/// `IAS_MODEL_SHA256`. Any real model will fail this digest check at load
/// time — that's the point. Production builds set `IAS_MODEL_SHA256` to the
/// real digest of the released model artifact.
const DEV_MODEL_SHA256: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Build-time pinned constants for the IAS client.
///
/// See module docs for the env-var surface and dev-build behavior.
pub struct BuildConfig;

impl BuildConfig {
    /// IAS backend base URL.
    pub const BACKEND_URL: &'static str =
        match option_env!("IAS_BACKEND_URL") {
            Some(v) => v,
            None => "https://localhost:8000",
        };

    /// ONNX model download URL.
    pub const MODEL_URL: &'static str =
        match option_env!("IAS_MODEL_URL") {
            Some(v) => v,
            None => "https://example.invalid/ias-model-placeholder.onnx",
        };

    /// Hex-encoded SHA-256 of the pinned model file. The runtime model loader
    /// refuses to load a file whose digest doesn't match this value — that's
    /// what makes the client refuse a swapped model.
    pub const MODEL_SHA256: &'static str =
        match option_env!("IAS_MODEL_SHA256") {
            Some(v) => v,
            None => DEV_MODEL_SHA256,
        };

    /// Semver-ish identifier for the pinned model.
    pub const MODEL_VERSION: &'static str =
        match option_env!("IAS_MODEL_VERSION") {
            Some(v) => v,
            None => "0.0.0-dev",
        };

    /// Client app version. Cargo always sets `CARGO_PKG_VERSION`, so this can
    /// safely fall back to it via `env!`.
    pub const APP_VERSION: &'static str =
        match option_env!("IAS_APP_VERSION") {
            Some(v) => v,
            None => env!("CARGO_PKG_VERSION"),
        };

    /// Tauri updater public key. Empty in dev → updates disabled (per CL-23).
    pub const UPDATER_PUBKEY: &'static str =
        match option_env!("IAS_UPDATER_PUBKEY") {
            Some(v) => v,
            None => "",
        };

    /// Updater manifest endpoint.
    pub const UPDATER_MANIFEST_URL: &'static str =
        match option_env!("IAS_UPDATER_MANIFEST_URL") {
            Some(v) => v,
            None => "https://example.invalid/updates.json",
        };

    /// On-disk cache filename for the downloaded model.
    ///
    /// Derived from the terminal path segment of [`MODEL_URL`] (OQ2 decision,
    /// 2026-06-08): the file the CL-24 download writes and the file
    /// `evaluation::orchestrator::resolve_model_path` looks for are the same
    /// name *by construction*, so a model-version bump (which changes
    /// `IAS_MODEL_URL`) needs no lockstep code edit. Falls back to a stable
    /// default if the URL has no usable segment (defensive — a real
    /// `IAS_MODEL_URL` always ends in `…/<file>.onnx`).
    pub fn model_filename() -> &'static str {
        let seg = terminal_path_segment(Self::MODEL_URL);
        if seg.is_empty() {
            "ias-model.onnx"
        } else {
            seg
        }
    }
}

/// Last path segment of a URL, with any `?query` / `#fragment` stripped.
/// Pure + lifetime-preserving so [`BuildConfig::model_filename`] can hand back
/// a `&'static str` and unit tests can cover the parsing without touching the
/// compile-time const.
fn terminal_path_segment(url: &str) -> &str {
    match url.rsplit('/').next() {
        Some(seg) => match seg.split(['?', '#']).next() {
            Some(name) => name,
            None => seg,
        },
        None => "",
    }
}

// ---------------------------------------------------------------------------
// RuntimeConfig
// ---------------------------------------------------------------------------

/// Filenames + subdir names beneath the OS app-data dir. Kept as `const`s so
/// the tests can refer to them by name rather than re-spelling literals.
const DB_FILENAME: &str = "ias.db";
const MODEL_CACHE_SUBDIR: &str = "models";

/// Abstraction over the three Tauri-provided directory lookups
/// [`RuntimeConfig`] needs. Production uses the [`tauri::AppHandle`] impl;
/// tests use a [`tempfile::TempDir`]-backed fake.
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
/// - `ias.db` — SQLite database (see CL-6).
/// - `models/` — model cache directory (see CL-14).
///
/// The log directory is whatever Tauri's `app_log_dir()` returns — on
/// Windows that's `%LOCALAPPDATA%\<bundle_identifier>\logs`; on macOS it's
/// `~/Library/Logs/<bundle_identifier>`; on Linux it's
/// `~/.local/share/<bundle_identifier>/logs`.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub db_path: PathBuf,
    pub model_cache_dir: PathBuf,
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
            model_cache_dir: data_root.join(MODEL_CACHE_SUBDIR),
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
        // Smoke test: the constants compile + link and are non-empty in a
        // dev build. Production-env-injected values will be covered when the
        // release-build automation lands.
        assert!(!BuildConfig::BACKEND_URL.is_empty());
        assert!(!BuildConfig::MODEL_URL.is_empty());
        assert!(!BuildConfig::MODEL_SHA256.is_empty());
        assert!(!BuildConfig::MODEL_VERSION.is_empty());
        assert!(!BuildConfig::APP_VERSION.is_empty());
        // UPDATER_PUBKEY is intentionally empty in dev — assert it's the
        // empty string rather than non-empty.
        assert_eq!(BuildConfig::UPDATER_PUBKEY, "");
        assert!(!BuildConfig::UPDATER_MANIFEST_URL.is_empty());
    }

    #[test]
    fn build_config_dev_model_sha256_is_64_zeros() {
        // The dev sentinel is what guarantees the runtime digest check fails
        // on any real model — keep that invariant under test so a future
        // edit to the literal can't silently weaken it.
        assert_eq!(BuildConfig::MODEL_SHA256.len(), 64);
        assert!(BuildConfig::MODEL_SHA256.chars().all(|c| c == '0'));
    }

    #[test]
    fn build_config_dev_model_version_is_pinned_sentinel() {
        // Dev fallback for MODEL_VERSION is a deliberate sentinel surfaced
        // when a release build forgets to set IAS_MODEL_VERSION. Don't drift.
        assert_eq!(BuildConfig::MODEL_VERSION, "0.0.0-dev");
    }

    #[test]
    fn terminal_path_segment_extracts_filename() {
        assert_eq!(
            terminal_path_segment(
                "https://github.com/Pace-IAS/pronounce/releases/download/model-v0.1.0/ias-model-0.1.0.onnx"
            ),
            "ias-model-0.1.0.onnx"
        );
    }

    #[test]
    fn terminal_path_segment_strips_query_and_fragment() {
        assert_eq!(
            terminal_path_segment("https://cdn.example/ias-model-0.2.0.onnx?token=abc"),
            "ias-model-0.2.0.onnx"
        );
        assert_eq!(
            terminal_path_segment("https://cdn.example/ias-model-0.2.0.onnx#sha"),
            "ias-model-0.2.0.onnx"
        );
    }

    #[test]
    fn terminal_path_segment_empty_for_trailing_slash() {
        // A URL ending in `/` has no filename segment — model_filename()
        // falls back rather than producing an empty path component.
        assert_eq!(terminal_path_segment("https://cdn.example/models/"), "");
    }

    #[test]
    fn model_filename_matches_model_url_terminal_segment() {
        // The invariant OQ2 is protecting: the cache filename is exactly the
        // last segment of MODEL_URL, so download-target == resolve-path. The
        // dev fallback URL ends in `ias-model-placeholder.onnx`.
        assert_eq!(
            BuildConfig::model_filename(),
            terminal_path_segment(BuildConfig::MODEL_URL)
        );
        assert!(!BuildConfig::model_filename().is_empty());
    }

    #[test]
    fn build_config_dev_urls_use_invalid_tld() {
        // Dev fallback URLs must use the `.invalid` TLD (RFC 6761) so that a
        // misconfigured dev build cannot accidentally reach a real endpoint.
        assert!(
            BuildConfig::MODEL_URL.contains("example.invalid"),
            "MODEL_URL dev fallback should use example.invalid TLD"
        );
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
    fn runtime_config_model_cache_dir_is_under_data_dir() {
        let root = TempDir::new().unwrap();
        let src = FakePathSource::under(&root);
        let cfg = RuntimeConfig::from_path_source(&src).unwrap();

        assert!(
            cfg.model_cache_dir.starts_with(&src.data_dir),
            "model_cache_dir {:?} should be under data_dir {:?}",
            cfg.model_cache_dir,
            src.data_dir
        );
        assert_eq!(
            cfg.model_cache_dir.file_name().unwrap(),
            MODEL_CACHE_SUBDIR
        );
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
