/**
 * Tests for the typed Tauri event subscribers.
 *
 * `@tauri-apps/api/event::listen` is mocked so each test can assert that the
 * subscriber wires the right channel name (via the exported `EVT_*` const)
 * and that the payload-extraction `(e) => cb(e.payload)` wrapper applies.
 *
 * Covers the surviving updater event surface.
 */

import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

import { listen } from "@tauri-apps/api/event";

import {
  EVT_UPDATE_DOWNLOAD_PROGRESS,
  EVT_UPDATE_ERROR,
  EVT_UPDATE_READY,
  listenUpdateDownloadProgress,
  listenUpdateError,
  listenUpdateReady,
  subscribe,
} from "../events";

const mockListen = vi.mocked(listen);

afterEach(() => {
  mockListen.mockReset();
});

/**
 * Trigger the mocked `listen` and capture the inner `(e) => cb(e.payload)`
 * wrapper Tauri receives. Lets us assert that the helper unwraps `.payload`.
 */
function captureListener(): (event: { payload: unknown }) => void {
  const calls = mockListen.mock.calls;
  expect(calls.length).toBeGreaterThan(0);
  const lastCall = calls[calls.length - 1];
  return lastCall[1] as (event: { payload: unknown }) => void;
}

// ---------------------------------------------------------------------------
// Channel name constants — typo-safety
// ---------------------------------------------------------------------------

describe("event channel name constants", () => {
  it("each EVT_* const matches its channel string", () => {
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

    await subscribe<{ pct: number }>(EVT_UPDATE_DOWNLOAD_PROGRESS, cb);

    expect(mockListen).toHaveBeenCalledWith(
      EVT_UPDATE_DOWNLOAD_PROGRESS,
      expect.any(Function)
    );

    const wrapper = captureListener();
    wrapper({ payload: { pct: 0.5 } });
    expect(cb).toHaveBeenCalledWith({ pct: 0.5 });
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
