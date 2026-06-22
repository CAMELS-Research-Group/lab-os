/**
 * Tests for the typed Tauri event subscribers.
 *
 * `@tauri-apps/api/event::listen` is mocked so each test can assert that the
 * subscriber wires the right channel name (via the exported `EVT_*` const)
 * and that the payload-extraction `(e) => cb(e.payload)` wrapper applies.
 *
 * Spec: ADD §3.7 (event surface), task CL-25.
 */

import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

import { listen } from "@tauri-apps/api/event";

import {
  EVT_EVAL_DONE,
  EVT_EVAL_ERROR,
  EVT_EVAL_PROGRESS,
  EVT_IDENTITY_REGISTRATION_FAILED,
  EVT_IDENTITY_REGISTRATION_SUCCEEDED,
  EVT_MODEL_DOWNLOAD_DONE,
  EVT_MODEL_DOWNLOAD_ERROR,
  EVT_MODEL_DOWNLOAD_PROGRESS,
  EVT_RECORDING_ERROR,
  EVT_RECORDING_LEVEL,
  EVT_UPDATE_DOWNLOAD_PROGRESS,
  EVT_UPDATE_ERROR,
  EVT_UPDATE_READY,
  EVT_UPLOAD_SUCCEEDED,
  EVT_UPLOAD_TERMINAL_ERROR,
  listenEvalDone,
  listenEvalError,
  listenEvalProgress,
  listenIdentityRegistrationFailed,
  listenIdentityRegistrationSucceeded,
  listenModelDownloadDone,
  listenModelDownloadError,
  listenModelDownloadProgress,
  listenRecordingError,
  listenRecordingLevel,
  listenUpdateDownloadProgress,
  listenUpdateError,
  listenUpdateReady,
  listenUploadSucceeded,
  listenUploadTerminalError,
  subscribe,
} from "../events";
import type { EvalDoneEvent } from "../events";
import { makeEvaluationResult, makeFeedbackEntry } from "./fixtures";

const mockListen = vi.mocked(listen);

afterEach(() => {
  mockListen.mockReset();
});

/**
 * Trigger the mocked `listen` and capture the inner `(e) => cb(e.payload)`
 * wrapper Tauri receives. Lets us assert that the helper unwraps `.payload`.
 */
function captureListener(): (event: { payload: unknown }) => void {
  // The first arg-call is `(channel, handler)`. Grab the handler.
  const calls = mockListen.mock.calls;
  expect(calls.length).toBeGreaterThan(0);
  const lastCall = calls[calls.length - 1];
  // listen<T> signature: (event: string, handler: (e: Event<T>) => void) => Promise<UnlistenFn>
  return lastCall[1] as (event: { payload: unknown }) => void;
}

// ---------------------------------------------------------------------------
// Channel name constants — typo-safety
// ---------------------------------------------------------------------------

describe("event channel name constants", () => {
  it("each EVT_* const matches the ADD §3.7 channel string", () => {
    expect(EVT_IDENTITY_REGISTRATION_SUCCEEDED).toBe("identity:registration_succeeded");
    expect(EVT_IDENTITY_REGISTRATION_FAILED).toBe("identity:registration_failed");
    expect(EVT_RECORDING_LEVEL).toBe("recording:level");
    expect(EVT_RECORDING_ERROR).toBe("recording:error");
    expect(EVT_EVAL_PROGRESS).toBe("eval:progress");
    expect(EVT_EVAL_DONE).toBe("eval:done");
    expect(EVT_EVAL_ERROR).toBe("eval:error");
    expect(EVT_UPLOAD_SUCCEEDED).toBe("upload:succeeded");
    expect(EVT_UPLOAD_TERMINAL_ERROR).toBe("upload:terminal_error");
    expect(EVT_MODEL_DOWNLOAD_PROGRESS).toBe("model_download:progress");
    expect(EVT_MODEL_DOWNLOAD_DONE).toBe("model_download:done");
    expect(EVT_MODEL_DOWNLOAD_ERROR).toBe("model_download:error");
    expect(EVT_UPDATE_DOWNLOAD_PROGRESS).toBe("update:download_progress");
    expect(EVT_UPDATE_READY).toBe("update:ready");
    expect(EVT_UPDATE_ERROR).toBe("update:error");
  });
});

// ---------------------------------------------------------------------------
// subscribe<T> helper — payload unwrap behaviour
// ---------------------------------------------------------------------------

describe("subscribe<T> helper", () => {
  it("registers the inner wrapper on listen() and forwards e.payload to the cb", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();

    await subscribe<{ x: number }>(EVT_RECORDING_LEVEL, cb);

    expect(mockListen).toHaveBeenCalledWith(EVT_RECORDING_LEVEL, expect.any(Function));

    const wrapper = captureListener();
    wrapper({ payload: { x: 42 } });
    expect(cb).toHaveBeenCalledWith({ x: 42 });
  });
});

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

