//! V1 allophone map — bundled loader for the evaluation pipeline.
//!
//! Source of truth: `resources/allophone_map_v1.json`, transcribed verbatim
//! from SPIKE-11 `ACCEPTABLE_ALLOPHONES` at
//! `spike/eval/l2arctic_accuracy.py:252-270`.
//!
//! The JSON is compiled into the binary via [`include_str!`] so there is no
//! disk access at runtime — in contrast with the threshold table, which is
//! loaded from disk at startup (because it may be regenerated per IAS adapter).
//! The allophone map is V1-stable; it is only expected to change when the model
//! vocab changes (post-V1 adapter path).
//!
//! [`AllophoneMap::load`] accepts `vocab: &[String]` so tests can inject a
//! synthetic vocab; production callers pass the slice returned by
//! `orchestrator::load_vocab`.

use std::collections::HashMap;

use serde::Deserialize;

use crate::evaluation::thresholds::V1_TARGET_PHONEMES;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by the allophone map loader. Kept separate from
/// [`crate::evaluation::error::EvaluationError`] so callers can distinguish
/// map-parse failures (config problem) from runtime inference failures.
#[derive(Debug, thiserror::Error)]
pub enum AllophoneError {
    /// The bundled JSON could not be parsed.
    #[error("could not parse allophone map: {0}")]
    Parse(String),

    /// An allophone string listed for `target` is absent from the model vocab.
    /// This is a hard configuration error: the JSON was authored against a
    /// different vocab than the one bundled with this binary.
    #[error(
        "allophone symbol {symbol:?} (for target {target:?}) is not in the model vocab"
    )]
    SymbolNotInVocab { target: String, symbol: String },

    /// One or more required V1 target symbols are missing from the allophone
    /// map. The payload is a comma-joined list so the full repair surface is
    /// visible in a single error (mirrors `thresholds::validate_completeness`).
    #[error("missing required target symbol(s) in allophone map: {0}")]
    MissingTarget(String),

    /// Unsupported schema_version in the allophone map header. V1 expects 1.
    #[error("unsupported allophone map schema_version {0}: expected 1")]
    UnsupportedSchemaVersion(i32),
}

// ---------------------------------------------------------------------------
// On-disk (deserialization) shape
// ---------------------------------------------------------------------------

