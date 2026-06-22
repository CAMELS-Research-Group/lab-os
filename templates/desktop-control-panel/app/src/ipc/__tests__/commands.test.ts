/**
 * Tests for the typed Tauri command wrappers.
 *
 * `@tauri-apps/api/core` is mocked so each test can assert that the wrapper
 * passes the right command name + argument shape to `invoke()`, returns the
 * right type on success, and raises a typed `IpcError` with the parsed
 * `kind` / `message` on failure.
 *
 * Spec: ADD §3.6 (command surface), ADD §3.10 (error envelope), task CL-25.
 */

import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

import {
  IpcError,
  acceptConsent,
  applyUpdate,
  cancelSession,
  checkForUpdate,
  endSession,
  getAppVersion,
  getEvaluationResult,
  getFirstRunPhase,
  getInstallState,
  getModelVersion,
  getPassage,
  getPhonemeTrends,
  getQueueStatus,
  getSessionHistory,
  getSettings,
  pauseSession,
  resumeSession,
  revokeConsent,
  setDifficulty,
  setL1,
  setReportUploadsEnabled,
  startFirstRunModelDownload,
  startSession,
  submitSessionFeedback,
} from "../commands";
import type {
  EvaluationResult,
  InstallState,
  Passage,
  PhonemeTrend,
  QueueStatus,
  SessionSummary,
  Settings,
  UpdateInfo,
} from "../types";
import { makeEvaluationResult } from "./fixtures";

// Re-cast the mocked module so tests get the vi.Mock interface for assertions.
const mockInvoke = vi.mocked(invoke);

afterEach(() => {
  mockInvoke.mockReset();
});

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

describe("identity wrappers", () => {
  it("getInstallState calls invoke with the right command name and returns the payload", async () => {
    const payload: InstallState = {
      uuid: "11111111-2222-3333-4444-555555555555",
      consent_state: "granted",
    };
    mockInvoke.mockResolvedValueOnce(payload);

    const result = await getInstallState();

    expect(mockInvoke).toHaveBeenCalledWith("get_install_state", undefined);
    expect(result).toEqual(payload);
  });

  it("acceptConsent calls invoke with no args", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await acceptConsent();
    expect(mockInvoke).toHaveBeenCalledWith("accept_consent", undefined);
  });

  it("revokeConsent calls invoke with no args", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await revokeConsent();
    expect(mockInvoke).toHaveBeenCalledWith("revoke_consent", undefined);
  });
});

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

describe("settings wrappers", () => {
  it("getSettings calls invoke with the right command name and returns the payload", async () => {
    const settings: Settings = {
      l1: "spa",
      regional_variety: "Caribbean",
      difficulty: "standard",
      report_uploads_enabled: true,
      update_checks_enabled: false,
    };
    mockInvoke.mockResolvedValueOnce(settings);

    const result = await getSettings();

    expect(mockInvoke).toHaveBeenCalledWith("get_settings", undefined);
    expect(result).toEqual(settings);
  });

  it("setL1 wraps l1 + variety in an `args` struct (normalizing undefined to null)", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setL1("spa", "Caribbean");
    expect(mockInvoke).toHaveBeenCalledWith("set_l1", {
      args: { l1: "spa", variety: "Caribbean" },
    });

    mockInvoke.mockResolvedValueOnce(undefined);
    await setL1("cmn");
    expect(mockInvoke).toHaveBeenLastCalledWith("set_l1", {
      args: { l1: "cmn", variety: null },
    });
  });

  it("setDifficulty wraps the named level in an `args` struct", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setDifficulty("strict");
    expect(mockInvoke).toHaveBeenCalledWith("set_difficulty", { args: { level: "strict" } });
  });

  it("setReportUploadsEnabled wraps the bool flag in an `args` struct", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setReportUploadsEnabled(false);
    expect(mockInvoke).toHaveBeenCalledWith("set_report_uploads_enabled", {
      args: { enabled: false },
    });
  });
});

// ---------------------------------------------------------------------------
// Recording lifecycle
// ---------------------------------------------------------------------------

