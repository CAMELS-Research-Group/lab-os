//! IAS client library entry. Feature modules are declared here and filled in
//! by subsequent tasks per `planning/mvp-pilot/2026-05-20-ias-client.md`.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tauri::Manager;

pub mod recording;
pub mod evaluation;
pub mod reporting;
pub mod identity;
pub mod settings;
pub mod update;
pub mod storage;
pub mod shared;

use recording::SessionLifecycle;
use shared::config::RuntimeConfig;
use storage::Connection;

/// Application-wide state stored on the Tauri app via `manage`. Holds the
/// SQLite handle (CL-6) and the recording session lifecycle (CL-13) behind
/// `Mutex`es for shared access from `#[tauri::command]` bodies. Later feature
/// modules (CL-19 evaluation orchestrator) will add their own fields here
/// rather than registering separate manage()'d structs — one shared state is
/// simpler to reason about as the surface grows.
///
/// The `lifecycle` mutex also serves a threading-safety role: `CpalAdapter`
/// holds a `cpal::Stream` that is `!Send` on some platforms (e.g. macOS)
/// while a stream is alive. Tauri serializes command invocations onto its
/// async-runtime workers, so as long as every command body locks, mutates,
/// and unlocks on the same worker thread, the lifecycle's `!Send` adapter
/// never crosses thread boundaries. The mutex guards against any future
/// caller violating that invariant.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub lifecycle: Mutex<SessionLifecycle>,
    /// Re-entrancy guard for the CL-24 first-run model download. Set true while
    /// a download is in flight so `start_first_run_model_download` rejects a
    /// second concurrent invocation (two writers onto the same `*.part`). An
    /// `Arc` so the spawned download task can clear it on completion without
    /// holding a non-`'static` `State` reference.
    pub model_download_in_progress: Arc<AtomicBool>,
}

/// Dev override pointing at an ONNXRuntime dynamic library. When set it wins
/// over the bundled copy, so a developer can run against any local build
/// without touching the installer. Installed builds leave it unset and resolve
/// the DLL vendored into the Tauri resource dir instead (see
/// [`BUNDLED_ORT_DYLIB_REL`]). There is intentionally no hardcoded path
/// fallback — the prior `mx/.venv/...` literal pointed at a worktree path that
/// no longer exists on every dev box, masking the config problem behind a quiet
/// runtime failure on first inference. Resolution failures are logged, not
/// panicked.
const ORT_DYLIB_ENV: &str = "IAS_ORT_DYLIB_PATH";

/// Relative path (under the Tauri resource dir) of the bundled ONNXRuntime
/// loadable library. Mirrors the per-platform `bundle.resources` mapping
/// (`tauri.windows.conf.json` / `tauri.macos.conf.json`). The binary itself is
/// operator-placed and gitignored (>5 MB); the installer copies it into the
/// resource dir from here so a clean install resolves it with no env var.
/// Matched to the `ort` `api-24` pin (ONNX Runtime >=1.24, Cargo.toml).
#[cfg(target_os = "windows")]
const BUNDLED_ORT_DYLIB_REL: &str = "resources/onnxruntime/onnxruntime.dll";
#[cfg(target_os = "macos")]
const BUNDLED_ORT_DYLIB_REL: &str = "resources/onnxruntime/libonnxruntime.dylib";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const BUNDLED_ORT_DYLIB_REL: &str = "resources/onnxruntime/libonnxruntime.so";

/// Resolve the ONNXRuntime DLL path. `IAS_ORT_DYLIB_PATH` wins when it names an
/// existing file (dev override); otherwise an installed build falls back to the
/// DLL vendored into the Tauri resource dir (Task 1). Returns `None`, with a
/// logged reason, when neither is reachable so the caller logs-not-panics.
fn resolve_ort_dylib(app: &tauri::AppHandle) -> Option<PathBuf> {
    // Dev override: an explicit env-var path takes precedence over the bundle.
    if let Ok(v) = std::env::var(ORT_DYLIB_ENV) {
        let p = PathBuf::from(v);
        if p.exists() {
            return Some(p);
        }
        log::error!(
            "{} is set but no DLL exists at {:?}; falling back to the bundled runtime.",
            ORT_DYLIB_ENV,
            p
        );
    }
    // Bundled resource: the operator-vendored DLL shipped inside the installer.
    match app
        .path()
        .resolve(BUNDLED_ORT_DYLIB_REL, tauri::path::BaseDirectory::Resource)
    {
        Ok(p) if p.exists() => Some(p),
        Ok(p) => {
            log::error!(
                "ort::init_from skipped: no DLL at the bundled resource path {:?} \
                 and {} is unset/invalid. Inference will fail with RuntimeFailure \
                 on first eval.",
                p,
                ORT_DYLIB_ENV
            );
            None
        }
        Err(e) => {
            log::error!(
                "ort::init_from skipped: could not resolve the bundled runtime \
                 resource {}: {e}. Set {} to an `onnxruntime.dll` for dev runs.",
                BUNDLED_ORT_DYLIB_REL,
                ORT_DYLIB_ENV
            );
            None
        }
    }
}

