//! IAS client update feature — manifest model, semver comparison, and error type.
//!
//! This module owns the pure, network-free core of the update feature:
//!
//! - [`Manifest`] — the `latest.json` data model (CI-11 schema), including
//!   the optional [`platforms`][Manifest::platforms] map added in CI-6.
//! - [`parse_manifest`] — deserialize a JSON string into a [`Manifest`],
//!   returning a typed [`UpdateError`] on parse failure.
//! - [`is_update_available`] — semver comparison between the running version
//!   and a [`Manifest`]; returns `true` only when the manifest version is
//!   strictly greater. Non-semver inputs produce a typed error rather than a
//!   false positive.
//! - [`UpdateError`] — feature-owned `thiserror`-derived enum; bubbles into
//!   [`crate::shared::error::AppError`] via `#[from]`.
//!
//! HTTP fetching and Tauri commands are in a later task. Nothing here touches
//! the network or the Tauri runtime.

pub mod commands;

use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

// ---------------------------------------------------------------------------
// UpdateError
// ---------------------------------------------------------------------------

/// Errors produced by the update feature.
///
/// Per ADD §3.10 this enum is owned by the `update` module and bubbles into
/// [`crate::shared::error::AppError::Update`] via `#[from]`.
///
/// # Privacy note
///
/// The inner `String` payloads of these variants are surfaced verbatim into
/// [`crate::shared::error::AppError`]'s serialized `message` field, which
/// crosses the IPC boundary and is readable by the frontend. They carry only
/// transport/OS error text and the manifest host — they MUST NEVER contain an
/// install UUID or any user identifier. The update request is intentionally
/// identifier-free by design (NF-PRV-3).
#[derive(Debug, Error)]
pub enum UpdateError {
    /// A network or HTTP error prevented fetching the update manifest.
    /// The inner string is the verbatim transport-layer message so log
    /// triage can distinguish DNS failures from HTTP 4xx/5xx responses.
    /// (Populated by the fetch task; defined here so the error shape is
    /// stable before the command layer lands.)
    #[error("update manifest fetch failed: {0}")]
    FetchFailed(String),

    /// The server's response body was not valid JSON, or the JSON did not
    /// match the [`Manifest`] schema (missing required fields, wrong types).
    #[error("update manifest parse failed: {0}")]
    ManifestParse(String),

    /// A version string — either the running app version or the `version`
    /// field from a [`Manifest`] — was not valid semver. The inner string
    /// is the offending raw version text.
    #[error("invalid semver version string: {0:?}")]
    InvalidVersion(String),

    /// The OS failed to open the download URL in the default browser.
    /// (Consumed by the "open browser" command in a later task; defined
    /// here so the error enum is stable across tasks.)
    #[error("failed to open download URL in browser: {0}")]
    OpenBrowserFailed(String),

    /// `apply_update` was invoked while the opt-in gate (`update_checks_enabled`)
    /// is off. Surfaced as a typed error instead of a silent `Ok(())` so the
    /// caller (and the frontend) can distinguish "gate disabled" from a
    /// successful browser-open.
    #[error("update checks are disabled")]
    ChecksDisabled,
}

// ---------------------------------------------------------------------------
// PlatformEntry
// ---------------------------------------------------------------------------

/// Per-platform download URL entry inside [`Manifest::platforms`].
///
/// Carries the `download_url` for a single target triple. Additional fields
/// may be added in future CI-11 schema versions; unknown keys are silently
/// ignored.
///
/// # Key format
///
/// Map keys are `"{OS}-{ARCH}"` where `OS` and `ARCH` come from
/// [`std::env::consts::OS`] / [`std::env::consts::ARCH`] at build time.
/// Canonical keys for the V1 release targets are:
/// - `windows-x86_64`
/// - `macos-aarch64` (Apple silicon — `mac-stable` channel)
/// - `macos-x86_64` (Intel macOS — `mac-intel-stable` channel)
///
/// Note: the OS segment uses Rust's convention (`"macos"`, NOT `"darwin"`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PlatformEntry {
    /// Platform-specific installer or archive URL.
    pub download_url: String,
}

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

