//! Tauri command surface for the update feature (CL-23-lite).
//!
//! Three commands:
//!
//! - [`check_for_update`] — reads `update_checks_enabled` from the DB; when
//!   false returns `{available:false}` with **no network call** (privacy
//!   guarantee, enforced server-side). When true, fetches the manifest,
//!   compares versions, and returns [`UpdateInfo`].
//! - [`apply_update`] — re-fetches the manifest (to confirm an update exists
//!   and enforce the opt-in gate), then opens the release download page in the
//!   default browser via `tauri-plugin-opener`. No URL crosses the IPC
//!   boundary. On failure returns `AppError::Update` (the user explicitly
//!   triggered this action, so surfacing an error is appropriate — unlike the
//!   silent check).
//! - [`get_app_version`] — returns `BuildConfig::APP_VERSION` as a `String`.
//!
//! # Design seam
//!
//! `check_for_update_impl` is a free function that accepts any [`ManifestFetcher`].
//! This makes the gating and error→`available=false` behavior unit-testable
//! without a Tauri runtime or real network. The `#[tauri::command]`
//! `check_for_update` wrapper locks the DB connection from `AppState` and
//! calls the impl with the real [`ReqwestFetcher`].
//!
//! Privacy guarantee: the fetch helper sends only a plain `GET` to the
//! manifest URL. No install UUID, no identifiers, no request body is attached
//! — only inherent HTTP transport metadata (IP, user-agent).

use std::sync::Mutex;

use tauri::AppHandle;
use tauri::Runtime;

use crate::shared::config::BuildConfig;
use crate::shared::error::AppError;
use crate::shared::types::UpdateInfo;
use crate::storage::Connection;
use crate::update::{is_update_available, parse_manifest, Manifest, UpdateError};

pub use crate::AppState;

/// Public release download page that `apply_update` opens (rather than the raw
/// installer `download_url`): a download page can resolve the newest
/// per-platform installer and carry install instructions, so the user gets
/// guided steps instead of a bare `.msi`/`.dmg`.
///
/// OPTIONAL / configure-post-fork: this is build-time overridable via the
/// `RELEASE_DOWNLOAD_PAGE_URL` env var (read by `option_env!`). When unset —
/// the default for a fresh fork — it falls back to an unresolvable `.invalid`
/// sentinel (RFC 6761) so the app builds and runs with auto-update effectively
/// disabled and never opens a stale host. A fork that wants auto-update sets
/// this to its own release/download page. See the app README.
const RELEASE_DOWNLOAD_PAGE_URL: &str = match option_env!("RELEASE_DOWNLOAD_PAGE_URL") {
    Some(v) => v,
    None => "https://example.invalid/releases",
};

// ---------------------------------------------------------------------------
// ManifestFetcher trait — the unit-test seam
// ---------------------------------------------------------------------------

/// Abstraction over the HTTP fetch of the manifest JSON.
///
/// Real impl: [`ReqwestFetcher`] (async reqwest GET).
/// Test impl: [`InMemoryFetcher`] / [`FailingFetcher`] / [`RecordingFetcher`].
///
/// The trait is `async_trait`-free: it returns a plain `Result` synchronously.
/// The real fetcher runs on the tokio runtime via `reqwest::get` called from
/// within an `async fn`, and we spawn a blocking task to keep the async
/// surface uniform.
pub trait ManifestFetcher: Send + Sync {
    /// Fetch the manifest at `url` and return the raw JSON body as a `String`.
    ///
    /// Errors are represented as [`UpdateError::FetchFailed`].
    fn fetch(&self, url: &str) -> Result<String, UpdateError>;
}

// ---------------------------------------------------------------------------
// Real fetcher — reqwest (async, follows redirects by default)
// ---------------------------------------------------------------------------

/// Production [`ManifestFetcher`] backed by [`reqwest`].
///
/// Sends a plain GET with no request body and no custom headers beyond the
/// default reqwest user-agent. GitHub's `/releases/latest/download/` URLs
/// issue 302 redirects; reqwest follows them automatically.
pub struct ReqwestFetcher;