describe("recording lifecycle wrappers", () => {
  it("startSession returns the SessionId as a bare string", async () => {
    mockInvoke.mockResolvedValueOnce("session-abc-123");
    const id = await startSession();
    expect(mockInvoke).toHaveBeenCalledWith("start_session", undefined);
    expect(id).toBe("session-abc-123");
  });

  it("pauseSession / resumeSession / cancelSession / endSession take no args", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await pauseSession();
    expect(mockInvoke).toHaveBeenLastCalledWith("pause_session", undefined);

    mockInvoke.mockResolvedValueOnce(undefined);
    await resumeSession();
    expect(mockInvoke).toHaveBeenLastCalledWith("resume_session", undefined);

    mockInvoke.mockResolvedValueOnce(undefined);
    await cancelSession();
    expect(mockInvoke).toHaveBeenLastCalledWith("cancel_session", undefined);

    mockInvoke.mockResolvedValueOnce(undefined);
    await endSession();
    expect(mockInvoke).toHaveBeenLastCalledWith("end_session", undefined);
  });
});

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

describe("evaluation wrappers", () => {
  it("getEvaluationResult sends sessionId and returns the typed result", async () => {
    const payload: EvaluationResult = makeEvaluationResult();
    mockInvoke.mockResolvedValueOnce(payload);

    const result = await getEvaluationResult("session-xyz");

    expect(mockInvoke).toHaveBeenCalledWith("get_evaluation_result", {
      sessionId: "session-xyz",
    });
    expect(result).toEqual(payload);
  });

  it("getEvaluationResult passes through null on a missing row", async () => {
    mockInvoke.mockResolvedValueOnce(null);
    const result = await getEvaluationResult("session-missing");
    expect(result).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Reporting
// ---------------------------------------------------------------------------

describe("reporting wrappers", () => {
  it("submitSessionFeedback normalizes optional note to null", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await submitSessionFeedback("session-abc", 4);
    expect(mockInvoke).toHaveBeenCalledWith("submit_session_feedback", {
      sessionId: "session-abc",
      rating: 4,
      note: null,
    });

    mockInvoke.mockResolvedValueOnce(undefined);
    await submitSessionFeedback("session-abc", 5, "thanks!");
    expect(mockInvoke).toHaveBeenLastCalledWith("submit_session_feedback", {
      sessionId: "session-abc",
      rating: 5,
      note: "thanks!",
    });
  });

  it("getQueueStatus returns the queue snapshot", async () => {
    const status: QueueStatus = {
      pending_count: 2,
      last_attempt_at: "2026-06-05T11:55:00Z",
      last_terminal_error: { code: "410", at: "2026-06-05T11:50:00Z" },
    };
    mockInvoke.mockResolvedValueOnce(status);
    const result = await getQueueStatus();
    expect(mockInvoke).toHaveBeenCalledWith("get_queue_status", undefined);
    expect(result).toEqual(status);
  });
});

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

describe("update wrappers", () => {
  it("checkForUpdate returns the typed UpdateInfo", async () => {
    const info: UpdateInfo = { available: true, version: "1.2.3", notes: "Bug fixes" };
    mockInvoke.mockResolvedValueOnce(info);
    const result = await checkForUpdate();
    expect(mockInvoke).toHaveBeenCalledWith("check_for_update", undefined);
    expect(result).toEqual(info);
  });

  it("applyUpdate takes no args", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await applyUpdate();
    expect(mockInvoke).toHaveBeenCalledWith("apply_update", undefined);
  });
});

// ---------------------------------------------------------------------------
// Storage (read-only)
// ---------------------------------------------------------------------------

describe("storage read wrappers", () => {
  it("getSessionHistory returns the wrapped sessions array", async () => {
    const summary: SessionSummary = {
      session_id: "session-abc",
      ended_at: "2026-06-05T12:30:00Z",
      duration_seconds: 180,
      flagged_count: 3,
      highest_error_phoneme: "ð",
    };
    mockInvoke.mockResolvedValueOnce({ sessions: [summary] });
    const result = await getSessionHistory();
    expect(mockInvoke).toHaveBeenCalledWith("get_session_history", undefined);
    expect(result).toEqual({ sessions: [summary] });
  });

  it("getPhonemeTrends returns the wrapped per_phoneme array", async () => {
    const trend: PhonemeTrend = {
      phoneme: "θ",
      example_word: "think",
      attempts_total: 24,
      flagged_total: 8,
      trend_direction: "improving",
      sessions_observed: 6,
      session_flag_rate: [0.5, 0.45, 0.4, 0.35, 0.3, 0.25],
    };
    mockInvoke.mockResolvedValueOnce({ per_phoneme: [trend] });
    const result = await getPhonemeTrends();
    expect(mockInvoke).toHaveBeenCalledWith("get_phoneme_trends", undefined);
    expect(result).toEqual({ per_phoneme: [trend] });
  });

  it("getPassage returns the bundled passage", async () => {
    const passage: Passage = {
      text: "The quick brown fox.",
      expected_ipa_per_word: [{ word: "The", ipa: ["ð", "ə"] }],
    };
    mockInvoke.mockResolvedValueOnce(passage);
    const result = await getPassage();
    expect(mockInvoke).toHaveBeenCalledWith("get_passage", undefined);
    expect(result).toEqual(passage);
  });
});

// ---------------------------------------------------------------------------
// Shared / first-run
// ---------------------------------------------------------------------------

describe("shared / first-run wrappers", () => {
  it("getAppVersion returns the bare version string", async () => {
    mockInvoke.mockResolvedValueOnce("1.0.0");
    const result = await getAppVersion();
    expect(mockInvoke).toHaveBeenCalledWith("get_app_version", undefined);
    expect(result).toBe("1.0.0");
  });

  it("getModelVersion passes through null when no model is installed", async () => {
    mockInvoke.mockResolvedValueOnce(null);
    const result = await getModelVersion();
    expect(mockInvoke).toHaveBeenCalledWith("get_model_version", undefined);
    expect(result).toBeNull();
  });

  it("getFirstRunPhase returns the PascalCase phase string", async () => {
    mockInvoke.mockResolvedValueOnce("ConsentPending");
    const result = await getFirstRunPhase();
    expect(mockInvoke).toHaveBeenCalledWith("get_first_run_phase", undefined);
    expect(result).toBe("ConsentPending");
  });

  it("startFirstRunModelDownload takes no args", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await startFirstRunModelDownload();
    expect(mockInvoke).toHaveBeenCalledWith("start_first_run_model_download", undefined);
  });
});

// ---------------------------------------------------------------------------
// IpcError parsing
// ---------------------------------------------------------------------------

describe("IpcError parsing", () => {
  it("parses the canonical { kind, message, recoverable } object", async () => {
    mockInvoke.mockRejectedValueOnce({
      kind: "storage",
      message: "storage error: db locked",
      recoverable: false,
    });

    await expect(getSettings()).rejects.toBeInstanceOf(IpcError);

    mockInvoke.mockRejectedValueOnce({
      kind: "microphone",
      message: "microphone error: permission denied",
      recoverable: true,
    });
    try {
      await startSession();
      throw new Error("expected throw");
    } catch (e) {
      expect(e).toBeInstanceOf(IpcError);
      const ipc = e as IpcError;
      expect(ipc.kind).toBe("microphone");
      expect(ipc.message).toBe("microphone error: permission denied");
      expect(ipc.recoverable).toBe(true);
    }
  });

  it("falls back to splitting a legacy 'kind: message' string", async () => {
    mockInvoke.mockRejectedValueOnce("invalid_state: bad transition");
    try {
      await pauseSession();
      throw new Error("expected throw");
    } catch (e) {
      expect(e).toBeInstanceOf(IpcError);
      const ipc = e as IpcError;
      expect(ipc.kind).toBe("invalid_state");
      expect(ipc.message).toBe("bad transition");
      expect(ipc.recoverable).toBe(false);
    }
  });

  it("classifies an unparseable rejection as kind = 'unknown'", async () => {
    mockInvoke.mockRejectedValueOnce("nope just plain string");
    try {
      await cancelSession();
      throw new Error("expected throw");
    } catch (e) {
      expect(e).toBeInstanceOf(IpcError);
      const ipc = e as IpcError;
      expect(ipc.kind).toBe("unknown");
      expect(ipc.message).toBe("nope just plain string");
    }
  });

  it("classifies a non-string, non-object rejection as 'unknown'", async () => {
    mockInvoke.mockRejectedValueOnce(42);
    try {
      await getAppVersion();
      throw new Error("expected throw");
    } catch (e) {
      expect(e).toBeInstanceOf(IpcError);
      const ipc = e as IpcError;
      expect(ipc.kind).toBe("unknown");
      expect(ipc.message).toBe("42");
    }
  });

  it("preserves the raw value in cause for debugging", async () => {
    const raw = { kind: "config", message: "bad path", recoverable: false };
    mockInvoke.mockRejectedValueOnce(raw);
    try {
      await getAppVersion();
      throw new Error("expected throw");
    } catch (e) {
      expect(e).toBeInstanceOf(IpcError);
      expect((e as IpcError).cause).toBe(raw);
    }
  });
});