/// The `latest.json` update manifest emitted by CI-11.
///
/// Schema:
/// - `version` — the release version string (expected to be valid semver).
/// - `download_url` — fallback download URL for the installer / archive.
///   Used when `platforms` is absent or contains no entry for the running
///   target. Always required for backward compatibility.
/// - `notes` — human-readable release notes for the update banner.
/// - `platforms` — optional map of platform-specific download URLs (CI-6).
///   Keys are `"{OS}-{ARCH}"` (e.g. `"windows-x86_64"`, `"macos-aarch64"`,
///   `"macos-x86_64"` for Intel macOS).
///   When present and the running target has an entry, [`Manifest::download_url_for`]
///   returns the platform-specific URL instead of the flat fallback.
///
/// Unknown JSON keys are silently ignored (`deny_unknown_fields` is
/// intentionally omitted) so the manifest schema can grow without breaking
/// older client builds.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub download_url: String,
    pub notes: String,
    /// Optional per-platform download URL map (CI-6).
    ///
    /// Keys are `"{OS}-{ARCH}"` strings matching the running host's
    /// [`std::env::consts::OS`] and [`std::env::consts::ARCH`].
    /// Absent when the manifest was produced by a pre-CI-6 pipeline.
    #[serde(default)]
    pub platforms: Option<HashMap<String, PlatformEntry>>,
}

impl Manifest {
    /// Return the download URL appropriate for the **running** host.
    ///
    /// Delegates to [`Self::download_url_for_target`] with the target constants
    /// [`std::env::consts::OS`] and [`std::env::consts::ARCH`].
    ///
    /// # Fallback behaviour
    ///
    /// When `platforms` is absent, or when the map contains no entry for the
    /// running target, the flat `download_url` field is returned. This
    /// preserves full backward compatibility with manifests produced before
    /// CI-6.
    pub fn download_url_for(&self) -> &str {
        self.download_url_for_target(std::env::consts::OS, std::env::consts::ARCH)
    }

    /// Return the download URL for an explicit `(os, arch)` pair.
    ///
    /// This is the testable core of the platform resolver. The public
    /// [`Self::download_url_for`] calls this with the compile-time constants;
    /// tests call it directly with controlled values so the BLOCKER-guard
    /// assertion can run on any host without recompilation.
    ///
    /// # Key derivation
    ///
    /// The lookup key is `"{os}-{arch}"`, matching the canonical format used
    /// in the CI-11 manifest schema (`"windows-x86_64"`, `"macos-aarch64"`).
    ///
    /// # Fallback behaviour
    ///
    /// Returns `self.platforms[key].download_url` when the map exists and
    /// contains the key; otherwise falls back to the flat `download_url`.
    pub fn download_url_for_target(&self, os: &str, arch: &str) -> &str {
        let key = format!("{os}-{arch}");
        self.platforms
            .as_ref()
            .and_then(|map| map.get(&key))
            .map(|entry| entry.download_url.as_str())
            .unwrap_or(self.download_url.as_str())
    }
}

// ---------------------------------------------------------------------------
// parse_manifest
// ---------------------------------------------------------------------------

/// Parse a JSON string into a [`Manifest`].
///
/// Returns [`UpdateError::ManifestParse`] if the JSON is malformed or the
/// required fields are absent or of the wrong type.
pub fn parse_manifest(json: &str) -> Result<Manifest, UpdateError> {
    serde_json::from_str(json).map_err(|e| UpdateError::ManifestParse(e.to_string()))
}

// ---------------------------------------------------------------------------
// is_update_available
// ---------------------------------------------------------------------------

