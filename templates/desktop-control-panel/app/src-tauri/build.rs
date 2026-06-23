fn main() {
    // Rebuild whenever a build-time env var that feeds `BuildConfig`
    // (src/shared/config.rs) changes. These are all OPTIONAL — unset, the app
    // builds clean with auto-update effectively disabled (see config.rs and the
    // app README). A fork that wants auto-update sets them after forking.
    println!("cargo:rerun-if-env-changed=APP_VERSION");
    println!("cargo:rerun-if-env-changed=UPDATER_PUBKEY");
    println!("cargo:rerun-if-env-changed=UPDATER_MANIFEST_URL");
    println!("cargo:rerun-if-env-changed=RELEASE_DOWNLOAD_PAGE_URL");

    tauri_build::build()
}