describe("identity listeners", () => {
  it("listenIdentityRegistrationSucceeded wires the success channel", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenIdentityRegistrationSucceeded(cb);
    expect(mockListen).toHaveBeenCalledWith(
      EVT_IDENTITY_REGISTRATION_SUCCEEDED,
      expect.any(Function)
    );
    const wrapper = captureListener();
    wrapper({ payload: {} });
    expect(cb).toHaveBeenCalledWith({});
  });

  it("listenIdentityRegistrationFailed wires the failure channel + forwards the error string", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenIdentityRegistrationFailed(cb);
    expect(mockListen).toHaveBeenCalledWith(
      EVT_IDENTITY_REGISTRATION_FAILED,
      expect.any(Function)
    );
    const wrapper = captureListener();
    wrapper({ payload: { error: "backend 503" } });
    expect(cb).toHaveBeenCalledWith({ error: "backend 503" });
  });
});

// ---------------------------------------------------------------------------
// Recording
// ---------------------------------------------------------------------------

describe("recording listeners", () => {
  it("listenRecordingLevel wires recording:level + forwards rms", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenRecordingLevel(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_RECORDING_LEVEL, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { rms: 0.42 } });
    expect(cb).toHaveBeenCalledWith({ rms: 0.42 });
  });

  it("listenRecordingError wires recording:error + forwards kind/message", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenRecordingError(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_RECORDING_ERROR, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { kind: "device_lost", message: "default input changed" } });
    expect(cb).toHaveBeenCalledWith({ kind: "device_lost", message: "default input changed" });
  });
});

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

describe("evaluation listeners", () => {
  it("listenEvalProgress wires eval:progress + forwards the progress payload", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenEvalProgress(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_EVAL_PROGRESS, expect.any(Function));
    const wrapper = captureListener();
    wrapper({
      payload: {
        session_id: "sess",
        stage: "chunk",
        pct: 0.5,
        partial_result: null,
      },
    });
    expect(cb).toHaveBeenCalledWith({
      session_id: "sess",
      stage: "chunk",
      pct: 0.5,
      partial_result: null,
    });
  });

  it("listenEvalDone wires eval:done + forwards { result, feedback }", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenEvalDone(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_EVAL_DONE, expect.any(Function));
    const wrapper = captureListener();
    const payload: EvalDoneEvent = {
      result: makeEvaluationResult({ session_id: "sess" }),
      feedback: [makeFeedbackEntry()],
    };
    wrapper({ payload });
    expect(cb).toHaveBeenCalledWith(payload);
  });

  it("listenEvalError wires eval:error + forwards the error payload", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenEvalError(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_EVAL_ERROR, expect.any(Function));
    const wrapper = captureListener();
    wrapper({
      payload: { session_id: "sess", kind: "inference_runtime", message: "bad" },
    });
    expect(cb).toHaveBeenCalledWith({
      session_id: "sess",
      kind: "inference_runtime",
      message: "bad",
    });
  });
});

// ---------------------------------------------------------------------------
// Upload
// ---------------------------------------------------------------------------

describe("upload listeners", () => {
  it("listenUploadSucceeded wires upload:succeeded + forwards session_id", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenUploadSucceeded(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_UPLOAD_SUCCEEDED, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { session_id: "sess-123" } });
    expect(cb).toHaveBeenCalledWith({ session_id: "sess-123" });
  });

  it("listenUploadTerminalError wires upload:terminal_error + forwards the literal code string", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenUploadTerminalError(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_UPLOAD_TERMINAL_ERROR, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { code: "410" } });
    expect(cb).toHaveBeenCalledWith({ code: "410" });
  });
});

// ---------------------------------------------------------------------------
// Model download
// ---------------------------------------------------------------------------

describe("model download listeners", () => {
  it("listenModelDownloadProgress wires model_download:progress", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenModelDownloadProgress(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_MODEL_DOWNLOAD_PROGRESS, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { bytes_done: 5000, bytes_total: 10000 } });
    expect(cb).toHaveBeenCalledWith({ bytes_done: 5000, bytes_total: 10000 });
  });

  it("listenModelDownloadDone wires model_download:done", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenModelDownloadDone(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_MODEL_DOWNLOAD_DONE, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: {} });
    expect(cb).toHaveBeenCalledWith({});
  });

  it("listenModelDownloadError wires model_download:error", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenModelDownloadError(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_MODEL_DOWNLOAD_ERROR, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { error: "network down" } });
    expect(cb).toHaveBeenCalledWith({ error: "network down" });
  });
});

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

describe("update listeners", () => {
  it("listenUpdateDownloadProgress wires update:download_progress", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenUpdateDownloadProgress(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_UPDATE_DOWNLOAD_PROGRESS, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { pct: 0.75 } });
    expect(cb).toHaveBeenCalledWith({ pct: 0.75 });
  });

  it("listenUpdateReady wires update:ready", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenUpdateReady(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_UPDATE_READY, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: {} });
    expect(cb).toHaveBeenCalledWith({});
  });

  it("listenUpdateError wires update:error", async () => {
    mockListen.mockResolvedValueOnce(() => {});
    const cb = vi.fn();
    await listenUpdateError(cb);
    expect(mockListen).toHaveBeenCalledWith(EVT_UPDATE_ERROR, expect.any(Function));
    const wrapper = captureListener();
    wrapper({ payload: { error: "downloader failed" } });
    expect(cb).toHaveBeenCalledWith({ error: "downloader failed" });
  });
});
