/**
 * Unit tests for the Results screen — unflagged-target vs non-target copy.
 *
 * Regression guard for #115: phonemes in the V1 target set that the learner
 * pronounced acceptably (no flags from Rust, so feedback === null) must show
 * "Clear — no correction needed." rather than the non-target copy.
 *
 * Test matrix:
 *   /θ/  — unflagged V1 target  → "Clear — no correction needed."
 *   /t/  — non-target phoneme   → "not part of the V1 target set"
 */

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import Results from "../Results";
import { useSession } from "../../store/useSession";
import type { EvaluationResult } from "../../ipc/types";

// Minimal EvaluationResult containing only the fields Results.tsx reads.
// Cast via `as unknown as EvaluationResult` to avoid filling every field.
const minimalEval = {
  session_id: "test-session-id",
  started_at: "2026-06-09T00:00:00Z",
  ended_at: "2026-06-09T00:01:00Z",
  duration_seconds: 60,
  phoneme_attempts: {
    // Unflagged V1 target — Rust emits no FeedbackEntry for it.
    θ: { occurrences: 3, flagged: 0, mean_certainty: 0.92 },
    // Genuine non-target — also no FeedbackEntry.
    t: { occurrences: 4, flagged: 0, mean_certainty: 0.95 },
  },
  flagged_phonemes_ordered: [],
  difficulty_level: "standard",
  difficulty_thresholds: {},
  threshold_table_version: 1,
  reattempt_counts_by_sentence: [],
  highest_error_phoneme: null,
  model_version: "test",
} as unknown as EvaluationResult;

function renderResults() {
  return render(
    <MemoryRouter initialEntries={["/results"]}>
      <Results />
    </MemoryRouter>
  );
}

describe("Results screen — unflagged-target vs non-target copy", () => {
  beforeEach(() => {
    useSession.setState({
      lastEvaluation: minimalEval,
      lastFeedback: [],
      // Non-zero duration so the component is not treated as preview-only.
      recordingDurationMs: 60000,
      recordingObjectUrl: null,
      lastEvalError: null,
    });
  });

  it("shows 'Clear — no correction needed.' for an unflagged V1 target (/θ/)", () => {
    renderResults();
    expect(screen.getByText(/clear — no correction needed\./i)).toBeInTheDocument();
  });

  it("does NOT show the non-target copy for /θ/", () => {
    renderResults();
    // The /θ/ row must not carry the non-target message.
    // We find all instances of the non-target text and verify /θ/ is not among them.
    // Simplest: the non-target text should only appear once (for /t/), not twice.
    const nonTargetMessages = screen.queryAllByText(/not part of the V1 target set/i);
    // The /θ/ phoneme row should not contain this copy — only /t/ should.
    expect(nonTargetMessages).toHaveLength(1);
  });

  it("shows 'not part of the V1 target set' for a genuine non-target (/t/)", () => {
    renderResults();
    expect(
      screen.getByText(/not part of the V1 target set/i)
    ).toBeInTheDocument();
  });
});
