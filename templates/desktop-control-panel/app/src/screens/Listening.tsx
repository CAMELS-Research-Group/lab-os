import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PASSAGE_PARAGRAPHS, PASSAGE_TITLE } from "../data/passage";
import {
  cancelSession as cancelSessionCmd,
  endSession as endSessionCmd,
  pauseSession as pauseSessionCmd,
  resumeSession as resumeSessionCmd,
  startSession as startSessionCmd,
} from "../ipc/commands";
import {
  listenEvalDone,
  listenEvalError,
  listenRecordingLevel,
} from "../ipc/events";
import { Recorder } from "../lib/recorder";
import { useSession } from "../store/useSession";
import "./Listening.css";

type Mode = "starting" | "recording" | "paused" | "stopping" | "denied";

function fmtTimer(ms: number) {
  const totalSeconds = Math.floor(ms / 1000);
  const m = Math.floor(totalSeconds / 60);
  const s = totalSeconds % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export default function Listening() {
  const nav = useNavigate();
  const setRecording = useSession((s) => s.setRecording);
  const setLastEvaluation = useSession((s) => s.setLastEvaluation);
  const setLastEvalError = useSession((s) => s.setLastEvalError);

  // Simulated-mode fallback recorder. Only constructed when start_session
  // throws (mic denied, no Tauri runtime, etc.). Kept in tree exactly for
  // this preview path — DEMO-5 must not regress.
  const recorderRef = useRef<Recorder | null>(null);
  const [mode, setMode] = useState<Mode>("starting");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [simulated, setSimulated] = useState(false);
  const startedAtRef = useRef<number | null>(null);

  // Best-effort, in-memory webview recorder used ONLY to provide "Listen back"
  // playback on Results. In the real path the Rust side owns the audio capture
  // that feeds evaluation; this parallel webview capture exists solely so the
  // learner can replay their reading. The blob lives in memory (an object URL)
  // and is NEVER written to disk or localStorage (FRD: no raw audio persisted),
  // so playback is lost on reload — that's acceptable. If the mic can't be
  // opened here (already held, or webview permission denied) playback is simply
  // unavailable; the real evaluation flow is unaffected.
  const startPlaybackRecorder = () => {
    const r = new Recorder();
    recorderRef.current = r;
    r.start().catch(() => {
      recorderRef.current = null;
    });
  };

  // Boot: try to start the real session first, fall back to simulated.
  useEffect(() => {
    let cancelled = false;

    startSessionCmd()
      .then(() => {
        if (cancelled) return;
        startedAtRef.current = performance.now();
        // Parallel in-memory capture for "Listen back" (best-effort, see
        // startPlaybackRecorder). The Rust session owns the eval audio.
        startPlaybackRecorder();
        setMode("recording");
      })
      .catch((err) => {
        if (cancelled) return;
        // StrictMode (and any other re-mount) double-invokes this effect in
        // dev: the first call succeeds and leaves the Rust lifecycle in
        // Recording, the second call's startSession then rejects with
        // `invalid_state` because the state machine won't transition
        // Recording → Recording. That second rejection is NOT a real
        // failure — the mic is already capturing on the Rust side from the
        // first call, so we adopt that session and proceed.
        //
        // Match on the canonical message prefix from `session.rs::invalid_state`
        // (`"cannot {op} from {from:?}; ..."`). Pinning the op AND the from-
        // state to "start_session" / "Recording" avoids a loose substring
        // sniff that would also fire for `"cannot pause from Recording"` or
        // an Evaluating→Recording probe.
        const isAlreadyRecording =
          err &&
          typeof err === "object" &&
          (err as { kind?: string }).kind === "invalid_state" &&
          typeof (err as { message?: string }).message === "string" &&
          (err as { message: string }).message
            .includes("cannot start_session from Recording");
        if (isAlreadyRecording) {
          startedAtRef.current = performance.now();
          // Adopted an already-running Rust session (StrictMode re-mount);
          // still capture in-memory audio for playback.
          startPlaybackRecorder();
          setMode("recording");
          return;
        }
        // Genuine startSession failure (mic denied, no Tauri runtime, etc.).
        // Spin up the React-side recorder so the user can still walk through
        // the flow without a working mic. `recorder.stop()` returning
        // durationMs: 0 is the signal Results uses to switch to the
        // preview-only branch.
        const r = new Recorder();
        recorderRef.current = r;
        r.start()
          .then(() => {
            if (cancelled) return;
            startedAtRef.current = performance.now();
            setSimulated(true);
            setMode("recording");
          })
          .catch(() => {
            if (cancelled) return;
            setSimulated(true);
            startedAtRef.current = performance.now();
            setMode("recording");
          });
      });

    return () => {
      cancelled = true;
      // Best-effort teardown for the simulated path. The real path's audio
      // capture is owned by the Rust lifecycle; end_session / cancel_session
      // are dispatched from the explicit user actions below.
      recorderRef.current?.stop().catch(() => {});
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Subscribe to recording:level once on mount. The handler is a stub today
  // (no visible level meter in V1 walking-skeleton), but the subscription
  // proves the IPC wiring and gives the Rust side a listener — necessary so
  // emit() does not silently no-op against a closed webview channel.
  useEffect(() => {
    const unlistenP = listenRecordingLevel(() => {
      // Intentionally empty: V1 walking-skeleton does not surface RMS yet.
    });
    return () => {
      unlistenP.then((u) => u()).catch(() => {});
    };
  }, []);

  // Drive the elapsed timer at RAF rate (smooth 1-second clock display).
  useEffect(() => {
    if (mode !== "recording") return;
    const tickStart = performance.now();
    const baseElapsed = elapsedMs;
    let raf = 0;

    const tick = () => {
      const now = baseElapsed + (performance.now() - tickStart);
      setElapsedMs(now);

      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode]);

  const onPause = async () => {
    if (simulated) {
      recorderRef.current?.pause();
      setMode("paused");
      return;
    }
    try {
      await pauseSessionCmd();
      // Keep the best-effort playback capture aligned with the session.
      recorderRef.current?.pause();
      setMode("paused");
    } catch (err) {
      // Keep the UI in `recording` so it reflects the Rust state machine's
      // actual transition (or lack of one). Logging the rejection here is the
      // minimum signal until CL-22 wires a pause/resume failure toast.
      console.warn("pause_session rejected:", err);
    }
  };

  const onResume = async () => {
    if (simulated) {
      recorderRef.current?.resume();
      setMode("recording");
      return;
    }
    try {
      await resumeSessionCmd();
      recorderRef.current?.resume();
      setMode("recording");
    } catch (err) {
      // Same rationale as onPause: do not flip mode when Rust refused the
      // transition; surface the rejection in the console for triage.
      console.warn("resume_session rejected:", err);
    }
  };

  // Discard the current take and begin a fresh recording in place. Mirrors the
  // boot sequence (real → simulated fallback) but, being user-triggered, skips
  // the StrictMode adopt-already-recording branch. The Rust session is in
  // Recording/Paused, so cancel it before starting a new one; errors there are
  // non-fatal (we fall back to a fresh start / simulated capture).
  const onStartOver = async () => {
    setMode("starting");
    setElapsedMs(0);
    startedAtRef.current = null;

    const prevPlayback = recorderRef.current;
    recorderRef.current = null;
    await prevPlayback?.stop().catch(() => {});

    if (simulated) {
      const r = new Recorder();
      recorderRef.current = r;
      await r.start().catch(() => {});
      startedAtRef.current = performance.now();
      setMode("recording");
      return;
    }

    await cancelSessionCmd().catch(() => {});
    try {
      await startSessionCmd();
      startedAtRef.current = performance.now();
      startPlaybackRecorder();
      setMode("recording");
    } catch {
      // Mic/session unavailable on restart — drop to the simulated walkthrough,
      // matching the boot-path fallback contract.
      const r = new Recorder();
      recorderRef.current = r;
      r.start().catch(() => {});
      setSimulated(true);
      startedAtRef.current = performance.now();
      setMode("recording");
    }
  };

  const onDone = async () => {
    setMode("stopping");

    // Simulated path: skip the Rust eval pipeline entirely. Results uses
    // recordingDurationMs === 0 as the preview-only signal.
    if (simulated) {
      const r = recorderRef.current;
      if (r) {
        try {
          const result = await r.stop();
          setRecording(result.objectUrl, result.durationMs);
        } catch (err) {
          // Same diagnostic contract as the real-path failure surfaces: the
          // user lands on Results and the empty-result fallback renders the
          // kind+message instead of silently swallowing the capture failure.
          setRecording(null, 0);
          setLastEvalError({
            kind: "simulated_capture_failed",
            message: err instanceof Error ? err.message : String(err),
          });
        }
      } else {
        setRecording(null, 0);
      }
      setLastEvaluation(null, null);
      nav("/results");
      return;
    }

    // Real path. First stop the best-effort playback recorder (if it started)
    // to capture an in-memory object URL for "Listen back" on Results. This is
    // independent of the Rust-side eval audio and is held in memory only —
    // never written to disk or localStorage (FRD: no raw audio persisted).
    let playbackUrl: string | null = null;
    const playbackRec = recorderRef.current;
    recorderRef.current = null;
    if (playbackRec) {
      try {
        const res = await playbackRec.stop();
        playbackUrl = res.objectUrl || null;
      } catch {
        playbackUrl = null;
      }
    }

    // Subscribe to eval:done / eval:error BEFORE dispatching end_session so we
    // cannot race the emit. The orchestrator runs fire-and-forget on the Rust
    // side.
    let unlistenDone: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;

    const cleanup = () => {
      unlistenDone?.();
      unlistenError?.();
    };

    try {
      const [doneUnlisten, errorUnlisten] = await Promise.all([
        listenEvalDone((e) => {
          setLastEvaluation(e.result, e.feedback);
          // Mirror the existing setRecording shape so DEMO-3's promote-guard
          // (recordingDurationMs > 0) recognises this as a real session.
          const durationMs = Math.round(
            (e.result.duration_seconds ?? 0) * 1000
          );
          setRecording(playbackUrl, durationMs);
          cleanup();
          nav("/results");
        }),
        listenEvalError((e) => {
          // Capture the kind+message BEFORE clearing the result slot so
          // Results can render the diagnostic alongside the empty-result
          // fallback. The setLastEvaluation(null, ...) call below leaves
          // lastEvalError in place; only a successful result clears it.
          setLastEvalError({ kind: e.kind, message: e.message });
          setLastEvaluation(null, null);
          const fallbackMs = startedAtRef.current
            ? Math.max(1, Math.round(performance.now() - startedAtRef.current))
            : 1;
          setRecording(playbackUrl, fallbackMs);
          cleanup();
          nav("/results");
        }),
      ]);
      unlistenDone = doneUnlisten;
      unlistenError = errorUnlisten;

      await endSessionCmd();
    } catch (err) {
      // end_session itself rejected (e.g. lifecycle take_audio returned
      // None, or any other AppError before the orchestrator spawned). The
      // Rust side did NOT fire eval:error in this path, so we synthesise
      // an error payload tagged with a recognisable kind.
      cleanup();
      setLastEvalError({
        kind: "end_session_failed",
        message: err instanceof Error ? err.message : String(err),
      });
      setLastEvaluation(null, null);
      const fallbackMs = startedAtRef.current
        ? Math.max(1, Math.round(performance.now() - startedAtRef.current))
        : 1;
      setRecording(playbackUrl, fallbackMs);
      nav("/results");
    }
  };

  const wordsByParagraph = useMemo(
    () => PASSAGE_PARAGRAPHS.map((p) => p.split(/(\s+)/)),
    []
  );

  const isRecording = mode === "recording";
  const isPaused = mode === "paused";

  // Single status line under the recorder. The simulated branch keeps the
  // existing "nothing is being recorded" warning prominent (styled warn).
  const hint =
    mode === "starting"
      ? "Starting…"
      : mode === "stopping"
        ? "Wrapping up…"
        : isPaused
          ? "Paused — resume when you're ready."
          : simulated
            ? "Preview only — microphone access denied; nothing is being recorded."
            : "Recording. Pause any time — evaluating is a separate step.";

  return (
    <div className="screen listening-screen">
      <div className="rec-stage">
        <div className="card rec-passage">
          <h2 className="rec-passage-title">{PASSAGE_TITLE}</h2>
          {wordsByParagraph.map((words, pi) => (
            <p key={pi} className="rec-passage-p">
              {words.map((token, ti) => (
                <span key={ti}>{token}</span>
              ))}
            </p>
          ))}
        </div>

        <div className="rec-wrap">
          <div
            className={
              "rec-pill" +
              (isPaused ? " rec-pill--paused" : "") +
              (simulated ? " rec-pill--sim" : "")
            }
          >
            <span className="rec-dot" aria-hidden="true" />
            <span className="rec-time">{fmtTimer(elapsedMs)}</span>
            <div
              className={"rec-wave" + (isRecording ? " rec-wave--on" : "")}
              aria-hidden="true"
            >
              <span />
              <span />
              <span />
              <span />
              <span />
              <span />
            </div>
            <button
              className="rec-pause"
              onClick={isPaused ? onResume : onPause}
              disabled={mode === "starting" || mode === "stopping"}
              aria-label={isPaused ? "Resume" : "Pause"}
            >
              {isPaused ? "▶" : "⏸"}
            </button>
          </div>

          <div className="rec-actions">
            <button
              className="ghost rec-ghost"
              onClick={onStartOver}
              disabled={mode === "starting" || mode === "stopping"}
            >
              Start over
            </button>
            <button
              className="rec-evaluate"
              onClick={onDone}
              disabled={mode === "stopping"}
            >
              Stop &amp; evaluate →
            </button>
          </div>

          <div
            className={
              "rec-hint" + (simulated && isRecording ? " rec-hint--warn" : "")
            }
          >
            {hint}
          </div>
        </div>
      </div>
    </div>
  );
}
