/**
 * Unit tests for useUpdate Zustand store.
 *
 * Mocks the `ipc/commands` module so no Tauri runtime is required.
 * Uses the zustand store's `setState` to reset between tests, ensuring test
 * isolation without needing `beforeEach` lifecycle ordering tricks.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { useUpdate } from "../useUpdate";

// ---------------------------------------------------------------------------
// Module mock — vi.mock is hoisted, so the factory runs before any imports.
// ---------------------------------------------------------------------------

vi.mock("../../ipc/commands", async (importOriginal) => {
  // Spread the real module so IpcError (a class, not a function) is preserved.
  // Only checkForUpdate and applyUpdate need to be swappable per-test.
  const real = await importOriginal<typeof import("../../ipc/commands")>();
  return {
    ...real,
    checkForUpdate: vi.fn(),
    applyUpdate: vi.fn(),
  };
});

// Typed references to the mocked functions for easy per-test configuration.
import { checkForUpdate, applyUpdate, IpcError } from "../../ipc/commands";

const mockCheckForUpdate = vi.mocked(checkForUpdate);
const mockApplyUpdate = vi.mocked(applyUpdate);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Reset the store to its initial state and clear all mock call history. */
function resetStore() {
  useUpdate.setState({
    checking: false,
    available: false,
    version: null,
    notes: null,
    dismissed: false,
    applyFailed: false,
  });
  vi.clearAllMocks();
}

function makeIpcError(kind = "update", message = "failed"): IpcError {
  return new IpcError(kind, message, false, { kind, message, recoverable: false });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("useUpdate store — initial state", () => {
  beforeEach(resetStore);

  it("exposes the correct initial shape", () => {
    const s = useUpdate.getState();
    expect(s.checking).toBe(false);
    expect(s.available).toBe(false);
    expect(s.version).toBeNull();
    expect(s.notes).toBeNull();
    expect(s.dismissed).toBe(false);
    expect(s.applyFailed).toBe(false);
    expect(typeof s.check).toBe("function");
    expect(typeof s.apply).toBe("function");
    expect(typeof s.dismiss).toBe("function");
  });
});

describe("useUpdate store — check() success", () => {
  beforeEach(resetStore);

  it("populates available/version/notes and clears checking when update is available", async () => {
    mockCheckForUpdate.mockResolvedValueOnce({
      available: true,
      version: "0.3.0",
      notes: "Bug fixes and improvements.",
    });

    await useUpdate.getState().check();

    const s = useUpdate.getState();
    expect(s.checking).toBe(false);
    expect(s.available).toBe(true);
    expect(s.version).toBe("0.3.0");
    expect(s.notes).toBe("Bug fixes and improvements.");
  });

  it("sets available=false and clears version/notes when no update is available", async () => {
    mockCheckForUpdate.mockResolvedValueOnce({
      available: false,
      version: null,
      notes: null,
    });

    await useUpdate.getState().check();

    const s = useUpdate.getState();
    expect(s.checking).toBe(false);
    expect(s.available).toBe(false);
    expect(s.version).toBeNull();
    expect(s.notes).toBeNull();
  });

  it("sets checking=true before the call resolves", async () => {
    let checkingDuringCall = false;

    mockCheckForUpdate.mockImplementationOnce(async () => {
      checkingDuringCall = useUpdate.getState().checking;
      return { available: true, version: "0.3.0", notes: null };
    });

    await useUpdate.getState().check();

    expect(checkingDuringCall).toBe(true);
    // Cleared after resolution
    expect(useUpdate.getState().checking).toBe(false);
  });
});

describe("useUpdate store — check() failure", () => {
  beforeEach(resetStore);

  it("leaves available=false and checking=false when checkForUpdate rejects with IpcError", async () => {
    mockCheckForUpdate.mockRejectedValueOnce(makeIpcError("update_check_failed", "network error"));

    // Must not throw out of the action
    await expect(useUpdate.getState().check()).resolves.toBeUndefined();

    const s = useUpdate.getState();
    expect(s.checking).toBe(false);
    expect(s.available).toBe(false);
    expect(s.version).toBeNull();
    expect(s.notes).toBeNull();
  });

  it("leaves available=false and checking=false when checkForUpdate rejects with a generic error", async () => {
    mockCheckForUpdate.mockRejectedValueOnce(new Error("unexpected failure"));

    await expect(useUpdate.getState().check()).resolves.toBeUndefined();

    const s = useUpdate.getState();
    expect(s.checking).toBe(false);
    expect(s.available).toBe(false);
  });

  it("does not throw out of the action even on rejection", async () => {
    mockCheckForUpdate.mockRejectedValueOnce(makeIpcError());

    // Verify check() promise resolves (not rejects).
    await expect(useUpdate.getState().check()).resolves.not.toThrow();
  });
});

describe("useUpdate store — apply()", () => {
  beforeEach(resetStore);

  it("calls applyUpdate()", async () => {
    mockApplyUpdate.mockResolvedValueOnce(undefined);

    await useUpdate.getState().apply();

    expect(mockApplyUpdate).toHaveBeenCalledOnce();
  });

  it("swallows IpcError from applyUpdate without crashing", async () => {
    mockApplyUpdate.mockRejectedValueOnce(makeIpcError("apply_update_failed", "launcher error"));

    await expect(useUpdate.getState().apply()).resolves.toBeUndefined();
  });

  it("swallows generic errors from applyUpdate without crashing", async () => {
    mockApplyUpdate.mockRejectedValueOnce(new Error("unexpected"));

    await expect(useUpdate.getState().apply()).resolves.toBeUndefined();
  });

  // -- applyFailed surface-failure tests (PR #64 review) ----------------------

  it("sets applyFailed=true when applyUpdate() rejects", async () => {
    mockApplyUpdate.mockRejectedValueOnce(
      makeIpcError("apply_update_failed", "checks disabled"),
    );

    await useUpdate.getState().apply();

    expect(useUpdate.getState().applyFailed).toBe(true);
  });

  it("does not throw even when applyUpdate() rejects (applyFailed path)", async () => {
    mockApplyUpdate.mockRejectedValueOnce(makeIpcError());

    await expect(useUpdate.getState().apply()).resolves.toBeUndefined();
  });

  it("leaves applyFailed=false when applyUpdate() resolves", async () => {
    mockApplyUpdate.mockResolvedValueOnce(undefined);

    await useUpdate.getState().apply();

    expect(useUpdate.getState().applyFailed).toBe(false);
  });

  it("resets applyFailed to false at the start of a new apply() call", async () => {
    // Pre-set a prior failure
    useUpdate.setState({ applyFailed: true });

    // Next call succeeds
    mockApplyUpdate.mockResolvedValueOnce(undefined);

    await useUpdate.getState().apply();

    expect(useUpdate.getState().applyFailed).toBe(false);
  });
});

describe("useUpdate store — dismiss()", () => {
  beforeEach(resetStore);

  it("sets dismissed=true", () => {
    expect(useUpdate.getState().dismissed).toBe(false);

    useUpdate.getState().dismiss();

    expect(useUpdate.getState().dismissed).toBe(true);
  });

  it("does not affect other state fields when dismissed", () => {
    // Pre-populate some state
    useUpdate.setState({ available: true, version: "0.3.0", notes: "notes" });

    useUpdate.getState().dismiss();

    const s = useUpdate.getState();
    expect(s.dismissed).toBe(true);
    expect(s.available).toBe(true);
    expect(s.version).toBe("0.3.0");
    expect(s.notes).toBe("notes");
  });
});
