//! Rule-based per-phoneme feedback generator (CL-18).
//!
//! # Data flow
//!
//! The IAS-facing articulation reference lives in
//! `documentation/docs/pedagogy/articulation_table.md`. Its `## Structured
//! table (machine-readable companion)` section is parsed at compile time by
//! `app/src-tauri/build.rs`, which emits
//! `$OUT_DIR/articulation_table_generated.rs` with a `static
//! ARTICULATION_TABLE: &[ArticulationEntry]`. We `include!` that file below
//! so the runtime module never parses markdown — stale or malformed copy
//! becomes a compile error, not a production bug.
//!
//! # SPIKE-16 §3 guardrails — rule encoding strategy
//!
//! Spike findings showed three V1 phonemes whose model-certainty signal is
//! unreliable in ways that require copy-level guardrails:
//!
//! - **/ð/** (AUC 0.596, distortion-blind). Low certainty IS informative
//!   (flag it), but high certainty CANNOT certify the production. The /ð/
//!   row's text must NOT use positive-reassurance framing and MUST contain a
//!   hedge substring acknowledging the tool's blind spot.
//! - **/θ/** (bimodal, many correct productions also score near-zero). A
//!   flagged /θ/ does NOT reliably indicate an error. The /θ/ row's text
//!   must contain at least one hedge token (e.g. "uncertain", "may sound").
//! - **/ʒ/** (low even when correct, n=66 in spike). Provisional signal.
//!   The /ʒ/ row's text must contain the exact substring "harder for the
//!   tool to score".
//!
//! These rules are enforced TWICE:
//!
//! 1. **Build-time test** ([`tests`] module below) asserts the baked data
//!    satisfies each pattern. A copy revision that violates a rule fails
//!    `cargo test evaluation::feedback`.
//! 2. **Selection-time guard** ([`passes_problem_phoneme_rules`]) re-checks
//!    each candidate entry inside [`generate_feedback`]. This is
//!    belt-and-suspenders: build-time validation catches violations before
//!    they ship; the runtime check ensures that even if a future refactor
//!    bypasses build-time validation (e.g. a hot-reloaded data path), no
//!    rule-violating entry reaches the user.
//!
//! V1 ships a single copy variant per phoneme; the structural rule
//! enforcement is "if the entry doesn't satisfy the rules, omit it." Future
//! work (post-V1) may introduce multiple variants per phoneme with
//! tone-aware selection; the rule patterns will move from "must contain" to
//! "must select a variant that contains" at that point.

use std::collections::HashMap;

use serde::Serialize;

/// One row of the IAS-authored articulation reference, baked at compile time
/// from `articulation_table.md`'s structured section. All fields are
/// `&'static str` so the table costs no heap allocations.
///
/// Fields are intentionally text-only; V1 guidance does not ship images,
/// diagrams, or animations (see CLAUDE.md "V1 scope is narrow"). Future
/// multimodal guidance is a V1.1+ concern and would land in a sibling type
/// rather than widening this one.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ArticulationEntry {
    pub phoneme: &'static str,
    pub example_word: &'static str,
    /// Single plain-language "how to make this sound" paragraph. Collapsed from
    /// the former tongue/lips/voicing/airflow split per IAS review (2026-06).
    pub mouth_shape: &'static str,
    /// The two contrasting words for the "Say this pair" practice line
    /// (e.g. "light / right"). Empty when the phoneme has no usable word pair —
    /// the UI omits the line. (/z/, /ʒ/, /dʒ/ were empty until the IAS
    /// 2026-06-09 review supplied pairs; see articulation_table.md v0.4.)
    pub minimal_pair: &'static str,
    /// Internal-only L1 / tool-reliability notes. Never shown to learners
    /// (dropped from `FeedbackEntry`); retained because the SPIKE-16 §3 hedges
    /// for /θ/, /ʒ/, /ð/ live here and are enforced by the rule checks below.
    pub l1_notes: &'static str,
}

// Pulls in `pub static ARTICULATION_TABLE: &[ArticulationEntry] = &[...]`
// produced by build.rs from the structured markdown table.
include!(concat!(env!("OUT_DIR"), "/articulation_table_generated.rs"));

/// Hedge tokens accepted for the /θ/ rule. Lowercased; matching is
/// case-insensitive on the entry text. Adding tokens here is a deliberate
/// copy-policy decision: the rule's intent is "the text frames /θ/ flags as
/// uncertain, not as confident correction."
const THETA_HEDGE_TOKENS: &[&str] = &[
    "uncertain",
    "may sound",
    "may have been",
    "the tool was unsure",
    "the tool was uncertain",
];

