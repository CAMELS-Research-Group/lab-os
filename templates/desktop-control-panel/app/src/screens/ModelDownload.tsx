import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { startFirstRunModelDownload } from "../ipc/commands";
import {
  listenModelDownloadProgress,
  listenModelDownloadDone,
  listenModelDownloadError,
} from "../ipc/events";
import { useSession } from "../store/useSession";
import ErrorNotice from "../components/ErrorNotice";
import "./ModelDownload.css";

type Phase = "downloading" | "error";

function fmtBytes(n: number): string {
  if (n <= 0) return "0 MB";
  const mb = n / (1024 * 1024);
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(0)} MB`;
}

/**
 * First-run model self-download screen (CL-24).
 *
 * On mount it kicks off `start_first_run_model_download` and subscribes to the
 * three `model_download:*` events. The bundled ONNX model (~339 MB) downloads
 * once on first run with live progress here.
 *
 * The Rust handler is idempotent: if the model is already cached it fires
 * `:done` immediately, so this screen also correctly handles the
 * already-downloaded case (it just advances straight to /passage).
 *
 * Listener cleanup: every `listen*` helper returns an UnlistenFn promise; all
 * three are torn down on unmount. A `mountedRef` guards against navigating or
 * setting state after the component has unmounted.
 */
export default function ModelDownload() {
  const nav = useNavigate();
  const setModelReady = useSession((s) => s.setModelReady);

  const [phase, setPhase] = useState<Phase>("downloading");
  const [bytesDone, setBytesDone] = useState(0);
  const [bytesTotal, setBytesTotal] = useState(0);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  // Guards state updates / navigation after unmount, and re-subscription.
  const mountedRef = useRef(true);
  // Bumped on Retry to force the subscribe/start effect to re-run.
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  // Subscribe to the three channels, then invoke the download command. Runs on
  // mount and on every Retry (via `attempt`). Cleanup tears down all three
  // listeners so a re-subscribe (Retry) or unmount never leaks.
  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;
    let unlistenDone: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;
    let cancelled = false;

    const cleanup = () => {
      unlistenProgress?.();
      unlistenDone?.();
      unlistenError?.();
    };

    Promise.all([
      listenModelDownloadProgress((e) => {
        if (!mountedRef.current) return;
        setBytesDone(e.bytes_done);
        setBytesTotal(e.bytes_total);
      }),
      listenModelDownloadDone(() => {
        if (!mountedRef.current) return;
        setModelReady();
        nav("/passage");
      }),
      listenModelDownloadError((e) => {
        if (!mountedRef.current) return;
        setErrorMsg(e.error);
        setPhase("error");
      }),
    ])
      .then(([p, d, er]) => {
        // If the component unmounted (or retried) while listeners were being
        // registered, tear them down immediately.
        if (cancelled) {
          p();
          d();
          er();
          return;
        }
        unlistenProgress = p;
        unlistenDone = d;
        unlistenError = er;

        // Listeners are live — now kick off the download. A synchronous
        // rejection from the command (e.g. handler missing) surfaces as an
        // error state rather than an unhandled rejection.
        startFirstRunModelDownload().catch((err) => {
          if (!mountedRef.current) return;
          setErrorMsg(err instanceof Error ? err.message : String(err));
          setPhase("error");
        });
      })
      .catch((err) => {
        if (!mountedRef.current) return;
        setErrorMsg(err instanceof Error ? err.message : String(err));
        setPhase("error");
      });

    return () => {
      cancelled = true;
      cleanup();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [attempt]);

  const onRetry = useCallback(() => {
    // Reset local progress/error and bump `attempt` so the subscribe/start
    // effect re-runs with fresh listeners and a fresh command invocation.
    setErrorMsg(null);
    setBytesDone(0);
    setBytesTotal(0);
    setPhase("downloading");
    setAttempt((a) => a + 1);
  }, []);

  const determinate = bytesTotal > 0;
  const pct = determinate
    ? Math.min(100, Math.round((bytesDone / bytesTotal) * 100))
    : 0;

  return (
    <div className="screen model-download-screen">
      <p className="md-kicker">One-time setup</p>
      <h1>Getting your practice model ready</h1>
      <p className="lede">
        We're downloading the on-device pronunciation model. This happens once —
        after this, everything runs locally with no further downloads.
      </p>

      <div className="card md-card">
        {phase === "downloading" && (
          <div
            className="md-progress"
            role="progressbar"
            aria-label="Model download progress"
            aria-valuemin={0}
            aria-valuemax={determinate ? 100 : undefined}
            aria-valuenow={determinate ? pct : undefined}
          >
            <div className="md-progress-head">
              {determinate ? (
                <>
                  <span className="md-pct">{pct}%</span>
                  <span className="md-bytes">
                    {fmtBytes(bytesDone)} of {fmtBytes(bytesTotal)}
                  </span>
                </>
              ) : (
                <span className="md-pct">Downloading…</span>
              )}
            </div>

            <div className={`md-track ${determinate ? "" : "indeterminate"}`}>
              <div
                className="md-fill"
                style={determinate ? { width: `${pct}%` } : undefined}
              />
            </div>

            <p className="md-hint">
              Keep the app open while this finishes.
            </p>
          </div>
        )}

        {phase === "error" && (
          <ErrorNotice
            kind="model_download_failed"
            message={errorMsg}
            actionLabel="Retry"
            onAction={onRetry}
          />
        )}
      </div>
    </div>
  );
}
