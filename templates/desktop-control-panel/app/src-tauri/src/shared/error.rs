//! Cross-feature error type for the IAS client.
//!
//! Per ADD §3.10: each feature module owns its own `thiserror`-derived error
//! enum; cross-feature errors bubble up via the top-level [`AppError`]. Tauri
//! commands return [`AppError`] and the custom [`Serialize`] impl emits a
//! `{ kind, message, recoverable }` JSON object the React frontend switches on.
//!
//! The remaining sub-error types below (`BackendError`, `KeyringError`) are
//! **stubs** so the `#[from]` derives on `AppError` compile while those
//! feature modules are still empty. They are replaced by the real
//! feature-owned types in later CL-* tasks; see the per-stub `TODO(CL-N)`
//! markers. `StorageError` (CL-6), `MicrophoneError` (CL-12),
//! `EvaluationError` (CL-9a / CL-14), and `UpdateError` (CL-23) are already
//! real — they are re-exported here from their owning modules. The
//! `AppError::Inference` variant name is retained as the IPC-level kind
//! (`"inference"`) per CL-2's contract; the inner type is the broader
//! feature-level `EvaluationError`.

use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::storage::StorageError;
use crate::update::UpdateError;

// ---------------------------------------------------------------------------
// Sub-error stubs
//
// Each of these will be replaced by a feature-module-owned error type in a
// later task. Until then they exist as single-variant `thiserror` enums so
// that `AppError`'s `#[from]` conversions resolve.
// ---------------------------------------------------------------------------

// TODO(CL-10): replaced by reporting::BackendError when the backend client lands.
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("backend stub: {0}")]
    Placeholder(String),
}

// TODO(CL-7): replaced by identity::KeyringError when the keyring module lands.
#[derive(Debug, thiserror::Error)]
pub enum KeyringError {
    #[error("keyring stub: {0}")]
    Placeholder(String),
}

// ---------------------------------------------------------------------------
// AppError
// ---------------------------------------------------------------------------