/// Best-effort initialization of the ONNXRuntime dynamic library. The crate
/// is configured with `ort = "...load-dynamic..."`, so any `Session::builder`
/// call requires this to have run first (per `ort::init_from(...).commit()`).
///
/// On failure we log and return — the orchestrator's
/// [`crate::evaluation::OnnxPhonemizer::load`] will surface the missing init
/// as [`crate::evaluation::EvaluationError::RuntimeFailure`] which becomes
/// an `eval:error` to the UI. NEVER panic here — a dev box without the DLL
/// path set should still launch the app and let the user see the failure
/// surface through to Results.
fn init_ort_runtime(app: &tauri::AppHandle) {
    let dylib_path = match resolve_ort_dylib(app) {
        Some(p) => p,
        None => return,
    };
    // ort::init_from returns Result<EnvironmentBuilder, OrtError>; .commit()
    // installs the global environment and returns a `bool` (true on first
    // commit). Mirrors the spike's call shape at spike/rust-poc/src/main.rs:96.
    let builder = match ort::init_from(dylib_path.to_string_lossy().as_ref()) {
        Ok(b) => b,
        Err(e) => {
            log::error!(
                "ort::init_from failed for {:?}: {e}. \
                 Inference will fail until the DLL is reachable.",
                dylib_path
            );
            return;
        }
    };
    let _first_commit = builder.commit();
    log::info!("ort: dynamic library initialised from {:?}", dylib_path);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      // Resolve the log directory up front; tauri-plugin-log needs it eagerly.
      let app_log_dir = app
        .path()
        .app_log_dir()
        .expect("app_log_dir resolution failed");

      // Attach the log plugin FIRST so every subsequent log line in setup is
      // captured. Anything logged before this point fires against an
      // uninstalled global logger and is silently dropped — that previously
      // hid every `init_ort_runtime` line, masking config errors as a
      // generic eval:error downstream.
      app.handle().plugin(shared::log::init::<tauri::Wry>(&app_log_dir))?;

      // Initialise the ONNXRuntime DLL before any Session::builder call.
      // Required because the `ort` crate is built with `load-dynamic`
      // (see Cargo.toml comment). Init failure is logged, not panicked —
      // the orchestrator surfaces a RuntimeFailure on first inference call.
      init_ort_runtime(app.handle());

      // Resolve the on-disk paths (db, model cache, log dir) and open the
      // SQLite connection. The connection runs PRAGMAs + migrations as part
      // of `new()`, so by the time it's `manage`'d the schema is current.
      //
      // TODO(CL-8): the identity module's first-run setup will also need
      //             access to this AppState (to create the install_identity
      //             row during accept_consent). Chain that initialization
      //             after manage() lands so it can take a State<'_, AppState>.
      let runtime_config = RuntimeConfig::from_app_handle(app.handle())
        .expect("RuntimeConfig should resolve at startup");

      // Ensure the model cache directory exists before the orchestrator
      // tries to resolve a model path under it. CL-24 landed — the model
      // auto-downloads from IAS_MODEL_URL on first launch into this directory.
      // Creating it up front makes the download step a single
      // write rather than mkdir + write, and matches Connection::new's
      // parent-dir create behaviour. Logged-not-panicked: a read-only volume
      // or AV quarantine should not kill the app — the orchestrator will
      // surface ModelNotFound on first eval with a user-visible error.
      if let Err(e) = std::fs::create_dir_all(&runtime_config.model_cache_dir) {
        log::error!(
          "model cache dir create failed at {:?}: {e}. \
           Inference will fail with ModelNotFound on first eval until the \
           directory is reachable.",
          runtime_config.model_cache_dir
        );
      }

      let conn = Connection::new(&runtime_config.db_path)
        .expect("storage open should succeed");
      app.manage(AppState {
        db: Mutex::new(conn),
        lifecycle: Mutex::new(SessionLifecycle::new()),
        model_download_in_progress: Arc::new(AtomicBool::new(false)),
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      settings::commands::get_settings,
      settings::commands::set_l1,
      settings::commands::set_difficulty,
      settings::commands::set_report_uploads_enabled,
      settings::commands::set_update_checks_enabled,
      recording::commands::start_session,
      recording::commands::pause_session,
      recording::commands::resume_session,
      recording::commands::cancel_session,
      recording::commands::end_session,
      evaluation::reference_ipa::get_passage,
      evaluation::commands::get_evaluation_result,
      evaluation::model_download::start_first_run_model_download,
      update::commands::check_for_update,
      update::commands::apply_update,
      update::commands::get_app_version,
      storage::commands::clear_session_data,
      storage::commands::get_session_history,
      storage::commands::get_phoneme_trends,
      reporting::commands::submit_feedback,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
