//! Canonical Rust side of the IPC contract.
//!
//! Every payload that crosses the WebView boundary is defined here. The
//! TypeScript counterpart lives in `app/src/ipc/types.ts` (CL-11) and **must
//! mirror this file exactly** until codegen lands post-V1.
//!
//! Source of truth: ADD §3.6 (IPC payload types). See also FRD F-IPC for the
//! requirement that the two sides stay in lockstep.
//!
//! Conventions enforced here:
//!
//! - `#[serde(rename_all = "snake_case")]` on every struct so JSON keys match
//!   the TS snake_case shape (e.g. `regional_variety`, `session_id`).
//! - String-literal unions on the TS side are modelled as Rust enums with the
//!   appropriate `rename_all` (`snake_case` for lowercase tokens, `PascalCase`
//!   for [`FirstRunPhase`]).
//! - Timestamps (ISO 8601 UTC) and UUIDs are conveyed as plain `String` — the
//!   IPC boundary is a JSON wire format, not a domain model. Validation /
//!   parsing happens at construction sites, not at the type definition.
//! - Newtype wrappers (`SessionId`, `PhonemeAttempts`, `PhonemeThresholds`)
//!   use `#[serde(transparent)]` so they serialize as their inner value rather
//!   than a single-field object.
//!
//! If you change a type here, you **must** update `app/src/ipc/types.ts` in
//! the same PR. Keep field order and naming aligned between the two files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Install / consent
// ---------------------------------------------------------------------------

/// Per-install identity and consent posture. Returned by the install / first-run
/// IPC commands. Mirrors the TS `InstallState`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct InstallState {
    pub uuid: String,
    pub consent_state: ConsentState,
}

/// Tri-state consent status. Serialized as `"pending" | "granted" | "revoked"`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConsentState {
    Pending,
    Granted,
    Revoked,
}

// ---------------------------------------------------------------------------
// Difficulty level + thresholds
// ---------------------------------------------------------------------------

/// Named difficulty level. Resolves to a per-phoneme threshold map via the
/// bundled threshold table. Serialized as `"gentle" | "standard" | "strict"`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DifficultyLevel {
    Gentle,
    Standard,
    Strict,
}

/// Per-phoneme certainty cutoff map for the active difficulty level.
/// Keys are IPA symbols; values are in `[0, 1]`. Newtype around
/// `HashMap<String, f64>` so it serializes transparently as a JSON object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(transparent)]
pub struct PhonemeThresholds(pub HashMap<String, f64>);

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// User-mutable settings persisted across sessions. Mirrors the TS `Settings`.
///
/// `difficulty` is the **named level** (not a scalar). The per-phoneme cutoff
/// map is derived at evaluation time and surfaced on [`EvaluationResult`].
///
/// `update_checks_enabled` gates periodic network egress to check for a new
/// client version. Off by default (CL-23-lite); the user opts in via the
/// Settings screen.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Settings {
    pub l1: String,
    pub regional_variety: Option<String>,
    pub difficulty: DifficultyLevel,
    pub report_uploads_enabled: bool,
    pub update_checks_enabled: bool,
}

// ---------------------------------------------------------------------------
// Session identifiers
// ---------------------------------------------------------------------------

/// Client-generated v4 UUID for a single practice session. Transparent newtype
/// over `String` — serializes as a bare JSON string.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionId(pub String);

// ---------------------------------------------------------------------------
// Evaluation results
// ---------------------------------------------------------------------------

