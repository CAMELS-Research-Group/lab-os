/**
 * Tests for the typed Tauri command wrappers.
 *
 * `@tauri-apps/api/core` is mocked so each test can assert that the wrapper
 * passes the right command name + argument shape to `invoke()`, returns the
 * right type on success, and raises a typed `IpcError` with the parsed
 * `kind` / `message` on failure.
 *
 * Covers exactly the surviving eight-command surface:
 *   get_settings, set_theme, set_report_uploads_enabled,
 *   set_update_checks_enabled, check_for_update, apply_update,
 *   get_app_version, clear_app_data
 */

import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

import {
  IpcError,
  applyUpdate,
  checkForUpdate,
  clearAppData,
  getAppVersion,
  getSettings,
  setReportUploadsEnabled,
  setTheme,
  setUpdateChecksEnabled,
} from "../commands";
import type { Settings, UpdateInfo } from "../types";

// Re-cast the mocked module so tests get the vi.Mock interface for assertions.
const mockInvoke = vi.mocked(invoke);

afterEach(() => {
  mockInvoke.mockReset();
});

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

describe("settings wrappers", () => {
  it("getSettings calls invoke with the right command name and returns the payload", async () => {
    const settings: Settings = {
      theme: "system",
      report_uploads_enabled: true,
      update_checks_enabled: false,
    };
    mockInvoke.mockResolvedValueOnce(settings);

    const result = await getSettings();

    expect(mockInvoke).toHaveBeenCalledWith("get_settings", undefined);
    expect(result).toEqual(settings);
  });

  it("setTheme wraps the theme in an `args` struct", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setTheme("dark");
    expect(mockInvoke).toHaveBeenCalledWith("set_theme", { args: { theme: "dark" } });
  });

  it("setReportUploadsEnabled wraps the bool flag in an `args` struct", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setReportUploadsEnabled(false);
    expect(mockInvoke).toHaveBeenCalledWith("set_report_uploads_enabled", {
      args: { enabled: false },
    });
  });

  it("setUpdateChecksEnabled wraps the bool flag in an `args` struct", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await setUpdateChecksEnabled(true);
    expect(mockInvoke).toHaveBeenCalledWith("set_update_checks_enabled", {
      args: { enabled: true },
    });
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
// App version + data
// ---------------------------------------------------------------------------

describe("app version + data wrappers", () => {
  it("getAppVersion returns the bare version string", async () => {
    mockInvoke.mockResolvedValueOnce("1.0.0");
    const result = await getAppVersion();
    expect(mockInvoke).toHaveBeenCalledWith("get_app_version", undefined);
    expect(result).toBe("1.0.0");
  });

  it("clearAppData returns the number of rows removed", async () => {
    mockInvoke.mockResolvedValueOnce(7);
    const result = await clearAppData();
    expect(mockInvoke).toHaveBeenCalledWith("clear_app_data", undefined);
    expect(result).toBe(7);
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
      kind: "invalid_state",
      message: "invalid_state error: bad transition",
      recoverable: true,
    });
    try {
      await clearAppData();
      throw new Error("expected throw");
    } catch (e) {
      expect(e).toBeInstanceOf(IpcError);
      const ipc = e as IpcError;
      expect(ipc.kind).toBe("invalid_state");
      expect(ipc.message).toBe("invalid_state error: bad transition");
      expect(ipc.recoverable).toBe(true);
    }
  });

  it("falls back to splitting a legacy 'kind: message' string", async () => {
    mockInvoke.mockRejectedValueOnce("invalid_state: bad transition");
    try {
      await getSettings();
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
      await getAppVersion();
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
