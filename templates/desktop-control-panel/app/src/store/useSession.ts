import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { EvaluationResult, FeedbackEntry } from "../ipc/types";

// Snake_case field names mirror the CL-4 Rust IPC payload type so the demo's
// persisted shape is forward-compatible with production storage, even though
// the demo doesn't IPC anything yet.
export type SessionSummary = {
  session_id: string;
  ended_at: string;
  duration_seconds: number;
};

export type SessionState = {
  // First-run profile (FRD §4.2). The only persisted personal data.
  l1: string;
  regionalVariety: string;
  hasCompletedFirstRun: boolean;

  // First-run gating (CL-24, offline reviewer build). `consentAcknowledged`
  // is a LOCAL acknowledgment only — this build has no backend, so the
  // consent screen records acceptance here and does not call `accept_consent`
  // or do any Rust identity work. `modelReady` flips true once the bundled
  // ONNX model has finished its first-run self-download (or was already
  // cached); it gates entry into the practice flow so a launch with no model
  // routes to the download screen.
  consentAcknowledged: boolean;
  modelReady: boolean;

  // Persisted session history (DEMO-3). Stored oldest-first.
  sessions: SessionSummary[];

  // Volatile per-session state — not persisted; recreated each demo run.
  recordingObjectUrl: string | null;
  recordingDurationMs: number;

  // BRIDGE-1: most recent real eval result + feedback. Persisted so a window
  // reload while sitting on /results still shows the per-phoneme content.
  lastEvaluation: EvaluationResult | null;
  lastFeedback: FeedbackEntry[] | null;

  // Surface for `eval:error` payloads (and `end_session` RPC rejections,
  // tagged with `kind: "end_session_failed"`). Persisted alongside
  // lastEvaluation so a reload after a failure preserves the diagnostic.
  // Cleared automatically on a successful setLastEvaluation(result, ...).
  lastEvalError: { kind: string; message: string } | null;

  setL1: (l1: string, variety: string) => void;
  acknowledgeConsent: () => void;
  setModelReady: () => void;
  setRecording: (url: string | null, durationMs: number) => void;
  setSessions: (sessions: SessionSummary[]) => void;
  setLastEvaluation: (
    result: EvaluationResult | null,
    feedback: FeedbackEntry[] | null
  ) => void;
  setLastEvalError: (err: { kind: string; message: string } | null) => void;
  // Clears the in-memory session history + last-eval slots so the UI reflects
  // an on-disk session-data wipe without a restart. Does NOT touch first-run
  // identity/preferences — that is `resetAll`'s job. The two clears are
  // intentionally independent (see Settings "Clear session data" vs "Reset
  // app / UI state").
  clearSessions: () => void;
  resetAll: () => void;
};

export const useSession = create<SessionState>()(
  persist(
    (set) => ({
      l1: "",
      regionalVariety: "",
      hasCompletedFirstRun: false,
      consentAcknowledged: false,
      modelReady: false,
      sessions: [],
      recordingObjectUrl: null,
      recordingDurationMs: 0,
      lastEvaluation: null,
      lastFeedback: null,
      lastEvalError: null,

      setL1: (l1, variety) =>
        set({ l1, regionalVariety: variety, hasCompletedFirstRun: true }),

      acknowledgeConsent: () => set({ consentAcknowledged: true }),

      setModelReady: () => set({ modelReady: true }),

      setRecording: (url, durationMs) =>
        set((state) => {
          // Release the previous session's in-memory blob URL when it is
          // replaced so playback recordings don't accumulate for the process
          // lifetime. The prior <audio> is already unmounted by the time a new
          // session calls this, so revoking is safe.
          if (state.recordingObjectUrl && state.recordingObjectUrl !== url) {
            URL.revokeObjectURL(state.recordingObjectUrl);
          }
          return { recordingObjectUrl: url, recordingDurationMs: durationMs };
        }),

      setSessions: (sessions) => set({ sessions }),

      setLastEvaluation: (result, feedback) => {
        // A successful evaluation supersedes any stored error. A null
        // result leaves the existing error in place so Results can render
        // kind + message alongside the empty-result fallback.
        if (result !== null) {
          set({
            lastEvaluation: result,
            lastFeedback: feedback,
            lastEvalError: null,
          });
        } else {
          set({ lastEvaluation: null, lastFeedback: feedback });
        }
      },

      setLastEvalError: (err) => set({ lastEvalError: err }),

      clearSessions: () =>
        set({
          sessions: [],
          lastEvaluation: null,
          lastFeedback: null,
          lastEvalError: null,
        }),

      resetAll: () =>
        set({
          l1: "",
          regionalVariety: "",
          hasCompletedFirstRun: false,
          consentAcknowledged: false,
          modelReady: false,
          sessions: [],
          recordingObjectUrl: null,
          recordingDurationMs: 0,
          lastEvaluation: null,
          lastFeedback: null,
          lastEvalError: null,
        }),
    }),
    {
      name: "p3-platform:session",
      // Persist first-run identity, session history, and the most recent
      // eval result + feedback (plus any error) so /results survives a
      // reload. Volatile recording state (object URL, duration) stays
      // in-memory.
      partialize: (s) => ({
        l1: s.l1,
        regionalVariety: s.regionalVariety,
        hasCompletedFirstRun: s.hasCompletedFirstRun,
        consentAcknowledged: s.consentAcknowledged,
        modelReady: s.modelReady,
        sessions: s.sessions,
        lastEvaluation: s.lastEvaluation,
        lastFeedback: s.lastFeedback,
        lastEvalError: s.lastEvalError,
      }),
    }
  )
);