/// Rollup of attempts for a single phoneme within a session. Inner value of
/// [`PhonemeAttempts`].
///
/// **Fallback-occurrence filter:** `flagged` and `mean_certainty` count only
/// label positions the Viterbi pass anchored on real audio frames; positions
/// filled via CL-17's zero-frame fallback (a flanking-blank window when the
/// learner never reached that label — typical on a partial / fast read)
/// contribute to `occurrences` but NOT to `flagged` or `mean_certainty`. The
/// effect: an unread phoneme reports honest `occurrences > 0` against the
/// reference but is not penalised as a pronunciation error. Whether this is
/// the right pedagogy is an open question with IAS — see
/// `documentation/docs/updates/asks.md` (Q: partial-read flag suppression).
///
/// `mean_certainty` is `None` when there are zero non-fallback occurrences
/// (e.g. the phoneme never occurred in the reference, or every label position
/// was filled via fallback).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AttemptRollup {
    /// Total reference label positions for this symbol — includes both
    /// real-frame-anchored and fallback occurrences. Honest wire-shape
    /// reflection of "how many times the passage asks for this sound."
    pub occurrences: u32,
    /// Real-frame-anchored occurrences with certainty below the difficulty
    /// threshold. Fallback occurrences are excluded — see struct doc.
    pub flagged: u32,
    /// Mean of certainties over real-frame-anchored occurrences only.
    /// `None` when every occurrence for this symbol was fallback (no
    /// real-frame signal to average), or when the phoneme never occurred.
    pub mean_certainty: Option<f64>,
}

/// Map of IPA symbol to its per-session [`AttemptRollup`]. Transparent newtype
/// so the JSON shape is `{ "θ": { ... }, "ð": { ... } }` rather than nested
/// under a `0` field.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(transparent)]
pub struct PhonemeAttempts(pub HashMap<String, AttemptRollup>);

/// Single entry in `EvaluationResult.flagged_phonemes_ordered`. Carries the
/// example word copy so the results screen does not have to look it up.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FlaggedPhoneme {
    pub phoneme: String,
    pub example_word: String,
    pub flag_count: u32,
    pub mean_certainty: f64,
}

/// Full result payload for a completed session. Mirrors the TS
/// `EvaluationResult`. Carries per-phoneme certainty (FRD F-EVL-3) and the
/// three v0.4 fields (`difficulty_level`, `difficulty_thresholds`,
/// `threshold_table_version`) applied at evaluation time.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct EvaluationResult {
    pub session_id: SessionId,
    pub started_at: String,
    pub ended_at: String,
    pub duration_seconds: f64,
    pub phoneme_attempts: PhonemeAttempts,
    pub difficulty_level: DifficultyLevel,
    pub difficulty_thresholds: PhonemeThresholds,
    pub threshold_table_version: i32,
    pub reattempt_counts_by_sentence: Vec<u32>,
    pub flagged_phonemes_ordered: Vec<FlaggedPhoneme>,
    pub highest_error_phoneme: Option<String>,
    pub model_version: String,
}

// ---------------------------------------------------------------------------
// History / progress
// ---------------------------------------------------------------------------

/// Compact summary of a completed session, used in the history list.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub ended_at: String,
    pub duration_seconds: f64,
    pub flagged_count: u32,
    pub highest_error_phoneme: Option<String>,
}

/// Direction a phoneme's flag rate has moved over the observed window.
/// Serialized as `"improving" | "worsening" | "flat"`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Worsening,
    Flat,
}

/// Cross-session trend for a single phoneme.
///
/// `Eq` is intentionally NOT derived: `session_flag_rate` is `Vec<f64>` and
/// `f64` is not `Eq`. `PartialEq` is sufficient for the round-trip test.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PhonemeTrend {
    pub phoneme: String,
    pub example_word: String,
    pub attempts_total: u32,
    pub flagged_total: u32,
    pub trend_direction: TrendDirection,
    pub sessions_observed: u32,
    /// Per-session flag rate (`flagged / occurrences`) for each session in
    /// which this sound **occurred** (`occurrences > 0`), oldest→newest. One
    /// entry per observed session, no gaps — `len()` equals `sessions_observed`.
    pub session_flag_rate: Vec<f64>,
}

// ---------------------------------------------------------------------------
// Upload queue status
// ---------------------------------------------------------------------------

/// HTTP-status discriminator for the most recent terminal upload error.
/// Serialized as the literal string `"401" | "403" | "410"` — matches the TS
/// union shape exactly. Do NOT use `#[repr(u16)]`; the wire format is string.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TerminalErrorCode {
    #[serde(rename = "401")]
    Unauthorized,
    #[serde(rename = "403")]
    Forbidden,
    #[serde(rename = "410")]
    Gone,
}

