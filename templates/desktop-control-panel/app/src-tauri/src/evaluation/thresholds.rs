//! Per-phoneme threshold table for the V1 evaluation pipeline.
//!
//! Source of truth: ADD T-BKE-15a, FRD F-EVL-4 / F-SET-3.
//!
//! **v3 (2026-06-09) — restored SPIKE-11 values (v1 scale).** The bundled
//! values are the original SPIKE-11 PRIMARY-measure thresholds (byte-identical
//! to v1 / PR #32). The v2 Option-A stop-gap (PR #66) was reverted: it had
//! anchored an absolute scale to a single un-normalized, quiet native read, but
//! issue #67/#70 root-caused that deficit as a MISSING wav2vec2 input
//! normalization (`do_normalize`), fixed in the input-normalization PR. With
//! normalized input the SPIKE-11 scale is reachable at any capture level, so the
//! band-aid is obsolete. See `threshold_table_v1.json::_metadata`.
//!
//! Calibration semantics (SPIKE-11 correct-production distribution):
//! - **Gentle**  = p10 (catches only clear errors; ~10% flag rate on correct).
//! - **Standard** = p25 (moderate sensitivity; FRD F-SET-3 default).
//! - **Strict**  = p50 (catches the most; higher nuisance rate).
//!
//! Depends on the #67 input-normalization fix being present in the binary;
//! without it, quiet captures re-introduce the over-flagging these thresholds
//! assume normalized input avoids. A clean re-anchor (re-run the SPIKE sweep
//! WITH normalization) is an optional follow-up before broader rollout.
//!
//! The table loads once at startup (orchestrator, CL-19) and is cached by the
//! caller. `resolve_for_level` returns a cloned [`PhonemeThresholds`] map so the
//! evaluator (CL-17) can attach it to each [`EvaluationResult`] without holding
//! a borrow on the table.
//!
//! Post-V1, MX-11 regenerates this table from the IAS-trained adapter's
//! L2-Arctic correct-production distributions and bumps `version`.
//!
//! The `_v1` filename suffix is frozen and decoupled from the `version` field:
//! the file is renamed only on a schema change, while in-place recalibrations
//! bump `version` instead — so `threshold_table_v1.json` legitimately carries
//! `version: 3`. The version is monotonic (1 = original SPIKE-11, 2 = Option-A
//! band-aid, 3 = restored SPIKE-11) so the persisted `threshold_table_version`
//! identifies which physical table scored each session, even though v3's values
//! equal v1's.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::evaluation::error::EvaluationError;
use crate::shared::types::{DifficultyLevel, PhonemeThresholds};

/// IPA symbols the V1 inventory targets. Both `load` and the build-time test
/// assert the bundled JSON covers every symbol here at every difficulty level.
///
/// The rhotic is keyed as `ɹ` (U+0279), not `r` — espeak-ng emits `ɹ` for the
/// English rhotic and that's what the alignment layer sees (SPIKE-4 / FINDINGS
/// §1b).
pub const V1_TARGET_PHONEMES: &[&str] = &[
    "w", "i", "l", "ʒ", "v", "z", "θ", "æ", "ɛ", "dʒ", "ɪ", "ɹ", "ð",
];

/// On-disk shape of `threshold_table_v1.json`. Kept private — callers see the
/// validated [`ThresholdTable`] type instead.
#[derive(Debug, Deserialize)]
struct OnDiskTable {
    version: i32,
    table: HashMap<DifficultyLevel, PhonemeThresholds>,
}

/// Validated, in-memory threshold table. Constructed via [`Self::load`]; every
/// instance is guaranteed to cover every (level × V1 target phoneme) cell.
#[derive(Debug, Clone)]
pub struct ThresholdTable {
    version: i32,
    table: HashMap<DifficultyLevel, PhonemeThresholds>,
}

impl ThresholdTable {
    /// Read `path`, parse the JSON, and assert completeness against
    /// [`V1_TARGET_PHONEMES`].
    ///
    /// Any failure (missing file, malformed JSON, missing level, missing
    /// phoneme for a level) surfaces as [`EvaluationError::ThresholdTableLoad`]
    /// with a message naming the failing cell(s) where applicable.
    pub fn load(path: &Path) -> Result<Self, EvaluationError> {
        let bytes = std::fs::read(path).map_err(|e| {
            EvaluationError::ThresholdTableLoad(format!(
                "could not read {}: {}",
                path.display(),
                e
            ))
        })?;

        let on_disk: OnDiskTable = serde_json::from_slice(&bytes).map_err(|e| {
            EvaluationError::ThresholdTableLoad(format!(
                "could not parse {}: {}",
                path.display(),
                e
            ))
        })?;

        validate_completeness(&on_disk.table)?;

        Ok(Self {
            version: on_disk.version,
            table: on_disk.table,
        })
    }

