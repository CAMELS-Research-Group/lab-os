/**
 * Shared test fixtures for the IPC command + event suites.
 *
 * Keeps realistic `EvaluationResult` literals out of the individual test files
 * so the typed round-trip is exercised consistently (commands.test.ts asserts
 * the wrapper passes the shape through; events.test.ts asserts the
 * `eval:done` subscriber forwards a full payload).
 *
 * Spec: ADD §3.6 (EvaluationResult), ADD §3.7 (eval:done payload).
 */

import type { EvaluationResult, FeedbackEntry, SessionId } from "../types";

/**
 * Realistic `EvaluationResult` literal for typed round-trip assertions.
 * Override individual fields per test by spreading: `{ ...makeEvaluationResult(), session_id: "other" }`.
 */
export function makeEvaluationResult(
  overrides: Partial<EvaluationResult> = {}
): EvaluationResult {
  return {
    session_id: "session-xyz" as SessionId,
    started_at: "2026-06-05T12:00:00Z",
    ended_at: "2026-06-05T12:03:45Z",
    duration_seconds: 225.5,
    phoneme_attempts: {},
    difficulty_level: "standard",
    difficulty_thresholds: { θ: 0.7 },
    threshold_table_version: 7,
    reattempt_counts_by_sentence: [0, 1],
    flagged_phonemes_ordered: [],
    highest_error_phoneme: null,
    model_version: "camels-v0.4.0",
    ...overrides,
  };
}

/**
 * Realistic `FeedbackEntry` literal mirroring `evaluation::feedback::FeedbackEntry`
 * on the Rust side. Used for the `eval:done` payload round-trip.
 */
export function makeFeedbackEntry(
  overrides: Partial<FeedbackEntry> = {}
): FeedbackEntry {
  return {
    phoneme: "θ",
    example_word: "thin",
    mouth_shape:
      "Place the tip of your tongue lightly between your upper and lower front teeth and blow air gently through the gap; your throat does not vibrate.",
    minimal_pair: "thin / then",
    flag_count: 2,
    learn_more_url: null,
    ...overrides,
  };
}