/// Cross-feature error type for Tauri commands.
///
/// Serializes to `{ kind: string, message: string, recoverable: bool }`.
/// `kind` is the snake_case form of the variant name.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("backend error: {0}")]
    Backend(#[from] BackendError),

    #[error("keyring error: {0}")]
    Keyring(#[from] KeyringError),

    #[error("update error: {0}")]
    Update(#[from] UpdateError),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("config error: {0}")]
    Config(String),
}

impl AppError {
    /// Snake_case variant name used as the `kind` field in the Tauri payload.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            AppError::Storage(_) => "storage",
            AppError::Backend(_) => "backend",
            AppError::Keyring(_) => "keyring",
            AppError::Update(_) => "update",
            AppError::InvalidState(_) => "invalid_state",
            AppError::Config(_) => "config",
        }
    }

    /// Whether the frontend should treat the error as recoverable (retry /
    /// actionable toast) vs terminal (banner / modal / log). See the
    /// per-variant rationale below — these mirror ADD §3.10's "Frontend
    /// convention" guidance and the CL-2 plan acceptance bullet.
    pub(crate) fn recoverable(&self) -> bool {
        match self {
            // Data loss / corruption — terminal. All current StorageError
            // variants are programmer / data-integrity errors with no
            // retry semantics.
            AppError::Storage(_) => false,
            // Network is the typical cause — default to recoverable; the
            // inner BackendError will distinguish transient vs terminal once
            // it's fleshed out in CL-10.
            // TODO(CL-10): match into BackendError once real variants exist (transient vs terminal).
            AppError::Backend(_) => true,
            // Locked keychain — terminal; per ADD §3.9 the caller falls back
            // to read-only mode rather than retrying.
            // TODO(CL-7): match into KeyringError once real variants exist.
            AppError::Keyring(_) => false,
            // Next update check / retry can succeed. `FetchFailed` and
            // `ManifestParse` are transient; `InvalidVersion` is a build
            // misconfiguration (terminal in practice, but the banner can
            // dismiss and won't block core functionality). Keeping true
            // here to match the original contract; CL-23 can refine per
            // variant if a terminal case emerges in later tasks.
            AppError::Update(_) => true,
            // Programmer error per the CL-2 acceptance bullet.
            AppError::InvalidState(_) => false,
            // App is misconfigured — terminal.
            AppError::Config(_) => false,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 3)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("recoverable", &self.recoverable())?;
        s.end()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    /// Helper: build one of each `AppError` variant with a trivial inner value.
    fn all_variants() -> Vec<AppError> {
        vec![
            AppError::Storage(StorageError::InvalidState("x".into())),
            AppError::Backend(BackendError::Placeholder("x".into())),
            AppError::Keyring(KeyringError::Placeholder("x".into())),
            AppError::Update(UpdateError::FetchFailed("x".into())),
            AppError::InvalidState("x".into()),
            AppError::Config("x".into()),
        ]
    }

    fn to_json(err: &AppError) -> Value {
        serde_json::to_value(err).expect("AppError serializes to JSON")
    }

    #[test]
    fn kind_is_snake_case_variant_name_storage() {
        let err = AppError::Storage(StorageError::InvalidState("x".into()));
        assert_eq!(to_json(&err)["kind"], json!("storage"));
    }

    #[test]
    fn kind_is_snake_case_variant_name_backend() {
        let err = AppError::Backend(BackendError::Placeholder("x".into()));
        assert_eq!(to_json(&err)["kind"], json!("backend"));
    }

    #[test]
    fn kind_is_snake_case_variant_name_keyring() {
        let err = AppError::Keyring(KeyringError::Placeholder("x".into()));
        assert_eq!(to_json(&err)["kind"], json!("keyring"));
    }

    #[test]
    fn kind_is_snake_case_variant_name_update() {
        let err = AppError::Update(UpdateError::FetchFailed("x".into()));
        assert_eq!(to_json(&err)["kind"], json!("update"));
    }

    #[test]
    fn kind_is_snake_case_variant_name_invalid_state() {
        let err = AppError::InvalidState("x".into());
        assert_eq!(to_json(&err)["kind"], json!("invalid_state"));
    }

    #[test]
    fn kind_is_snake_case_variant_name_config() {
        let err = AppError::Config("x".into());
        assert_eq!(to_json(&err)["kind"], json!("config"));
    }

    #[test]
    fn recoverable_storage_is_false() {
        let err = AppError::Storage(StorageError::InvalidState("x".into()));
        assert_eq!(to_json(&err)["recoverable"], json!(false));
    }

    #[test]
    fn recoverable_backend_is_true() {
        let err = AppError::Backend(BackendError::Placeholder("x".into()));
        assert_eq!(to_json(&err)["recoverable"], json!(true));
    }

    #[test]
    fn recoverable_keyring_is_false() {
        let err = AppError::Keyring(KeyringError::Placeholder("x".into()));
        assert_eq!(to_json(&err)["recoverable"], json!(false));
    }

    #[test]
    fn recoverable_update_is_true() {
        let err = AppError::Update(UpdateError::FetchFailed("x".into()));
        assert_eq!(to_json(&err)["recoverable"], json!(true));
    }

    #[test]
    fn recoverable_invalid_state_is_false() {
        let err = AppError::InvalidState("x".into());
        assert_eq!(to_json(&err)["recoverable"], json!(false));
    }

    #[test]
    fn recoverable_config_is_false() {
        let err = AppError::Config("x".into());
        assert_eq!(to_json(&err)["recoverable"], json!(false));
    }

    #[test]
    fn serialized_payload_has_exactly_three_keys() {
        for err in all_variants() {
            let value = to_json(&err);
            let obj = value.as_object().expect("AppError serializes to a JSON object");
            assert_eq!(obj.len(), 3, "expected 3 keys, got {:?}", obj.keys().collect::<Vec<_>>());
            assert!(obj.contains_key("kind"), "missing `kind` for {:?}", err);
            assert!(obj.contains_key("message"), "missing `message` for {:?}", err);
            assert!(obj.contains_key("recoverable"), "missing `recoverable` for {:?}", err);
        }
    }

    #[test]
    fn message_field_matches_display() {
        let err = AppError::InvalidState("bad transition".into());
        let value = to_json(&err);
        assert_eq!(value["message"], json!("invalid state: bad transition"));
    }

    #[test]
    fn from_storage_error_converts_into_app_error() {
        // Smoke-test that `#[from]` on the sub-error stubs actually wires up;
        // this is what feature modules will rely on once they own their
        // error types.
        fn returns_app_error() -> Result<(), AppError> {
            Err(StorageError::InvalidState("nope".into()))?;
            Ok(())
        }
        let err = returns_app_error().unwrap_err();
        assert_eq!(serde_json::to_value(&err).unwrap()["kind"], json!("storage"));
    }
}
