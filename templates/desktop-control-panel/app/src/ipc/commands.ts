/**
 * Typed wrappers for the Tauri command surface.
 *
 * Each wrapper hides the `invoke()` call site so screens don't need to remember
 * the Rust command name or hand-type the response shape. The frontend MUST go
 * through this layer — never call `@tauri-apps/api/core::invoke` directly —
 * so the type contract is single-source.
 *
 * Mirrors the Rust-side registration in `src-tauri/src/lib.rs::invoke_handler!`.
 * The surviving surface is exactly eight commands:
 *
 *   get_settings, set_theme, set_report_uploads_enabled,
 *   set_update_checks_enabled, check_for_update, apply_update,
 *   get_app_version, clear_app_data
 *
 * Tauri 2.x default: invoke args are matched **by parameter name**. Handlers
 * that take a single struct param (e.g. `set_theme(args: SetThemeArgs, …)`) do
 * NOT flatten — the wrapper must send `{ args: { theme } }`, because the
 * parameter Tauri is looking for is literally named `args`. The struct-arg
 * wrappers (`setTheme`, `setReportUploadsEnabled`, `setUpdateChecksEnabled`)
 * wrap accordingly.
 *
 * Error wire shape: `AppError` on the Rust side serializes as
 * `{ kind, message, recoverable }` (see `shared/error.rs`). The wrappers catch
 * the rejected `invoke()` promise and rethrow as a typed `IpcError`. If the
 * rejection value is a string (legacy or pre-AppError path), the wrapper falls
 * back to `kind: "unknown"` with the raw string in `message`.
 */

import { invoke } from "@tauri-apps/api/core";

import type { Settings, Theme, UpdateInfo } from "./types";

// ---------------------------------------------------------------------------
// IpcError — typed rejection wrapper
// ---------------------------------------------------------------------------

/**
 * Discriminator for a Tauri-command failure. Mirrors `AppError::kind()` in
 * `src-tauri/src/shared/error.rs` plus a `"unknown"` fallback for any error
 * the parser can't classify (legacy path, plugin errors, malformed strings).
 *
 * Open-ended `string` rather than a closed union so new Rust kinds don't force
 * a coordinated TS update — the frontend switches on the known set and treats
 * the rest as "unknown".
 */
export type IpcErrorKind = string;

/**
 * Typed error thrown by every command wrapper on failure. `kind` is the Rust
 * `AppError` variant in snake_case (e.g. `"storage"`, `"invalid_state"`);
 * `message` is the Display string; `recoverable` is the Rust-side hint for
 * retry-vs-banner UX.
 *
 * `cause` is the raw rejection value (the original `invoke()` reject) — kept
 * for debugging / logging, but UI code should switch on `kind`, not `cause`.
 */
export class IpcError extends Error {
  readonly kind: IpcErrorKind;
  readonly recoverable: boolean;
  readonly cause: unknown;

  constructor(kind: IpcErrorKind, message: string, recoverable: boolean, cause: unknown) {
    super(message);
    this.name = "IpcError";
    this.kind = kind;
    this.recoverable = recoverable;
    this.cause = cause;
  }
}

/**
 * Parse a rejection from `invoke()` into an `IpcError`. Handles three shapes:
 *
 * 1. The canonical `{ kind, message, recoverable }` object (what
 *    `AppError`'s custom `Serialize` impl emits today).
 * 2. A bare string `"<kind>: <message>"` — split on the first `": "` to
 *    extract the kind, with the remainder as `message`. Fallback path for
 *    legacy emitters that haven't migrated to `AppError` yet.
 * 3. Anything else — `kind = "unknown"`, message = `String(value)`.
 *
 * Defensive defaults: `recoverable = false` when the input is a bare string
 * or doesn't carry the field.
 */
function toIpcError(raw: unknown): IpcError {
  if (raw instanceof IpcError) return raw;

  // Canonical shape — object with `kind` and `message`.
  if (raw && typeof raw === "object") {
    const obj = raw as Record<string, unknown>;
    const kind = typeof obj.kind === "string" ? obj.kind : undefined;
    const message = typeof obj.message === "string" ? obj.message : undefined;
    const recoverable = typeof obj.recoverable === "boolean" ? obj.recoverable : false;
    if (kind && message !== undefined) {
      return new IpcError(kind, message, recoverable, raw);
    }
  }

  // String shape — try `"<kind>: <message>"`, otherwise opaque.
  if (typeof raw === "string") {
    const idx = raw.indexOf(": ");
    if (idx > 0) {
      const kind = raw.slice(0, idx);
      const message = raw.slice(idx + 2);
      return new IpcError(kind, message, false, raw);
    }
    return new IpcError("unknown", raw, false, raw);
  }

  return new IpcError("unknown", String(raw), false, raw);
}

/**
 * Shared `invoke()` shim. Awaits the underlying promise; on rejection wraps
 * the error and rethrows as a typed `IpcError`. Callers do not need
 * try/catch unless they intend to handle a specific `kind`.
 */
async function ipcInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (raw) {
    throw toIpcError(raw);
  }
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export async function getSettings(): Promise<Settings> {
  return ipcInvoke<Settings>("get_settings");
}

export async function setTheme(theme: Theme): Promise<void> {
  await ipcInvoke<void>("set_theme", { args: { theme } });
}

export async function setReportUploadsEnabled(enabled: boolean): Promise<void> {
  await ipcInvoke<void>("set_report_uploads_enabled", { args: { enabled } });
}

export async function setUpdateChecksEnabled(enabled: boolean): Promise<void> {
  await ipcInvoke<void>("set_update_checks_enabled", { args: { enabled } });
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

export async function checkForUpdate(): Promise<UpdateInfo> {
  return ipcInvoke<UpdateInfo>("check_for_update");
}

export async function applyUpdate(): Promise<void> {
  await ipcInvoke<void>("apply_update");
}

// ---------------------------------------------------------------------------
// App version + data
// ---------------------------------------------------------------------------

export async function getAppVersion(): Promise<string> {
  return ipcInvoke<string>("get_app_version");
}

/**
 * Deletes all on-disk app data managed by the backend and returns the number
 * of rows removed. Backs the Settings "Clear app data" control. Independent of
 * the frontend "Reset app / UI state" action, which only clears
 * Zustand/localStorage.
 */
export async function clearAppData(): Promise<number> {
  return ipcInvoke<number>("clear_app_data");
}
