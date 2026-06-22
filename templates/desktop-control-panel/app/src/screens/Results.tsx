import { useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useSession } from "../store/useSession";
import ErrorNotice from "../components/ErrorNotice";
import type { AttemptRollup, FeedbackEntry } from "../ipc/types";
import { TARGET_PHONEMES } from "../data/phonemes";
import "./Results.css";

function fmtDuration(ms: number) {
  const totalSeconds = Math.max(0, Math.round(ms / 1000));
  const m = Math.floor(totalSeconds / 60);
  const s = totalSeconds % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

// Render the per-phoneme acoustic-match figure. Null/NaN (no certainty datum)
// shows a bare em-dash; a real value gets the "% match" suffix. Both the
// null-handling and the suffix live here so the call site stays a single call.
function fmtMatch(value: number | null): string {
  if (value === null || Number.isNaN(value)) return "—";
  return `${Math.round(value * 100)}% match`;
}

type PhonemeRow = {
  phoneme: string;
  rollup: AttemptRollup;
  feedback: FeedbackEntry | null;
};

export default function Results() {
  const nav = useNavigate();
  const recordingUrl = useSession((s) => s.recordingObjectUrl);
  const recordingDurationMs = useSession((s) => s.recordingDurationMs);
  const lastEvaluation = useSession((s) => s.lastEvaluation);
  const lastFeedback = useSession((s) => s.lastFeedback);
  const lastEvalError = useSession((s) => s.lastEvalError);

  // Preview-only = the simulated/permission-denied path. recorder.stop()
  // returns durationMs: 0 when MediaRecorder never started, so a zero-length
  // session is the signal that nothing was captured. Reload on /results
  // clears recordingDurationMs (volatile) but keeps lastEvaluation and
  // lastEvalError (both persisted via BRIDGE-1's partialize list). Treat the
  // run as preview-only ONLY when all three signals are absent — no live
  // duration, no persisted evaluation, AND no persisted eval error.
  // A persisted eval error means a real session ran but analysis failed;
  // routing to preview-only would hide the diagnostic on reload.
  const isPreviewOnly =
    lastEvaluation === null &&
    recordingDurationMs === 0 &&
    lastEvalError === null;

  // Edge case: the user ran a real session but eval:error fired (or
  // end_session itself failed) — there's a duration but no evaluation.
  // Render a softer fallback card rather than crashing.
  const hasEvalResult = lastEvaluation !== null;

  // Build the per-phoneme rows. Preserve the Rust-side ordering
  // (flagged_phonemes_ordered comes back sorted by flag_count desc); for any
  // attempt rows that aren't in the flagged list we append them after,
  // sorted by mean_certainty ascending (lowest-confidence first).
  const phonemeRows = useMemo<PhonemeRow[]>(() => {
    if (!lastEvaluation) return [];
    const feedbackByPhoneme = new Map<string, FeedbackEntry>();
    for (const f of lastFeedback ?? []) {
      feedbackByPhoneme.set(f.phoneme, f);
    }

    const attempts = lastEvaluation.phoneme_attempts;
    const flaggedOrdered = lastEvaluation.flagged_phonemes_ordered ?? [];
    const seen = new Set<string>();
    const rows: PhonemeRow[] = [];

    for (const f of flaggedOrdered) {
      const rollup = attempts[f.phoneme];
      if (!rollup) continue;
      rows.push({
        phoneme: f.phoneme,
        rollup,
        feedback: feedbackByPhoneme.get(f.phoneme) ?? null,
      });
      seen.add(f.phoneme);
    }

    const remaining = Object.entries(attempts)
      .filter(([p]) => !seen.has(p))
      .sort((a, b) => {
        const ca = a[1].mean_certainty ?? Number.POSITIVE_INFINITY;
        const cb = b[1].mean_certainty ?? Number.POSITIVE_INFINITY;
        return ca - cb;
      });
    for (const [phoneme, rollup] of remaining) {
      rows.push({
        phoneme,
        rollup,
        feedback: feedbackByPhoneme.get(phoneme) ?? null,
      });
    }

    return rows;
  }, [lastEvaluation, lastFeedback]);

  // Persisted session history is authoritative from SQLite: Progress loads it
  // via getSessionHistory() on mount (#117), so this screen no longer writes a
  // client-fabricated session row. Advancing just navigates to /progress.
  const onSeeProgress = () => {
    nav("/progress");
  };

  return (
    <div className="screen results-screen">
      <h1>How that session went</h1>
      <p className="lede">
        {isPreviewOnly
          ? "This session was a preview — your microphone wasn't available, so nothing was captured. Grant microphone access and start a new session to record your reading."
          : hasEvalResult
          ? "Your reading was captured and analyzed. You can see your per-sound feedback below."
          : "Your reading was captured, but the on-device analysis didn't complete this time. You can continue to track your practice and try again on your next session."}
      </p>

      {recordingUrl && (
        <div className="card playback-card">
          <div className="playback-label">Listen back</div>
          <audio controls src={recordingUrl} />
        </div>
      )}

      {isPreviewOnly ? (
        <div className="card preview-notice-card">
          <h2>Preview only</h2>
          <p>
            This run was a preview — your microphone wasn't available, so
            nothing was captured. Grant microphone access and start a new
            session to record your reading.
          </p>
        </div>
      ) : (
        <div className="card session-summary-card">
          <div className="session-summary-row">
            <span className="label">Session length</span>
            <span className="value">{fmtDuration(recordingDurationMs)}</span>
          </div>
        </div>
      )}

      {isPreviewOnly ? (
        <div className="card placeholder-card">
          <h2>Per-phoneme analysis</h2>
          <p>
            This run was preview-only; once microphone access is granted, future
            sessions will surface per-sound feedback here.
          </p>
        </div>
      ) : hasEvalResult ? (
        <div className="card phoneme-analysis-card">
          <h2>Per-sound analysis</h2>
          {phonemeRows.length === 0 ? (
            <p className="phoneme-empty">
              No target sounds were detected in this reading. Try recording the
              full passage to get per-sound feedback.
            </p>
          ) : (
            <ul className="phoneme-list">
              {phonemeRows.map((row) => (
                <li key={row.phoneme} className="phoneme-row">
                  <div className="phoneme-row-header">
                    <span className="phoneme-symbol">/{row.phoneme}/</span>
                    <span className="phoneme-certainty">
                      {fmtMatch(row.rollup.mean_certainty)}
                    </span>
                    {row.rollup.flagged > 0 && (
                      <span className="phoneme-flagged-badge">
                        flagged {row.rollup.flagged}×
                      </span>
                    )}
                  </div>
                  {row.feedback ? (
                    <div className="phoneme-feedback">
                      <p className="phoneme-feedback-example">
                        Example: <em>{row.feedback.example_word}</em>
                      </p>
                      <h4 className="phoneme-feedback-heading">
                        How to make this sound
                      </h4>
                      <p className="phoneme-feedback-mouthshape">
                        {row.feedback.mouth_shape}
                      </p>
                      <h4 className="phoneme-feedback-heading">
                        How to Practice
                      </h4>
                      <ul className="phoneme-practice">
                        <li>Say this sound 20 times.</li>
                        <li>
                          Say this word 20 times:{" "}
                          <em>{row.feedback.example_word}</em>
                        </li>
                        {row.feedback.minimal_pair && (
                          <li>
                            Say this pair, making sure to pronounce the sounds
                            differently: <em>{row.feedback.minimal_pair}</em>
                          </li>
                        )}
                      </ul>
                    </div>
                  ) : TARGET_PHONEMES.includes(row.phoneme) ? (
                    <p className="phoneme-feedback-missing">
                      Clear — no correction needed.
                    </p>
                  ) : (
                    <p className="phoneme-feedback-missing">
                      Detected, but not part of the V1 target set — no
                      articulation guidance available.
                    </p>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : (
        <div className="card placeholder-card">
          <h2>Analysis unavailable</h2>
          <p>
            The on-device analyzer didn't complete this session. Your reading
            was still captured; per-sound feedback will be available again on
            your next session.
          </p>
          {lastEvalError && (
            <ErrorNotice
              kind={lastEvalError.kind}
              message={lastEvalError.message}
              variant="inline"
            />
          )}
        </div>
      )}

      <div className="results-actions">
        <button
          className="primary"
          onClick={onSeeProgress}
          disabled={isPreviewOnly}
        >
          See my progress
        </button>
      </div>
    </div>
  );
}