/// On-disk shape of `allophone_map_v1.json`. Private — callers see the
/// validated [`AllophoneMap`] instead.
#[derive(Debug, Deserialize)]
struct OnDiskMap {
    /// Header metadata. `schema_version` is enforced in `load`; the rest is
    /// retained for provenance tracing only.
    _header: OnDiskHeader,
    allophones: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OnDiskHeader {
    schema_version: i32,
    /// Informational only — not validated at runtime; retained for
    /// provenance tracing when debugging JSON mismatches.
    #[allow(dead_code)]
    source_spike: String,
    #[allow(dead_code)]
    source_file: String,
    #[allow(dead_code)]
    notes: String,
}

// ---------------------------------------------------------------------------
// Public type
// ---------------------------------------------------------------------------

/// Validated, in-memory allophone map. Every instance is guaranteed to:
/// - Cover every symbol in [`V1_TARGET_PHONEMES`].
/// - Have resolved each allophone string to a valid vocab column index.
/// - Contain no duplicate column indices within a single target's column set.
///
/// Constructed via [`Self::load`].
#[derive(Debug, Clone)]
pub struct AllophoneMap {
    /// Maps each V1 target IPA symbol to the sorted, deduplicated list of
    /// vocab column indices that are acceptable renditions of that target.
    columns: HashMap<String, Vec<usize>>,
    schema_version: i32,
}

impl AllophoneMap {
    /// Parse the bundled JSON and resolve allophone strings to vocab column
    /// indices using `vocab`.
    ///
    /// Fails loudly ([`AllophoneError`]) if:
    /// - The JSON cannot be parsed.
    /// - Any V1 target symbol is absent from the map.
    /// - Any allophone string is missing from `vocab`.
    ///
    /// Column-index sets are deduplicated; if two allophone strings hash to
    /// the same vocab id the resulting `Vec<usize>` will not double-count.
    pub fn load(json: &str, vocab: &[String]) -> Result<Self, AllophoneError> {
        let on_disk: OnDiskMap = serde_json::from_str(json)
            .map_err(|e| AllophoneError::Parse(e.to_string()))?;

        if on_disk._header.schema_version != 1 {
            return Err(AllophoneError::UnsupportedSchemaVersion(
                on_disk._header.schema_version,
            ));
        }

        // Build a single-pass lookup from symbol string → vocab index.
        let sym_to_idx: HashMap<&str, usize> = vocab
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect();

        // Assert every V1 target is present in the map. Collect all missing
        // in one pass so authors see the full repair list, not just the
        // first failure (mirrors thresholds::validate_completeness).
        let mut missing_targets: Vec<&str> = Vec::new();
        for target in V1_TARGET_PHONEMES {
            if !on_disk.allophones.contains_key(*target) {
                missing_targets.push(*target);
            }
        }
        if !missing_targets.is_empty() {
            return Err(AllophoneError::MissingTarget(missing_targets.join(", ")));
        }

        // Resolve each entry to column indices, failing loudly on any unknown
        // allophone string.
        let mut columns: HashMap<String, Vec<usize>> =
            HashMap::with_capacity(on_disk.allophones.len());

        for (target, allophone_strings) in &on_disk.allophones {
            let mut idxs: Vec<usize> = Vec::with_capacity(allophone_strings.len());
            for sym in allophone_strings {
                match sym_to_idx.get(sym.as_str()) {
                    Some(&idx) => idxs.push(idx),
                    None => {
                        return Err(AllophoneError::SymbolNotInVocab {
                            target: target.clone(),
                            symbol: sym.clone(),
                        });
                    }
                }
            }
            // Deduplicate: sort first so dedup is O(n).
            idxs.sort_unstable();
            idxs.dedup();
            columns.insert(target.clone(), idxs);
        }

        Ok(Self {
            columns,
            schema_version: on_disk._header.schema_version,
        })
    }

    /// Returns the sorted, deduplicated slice of vocab column indices that are
    /// acceptable renditions of `target`. Returns `None` if `target` is not a
    /// V1 phoneme (should not happen at production call sites after construction).
    pub fn columns_for_target(&self, target: &str) -> Option<&[usize]> {
        self.columns.get(target).map(|v| v.as_slice())
    }