    /// Per-phoneme certainty cutoff map for `level`. Cloned so callers do not
    /// hold a borrow on the table; the map is ≤13 entries so clone cost is
    /// negligible.
    ///
    /// Panics only if `level` is missing from a *constructed* `ThresholdTable`,
    /// which [`Self::load`]'s completeness check rules out by construction.
    pub fn resolve_for_level(&self, level: DifficultyLevel) -> PhonemeThresholds {
        self.table
            .get(&level)
            .cloned()
            .expect("ThresholdTable::load guarantees every DifficultyLevel is present")
    }

    /// Version of the bundled table; surfaced on every [`crate::shared::types::EvaluationResult`]
    /// and on the report upload payload (CL-19, CL-10).
    pub fn version(&self) -> i32 {
        self.version
    }
}

/// Assert every difficulty level has an entry, and every level's map covers
/// every symbol in [`V1_TARGET_PHONEMES`]. Reports all missing cells in one
/// pass so users see the full repair list, not just the first failure.
fn validate_completeness(
    table: &HashMap<DifficultyLevel, PhonemeThresholds>,
) -> Result<(), EvaluationError> {
    let expected_levels = [
        DifficultyLevel::Gentle,
        DifficultyLevel::Standard,
        DifficultyLevel::Strict,
    ];

    let mut missing: Vec<String> = Vec::new();

    for level in &expected_levels {
        match table.get(level) {
            None => missing.push(format!("level={:?}", level)),
            Some(PhonemeThresholds(map)) => {
                for phoneme in V1_TARGET_PHONEMES {
                    if !map.contains_key(*phoneme) {
                        missing.push(format!("({:?}, {})", level, phoneme));
                    }
                }
            }
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(EvaluationError::ThresholdTableLoad(format!(
            "threshold table missing required cells: {}",
            missing.join(", ")
        )))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Load the bundled JSON via its on-disk path relative to the crate root.
    /// `CARGO_MANIFEST_DIR` resolves to `app/src-tauri/` at compile time.
    fn load_bundled_for_test() -> Result<ThresholdTable, EvaluationError> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(manifest_dir).join("resources/threshold_table_v1.json");
        ThresholdTable::load(&path)
    }

    #[test]
    fn bundled_table_loads_and_covers_v1_inventory() {
        let table =
            load_bundled_for_test().expect("bundled threshold_table_v1.json must load cleanly");

        // Every (level, phoneme) cell must be populated. This is the
        // build-time completeness assertion the acceptance bullet names.
        for level in [
            DifficultyLevel::Gentle,
            DifficultyLevel::Standard,
            DifficultyLevel::Strict,
        ] {
            let map = table.resolve_for_level(level.clone());
            for phoneme in V1_TARGET_PHONEMES {
                assert!(
                    map.0.contains_key(*phoneme),
                    "missing cell for ({:?}, {})",
                    level,
                    phoneme
                );
            }
            assert_eq!(
                map.0.len(),
                V1_TARGET_PHONEMES.len(),
                "level {:?} has {} entries, expected exactly {}",
                level,
                map.0.len(),
                V1_TARGET_PHONEMES.len()
            );
        }
    }

    #[test]
    fn resolve_for_level_returns_correct_map() {
        let table = load_bundled_for_test().expect("bundled table loads");

        // Spot-check the restored v3 SPIKE-11 values (== v1 / PR #32).
        let gentle = table.resolve_for_level(DifficultyLevel::Gentle);
        assert_eq!(gentle.0.get("w"), Some(&0.60));
        assert_eq!(gentle.0.get("θ"), Some(&0.005));
        assert_eq!(gentle.0.get("ɹ"), Some(&0.02));

        let standard = table.resolve_for_level(DifficultyLevel::Standard);
        assert_eq!(standard.0.get("w"), Some(&0.81));
        assert_eq!(standard.0.get("dʒ"), Some(&0.84));
        assert_eq!(standard.0.get("ð"), Some(&0.86));

        let strict = table.resolve_for_level(DifficultyLevel::Strict);
        assert_eq!(strict.0.get("v"), Some(&0.98));
        assert_eq!(strict.0.get("ʒ"), Some(&0.45));
        assert_eq!(strict.0.get("ɪ"), Some(&0.92));
    }

    #[test]
    fn version_returns_three() {
        let table = load_bundled_for_test().expect("bundled table loads");
        assert_eq!(table.version(), 3);
    }

    #[test]
    fn load_fails_on_malformed_json() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("bad.json");
        {
            let mut f = std::fs::File::create(&path).expect("create");
            f.write_all(b"{ not valid json").expect("write");
        }

        let err = ThresholdTable::load(&path).expect_err("must reject malformed JSON");
        match err {
            EvaluationError::ThresholdTableLoad(msg) => {
                assert!(
                    msg.contains("could not parse"),
                    "message should mention parse failure, got: {msg}"
                );
            }
            other => panic!("expected ThresholdTableLoad, got {other:?}"),
        }
    }

    #[test]
    fn load_fails_on_missing_phoneme_for_level() {
        // Standard's "θ" entry is intentionally omitted.
        let json = r#"{
            "version": 1,
            "table": {
                "gentle":   {"w":0.60,"i":0.16,"l":0.43,"ʒ":0.01,"v":0.69,"z":0.80,"θ":0.005,"æ":0.47,"ɛ":0.07,"dʒ":0.48,"ɪ":0.20,"ɹ":0.02,"ð":0.06},
                "standard": {"w":0.81,"i":0.72,"l":0.81,"ʒ":0.25,"v":0.93,"z":0.94,            "æ":0.79,"ɛ":0.31,"dʒ":0.84,"ɪ":0.80,"ɹ":0.36,"ð":0.86},
                "strict":   {"w":0.93,"i":0.92,"l":0.96,"ʒ":0.45,"v":0.98,"z":0.97,"θ":0.81,"æ":0.90,"ɛ":0.69,"dʒ":0.91,"ɪ":0.92,"ɹ":0.90,"ð":0.97}
            }
        }"#;

        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("missing_phoneme.json");
        std::fs::write(&path, json).expect("write");

        let err = ThresholdTable::load(&path).expect_err("must reject missing phoneme");
        match err {
            EvaluationError::ThresholdTableLoad(msg) => {
                assert!(
                    msg.contains("missing required cells"),
                    "message should call out missing cells, got: {msg}"
                );
                assert!(
                    msg.contains("Standard") && msg.contains("θ"),
                    "message should name the missing (Standard, θ) pair, got: {msg}"
                );
            }
            other => panic!("expected ThresholdTableLoad, got {other:?}"),
        }
    }

    #[test]
    fn load_fails_on_missing_level() {
        // Strict level entirely omitted.
        let json = r#"{
            "version": 1,
            "table": {
                "gentle":   {"w":0.60,"i":0.16,"l":0.43,"ʒ":0.01,"v":0.69,"z":0.80,"θ":0.005,"æ":0.47,"ɛ":0.07,"dʒ":0.48,"ɪ":0.20,"ɹ":0.02,"ð":0.06},
                "standard": {"w":0.81,"i":0.72,"l":0.81,"ʒ":0.25,"v":0.93,"z":0.94,"θ":0.04,"æ":0.79,"ɛ":0.31,"dʒ":0.84,"ɪ":0.80,"ɹ":0.36,"ð":0.86}
            }
        }"#;

        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("missing_level.json");
        std::fs::write(&path, json).expect("write");

        let err = ThresholdTable::load(&path).expect_err("must reject missing level");
        match err {
            EvaluationError::ThresholdTableLoad(msg) => {
                assert!(
                    msg.contains("Strict"),
                    "message should name the missing Strict level, got: {msg}"
                );
            }
            other => panic!("expected ThresholdTableLoad, got {other:?}"),
        }
    }

    #[test]
    fn load_fails_on_missing_file() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does_not_exist.json");

        let err = ThresholdTable::load(&path).expect_err("must reject missing file");
        match err {
            EvaluationError::ThresholdTableLoad(msg) => {
                assert!(
                    msg.contains("could not read"),
                    "message should mention read failure, got: {msg}"
                );
            }
            other => panic!("expected ThresholdTableLoad, got {other:?}"),
        }
    }
}
