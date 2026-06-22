//! Bundled reference IPA loader + `get_passage` Tauri command.
//!
//! # Build-time → runtime data flow
//!
//! 1. The IAS-owned passage text lives at repo root: `passages/visiting_nyc.txt`.
//! 2. `tools/passages/gen_reference_ipa.py` phonemizes that text with
//!    espeak-ng (pinned 1.52.0, en-us) and emits `passages/visiting_nyc.ipa.json`
//!    next to it. Both files are **committed**; the JSON is regenerated when
//!    the passage changes.
//! 3. `tauri.conf.json` declares both as bundle resources, so they ship inside
//!    the installer (no runtime espeak dependency on the learner's machine).
//! 4. This module loads the JSON via Tauri's `BaseDirectory::Resource` lookup,
//!    validates that the bundled `v1_target_phonemes` array matches the code's
//!    [`V1_TARGET_PHONEMES`] (hard-fail on drift), and exposes:
//!    - [`get_passage`] — the Tauri command that returns the IPC
//!      [`Passage`] (text + per-word IPA) to the frontend.
//!    - [`load_expected_phonemes`] — the flat-with-context view CL-17's
//!      alignment task consumes.
//!
//! Per TRD §4.5.1: the reference IPA is **build-time generated and bundled**,
//! never invoked at runtime, so the client has no espeak runtime dependency.
//! Per ADD §3.10 errors bubble through [`EvaluationError`] → [`AppError`].

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::evaluation::error::EvaluationError;
use crate::evaluation::thresholds::V1_TARGET_PHONEMES;
use crate::shared::error::AppError;
use crate::shared::types::{ExpectedIpaPerWord, Passage};

/// Bundled-resource path, used both at runtime (via `BaseDirectory::Resource`)
/// and in tests (via `CARGO_MANIFEST_DIR`). Kept in one place so the wiring
/// stays in lockstep with `tauri.conf.json`'s `bundle.resources` map.
const BUNDLED_REFERENCE_REL_PATH: &str = "passages/visiting_nyc.ipa.json";

/// On-disk shape of `<passage>.ipa.json`. Kept private; callers receive
/// either the IPC [`Passage`] (via [`load_passage`]) or the flat
/// [`ExpectedPhoneme`] sequence (via [`load_expected_phonemes`]).
#[derive(Debug, Deserialize)]
struct OnDiskBundle {
    passage_file: String,
    /// Surfaced verbatim in
    /// [`EvaluationError::BundledReferenceInventoryMismatch`] so a drift log
    /// line auto-attributes which generator emitted the stale inventory.
    espeak_ng_version: String,
    /// Surfaced verbatim in
    /// [`EvaluationError::BundledReferenceInventoryMismatch`] alongside
    /// `espeak_ng_version` for the same drift-attribution reason.
    espeak_voice: String,
    v1_target_phonemes: Vec<String>,
    expected_ipa_per_word: Vec<OnDiskWord>,
}

#[derive(Debug, Deserialize)]
struct OnDiskWord {
    word: String,
    ipa: Vec<String>,
    is_target: Vec<bool>,
}

/// Flat-with-context view of the reference IPA, one entry per phoneme symbol
/// in passage order. CL-17 (forced alignment + per-phoneme certainty)
/// consumes this — the alignment runs over the flat sequence but needs the
/// word index to attribute each detected occurrence back to a word for the
/// UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedPhoneme {
    /// IPA symbol (e.g. `"ɛ"`, `"dʒ"`). Matches the V1 inventory verbatim for
    /// target positions; carries any other espeak-emitted symbol verbatim for
    /// non-target positions.
    pub symbol: String,
    /// `true` iff `symbol` is in [`V1_TARGET_PHONEMES`]. Pre-computed at
    /// build time so the alignment task does not re-lookup.
    pub is_target: bool,
    /// 0-based word index within the passage.
    pub word_index: usize,
    /// 0-based position within the word's IPA sequence.
    pub position_in_word: usize,
}

/// Load the bundled JSON from `path`, parse, and validate.
///
/// Validation enforces:
/// - `expected_ipa_per_word[i].ipa.len() == expected_ipa_per_word[i].is_target.len()`
///   for every word (the parallel arrays must agree).
/// - The bundled `v1_target_phonemes` matches [`V1_TARGET_PHONEMES`] exactly
///   (order-sensitive). A mismatch hard-fails with
///   [`EvaluationError::BundledReferenceInventoryMismatch`] so a stale bundle
///   never silently tags against the wrong inventory.
fn load_bundle(path: &Path) -> Result<OnDiskBundle, EvaluationError> {
    let bytes = std::fs::read(path).map_err(|e| {
        EvaluationError::BundledReferenceLoad(format!(
            "could not read {}: {}",
            path.display(),
            e
        ))
    })?;

    let bundle: OnDiskBundle = serde_json::from_slice(&bytes).map_err(|e| {
        EvaluationError::BundledReferenceLoad(format!(
            "could not parse {}: {}",
            path.display(),
            e
        ))
    })?;

    let expected: Vec<String> =
        V1_TARGET_PHONEMES.iter().map(|s| (*s).to_string()).collect();
    if bundle.v1_target_phonemes != expected {
        return Err(EvaluationError::BundledReferenceInventoryMismatch {
            bundle: bundle.v1_target_phonemes,
            expected,
            espeak_ng_version: bundle.espeak_ng_version,
            espeak_voice: bundle.espeak_voice,
        });
    }

    for (idx, w) in bundle.expected_ipa_per_word.iter().enumerate() {
        if w.ipa.len() != w.is_target.len() {
            return Err(EvaluationError::BundledReferenceLoad(format!(
                "word[{}] {:?}: ipa len {} != is_target len {}",
                idx,
                w.word,
                w.ipa.len(),
                w.is_target.len(),
            )));
        }
    }

    Ok(bundle)
}