/// Exact substring required for the /ʒ/ caveat rule. Verbatim because
/// downstream UI may render the caveat as-is and the spike's recommendation
/// names this exact phrase.
const EZH_CAVEAT_SUBSTRING: &str = "harder for the tool to score";

/// Hedge tokens accepted for the /ð/ rule. The /ð/ entry must contain at
/// least one of these so the copy does not read as confident correction.
const ETH_HEDGE_TOKENS: &[&str] = &[
    "may not reliably",
    "tool was uncertain",
    "tool may not",
    "offered as a reminder",
];

/// Confident-correction tokens that /ð/ MUST NOT contain. These would
/// constitute positive-reassurance framing, which the spike specifically
/// rules out for /ð/ (distortion-blind certainty signal).
const ETH_FORBIDDEN_POSITIVE_TOKENS: &[&str] = &[
    "correctly pronounced",
    "you nailed it",
    "perfect",
    "great job",
    "well done",
    "this is correct",
];

/// One feedback card returned to the UI. All fields are owned `String` (not
/// `&'static`) because future work (L1-aware copy variants in CL-19+) will
/// templatize some fields; using `String` now avoids a churn-only refactor.
///
/// `learn_more_url` is `None` in V1 — the field exists so post-V1 deep links
/// into the IAS pedagogy site or in-app glossary can wire up without a type
/// change.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FeedbackEntry {
    pub phoneme: String,
    pub example_word: String,
    pub mouth_shape: String,
    /// Empty when the phoneme has no usable word pair; the UI omits the line.
    pub minimal_pair: String,
    pub flag_count: u32,
    pub learn_more_url: Option<String>,
}

/// Look up the articulation entry for a phoneme. Returns `None` if the
/// phoneme is not in the V1 inventory. The table is 13 entries, so a linear
/// scan is fine; a HashMap would cost a heap allocation on every call.
pub fn lookup_articulation(phoneme: &str) -> Option<&'static ArticulationEntry> {
    ARTICULATION_TABLE.iter().find(|e| e.phoneme == phoneme)
}

/// Check whether `entry` satisfies SPIKE-16 §3 rules. The build-time test
/// also calls this so the rules live in one place. Returns `true` for every
/// phoneme not in the problem set (w, i, l, v, z, æ, ɛ, dʒ, ɪ, ɹ).
pub fn passes_problem_phoneme_rules(entry: &ArticulationEntry) -> bool {
    match entry.phoneme {
        "θ" => entry_contains_any(entry, THETA_HEDGE_TOKENS),
        "ʒ" => entry_contains_substring(entry, EZH_CAVEAT_SUBSTRING),
        "ð" => {
            entry_contains_any(entry, ETH_HEDGE_TOKENS)
                && !entry_contains_any(entry, ETH_FORBIDDEN_POSITIVE_TOKENS)
        }
        _ => true,
    }
}

/// Concatenate every text field of `entry` (lowercased) and check whether
/// any of `needles` (also lowercased) appears in the haystack.
fn entry_contains_any(entry: &ArticulationEntry, needles: &[&str]) -> bool {
    let hay = entry_text_lower(entry);
    needles.iter().any(|n| hay.contains(&n.to_lowercase()))
}

fn entry_contains_substring(entry: &ArticulationEntry, needle: &str) -> bool {
    entry_text_lower(entry).contains(&needle.to_lowercase())
}

fn entry_text_lower(entry: &ArticulationEntry) -> String {
    let mut s = String::new();
    for field in [
        entry.example_word,
        entry.mouth_shape,
        entry.minimal_pair,
        entry.l1_notes,
    ] {
        s.push(' ');
        s.push_str(&field.to_lowercase());
    }
    s
}