    /// Schema version from the JSON header. `1` for the V1 hand-authored map.
    pub fn schema_version(&self) -> i32 {
        self.schema_version
    }
}

// ---------------------------------------------------------------------------
// Build-time embedded JSON (used by production callers via `load_bundled`)
// ---------------------------------------------------------------------------

/// The bundled allophone map JSON, compiled into the binary.
const BUNDLED_JSON: &str =
    include_str!("../../resources/allophone_map_v1.json");

/// Load the bundled allophone map against `vocab`. This is the production
/// entry point; tests use [`AllophoneMap::load`] directly with synthetic
/// inputs.
pub fn load_bundled(vocab: &[String]) -> Result<AllophoneMap, AllophoneError> {
    AllophoneMap::load(BUNDLED_JSON, vocab)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal synthetic vocab that covers every symbol the V1 allophone map
    /// references. Built programmatically from the known SPIKE-11 set so the
    /// test doesn't depend on the real model vocab.
    fn synthetic_vocab() -> Vec<String> {
        // Include all allophone strings from the SPIKE-11 dict plus enough
        // padding for realistic index arithmetic.
        let symbols = [
            "<pad>", "<s>", "</s>", "<unk>",
            // V1 target symbols
            "w", "i", "l", "ʒ", "v", "z", "θ", "æ", "ɛ", "dʒ", "ɪ", "ɹ", "ð",
            // Extra allophones for ɹ
            "r", "ɾ", "ɚ", "ɑːɹ", "ɔːɹ", "oːɹ", "ɛɹ", "ɪɹ", "ʊɹ",
            // Extra allophones for l and ɪ
            "əl", "ᵻ",
            // Extra allophone for i
            "iː",
        ];
        symbols.iter().map(|s| s.to_string()).collect()
    }

    // (a) All 13 V1 target symbols present in the loaded map.
    #[test]
    fn all_13_target_symbols_present() {
        let vocab = synthetic_vocab();
        let map = AllophoneMap::load(BUNDLED_JSON, &vocab)
            .expect("bundled JSON must load against synthetic vocab");

        for target in V1_TARGET_PHONEMES {
            assert!(
                map.columns_for_target(target).is_some(),
                "target symbol {target:?} missing from loaded AllophoneMap"
            );
        }
        assert_eq!(
            V1_TARGET_PHONEMES.len(),
            13,
            "V1_TARGET_PHONEMES must have exactly 13 symbols"
        );
    }

    // (b) Every allophone string resolves to a real vocab id against the
    //     bundled manifest (model_vocab_v1.json loaded via CARGO_MANIFEST_DIR).
    #[test]
    fn all_allophones_resolve_against_real_vocab() {
        // Load the real vocab from disk (test-only; production uses load_vocab).
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let vocab_path =
            std::path::Path::new(manifest_dir).join("resources/model_vocab_v1.json");
        let bytes = std::fs::read(&vocab_path)
            .expect("model_vocab_v1.json must be readable from CARGO_MANIFEST_DIR");

        #[derive(serde::Deserialize)]
        struct VocabFile {
            vocabulary: Vec<String>,
        }
        let parsed: VocabFile =
            serde_json::from_slice(&bytes).expect("model_vocab_v1.json must be valid JSON");

        let map = load_bundled(&parsed.vocabulary)
            .expect("bundled allophone map must load cleanly against real model vocab");

        // Spot-check that each target returns at least one column index and
        // that each index is within the vocab bounds.
        let vocab_len = parsed.vocabulary.len();
        for target in V1_TARGET_PHONEMES {
            let cols = map
                .columns_for_target(target)
                .expect("every V1 target must have a column set");
            assert!(
                !cols.is_empty(),
                "column set for {target:?} must be non-empty"
            );
            for &idx in cols {
                assert!(
                    idx < vocab_len,
                    "column index {idx} for target {target:?} is out of vocab bounds ({vocab_len})"
                );
            }
        }
    }

    // (c) Loader fails loudly on a synthetic manifest missing one allophone.
    #[test]
    fn fails_loudly_on_vocab_missing_allophone() {
        // Vocab that omits "ɾ" (an allophone of ɹ).
        let vocab: Vec<String> = [
            "<pad>", "<s>", "</s>", "<unk>",
            "w", "i", "l", "ʒ", "v", "z", "θ", "æ", "ɛ", "dʒ", "ɪ", "ɹ", "ð",
            // "ɾ" deliberately omitted
            "r", "ɚ", "ɑːɹ", "ɔːɹ", "oːɹ", "ɛɹ", "ɪɹ", "ʊɹ",
            "əl", "ᵻ", "iː",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let err = AllophoneMap::load(BUNDLED_JSON, &vocab)
            .expect_err("must reject vocab missing an allophone");

        match err {
            AllophoneError::SymbolNotInVocab { target, symbol } => {
                assert_eq!(
                    symbol, "ɾ",
                    "error must name the missing symbol; got {symbol:?}"
                );
                assert_eq!(
                    target, "ɹ",
                    "error must name the target phoneme; got {target:?}"
                );
            }
            other => panic!("expected SymbolNotInVocab, got {other:?}"),
        }
    }

    // (d) JSON parses against a synthetic minimal manifest.
    #[test]
    fn json_parses_against_synthetic_minimal_manifest() {
        let vocab = synthetic_vocab();
        let map = AllophoneMap::load(BUNDLED_JSON, &vocab)
            .expect("bundled JSON must parse against synthetic minimal vocab");

        assert_eq!(map.schema_version(), 1);

        // Verify ɹ has 10 allophones and they are deduplicated.
        let rhotic_cols = map
            .columns_for_target("ɹ")
            .expect("ɹ must be present");
        assert_eq!(
            rhotic_cols.len(),
            10,
            "ɹ should have 10 distinct allophone columns, got {}",
            rhotic_cols.len()
        );

        // Verify single-element entries are still lists of one.
        for single in &["v", "w", "θ", "ð", "ʒ", "z", "ɛ", "æ"] {
            let cols = map
                .columns_for_target(single)
                .expect("single-element targets must be present");
            assert_eq!(
                cols.len(),
                1,
                "{single:?} should have exactly 1 column, got {}",
                cols.len()
            );
        }
    }

    // Dedup: duplicate allophone strings in the input JSON resolve to a
    // single column index in the loaded map.
    #[test]
    fn deduplicates_columns_when_vocab_aliases_collide() {
        let json = r#"{
            "_header": {
                "schema_version": 1,
                "source_spike": "test",
                "source_file": "test",
                "notes": "dedup test"
            },
            "allophones": {
                "w":  ["w", "w"],
                "i":  ["i"],
                "l":  ["l"],
                "ʒ":  ["ʒ"],
                "v":  ["v"],
                "z":  ["z"],
                "θ":  ["θ"],
                "æ":  ["æ"],
                "ɛ":  ["ɛ"],
                "dʒ": ["dʒ"],
                "ɪ":  ["ɪ"],
                "ɹ":  ["ɹ"],
                "ð":  ["ð"]
            }
        }"#;

        let vocab = synthetic_vocab();
        let map = AllophoneMap::load(json, &vocab)
            .expect("dedup-test JSON must load");

        let w_cols = map
            .columns_for_target("w")
            .expect("w must be present");
        assert_eq!(
            w_cols.len(),
            1,
            "duplicate allophone strings must be deduplicated; got {} columns for 'w'",
            w_cols.len()
        );
    }

    // Missing-target error names every absent V1 symbol (not just the first),
    // mirroring thresholds::validate_completeness.
    #[test]
    fn missing_target_error_lists_all_missing_symbols() {
        // JSON omits both ð and ʒ from the allophones map.
        let json = r#"{
            "_header": {
                "schema_version": 1,
                "source_spike": "test",
                "source_file": "test",
                "notes": "missing-targets test"
            },
            "allophones": {
                "w":  ["w"],
                "i":  ["i"],
                "l":  ["l"],
                "v":  ["v"],
                "z":  ["z"],
                "θ":  ["θ"],
                "æ":  ["æ"],
                "ɛ":  ["ɛ"],
                "dʒ": ["dʒ"],
                "ɪ":  ["ɪ"],
                "ɹ":  ["ɹ"]
            }
        }"#;

        let vocab = synthetic_vocab();
        let err = AllophoneMap::load(json, &vocab)
            .expect_err("must reject map missing required targets");

        match err {
            AllophoneError::MissingTarget(msg) => {
                assert!(
                    msg.contains("ð") && msg.contains("ʒ"),
                    "error message must name BOTH missing targets, got: {msg}"
                );
            }
            other => panic!("expected MissingTarget, got {other:?}"),
        }
    }

    // Schema-version guard fires on any value other than 1.
    #[test]
    fn fails_loudly_on_unsupported_schema_version() {
        let json = r#"{
            "_header": {
                "schema_version": 2,
                "source_spike": "test",
                "source_file": "test",
                "notes": "schema-version test"
            },
            "allophones": {
                "w":  ["w"], "i":  ["i"], "l":  ["l"], "ʒ":  ["ʒ"],
                "v":  ["v"], "z":  ["z"], "θ":  ["θ"], "æ":  ["æ"],
                "ɛ":  ["ɛ"], "dʒ": ["dʒ"], "ɪ": ["ɪ"], "ɹ": ["ɹ"], "ð": ["ð"]
            }
        }"#;

        let vocab = synthetic_vocab();
        let err = AllophoneMap::load(json, &vocab)
            .expect_err("must reject schema_version != 1");

        match err {
            AllophoneError::UnsupportedSchemaVersion(v) => {
                assert_eq!(v, 2, "error must carry the rejected version");
            }
            other => panic!("expected UnsupportedSchemaVersion, got {other:?}"),
        }
    }
}