/// Load the bundled reference IPA from `path` and project it to the flat
/// [`ExpectedPhoneme`] sequence CL-17 consumes.
pub fn load_expected_phonemes(path: &Path) -> Result<Vec<ExpectedPhoneme>, EvaluationError> {
    let bundle = load_bundle(path)?;
    let mut out: Vec<ExpectedPhoneme> = Vec::new();
    for (word_index, word) in bundle.expected_ipa_per_word.iter().enumerate() {
        for (position_in_word, (symbol, is_target)) in
            word.ipa.iter().zip(word.is_target.iter()).enumerate()
        {
            out.push(ExpectedPhoneme {
                symbol: symbol.clone(),
                is_target: *is_target,
                word_index,
                position_in_word,
            });
        }
    }
    Ok(out)
}

/// Load the bundled reference IPA from `path` and project it to the IPC
/// [`Passage`] (text + per-word IPA). Reads the passage text from the
/// `passage_file` sibling of `path` so the two stay in lockstep.
pub fn load_passage(path: &Path) -> Result<Passage, EvaluationError> {
    let bundle = load_bundle(path)?;
    let text_path = sibling_path(path, &bundle.passage_file);
    let text = std::fs::read_to_string(&text_path).map_err(|e| {
        EvaluationError::BundledReferenceLoad(format!(
            "could not read passage text {}: {}",
            text_path.display(),
            e
        ))
    })?;

    let expected_ipa_per_word = bundle
        .expected_ipa_per_word
        .into_iter()
        .map(|w| ExpectedIpaPerWord {
            word: w.word,
            ipa: w.ipa,
        })
        .collect();

    Ok(Passage {
        text,
        expected_ipa_per_word,
    })
}

fn sibling_path(reference_json: &Path, sibling_name: &str) -> PathBuf {
    match reference_json.parent() {
        Some(parent) => parent.join(sibling_name),
        None => PathBuf::from(sibling_name),
    }
}

