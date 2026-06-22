/**
 * Unit tests for the first-run ModelDownload screen (CL-24).
 *
 * The IPC command + event modules are mocked so no Tauri runtime is required.
 * The event mocks capture the registered callbacks into module-level slots so
 * each test can drive `:progress` / `:done` / `:error` by invoking them, and
 * return a stub UnlistenFn so the component's cleanup path runs.
 *
 * Coverage:
 *  - invokes start_first_run_model_download on mount
 *  - renders a determinate progress bar on a :progress event (bytes_total > 0)
 *  - renders an indeterminate state when bytes_total === 0
 *  - on :done flips modelReady in the store and navigates to /passage
 *  - on :error shows the message + a Retry that re-invokes the command
 *
 * Spec: CL-24 first-run model self-download.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";

// ---------------------------------------------------------------------------
// Captured event callbacks — populated by the mocked listen* helpers.
// ---------------------------------------------------------------------------
type ProgressCb = (e: { bytes_done: number; bytes_total: number }) => void;
type DoneCb = (e: Record<string, never>) => void;
type ErrorCb = (e: { error: string }) => void;

let progressCb: ProgressCb | null = null;
let doneCb: DoneCb | null = null;
let errorCb: ErrorCb | null = null;
const unlisten = vi.fn();

vi.mock("../../ipc/commands", () => ({
  startFirstRunModelDownload: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../../ipc/events", () => ({
  listenModelDownloadProgress: vi.fn((cb: ProgressCb) => {
    progressCb = cb;
    return Promise.resolve(unlisten);
  }),
  listenModelDownloadDone: vi.fn((cb: DoneCb) => {
    doneCb = cb;
    return Promise.resolve(unlisten);
  }),
  listenModelDownloadError: vi.fn((cb: ErrorCb) => {
    errorCb = cb;
    return Promise.resolve(unlisten);
  }),
}));

// Imported AFTER the mocks so the component picks up the mocked modules.
import ModelDownload from "../ModelDownload";
import { startFirstRunModelDownload } from "../../ipc/commands";
import { useSession } from "../../store/useSession";

function renderScreen() {
  return render(
    <MemoryRouter initialEntries={["/model-download"]}>
      <Routes>
        <Route path="/model-download" element={<ModelDownload />} />
        <Route path="/passage" element={<div>passage-probe</div>} />
      </Routes>
    </MemoryRouter>
  );
}

// The subscribe/start effect awaits Promise.all over the listen* helpers; flush
// microtasks so the captured callbacks + command invocation are in place.
async function flush() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

beforeEach(() => {
  progressCb = null;
  doneCb = null;
  errorCb = null;
  useSession.setState({ modelReady: false });
  vi.clearAllMocks();
});

describe("ModelDownload — mount", () => {
  it("invokes start_first_run_model_download on mount", async () => {
    renderScreen();
    await flush();
    expect(startFirstRunModelDownload).toHaveBeenCalledOnce();
  });
});

describe("ModelDownload — progress", () => {
  it("renders a determinate percentage on a progress event", async () => {
    renderScreen();
    await flush();

    act(() => {
      progressCb?.({ bytes_done: 50, bytes_total: 100 });
    });

    expect(screen.getByText("50%")).toBeInTheDocument();
    const bar = screen.getByRole("progressbar");
    expect(bar).toHaveAttribute("aria-valuenow", "50");
  });

  it("renders an indeterminate state when bytes_total is 0", async () => {
    renderScreen();
    await flush();

    act(() => {
      progressCb?.({ bytes_done: 0, bytes_total: 0 });
    });

    // The indeterminate head label is the literal "Downloading…" (ellipsis
    // char) — distinct from the lede prose that also contains "downloading".
    expect(screen.getByText("Downloading…")).toBeInTheDocument();
    const bar = screen.getByRole("progressbar");
    expect(bar).not.toHaveAttribute("aria-valuenow");
  });
});

describe("ModelDownload — done", () => {
  it("flips modelReady and navigates to /passage on done", async () => {
    renderScreen();
    await flush();
    expect(useSession.getState().modelReady).toBe(false);

    act(() => {
      doneCb?.({});
    });

    expect(useSession.getState().modelReady).toBe(true);
    expect(screen.getByText("passage-probe")).toBeInTheDocument();
  });
});

describe("ModelDownload — error + retry", () => {
  it("shows the error message and a Retry that re-invokes the command", async () => {
    renderScreen();
    await flush();
    expect(startFirstRunModelDownload).toHaveBeenCalledTimes(1);

    act(() => {
      errorCb?.({ error: "network unreachable" });
    });

    expect(screen.getByRole("alert")).toHaveTextContent(/network unreachable/i);
    const retry = screen.getByRole("button", { name: /retry/i });
    expect(retry).toBeInTheDocument();

    fireEvent.click(retry);
    await flush();

    // Retry first tears down the prior effect's three listeners (no leak across
    // attempts) before re-subscribing + re-invoking the command.
    expect(unlisten).toHaveBeenCalledTimes(3);
    // Retry re-runs the subscribe/start effect → command invoked again.
    expect(startFirstRunModelDownload).toHaveBeenCalledTimes(2);
    // Error UI cleared, back to the downloading state.
    expect(screen.queryByRole("alert")).toBeNull();
  });
});

describe("ModelDownload — listener teardown", () => {
  it("unlistens all three channels on unmount", async () => {
    const { unmount } = renderScreen();
    await flush();
    // Listeners are live, none torn down yet.
    expect(unlisten).not.toHaveBeenCalled();

    unmount();

    // Every model_download:* subscription is released — no leaked listeners
    // across a navigation away from the screen.
    expect(unlisten).toHaveBeenCalledTimes(3);
  });
});