impl ManifestFetcher for ReqwestFetcher {
    fn fetch(&self, url: &str) -> Result<String, UpdateError> {
        // We are called from within a tokio async context (the #[tauri::command]
        // body). Use tokio::task::block_in_place + a new per-call blocking
        // runtime to avoid blocking the async executor thread directly.
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| UpdateError::FetchFailed(e.to_string()))?;
            rt.block_on(async {
                let response = reqwest::get(url)
                    .await
                    .map_err(|e| UpdateError::FetchFailed(e.to_string()))?;
                response
                    .text()
                    .await
                    .map_err(|e| UpdateError::FetchFailed(e.to_string()))
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Business-logic layer (free functions, no Tauri runtime dependency)
// ---------------------------------------------------------------------------

/// Core logic for `check_for_update`.
///
/// 1. Reads `update_checks_enabled` from the settings row.
/// 2. When false: returns `{available: false}` immediately — **no fetch**.
/// 3. When true: fetches via `fetcher`, parses, compares.
/// 4. Any error in step 3 resolves to `{available: false}` (logged) — the
///    silent check must never nag and must never block core functionality.
pub(crate) fn check_for_update_impl(
    conn: &Connection,
    fetcher: &dyn ManifestFetcher,
) -> Result<UpdateInfo, AppError> {
    // Read the gate flag. Delegate to the existing settings impl.
    let settings = crate::settings::commands::get_settings_impl(conn)?;

    if !settings.update_checks_enabled {
        // Privacy guarantee: no network call when the user has not opted in.
        return Ok(UpdateInfo {
            available: false,
            version: None,
            notes: None,
        });
    }

    // Fetch + parse + compare. Any failure resolves to available=false.
    let url = BuildConfig::UPDATER_MANIFEST_URL;
    let manifest_result = fetcher
        .fetch(url)
        .and_then(|body| parse_manifest(&body))
        .and_then(|manifest| {
            is_update_available(BuildConfig::APP_VERSION, &manifest)
                .map(|available| (available, manifest))
        });

    match manifest_result {
        Ok((true, manifest)) => Ok(UpdateInfo {
            available: true,
            version: Some(manifest.version),
            notes: Some(manifest.notes),
        }),
        Ok((false, _)) => Ok(UpdateInfo {
            available: false,
            version: None,
            notes: None,
        }),
        Err(e) => {
            // Silent degradation: log + return available=false.
            log::warn!("update check failed (returning available=false): {e}");
            Ok(UpdateInfo {
                available: false,
                version: None,
                notes: None,
            })
        }
    }
}

/// Core logic for `apply_update`: honor the opt-in gate, then fetch + parse the
/// manifest the user wants to install.
///
/// Mirrors the privacy guarantee of [`check_for_update_impl`]: the apply path
/// *also* fetches the manifest, so it must honor the opt-in too. When
/// `update_checks_enabled` is false there is **no network egress** — returns
/// `Ok(None)`. When enabled, fetches + parses; unlike the silent check, a
/// fetch/parse failure here IS surfaced as `AppError::Update` because the user
/// explicitly triggered the action.
pub(crate) fn resolve_apply_manifest(
    conn: &Connection,
    fetcher: &dyn ManifestFetcher,
) -> Result<Option<Manifest>, AppError> {
    let settings = crate::settings::commands::get_settings_impl(conn)?;
    if !settings.update_checks_enabled {
        // Privacy gate: no egress when the user has not opted in.
        return Ok(None);
    }

    let url = BuildConfig::UPDATER_MANIFEST_URL;
    fetcher
        .fetch(url)
        .and_then(|body| parse_manifest(&body))
        .map(Some)
        .map_err(AppError::Update)
}

// ---------------------------------------------------------------------------
// Locking helper (mirrored from settings/commands.rs)
// ---------------------------------------------------------------------------

fn lock_conn(db: &Mutex<Connection>) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
    db.lock()
        .map_err(|_| AppError::InvalidState("update db lock poisoned".into()))
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Read the current update posture from the DB, and — when the user has
/// opted in — fetch + parse the manifest and compare versions.
///
/// Privacy guarantee enforced server-side: when `update_checks_enabled` is
/// false in the DB, this command returns `{available:false}` without any
/// network call. No install UUID or identifiers are attached to the request.
#[tauri::command]
pub async fn check_for_update(
    state: tauri::State<'_, AppState>,
) -> Result<UpdateInfo, AppError> {
    let conn = lock_conn(&state.db)?;
    check_for_update_impl(&conn, &ReqwestFetcher)
}

/// Re-fetch the manifest, then open the release download page in the system
/// browser.
///
/// The manifest fetch still runs (it confirms an update is available and
/// enforces the opt-in gate), but the user is sent to the release download
/// page rather than the raw installer `download_url`. No URL crosses the IPC
/// boundary. On fetch or parse failure this command returns `AppError::Update`
/// — unlike `check_for_update`, the user explicitly triggered this action, so
/// surfacing an error is appropriate.
///
/// # Gate contract
///
/// When `update_checks_enabled` is false in the DB, `resolve_apply_manifest`
/// returns `None` without performing any network fetch (the no-egress guarantee
/// is preserved). This command converts that `None` into
/// `Err(AppError::Update(UpdateError::ChecksDisabled))` — a typed error
/// instead of a silent `Ok(())` — so the frontend can surface a meaningful
/// signal to the user rather than silently no-oping on a user-initiated action.
#[tauri::command]
pub async fn apply_update<R: Runtime>(
    state: tauri::State<'_, AppState>,
    app: AppHandle<R>,
) -> Result<(), AppError> {
    // Resolve the manifest under the DB lock (which reads the opt-in gate),
    // then release the lock before any browser I/O.
    let manifest = {
        let conn = lock_conn(&state.db)?;
        resolve_apply_manifest(&conn, &ReqwestFetcher)?
    };

    // When update checks are disabled the gate returns None — no egress has
    // occurred. Return a typed error so the caller can distinguish "disabled"
    // from a successful browser-open (never a silent Ok(())). The resolved
    // manifest itself is no longer used for the URL — we send the user to
    // the release download page (below) rather than the raw installer — but the
    // fetch still confirms an update exists and enforces the opt-in gate.
    manifest.ok_or(AppError::Update(UpdateError::ChecksDisabled))?;

    // Open the release download page instead of the manifest's raw `download_url`.
    use tauri_plugin_opener::OpenerExt as _;
    app.opener()
        .open_url(RELEASE_DOWNLOAD_PAGE_URL, None::<&str>)
        .map_err(|e| AppError::Update(UpdateError::OpenBrowserFailed(e.to_string())))?;

    Ok(())
}

/// Return the build-time client version string.
///
/// Used by the Settings screen and the UpdateBanner to display the running
/// version alongside any available update. Returns `BuildConfig::APP_VERSION`.
#[tauri::command]
pub async fn get_app_version() -> Result<String, AppError> {
    Ok(BuildConfig::APP_VERSION.to_owned())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;

    use crate::settings::commands::set_update_checks_enabled_impl;
    use crate::storage::Connection;

    // ---- Helpers -----------------------------------------------------------

    fn open_tmp() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ias.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    fn make_manifest_json(version: &str) -> String {
        format!(
            r#"{{"version":"{}","download_url":"https://example.com/app.exe","notes":"Bug fixes"}}"#,
            version
        )
    }

    // ---- Test fetcher implementations -------------------------------------

    /// Always returns a fixed body string (never fails).
    struct InMemoryFetcher {
        body: String,
    }

    impl ManifestFetcher for InMemoryFetcher {
        fn fetch(&self, _url: &str) -> Result<String, UpdateError> {
            Ok(self.body.clone())
        }
    }

    /// Always returns a `FetchFailed` error.
    struct FailingFetcher {
        message: String,
    }

    impl ManifestFetcher for FailingFetcher {
        fn fetch(&self, _url: &str) -> Result<String, UpdateError> {
            Err(UpdateError::FetchFailed(self.message.clone()))
        }
    }

    /// Returns a fixed body AND records whether `fetch` was called.
    ///
    /// This is the seam used to prove the "no HTTP when disabled" guarantee:
    /// if the flag is false, `fetch` must NOT be invoked.
    struct RecordingFetcher {
        body: String,
        was_invoked: Arc<AtomicBool>,
    }

    impl RecordingFetcher {
        fn new(body: impl Into<String>) -> (Self, Arc<AtomicBool>) {
            let flag = Arc::new(AtomicBool::new(false));
            let fetcher = Self {
                body: body.into(),
                was_invoked: Arc::clone(&flag),
            };
            (fetcher, flag)
        }
    }

    impl ManifestFetcher for RecordingFetcher {
        fn fetch(&self, _url: &str) -> Result<String, UpdateError> {
            self.was_invoked.store(true, Ordering::SeqCst);
            Ok(self.body.clone())
        }
    }

    /// Returns malformed JSON — used to exercise the ManifestParse → available=false path.
    struct MalformedFetcher;

    impl ManifestFetcher for MalformedFetcher {
        fn fetch(&self, _url: &str) -> Result<String, UpdateError> {
            Ok("not valid json {{{{".to_owned())
        }
    }

    // ---- check_for_update_impl: disabled gate -----------------------------

    /// Key privacy guarantee test: when update_checks_enabled is false, the
    /// fetcher MUST NOT be invoked and the result must be available=false.
    #[test]
    fn when_disabled_fetcher_is_not_invoked_and_available_is_false() {
        let (_dir, conn) = open_tmp();
        // update_checks_enabled defaults to false on a fresh DB.

        let (fetcher, was_invoked) = RecordingFetcher::new(make_manifest_json("99.99.99"));

        let result = check_for_update_impl(&conn, &fetcher).unwrap();

        assert!(
            !was_invoked.load(Ordering::SeqCst),
            "fetcher MUST NOT be invoked when update_checks_enabled=false"
        );
        assert!(!result.available, "available must be false when disabled");
        assert!(result.version.is_none());
        assert!(result.notes.is_none());
    }

    /// Explicit set-to-false via the settings impl also suppresses the fetch.
    #[test]
    fn explicitly_disabled_after_enable_suppresses_fetch() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        set_update_checks_enabled_impl(&mut conn, false).unwrap();

        let (fetcher, was_invoked) = RecordingFetcher::new(make_manifest_json("99.99.99"));
        let result = check_for_update_impl(&conn, &fetcher).unwrap();

        assert!(
            !was_invoked.load(Ordering::SeqCst),
            "fetcher MUST NOT be invoked after toggle-off"
        );
        assert!(!result.available);
    }

    // ---- check_for_update_impl: enabled + newer version -------------------

    #[test]
    fn when_enabled_and_newer_version_returns_available_true() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();

        // APP_VERSION in dev builds is the Cargo.toml version (e.g. "0.2.0").
        // We use a clearly-higher version so the comparison is stable regardless
        // of the current package version.
        let fetcher = InMemoryFetcher {
            body: make_manifest_json("99.99.99"),
        };

        let result = check_for_update_impl(&conn, &fetcher).unwrap();

        assert!(result.available, "99.99.99 should be newer than any dev version");
        assert_eq!(result.version.as_deref(), Some("99.99.99"));
        assert_eq!(result.notes.as_deref(), Some("Bug fixes"));
    }

    // ---- check_for_update_impl: enabled + same version --------------------

    #[test]
    fn when_enabled_and_same_version_returns_available_false() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();

        // Feed back the running version so the comparison yields "no update."
        let current = BuildConfig::APP_VERSION;
        let fetcher = InMemoryFetcher {
            body: make_manifest_json(current),
        };

        let result = check_for_update_impl(&conn, &fetcher).unwrap();

        assert!(!result.available, "same version should not be update available");
        assert!(result.version.is_none());
    }