/// Resolve the bundled reference JSON via Tauri's `BaseDirectory::Resource`
/// lookup, then project to the IPC [`Passage`]. The frontend calls this
/// from the passage screen to fetch the read-aloud content + reference IPA.
#[tauri::command]
pub async fn get_passage(app: tauri::AppHandle) -> Result<Passage, AppError> {
    use tauri::Manager as _;
    let path = app
        .path()
        .resolve(BUNDLED_REFERENCE_REL_PATH, tauri::path::BaseDirectory::Resource)
        .map_err(|e| {
            EvaluationError::BundledReferenceLoad(format!(
                "could not resolve bundled resource {}: {}",
                BUNDLED_REFERENCE_REL_PATH, e
            ))
        })?;
    Ok(load_passage(&path)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Resolve the committed bundle via the repo-root `passages/` directory.
    /// `CARGO_MANIFEST_DIR` is `app/src-tauri/`, so the repo root is two
    /// levels up.
    fn bundled_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../passages/visiting_nyc.ipa.json")
    }

    #[test]
    fn bundled_reference_loads_and_deserializes() {
        let bundle = load_bundle(&bundled_path()).expect("bundled reference must load");
        assert_eq!(bundle.passage_file, "visiting_nyc.txt");
        assert_eq!(bundle.espeak_voice, "en-us");
        assert!(!bundle.expected_ipa_per_word.is_empty());
    }

    #[test]
    fn load_expected_phonemes_returns_non_empty_with_sensible_indices() {
        let phonemes = load_expected_phonemes(&bundled_path())
            .expect("expected phonemes must load");
        assert!(!phonemes.is_empty(), "passage must produce at least one phoneme");

        // word_index is monotone non-decreasing.
        let mut prev = 0usize;
        for p in &phonemes {
            assert!(
                p.word_index >= prev,
                "word_index should be monotone: saw {} after {}",
                p.word_index,
                prev
            );
            prev = p.word_index;
        }

        // position_in_word starts at 0 for each word and is contiguous.
        let mut current_word = usize::MAX;
        let mut expected_pos = 0usize;
        for p in &phonemes {
            if p.word_index != current_word {
                current_word = p.word_index;
                expected_pos = 0;
            }
            assert_eq!(
                p.position_in_word, expected_pos,
                "position_in_word should be 0-based and contiguous within a word"
            );
            expected_pos += 1;
        }
    }

    #[test]
    fn load_passage_returns_text_and_non_empty_reference() {
        let passage = load_passage(&bundled_path()).expect("passage must load");
        assert!(!passage.text.is_empty(), "passage text must be non-empty");
        assert!(
            !passage.expected_ipa_per_word.is_empty(),
            "expected_ipa_per_word must be non-empty"
        );
        // Every word entry's IPA list is non-empty.
        for w in &passage.expected_ipa_per_word {
            assert!(
                !w.ipa.is_empty(),
                "word {:?} has empty IPA — espeak-ng emitted nothing",
                w.word
            );
        }
    }

    #[test]
    fn target_tags_match_v1_inventory() {
        // For every position the bundle marks as a target, the symbol must
        // actually be in V1_TARGET_PHONEMES; for every non-target the symbol
        // must NOT be in the inventory.
        let phonemes = load_expected_phonemes(&bundled_path()).expect("load");
        let targets: std::collections::HashSet<&str> =
            V1_TARGET_PHONEMES.iter().copied().collect();
        for p in &phonemes {
            let in_inventory = targets.contains(p.symbol.as_str());
            assert_eq!(
                p.is_target, in_inventory,
                "tagging disagrees with inventory for symbol {:?}", p.symbol
            );
        }
    }

    #[test]
    fn load_fails_on_malformed_json() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("bad.json");
        {
            let mut f = std::fs::File::create(&path).expect("create");
            f.write_all(b"{ not valid json").expect("write");
        }

        let err = load_bundle(&path).expect_err("must reject malformed JSON");
        match err {
            EvaluationError::BundledReferenceLoad(msg) => {
                assert!(
                    msg.contains("could not parse"),
                    "message should mention parse failure, got: {msg}"
                );
            }
            other => panic!("expected BundledReferenceLoad, got {other:?}"),
        }
    }

    #[test]
    fn load_fails_on_missing_file() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does_not_exist.json");

        let err = load_bundle(&path).expect_err("must reject missing file");
        match err {
            EvaluationError::BundledReferenceLoad(msg) => {
                assert!(
                    msg.contains("could not read"),
                    "message should mention read failure, got: {msg}"
                );
            }
            other => panic!("expected BundledReferenceLoad, got {other:?}"),
        }
    }

    #[test]
    fn load_fails_on_inventory_mismatch() {
        // Bundle's v1_target_phonemes is missing the "ð" entry. Must
        // hard-fail with the InventoryMismatch variant rather than silently
        // mis-tagging.
        let json = r#"{
            "passage_file": "fake.txt",
            "espeak_ng_version": "1.52.0",
            "espeak_voice": "en-us",
            "v1_target_phonemes": ["w","i","l","ʒ","v","z","θ","æ","ɛ","dʒ","ɪ","ɹ"],
            "expected_ipa_per_word": []
        }"#;

        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("mismatch.json");
        std::fs::write(&path, json).expect("write");

        let err = load_bundle(&path).expect_err("must reject inventory mismatch");
        match err {
            EvaluationError::BundledReferenceInventoryMismatch {
                bundle,
                expected,
                espeak_ng_version,
                espeak_voice,
            } => {
                assert_eq!(bundle.len(), 12);
                assert_eq!(expected.len(), V1_TARGET_PHONEMES.len());
                assert!(expected.contains(&"ð".to_string()));
                assert_eq!(espeak_ng_version, "1.52.0");
                assert_eq!(espeak_voice, "en-us");
                // The Display impl must surface both attribution fields so a
                // drift log line auto-attributes which generator emitted the
                // stale bundle.
                let rendered = err_to_string(&EvaluationError::BundledReferenceInventoryMismatch {
                    bundle: vec![],
                    expected: vec![],
                    espeak_ng_version: "1.52.0".to_string(),
                    espeak_voice: "en-us".to_string(),
                });
                assert!(
                    rendered.contains("espeak-ng 1.52.0") && rendered.contains("voice en-us"),
                    "Display impl must surface espeak-ng version + voice, got: {rendered}"
                );
            }
            other => panic!("expected BundledReferenceInventoryMismatch, got {other:?}"),
        }
    }

    fn err_to_string(e: &EvaluationError) -> String {
        format!("{e}")
    }

    #[test]
    fn load_fails_on_parallel_array_length_mismatch() {
        let json = r#"{
            "passage_file": "fake.txt",
            "espeak_ng_version": "1.52.0",
            "espeak_voice": "en-us",
            "v1_target_phonemes": ["w","i","l","ʒ","v","z","θ","æ","ɛ","dʒ","ɪ","ɹ","ð"],
            "expected_ipa_per_word": [
                { "word": "bad", "ipa": ["b","æ","d"], "is_target": [false, true] }
            ]
        }"#;

        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("len_mismatch.json");
        std::fs::write(&path, json).expect("write");

        let err = load_bundle(&path).expect_err("must reject parallel-array length mismatch");
        match err {
            EvaluationError::BundledReferenceLoad(msg) => {
                assert!(
                    msg.contains("is_target len") && msg.contains("ipa len"),
                    "message should name the offending lengths, got: {msg}"
                );
            }
            other => panic!("expected BundledReferenceLoad, got {other:?}"),
        }
    }
}