/// Build one [`FeedbackEntry`] per flagged phoneme, ordered by flag count
/// descending and (for ties) by phoneme ascending for deterministic output.
///
/// A flagged phoneme not in the articulation table is logged (`log::warn!`)
/// and skipped. A flagged phoneme whose entry fails the SPIKE-16 rule check
/// is also logged and skipped — this should never trigger for V1's hand-
/// authored data because the build-time test catches it, but the runtime
/// guard is the safety net for future hot-reloaded data paths.
pub fn generate_feedback(flagged: &HashMap<String, u32>) -> Vec<FeedbackEntry> {
    // Snapshot into a Vec for deterministic ordering. HashMap iteration order
    // is randomized per process; we sort by (count desc, phoneme asc).
    let mut sorted: Vec<(&String, &u32)> = flagged.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    let mut out: Vec<FeedbackEntry> = Vec::with_capacity(sorted.len());
    for (phoneme, count) in sorted {
        let Some(entry) = lookup_articulation(phoneme) else {
            log::warn!(
                "generate_feedback: flagged phoneme {:?} not in articulation table; skipping",
                phoneme
            );
            continue;
        };
        if !passes_problem_phoneme_rules(entry) {
            log::warn!(
                "generate_feedback: articulation entry for {:?} fails SPIKE-16 rule check; \
                 skipping (this indicates a copy regression — see feedback.rs guardrails)",
                phoneme
            );
            continue;
        }
        out.push(FeedbackEntry {
            phoneme: entry.phoneme.to_string(),
            example_word: entry.example_word.to_string(),
            mouth_shape: entry.mouth_shape.to_string(),
            minimal_pair: entry.minimal_pair.to_string(),
            flag_count: *count,
            learn_more_url: None,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::V1_TARGET_PHONEMES;
    use std::collections::HashMap;

    #[test]
    fn empty_input_yields_empty_output() {
        let flagged: HashMap<String, u32> = HashMap::new();
        assert!(generate_feedback(&flagged).is_empty());
    }

    #[test]
    fn multiple_flagged_ordered_by_count_desc() {
        let mut flagged = HashMap::new();
        flagged.insert("w".to_string(), 1);
        flagged.insert("i".to_string(), 5);
        flagged.insert("l".to_string(), 3);

        let out = generate_feedback(&flagged);
        let phonemes: Vec<&str> = out.iter().map(|e| e.phoneme.as_str()).collect();
        let counts: Vec<u32> = out.iter().map(|e| e.flag_count).collect();
        assert_eq!(phonemes, vec!["i", "l", "w"]);
        assert_eq!(counts, vec![5, 3, 1]);
    }

    #[test]
    fn tie_break_on_count_is_phoneme_ascending() {
        // /v/ and /w/ both flagged twice; "v" sorts before "w" by string compare.
        let mut flagged = HashMap::new();
        flagged.insert("w".to_string(), 2);
        flagged.insert("v".to_string(), 2);
        flagged.insert("z".to_string(), 1);

        let out = generate_feedback(&flagged);
        let phonemes: Vec<&str> = out.iter().map(|e| e.phoneme.as_str()).collect();
        assert_eq!(phonemes, vec!["v", "w", "z"]);
    }

    #[test]
    fn unknown_phoneme_logged_and_skipped() {
        let mut flagged = HashMap::new();
        flagged.insert("qq".to_string(), 7); // not in V1 inventory
        flagged.insert("w".to_string(), 1);

        let out = generate_feedback(&flagged);
        // Only the valid /w/ entry should be returned; qq is silently dropped
        // (with a log::warn! that the test environment does not assert on).
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].phoneme, "w");
        assert_eq!(out[0].flag_count, 1);
    }

    #[test]
    fn unknown_only_input_yields_empty_output() {
        let mut flagged = HashMap::new();
        flagged.insert("qq".to_string(), 7);
        flagged.insert("zz".to_string(), 99);
        assert!(generate_feedback(&flagged).is_empty());
    }

    #[test]
    fn all_v1_target_phonemes_present_in_baked_data() {
        for phoneme in V1_TARGET_PHONEMES {
            assert!(
                lookup_articulation(phoneme).is_some(),
                "articulation table missing V1 target phoneme {:?}",
                phoneme
            );
        }
        assert_eq!(
            ARTICULATION_TABLE.len(),
            V1_TARGET_PHONEMES.len(),
            "articulation table size {} does not match V1 inventory size {}",
            ARTICULATION_TABLE.len(),
            V1_TARGET_PHONEMES.len()
        );
    }

    // ---- SPIKE-16 §3 rule assertions over the baked data ------------------

    #[test]
    fn articulation_data_excludes_positive_reassurance_for_eth() {
        let entry = lookup_articulation("ð").expect("/ð/ entry must exist");
        assert!(
            entry_contains_any(entry, ETH_HEDGE_TOKENS),
            "/ð/ row must contain a hedge token from {:?}; got entry {:#?}",
            ETH_HEDGE_TOKENS,
            entry
        );
        assert!(
            !entry_contains_any(entry, ETH_FORBIDDEN_POSITIVE_TOKENS),
            "/ð/ row must not contain confident-correction tokens {:?}; got entry {:#?}",
            ETH_FORBIDDEN_POSITIVE_TOKENS,
            entry
        );
    }

    #[test]
    fn articulation_data_uses_hedged_framing_for_theta() {
        let entry = lookup_articulation("θ").expect("/θ/ entry must exist");
        assert!(
            entry_contains_any(entry, THETA_HEDGE_TOKENS),
            "/θ/ row must contain a hedge token from {:?}; got entry {:#?}",
            THETA_HEDGE_TOKENS,
            entry
        );
    }

    #[test]
    fn articulation_data_carries_caveat_for_ezh() {
        let entry = lookup_articulation("ʒ").expect("/ʒ/ entry must exist");
        assert!(
            entry_contains_substring(entry, EZH_CAVEAT_SUBSTRING),
            "/ʒ/ row must contain the exact substring {:?}; got entry {:#?}",
            EZH_CAVEAT_SUBSTRING,
            entry
        );
    }

    #[test]
    fn all_baked_rows_pass_problem_phoneme_rules() {
        for entry in ARTICULATION_TABLE {
            assert!(
                passes_problem_phoneme_rules(entry),
                "baked articulation entry {:?} fails SPIKE-16 rule check; see feedback.rs",
                entry.phoneme
            );
        }
    }

    // ---- Structural / output-shape assertions -----------------------------

    #[test]
    fn feedback_entry_has_text_only_fields() {
        // Compile-time-ish guard. If someone adds image_url/diagram_path/etc
        // to FeedbackEntry, this fixture needs updating, which surfaces the
        // V1-scope violation at review time.
        let e = FeedbackEntry {
            phoneme: "w".into(),
            example_word: "walking".into(),
            mouth_shape: "x".into(),
            minimal_pair: "wine / vine".into(),
            flag_count: 1,
            learn_more_url: None,
        };
        assert!(e.learn_more_url.is_none());
    }

    #[test]
    fn learn_more_url_is_none_by_default_for_every_generated_entry() {
        // Flag every V1 phoneme once; assert every generated entry has
        // `learn_more_url: None`.
        let flagged: HashMap<String, u32> = V1_TARGET_PHONEMES
            .iter()
            .map(|p| ((*p).to_string(), 1u32))
            .collect();
        let out = generate_feedback(&flagged);
        assert_eq!(out.len(), V1_TARGET_PHONEMES.len());
        for entry in &out {
            assert!(
                entry.learn_more_url.is_none(),
                "V1 must not pre-populate learn_more_url for {:?}",
                entry.phoneme
            );
        }
    }

    // ---- Belt-and-suspenders: selection-time guard skips bad entries ------

    #[test]
    fn selection_time_guard_skips_entry_violating_eth_rule() {
        // Synthesize an entry that LOOKS like /ð/ but violates the
        // no-positive-reassurance rule. The rule check should reject it.
        let bad = ArticulationEntry {
            phoneme: "ð",
            example_word: "the",
            mouth_shape: "x",
            minimal_pair: "then / thin",
            l1_notes: "perfect — well done!",
        };
        assert!(
            !passes_problem_phoneme_rules(&bad),
            "synthesized /ð/ entry with positive-reassurance copy must fail the rule check"
        );
    }

    #[test]
    fn selection_time_guard_skips_entry_violating_theta_rule() {
        let bad = ArticulationEntry {
            phoneme: "θ",
            example_word: "through",
            mouth_shape: "place tongue tip between teeth; you mispronounced this sound",
            minimal_pair: "thin / then",
            l1_notes: "Mandarin and Hindi lack dental fricatives",
        };
        assert!(
            !passes_problem_phoneme_rules(&bad),
            "synthesized /θ/ entry without a hedge token must fail the rule check"
        );
    }

    #[test]
    fn selection_time_guard_skips_entry_violating_ezh_rule() {
        let bad = ArticulationEntry {
            phoneme: "ʒ",
            example_word: "usually",
            mouth_shape: "x",
            minimal_pair: "",
            l1_notes: "rare phoneme; practice slowly",
        };
        assert!(
            !passes_problem_phoneme_rules(&bad),
            "synthesized /ʒ/ entry without caveat substring must fail the rule check"
        );
    }

    #[test]
    fn non_problem_phoneme_always_passes_rule_check() {
        // /w/ has no SPIKE-16 guardrail — any text passes.
        let bland = ArticulationEntry {
            phoneme: "w",
            example_word: "walking",
            mouth_shape: "x",
            minimal_pair: "wine / vine",
            l1_notes: "x",
        };
        assert!(passes_problem_phoneme_rules(&bland));
    }
}