/// Return `true` when the manifest version is strictly greater than
/// `current_version` under semver ordering.
///
/// Both `current_version` and `manifest.version` must be valid semver strings;
/// either failing to parse yields [`UpdateError::InvalidVersion`] rather than
/// a false "update available."
///
/// Pre-release ordering follows semver rules: `1.0.0-alpha < 1.0.0`.
pub fn is_update_available(current_version: &str, manifest: &Manifest) -> Result<bool, UpdateError> {
    let current = semver::Version::parse(current_version)
        .map_err(|_| UpdateError::InvalidVersion(current_version.to_owned()))?;
    let manifest_ver = semver::Version::parse(&manifest.version)
        .map_err(|_| UpdateError::InvalidVersion(manifest.version.clone()))?;
    Ok(manifest_ver > current)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Manifest deserialization --

    #[test]
    fn manifest_deserializes_from_valid_json() {
        let json = r#"{"version":"1.2.3","download_url":"https://example.com/app.exe","notes":"Bug fixes"}"#;
        let m = parse_manifest(json).unwrap();
        assert_eq!(m.version, "1.2.3");
        assert_eq!(m.download_url, "https://example.com/app.exe");
        assert_eq!(m.notes, "Bug fixes");
    }

    #[test]
    fn manifest_ignores_unknown_keys() {
        let json = r#"{"version":"1.0.0","download_url":"https://example.com/app.exe","notes":"hi","extra":"ignored","future_field":42}"#;
        let m = parse_manifest(json).unwrap();
        assert_eq!(m.version, "1.0.0");
    }

    #[test]
    fn manifest_parse_returns_error_on_malformed_json() {
        let result = parse_manifest("not json at all {{{");
        assert!(
            matches!(result, Err(UpdateError::ManifestParse(_))),
            "expected ManifestParse, got {:?}",
            result
        );
    }

    #[test]
    fn manifest_parse_returns_error_when_field_missing() {
        // `notes` field absent
        let json = r#"{"version":"1.0.0","download_url":"https://example.com/app.exe"}"#;
        let result = parse_manifest(json);
        assert!(
            matches!(result, Err(UpdateError::ManifestParse(_))),
            "expected ManifestParse for missing field, got {:?}",
            result
        );
    }

    #[test]
    fn manifest_parse_returns_error_when_field_wrong_type() {
        // `version` is a number instead of a string
        let json = r#"{"version":123,"download_url":"https://example.com/app.exe","notes":"hi"}"#;
        let result = parse_manifest(json);
        assert!(
            matches!(result, Err(UpdateError::ManifestParse(_))),
            "expected ManifestParse for wrong field type, got {:?}",
            result
        );
    }

    // -- is_update_available: comparison semantics --

    fn make_manifest(version: &str) -> Manifest {
        Manifest {
            version: version.to_owned(),
            download_url: "https://example.com/app.exe".to_owned(),
            notes: "test".to_owned(),
            platforms: None,
        }
    }

    #[test]
    fn newer_manifest_version_means_update_available() {
        let result = is_update_available("1.0.0", &make_manifest("1.1.0")).unwrap();
        assert!(result, "1.1.0 > 1.0.0 should be update available");
    }

    #[test]
    fn equal_versions_means_no_update_available() {
        let result = is_update_available("1.0.0", &make_manifest("1.0.0")).unwrap();
        assert!(!result, "equal versions should not be update available");
    }

    #[test]
    fn older_manifest_version_means_no_update_available() {
        let result = is_update_available("2.0.0", &make_manifest("1.9.9")).unwrap();
        assert!(!result, "older manifest should not be update available");
    }

    #[test]
    fn semver_patch_ordering_is_respected() {
        let result = is_update_available("1.0.0", &make_manifest("1.0.1")).unwrap();
        assert!(result, "1.0.1 > 1.0.0 should be update available");
    }

    #[test]
    fn semver_major_ordering_is_respected() {
        let result = is_update_available("1.9.9", &make_manifest("2.0.0")).unwrap();
        assert!(result, "2.0.0 > 1.9.9 should be update available");
    }

    // Pre-release semantics: 1.0.0-alpha < 1.0.0 per semver spec.
    #[test]
    fn prerelease_manifest_is_not_greater_than_release() {
        let result = is_update_available("1.0.0", &make_manifest("1.0.0-alpha")).unwrap();
        assert!(!result, "1.0.0-alpha < 1.0.0 so no update available");
    }

    #[test]
    fn release_is_greater_than_prerelease_current() {
        let result = is_update_available("1.0.0-alpha", &make_manifest("1.0.0")).unwrap();
        assert!(result, "1.0.0 > 1.0.0-alpha so update is available");
    }

    #[test]
    fn prerelease_ordering_within_same_version() {
        // alpha < beta per semver identifier comparison
        let result = is_update_available("1.0.0-alpha", &make_manifest("1.0.0-beta")).unwrap();
        assert!(result, "1.0.0-beta > 1.0.0-alpha so update is available");
    }

    // -- is_update_available: invalid semver inputs yield typed errors --

    #[test]
    fn invalid_current_version_yields_typed_error() {
        let result = is_update_available("not-semver", &make_manifest("1.0.0"));
        assert!(
            matches!(result, Err(UpdateError::InvalidVersion(ref s)) if s == "not-semver"),
            "expected InvalidVersion for bad current version, got {:?}",
            result
        );
    }

    #[test]
    fn invalid_manifest_version_yields_typed_error() {
        let result = is_update_available("1.0.0", &make_manifest("1.x.bad"));
        assert!(
            matches!(result, Err(UpdateError::InvalidVersion(ref s)) if s == "1.x.bad"),
            "expected InvalidVersion for bad manifest version, got {:?}",
            result
        );
    }

    #[test]
    fn invalid_current_version_does_not_return_false_available() {
        // Critically: must NOT return Ok(true) or Ok(false) — must be Err.
        let result = is_update_available("garbage", &make_manifest("2.0.0"));
        assert!(result.is_err(), "non-semver current must not return Ok(...)");
    }

    #[test]
    fn invalid_manifest_version_does_not_return_false_available() {
        let result = is_update_available("1.0.0", &make_manifest("garbage"));
        assert!(result.is_err(), "non-semver manifest must not return Ok(...)");
    }

    // -- Platform-keyed URL resolver (CI-6 BLOCKER guard) --

    /// Build a manifest that carries per-platform entries for both canonical
    /// targets. Used across the resolver tests below.
    fn make_platform_manifest() -> Manifest {
        let mut platforms = HashMap::new();
        platforms.insert(
            "windows-x86_64".to_owned(),
            PlatformEntry {
                download_url: "https://example.com/app.msi".to_owned(),
            },
        );
        platforms.insert(
            "macos-aarch64".to_owned(),
            PlatformEntry {
                download_url: "https://example.com/app.dmg".to_owned(),
            },
        );
        platforms.insert(
            "macos-x86_64".to_owned(),
            PlatformEntry {
                download_url: "https://example.com/app-intel.dmg".to_owned(),
            },
        );
        Manifest {
            version: "2.0.0".to_owned(),
            download_url: "https://example.com/app-flat.exe".to_owned(),
            notes: "test".to_owned(),
            platforms: Some(platforms),
        }
    }

    /// BLOCKER-guard: a macOS aarch64 runtime must receive the `.dmg` URL, not
    /// the flat `.msi` fallback. Tests the resolver directly with an explicit
    /// target key so it runs on any build host without recompilation.
    #[test]
    fn resolver_returns_dmg_for_macos_aarch64() {
        let manifest = make_platform_manifest();
        let url = manifest.download_url_for_target("macos", "aarch64");
        assert_eq!(
            url, "https://example.com/app.dmg",
            "macos-aarch64 must resolve to the .dmg URL, not the flat fallback"
        );
    }

    /// Windows x86_64 must resolve to the `.msi` URL, not the flat fallback.
    #[test]
    fn resolver_returns_msi_for_windows_x86_64() {
        let manifest = make_platform_manifest();
        let url = manifest.download_url_for_target("windows", "x86_64");
        assert_eq!(
            url, "https://example.com/app.msi",
            "windows-x86_64 must resolve to the .msi URL"
        );
    }

    /// Intel macOS (x86_64) must resolve to its OWN `.dmg` URL, distinct from the
    /// arm64 entry — the Intel leg publishes a separate archive to mac-intel-stable
    /// with the `macos-x86_64` key. An Intel runtime must NOT receive the arm64 URL.
    #[test]
    fn resolver_returns_intel_dmg_for_macos_x86_64() {
        let manifest = make_platform_manifest();
        let url = manifest.download_url_for_target("macos", "x86_64");
        assert_eq!(
            url, "https://example.com/app-intel.dmg",
            "macos-x86_64 must resolve to the Intel .dmg URL, not the arm64 .dmg or the flat fallback"
        );
        // And the arm64 lookup must still resolve to the arm64 dmg (no cross-talk).
        let arm = manifest.download_url_for_target("macos", "aarch64");
        assert_eq!(
            arm, "https://example.com/app.dmg",
            "macos-aarch64 must still resolve to the arm64 .dmg, independent of the Intel entry"
        );
    }

    /// An absent platform key must fall back to the flat `download_url`.
    #[test]
    fn resolver_falls_back_to_flat_for_unknown_key() {
        let manifest = make_platform_manifest();
        let url = manifest.download_url_for_target("linux", "x86_64");
        assert_eq!(
            url, "https://example.com/app-flat.exe",
            "unknown platform key must fall back to the flat download_url"
        );
    }

    /// A manifest with NO platforms map must return the flat `download_url`
    /// for any target (full backward compatibility).
    #[test]
    fn resolver_falls_back_to_flat_when_platforms_absent() {
        let manifest = make_manifest("1.0.0"); // platforms: None
        let url = manifest.download_url_for_target("macos", "aarch64");
        assert_eq!(
            url, "https://example.com/app.exe",
            "absent platforms map must fall back to the flat download_url"
        );
    }

    /// Verify that the key derivation does NOT use "darwin" — the Rust OS
    /// constant is "macos". A manifest keyed with "darwin-aarch64" must NOT
    /// match a macos/aarch64 runtime (falls back to flat instead).
    #[test]
    fn resolver_does_not_use_darwin_key() {
        let mut platforms = HashMap::new();
        platforms.insert(
            "darwin-aarch64".to_owned(), // wrong key convention
            PlatformEntry {
                download_url: "https://example.com/app-darwin.dmg".to_owned(),
            },
        );
        let manifest = Manifest {
            version: "2.0.0".to_owned(),
            download_url: "https://example.com/app-flat.exe".to_owned(),
            notes: "test".to_owned(),
            platforms: Some(platforms),
        };
        // macos/aarch64 must NOT match the darwin-keyed entry.
        let url = manifest.download_url_for_target("macos", "aarch64");
        assert_eq!(
            url, "https://example.com/app-flat.exe",
            "resolver must use 'macos', not 'darwin', as the OS key segment"
        );
    }

    /// A manifest with a platforms map that has NO entry for the queried target
    /// must return the flat `download_url` (not panic or return an empty string).
    #[test]
    fn resolver_falls_back_to_flat_for_empty_platforms_map() {
        let manifest = Manifest {
            version: "1.0.0".to_owned(),
            download_url: "https://example.com/app-flat.exe".to_owned(),
            notes: "test".to_owned(),
            platforms: Some(HashMap::new()), // present but empty
        };
        let url = manifest.download_url_for_target("macos", "aarch64");
        assert_eq!(
            url, "https://example.com/app-flat.exe",
            "empty platforms map must fall back to flat download_url"
        );
    }

    /// Verify that a manifest with a platforms map still parses correctly from
    /// JSON, and that unknown keys inside `platforms` entries are ignored.
    #[test]
    fn manifest_with_platforms_parses_from_json() {
        let json = r#"{
            "version": "2.0.0",
            "download_url": "https://example.com/app.msi",
            "notes": "Release notes",
            "platforms": {
                "windows-x86_64": { "download_url": "https://example.com/app.msi", "future_key": 42 },
                "macos-aarch64":  { "download_url": "https://example.com/app.dmg" }
            }
        }"#;
        let m = parse_manifest(json).unwrap();
        assert_eq!(m.version, "2.0.0");
        let platforms = m.platforms.as_ref().expect("platforms must be Some");
        assert_eq!(
            platforms["windows-x86_64"].download_url,
            "https://example.com/app.msi"
        );
        assert_eq!(
            platforms["macos-aarch64"].download_url,
            "https://example.com/app.dmg"
        );
    }

    /// A manifest WITHOUT a platforms map still parses correctly (back-compat).
    #[test]
    fn manifest_without_platforms_parses_from_json() {
        let json = r#"{"version":"1.0.0","download_url":"https://example.com/app.exe","notes":"hi"}"#;
        let m = parse_manifest(json).unwrap();
        assert!(
            m.platforms.is_none(),
            "platforms must be None when absent from JSON"
        );
    }

    // -- UpdateError variants exist and display reasonably --

    #[test]
    fn update_error_variants_display() {
        let e1 = UpdateError::FetchFailed("timeout".into());
        assert!(e1.to_string().contains("timeout"));

        let e2 = UpdateError::ManifestParse("unexpected eof".into());
        assert!(e2.to_string().contains("unexpected eof"));

        let e3 = UpdateError::InvalidVersion("bad".into());
        assert!(e3.to_string().contains("bad"));

        let e4 = UpdateError::OpenBrowserFailed("permission denied".into());
        assert!(e4.to_string().contains("permission denied"));

        let e5 = UpdateError::ChecksDisabled;
        assert!(e5.to_string().contains("disabled"));
    }
}