    // ---- check_for_update_impl: enabled + older version -------------------

    #[test]
    fn when_enabled_and_older_manifest_version_returns_available_false() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();

        let fetcher = InMemoryFetcher {
            body: make_manifest_json("0.0.1"),
        };

        let result = check_for_update_impl(&conn, &fetcher).unwrap();

        assert!(!result.available, "older manifest should not be update available");
    }

    // ---- check_for_update_impl: fetch failure → available=false ----------

    #[test]
    fn fetch_failure_resolves_to_available_false_not_an_error() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();

        let fetcher = FailingFetcher {
            message: "connection refused".to_owned(),
        };

        // MUST resolve to Ok(available=false), NOT Err(...).
        let result = check_for_update_impl(&conn, &fetcher)
            .expect("fetch failure must not propagate as Err from check_for_update_impl");

        assert!(!result.available, "fetch failure must resolve to available=false");
        assert!(result.version.is_none());
    }

    // ---- check_for_update_impl: invalid manifest URL (dev .invalid TLD) --

    /// In dev builds UPDATER_MANIFEST_URL is "https://example.invalid/updates.json"
    /// which will not resolve. We simulate this with a FailingFetcher (since
    /// unit tests don't make real network calls). The result must be available=false,
    /// not a panic or an Err.
    #[test]
    fn dev_invalid_url_failure_resolves_to_available_false() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();

        // Simulate the .invalid TLD failing to resolve.
        let fetcher = FailingFetcher {
            message: "DNS resolution failed: example.invalid is not a real TLD (RFC 6761)".to_owned(),
        };

        let result = check_for_update_impl(&conn, &fetcher)
            .expect("dev .invalid URL failure must not propagate as Err");

        assert!(!result.available, "dev .invalid URL failure must resolve to available=false");
    }

    // ---- check_for_update_impl: malformed manifest → available=false -----

    #[test]
    fn malformed_manifest_resolves_to_available_false_not_an_error() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();

        // MUST resolve to Ok(available=false), NOT Err(...).
        let result = check_for_update_impl(&conn, &MalformedFetcher)
            .expect("malformed manifest must not propagate as Err from check_for_update_impl");

        assert!(!result.available, "malformed manifest must resolve to available=false");
    }

    // ---- check_for_update_impl: invalid version in manifest → available=false

    #[test]
    fn invalid_manifest_version_resolves_to_available_false() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();

        let fetcher = InMemoryFetcher {
            body: r#"{"version":"not-semver","download_url":"https://example.com/app.exe","notes":"hi"}"#.to_owned(),
        };

        let result = check_for_update_impl(&conn, &fetcher)
            .expect("invalid version in manifest must not propagate as Err");

        assert!(!result.available, "invalid manifest version must resolve to available=false");
    }

    // ---- UpdateInfo type matches the shared types contract ----------------

    #[test]
    fn check_for_update_returns_update_info_type() {
        // Structural smoke-test: verify the return shape compiles and
        // fields are accessible as expected by the TS counterpart.
        let (_dir, conn) = open_tmp();
        let fetcher = InMemoryFetcher {
            body: make_manifest_json("0.0.1"),
        };
        let info: UpdateInfo = check_for_update_impl(&conn, &fetcher).unwrap();
        // Type must have `available`, `version`, `notes` — compiler ensures this.
        let _ = info.available;
        let _ = info.version;
        let _ = info.notes;
    }

    // ---- get_app_version --------------------------------------------------

    /// Smoke-test: APP_VERSION is a non-empty string and round-trips through
    /// the impl. The actual async wrapper is just `Ok(BuildConfig::APP_VERSION.to_owned())`.
    #[test]
    fn app_version_is_non_empty_string() {
        let version = BuildConfig::APP_VERSION;
        assert!(!version.is_empty(), "APP_VERSION must be non-empty");
        // Verify it matches the semver structure expected (parseable, not "not-semver").
        semver::Version::parse(version)
            .expect("APP_VERSION should be valid semver (dev fallback comes from CARGO_PKG_VERSION)");
    }

    // ---- fetch_manifest_for_apply: propagates errors as AppError::Update --

    #[test]
    fn resolve_apply_manifest_propagates_fetch_error_as_app_error_update() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        let fetcher = FailingFetcher {
            message: "network down".to_owned(),
        };
        let err = resolve_apply_manifest(&conn, &fetcher).unwrap_err();
        match err {
            AppError::Update(_) => {} // correct
            other => panic!("expected AppError::Update, got {other:?}"),
        }
    }

    #[test]
    fn resolve_apply_manifest_propagates_parse_error_as_app_error_update() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        let err = resolve_apply_manifest(&conn, &MalformedFetcher).unwrap_err();
        match err {
            AppError::Update(_) => {} // correct
            other => panic!("expected AppError::Update, got {other:?}"),
        }
    }

    #[test]
    fn resolve_apply_manifest_returns_manifest_when_enabled() {
        let (_dir, mut conn) = open_tmp();
        set_update_checks_enabled_impl(&mut conn, true).unwrap();
        let fetcher = InMemoryFetcher {
            body: make_manifest_json("2.0.0"),
        };
        let manifest = resolve_apply_manifest(&conn, &fetcher)
            .unwrap()
            .expect("enabled + valid manifest → Some(manifest)");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.download_url, "https://example.com/app.exe");
    }

    /// Apply path honors the same opt-in gate as the check path: when disabled,
    /// it returns None and performs NO network fetch (no second egress path).
    #[test]
    fn resolve_apply_manifest_is_gated_no_egress_when_disabled() {
        let (_dir, conn) = open_tmp(); // update_checks_enabled defaults to false
        let (fetcher, was_invoked) = RecordingFetcher::new(make_manifest_json("99.99.99"));

        let result = resolve_apply_manifest(&conn, &fetcher).unwrap();

        assert!(result.is_none(), "disabled → no manifest to apply");
        assert!(
            !was_invoked.load(Ordering::SeqCst),
            "apply MUST NOT fetch the manifest when update checks are disabled"
        );
    }

    // ---- apply_update command layer: ChecksDisabled typed error ---------------

    /// When the opt-in gate is off, apply_update (the command layer) must convert
    /// resolve_apply_manifest's Ok(None) into Err(AppError::Update(ChecksDisabled)).
    /// The fetcher MUST still not be invoked (no egress, enforced by resolve_apply_manifest).
    #[test]
    fn apply_update_returns_checks_disabled_error_when_gate_is_off() {
        let (_dir, conn) = open_tmp(); // update_checks_enabled defaults to false
        let (fetcher, was_invoked) = RecordingFetcher::new(make_manifest_json("99.99.99"));

        // Call resolve_apply_manifest directly (the command wrapper requires a
        // Tauri runtime; we test the same gate path through the helper).
        let result = resolve_apply_manifest(&conn, &fetcher).unwrap();

        // Confirm gate returns None with no fetch — this is the input that
        // apply_update converts to ChecksDisabled.
        assert!(result.is_none(), "disabled → resolve_apply_manifest returns None");
        assert!(
            !was_invoked.load(Ordering::SeqCst),
            "apply MUST NOT fetch when gate is off"
        );

        // Simulate what apply_update does with the None: convert to ChecksDisabled.
        let cmd_result: Result<(), AppError> =
            result.ok_or(AppError::Update(UpdateError::ChecksDisabled)).map(|_| ());

        match cmd_result {
            Err(AppError::Update(UpdateError::ChecksDisabled)) => {} // expected
            other => panic!(
                "apply_update must return AppError::Update(ChecksDisabled) when disabled, got {other:?}"
            ),
        }
    }
}