/// Last terminal (non-retryable) upload error, plus the timestamp it occurred.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct LastTerminalError {
    pub code: TerminalErrorCode,
    pub at: String,
}

/// Snapshot of the report-upload queue.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct QueueStatus {
    pub pending_count: u32,
    pub last_attempt_at: Option<String>,
    pub last_terminal_error: Option<LastTerminalError>,
}

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

// ---------------------------------------------------------------------------
// Passage (read-aloud content)
// ---------------------------------------------------------------------------

/// One word of the passage along with its bundled reference IPA sequence.
/// See TRD §4.5.1 for the reference-IPA workflow.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ExpectedIpaPerWord {
    pub word: String,
    pub ipa: Vec<String>,
}

/// The passage the learner reads aloud, plus the bundled reference IPA
/// sequence. The text is the canonical written form; the IPA list is what the
/// inference pipeline aligns against.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Passage {
    pub text: String,
    pub expected_ipa_per_word: Vec<ExpectedIpaPerWord>,
}

// ---------------------------------------------------------------------------
// First-run state machine
// ---------------------------------------------------------------------------

/// First-run phase enum. Serialized in **PascalCase** to match the TS union
/// exactly (`"WelcomePending" | "L1Pending" | ...`).
///
/// V1 has no recovery-code variants — recovery codes are cut.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum FirstRunPhase {
    WelcomePending,
    L1Pending,
    ConsentPending,
    ModelDownloading,
    Ready,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    // ----- enum serialization -----

    #[test]
    fn difficulty_level_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&DifficultyLevel::Gentle).unwrap(),
            "\"gentle\""
        );
        assert_eq!(
            serde_json::to_string(&DifficultyLevel::Standard).unwrap(),
            "\"standard\""
        );
        assert_eq!(
            serde_json::to_string(&DifficultyLevel::Strict).unwrap(),
            "\"strict\""
        );
    }

    #[test]
    fn consent_state_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&ConsentState::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&ConsentState::Granted).unwrap(),
            "\"granted\""
        );
        assert_eq!(
            serde_json::to_string(&ConsentState::Revoked).unwrap(),
            "\"revoked\""
        );
    }

    #[test]
    fn trend_direction_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&TrendDirection::Improving).unwrap(),
            "\"improving\""
        );
        assert_eq!(
            serde_json::to_string(&TrendDirection::Worsening).unwrap(),
            "\"worsening\""
        );
        assert_eq!(
            serde_json::to_string(&TrendDirection::Flat).unwrap(),
            "\"flat\""
        );
    }

    #[test]
    fn terminal_error_code_serializes_as_numeric_strings() {
        assert_eq!(
            serde_json::to_string(&TerminalErrorCode::Unauthorized).unwrap(),
            "\"401\""
        );
        assert_eq!(
            serde_json::to_string(&TerminalErrorCode::Forbidden).unwrap(),
            "\"403\""
        );
        assert_eq!(
            serde_json::to_string(&TerminalErrorCode::Gone).unwrap(),
            "\"410\""
        );

        // And round-trip the other direction
        let parsed: TerminalErrorCode = serde_json::from_str("\"401\"").unwrap();
        assert_eq!(parsed, TerminalErrorCode::Unauthorized);
        let parsed: TerminalErrorCode = serde_json::from_str("\"403\"").unwrap();
        assert_eq!(parsed, TerminalErrorCode::Forbidden);
        let parsed: TerminalErrorCode = serde_json::from_str("\"410\"").unwrap();
        assert_eq!(parsed, TerminalErrorCode::Gone);
    }

    #[test]
    fn first_run_phase_serializes_pascal_case() {
        assert_eq!(
            serde_json::to_string(&FirstRunPhase::WelcomePending).unwrap(),
            "\"WelcomePending\""
        );
        assert_eq!(
            serde_json::to_string(&FirstRunPhase::L1Pending).unwrap(),
            "\"L1Pending\""
        );
        assert_eq!(
            serde_json::to_string(&FirstRunPhase::ConsentPending).unwrap(),
            "\"ConsentPending\""
        );
        assert_eq!(
            serde_json::to_string(&FirstRunPhase::ModelDownloading).unwrap(),
            "\"ModelDownloading\""
        );
        assert_eq!(
            serde_json::to_string(&FirstRunPhase::Ready).unwrap(),
            "\"Ready\""
        );

        // Round-trip — confirms discrimination still works on the receive side.
        let parsed: FirstRunPhase = serde_json::from_str("\"WelcomePending\"").unwrap();
        assert_eq!(parsed, FirstRunPhase::WelcomePending);
        let parsed: FirstRunPhase = serde_json::from_str("\"L1Pending\"").unwrap();
        assert_eq!(parsed, FirstRunPhase::L1Pending);
        let parsed: FirstRunPhase = serde_json::from_str("\"ConsentPending\"").unwrap();
        assert_eq!(parsed, FirstRunPhase::ConsentPending);
        let parsed: FirstRunPhase = serde_json::from_str("\"ModelDownloading\"").unwrap();
        assert_eq!(parsed, FirstRunPhase::ModelDownloading);
        let parsed: FirstRunPhase = serde_json::from_str("\"Ready\"").unwrap();
        assert_eq!(parsed, FirstRunPhase::Ready);
    }

    // ----- newtype transparency -----

    #[test]
    fn session_id_serializes_transparently() {
        let id = SessionId("abc-123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"abc-123\"");

        let parsed: SessionId = serde_json::from_str("\"abc-123\"").unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn phoneme_thresholds_serializes_transparently() {
        let mut map = HashMap::new();
        map.insert("θ".to_string(), 0.75);
        let pt = PhonemeThresholds(map);

        let value: Value = serde_json::to_value(&pt).unwrap();
        // Should be a JSON object, not `{ "0": { ... } }`.
        assert!(value.is_object());
        assert_eq!(value["θ"], json!(0.75));

        // Round-trip
        let s = serde_json::to_string(&pt).unwrap();
        let parsed: PhonemeThresholds = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, pt);
    }

    #[test]
    fn phoneme_attempts_serializes_transparently() {
        let mut map = HashMap::new();
        map.insert(
            "θ".to_string(),
            AttemptRollup {
                occurrences: 4,
                flagged: 1,
                mean_certainty: Some(0.82),
            },
        );
        let pa = PhonemeAttempts(map);

        let value: Value = serde_json::to_value(&pa).unwrap();
        assert!(value.is_object());
        assert_eq!(value["θ"]["occurrences"], json!(4));
        assert_eq!(value["θ"]["flagged"], json!(1));
        assert_eq!(value["θ"]["mean_certainty"], json!(0.82));

        // Round-trip
        let s = serde_json::to_string(&pa).unwrap();
        let parsed: PhonemeAttempts = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, pa);
    }

    #[test]
    fn empty_phoneme_attempts_round_trips() {
        let pa = PhonemeAttempts::default();
        let s = serde_json::to_string(&pa).unwrap();
        assert_eq!(s, "{}");
        let parsed: PhonemeAttempts = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, pa);
    }

    // ----- round-trip tests -----

    #[test]
    fn install_state_round_trips() {
        let state = InstallState {
            uuid: "11111111-2222-3333-4444-555555555555".to_string(),
            consent_state: ConsentState::Granted,
        };
        let s = serde_json::to_string(&state).unwrap();
        let parsed: InstallState = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, state);

        // Confirm wire shape
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["uuid"], json!("11111111-2222-3333-4444-555555555555"));
        assert_eq!(v["consent_state"], json!("granted"));
    }

    #[test]
    fn settings_round_trips_and_difficulty_is_named_level() {
        let settings = Settings {
            l1: "spa".to_string(),
            regional_variety: Some("Caribbean".to_string()),
            difficulty: DifficultyLevel::Strict,
            report_uploads_enabled: true,
            update_checks_enabled: false,
        };
        let s = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, settings);

        // Confirm `difficulty` is the named string, not a float.
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["difficulty"], json!("strict"));
        assert!(v["difficulty"].is_string());
        assert!(!v["difficulty"].is_number());
        assert_eq!(v["regional_variety"], json!("Caribbean"));
        assert_eq!(v["report_uploads_enabled"], json!(true));
        // update_checks_enabled must be present and boolean.
        assert_eq!(v["update_checks_enabled"], json!(false));
        assert!(v["update_checks_enabled"].is_boolean());
    }

    #[test]
    fn settings_with_null_regional_variety_round_trips() {
        let settings = Settings {
            l1: "cmn".to_string(),
            regional_variety: None,
            difficulty: DifficultyLevel::Gentle,
            report_uploads_enabled: false,
            update_checks_enabled: false,
        };
        let s = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, settings);

        let v: Value = serde_json::from_str(&s).unwrap();
        assert!(v["regional_variety"].is_null());
    }

    #[test]
    fn settings_update_checks_enabled_wire_shape() {
        // Verify the snake_case key name and that true/false both round-trip.
        let settings_on = Settings {
            l1: "".to_string(),
            regional_variety: None,
            difficulty: DifficultyLevel::Gentle,
            report_uploads_enabled: true,
            update_checks_enabled: true,
        };
        let v: Value = serde_json::to_value(&settings_on).unwrap();
        assert_eq!(v["update_checks_enabled"], json!(true));
        assert!(v["update_checks_enabled"].is_boolean());

        let settings_off = Settings {
            update_checks_enabled: false,
            ..settings_on.clone()
        };
        let v: Value = serde_json::to_value(&settings_off).unwrap();
        assert_eq!(v["update_checks_enabled"], json!(false));
        assert!(v["update_checks_enabled"].is_boolean());

        // Round-trip from JSON string with the field name exactly as specified.
        let json_str = r#"{"l1":"","regional_variety":null,"difficulty":"gentle","report_uploads_enabled":true,"update_checks_enabled":true}"#;
        let parsed: Settings = serde_json::from_str(json_str).unwrap();
        assert!(parsed.update_checks_enabled);
    }

    fn sample_evaluation_result() -> EvaluationResult {
        let mut attempts = HashMap::new();
        attempts.insert(
            "θ".to_string(),
            AttemptRollup {
                occurrences: 4,
                flagged: 2,
                mean_certainty: Some(0.55),
            },
        );
        attempts.insert(
            "ɹ".to_string(),
            AttemptRollup {
                occurrences: 6,
                flagged: 0,
                mean_certainty: Some(0.91),
            },
        );

        let mut thresholds = HashMap::new();
        thresholds.insert("θ".to_string(), 0.70);
        thresholds.insert("ɹ".to_string(), 0.65);

        EvaluationResult {
            session_id: SessionId("session-xyz".to_string()),
            started_at: "2026-06-01T12:00:00Z".to_string(),
            ended_at: "2026-06-01T12:03:45Z".to_string(),
            duration_seconds: 225.5,
            phoneme_attempts: PhonemeAttempts(attempts),
            difficulty_level: DifficultyLevel::Standard,
            difficulty_thresholds: PhonemeThresholds(thresholds),
            threshold_table_version: 7,
            reattempt_counts_by_sentence: vec![0, 1, 0, 2],
            flagged_phonemes_ordered: vec![FlaggedPhoneme {
                phoneme: "θ".to_string(),
                example_word: "the 'th' in 'think'".to_string(),
                flag_count: 2,
                mean_certainty: 0.55,
            }],
            highest_error_phoneme: Some("θ".to_string()),
            model_version: "camels-v0.4.0".to_string(),
        }
    }

    #[test]
    fn evaluation_result_round_trips() {
        let er = sample_evaluation_result();
        let s = serde_json::to_string(&er).unwrap();
        let parsed: EvaluationResult = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, er);
    }

    #[test]
    fn evaluation_result_carries_v04_fields() {
        let er = sample_evaluation_result();
        let v: Value = serde_json::to_value(&er).unwrap();

        // The three v0.4 fields must be present with the right shape.
        assert_eq!(v["difficulty_level"], json!("standard"));
        assert!(v["difficulty_thresholds"].is_object());
        assert_eq!(v["difficulty_thresholds"]["θ"], json!(0.70));
        assert_eq!(v["difficulty_thresholds"]["ɹ"], json!(0.65));
        assert_eq!(v["threshold_table_version"], json!(7));
        assert!(v["threshold_table_version"].is_number());

        // Spot-check other fields survived snake_case mapping.
        assert_eq!(v["session_id"], json!("session-xyz"));
        assert!(v["phoneme_attempts"].is_object());
        assert_eq!(v["phoneme_attempts"]["θ"]["mean_certainty"], json!(0.55));
        assert_eq!(v["highest_error_phoneme"], json!("θ"));
        assert_eq!(v["model_version"], json!("camels-v0.4.0"));
    }

    #[test]
    fn session_summary_round_trips() {
        let summary = SessionSummary {
            session_id: SessionId("session-abc".to_string()),
            ended_at: "2026-06-01T12:30:00Z".to_string(),
            duration_seconds: 180.0,
            flagged_count: 3,
            highest_error_phoneme: Some("ð".to_string()),
        };
        let s = serde_json::to_string(&summary).unwrap();
        let parsed: SessionSummary = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, summary);

        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["session_id"], json!("session-abc"));
        assert_eq!(v["flagged_count"], json!(3));
    }

    #[test]
    fn phoneme_trend_round_trips() {
        let trend = PhonemeTrend {
            phoneme: "θ".to_string(),
            example_word: "think".to_string(),
            attempts_total: 24,
            flagged_total: 8,
            trend_direction: TrendDirection::Improving,
            sessions_observed: 6,
            session_flag_rate: vec![0.5, 0.4, 0.3, 0.2, 0.1, 0.0],
        };
        let s = serde_json::to_string(&trend).unwrap();
        let parsed: PhonemeTrend = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, trend);

        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["trend_direction"], json!("improving"));
        assert_eq!(v["attempts_total"], json!(24));
        // snake_case array field on the wire.
        assert_eq!(v["session_flag_rate"], json!([0.5, 0.4, 0.3, 0.2, 0.1, 0.0]));
    }

    #[test]
    fn queue_status_round_trips_without_terminal_error() {
        let status = QueueStatus {
            pending_count: 2,
            last_attempt_at: Some("2026-06-01T11:55:00Z".to_string()),
            last_terminal_error: None,
        };
        let s = serde_json::to_string(&status).unwrap();
        let parsed: QueueStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, status);

        let v: Value = serde_json::from_str(&s).unwrap();
        assert!(v["last_terminal_error"].is_null());
    }

    #[test]
    fn queue_status_round_trips_with_terminal_error() {
        let status = QueueStatus {
            pending_count: 0,
            last_attempt_at: Some("2026-06-01T11:50:00Z".to_string()),
            last_terminal_error: Some(LastTerminalError {
                code: TerminalErrorCode::Gone,
                at: "2026-06-01T11:50:00Z".to_string(),
            }),
        };
        let s = serde_json::to_string(&status).unwrap();
        let parsed: QueueStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, status);

        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["last_terminal_error"]["code"], json!("410"));
        assert_eq!(v["last_terminal_error"]["at"], json!("2026-06-01T11:50:00Z"));
    }

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

    #[test]
    fn passage_round_trips() {
        let passage = Passage {
            text: "The quick brown fox.".to_string(),
            expected_ipa_per_word: vec![
                ExpectedIpaPerWord {
                    word: "The".to_string(),
                    ipa: vec!["ð".to_string(), "ə".to_string()],
                },
                ExpectedIpaPerWord {
                    word: "quick".to_string(),
                    ipa: vec![
                        "k".to_string(),
                        "w".to_string(),
                        "ɪ".to_string(),
                        "k".to_string(),
                    ],
                },
            ],
        };
        let s = serde_json::to_string(&passage).unwrap();
        let parsed: Passage = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed, passage);

        let v: Value = serde_json::from_str(&s).unwrap();
        assert!(v["expected_ipa_per_word"].is_array());
        assert_eq!(v["expected_ipa_per_word"][0]["word"], json!("The"));
        assert_eq!(
            v["expected_ipa_per_word"][1]["ipa"],
            json!(["k", "w", "ɪ", "k"])
        );
    }
}
