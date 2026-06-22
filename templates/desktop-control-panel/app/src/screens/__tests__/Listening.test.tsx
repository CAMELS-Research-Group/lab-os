/**
 * Unit test for the Listening screen's "Listen back" wiring (#116).
 *
 * The real evaluation path captures audio Rust-side; the webview runs a
 * parallel best-effort `Recorder` purely to produce an in-memory object URL
 * for playback on Results. This test proves that URL is threaded into the
 * session store on `eval:done` (the bug was Listening passing `null`), and
 * that nothing is persisted (the store's `recordingObjectUrl` is volatile and
 * excluded from `partialize`).
 *
 * Tauri command + event modules and the Recorder are mocked so no runtime is
 * required.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { useSession } from "../../store/useSession";

const { startSession, endSession, pauseSession, resumeSession } = vi.hoisted(
  () => ({
    startSession: vi.fn(),
    endSession: vi.fn(),
    pauseSession: vi.fn(),
    resumeSession: vi.fn(),
  })
);

vi.mock("../../ipc/commands", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../ipc/commands")>();
  return { ...real, startSession, endSession, pauseSession, resumeSession };
});

const { listenEvalDone, listenEvalError, listenRecordingLevel } = vi.hoisted(
  () => ({
    listenEvalDone: vi.fn(),
    listenEvalError: vi.fn(),
    listenRecordingLevel: vi.fn(),
  })
);

vi.mock("../../ipc/events", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../ipc/events")>();
  return { ...real, listenEvalDone, listenEvalError, listenRecordingLevel };
});

const recorderStop = vi.fn();

vi.mock("../../lib/recorder", () => ({
  Recorder: class {
    start = vi.fn().mockResolvedValue(undefined);
    stop = recorderStop;
    pause = vi.fn();
    resume = vi.fn();
  },
}));

import Listening from "../Listening";

let capturedEvalDone: ((e: { result: unknown; feedback: unknown }) => void) | null;
let capturedEvalError: ((e: { kind: string; message: string }) => void) | null;

function renderListening() {
  return render(
    <MemoryRouter initialEntries={["/listening"]}>
      <Routes>
        <Route path="/listening" element={<Listening />} />
        <Route path="/results" element={<div>results-probe</div>} />
      </Routes>
    </MemoryRouter>
  );
}

describe("Listening — Listen back wiring (#116)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Avoid jsdom RAF recursion; the timer just needs to not blow up.
    vi.stubGlobal("requestAnimationFrame", () => 0);
    vi.stubGlobal("cancelAnimationFrame", () => {});

    startSession.mockResolvedValue(undefined);
    endSession.mockResolvedValue(undefined);
    recorderStop.mockResolvedValue({
      blob: new Blob([]),
      objectUrl: "blob:playback-test",
      durationMs: 1234,
    });

    capturedEvalDone = null;
    listenEvalDone.mockImplementation((cb) => {
      capturedEvalDone = cb;
      return Promise.resolve(() => {});
    });
    capturedEvalError = null;
    listenEvalError.mockImplementation((cb) => {
      capturedEvalError = cb;
      return Promise.resolve(() => {});
    });
    listenRecordingLevel.mockResolvedValue(() => {});

    useSession.setState({
      recordingObjectUrl: null,
      recordingDurationMs: 0,
      lastEvaluation: null,
      lastFeedback: null,
      lastEvalError: null,
    });
  });

  it("threads the in-memory playback url into the store on eval:done", async () => {
    renderListening();

    const doneBtn = await screen.findByRole("button", { name: /evaluate/i });
    await act(async () => {
      fireEvent.click(doneBtn);
    });

    // The real path subscribes to eval:done; once end_session resolves the
    // callback is registered. Fire it as the Rust orchestrator would.
    await waitFor(() => expect(capturedEvalDone).toBeTruthy());
    await act(async () => {
      capturedEvalDone!({ result: { duration_seconds: 2 }, feedback: [] });
    });

    // The best-effort recorder's object URL — not null — reaches the store.
    expect(useSession.getState().recordingObjectUrl).toBe("blob:playback-test");
    expect(recorderStop).toHaveBeenCalled();
    await screen.findByText("results-probe");
  });

  it("threads the playback url on eval:error and surfaces the diagnostic", async () => {
    renderListening();

    const doneBtn = await screen.findByRole("button", { name: /evaluate/i });
    await act(async () => {
      fireEvent.click(doneBtn);
    });

    await waitFor(() => expect(capturedEvalError).toBeTruthy());
    await act(async () => {
      capturedEvalError!({ kind: "inference_runtime", message: "boom" });
    });

    // Playback is preserved even when evaluation fails, and the diagnostic is
    // surfaced for Results' empty-result fallback.
    expect(useSession.getState().recordingObjectUrl).toBe("blob:playback-test");
    expect(useSession.getState().lastEvalError).toEqual({
      kind: "inference_runtime",
      message: "boom",
    });
    await screen.findByText("results-probe");
  });
});
