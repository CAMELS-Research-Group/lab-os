//! Shared IPC payload types — structs that cross the WebView boundary and are
//! not owned by a single feature module. Currently just [`UpdateInfo`] (consumed
//! by the `update` feature); feature-specific payloads live in their own modules
//! (e.g. `settings`, `identity`, `storage`).
//!
//! The TypeScript counterparts live in `app/src/ipc/types.ts`; when a shared
//! payload here changes, update that file in the same PR so the two sides stay
//! aligned.
//!
//! Convention: `#[serde(rename_all = "snake_case")]` on every struct so JSON
//! keys match the TS snake_case shape. Timestamps (ISO 8601 UTC) cross as plain
//! `String` — the IPC boundary is a JSON wire format, not a domain model.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Update info
// ---------------------------------------------------------------------------

/// Information about an available client update. `version` and `notes` are
/// `None` when no update is pending.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn update_info_round_trips_available() {
        let info = UpdateInfo {
            available: true,
            version: Some("1.2.3".to_string()),
            notes: Some("Bug fixes".to_string()),
        };
        let s = serde_json::to_string(&info).unwrap();
        let parsed: UpdateInfo = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, info);
    }

    #[test]
    fn update_info_round_trips_unavailable() {
        let info = UpdateInfo {
            available: false,
            version: None,
            notes: None,
        };
        let s = serde_json::to_string(&info).unwrap();
        let parsed: UpdateInfo = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, info);

        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["available"], json!(false));
        assert!(v["version"].is_null());
        assert!(v["notes"].is_null());
    }
}
