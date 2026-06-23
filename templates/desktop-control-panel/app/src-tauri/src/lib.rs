//! IAS client library entry. Feature modules are declared here and filled in
//! by subsequent tasks per `planning/mvp-pilot/2026-05-20-ias-client.md`.

use std::sync::Mutex;

use tauri::Manager;

pub mod identity;
pub mod settings;
pub mod update;
pub mod storage;
pub mod shared;

use shared::config::RuntimeConfig;
use storage::Connection;

/// Application-wide state stored on the Tauri app via `manage`. Holds the
/// SQLite handle behind a `Mutex` for shared access from `#[tauri::command]`
/// bodies. One shared state is simpler to reason about than several
/// separately-`manage()`'d structs as the surface grows.
pub struct AppState {
    pub db: Mutex<Connection>,
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

      // Resolve the on-disk paths (db, log dir) and open the SQLite
      // connection. The connection runs PRAGMAs + migrations as part of
      // `new()`, so by the time it's `manage`'d the schema is current.
      let runtime_config = RuntimeConfig::from_app_handle(app.handle())
        .expect("RuntimeConfig should resolve at startup");

      let conn = Connection::new(&runtime_config.db_path)
        .expect("storage open should succeed");
      app.manage(AppState {
        db: Mutex::new(conn),
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      settings::commands::get_settings,
      settings::commands::set_theme,
      settings::commands::set_report_uploads_enabled,
      settings::commands::set_update_checks_enabled,
      update::commands::check_for_update,
      update::commands::apply_update,
      update::commands::get_app_version,
      storage::commands::clear_app_data,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
