/**
 * Typed wrappers for the Tauri command surface.
 *
 * Each wrapper hides the `invoke()` call site so screens don't need to remember
 * the Rust command name or hand-type the response shape. The frontend MUST go
 * through this layer — never call `@tauri-apps/api/core::invoke` directly —
 * so the type contract is single-source.
 *
 * Mirrors the Rust-side registration in `src-tauri/src/lib.rs::invoke_handler!`
 * (live handlers) and the ADD §3.6 command surface (full contract — some
 * handlers do not yet exist; runtime calls will reject as `IpcError` until
 * CL-7 / CL-8 / CL-10 / CL-20-22 / CL-23-lite land).
 *
 * Tauri 2.x default: invoke args are matched **by parameter name**. For
 * handlers with individual params (e.g. `get_evaluation_result(session_id:
 * String, …)`), Tauri's camel↔snake remapping lets the wrapper pass
 * `{ sessionId }`. For handlers that take a single struct param (e.g.
 * `set_l1(args: SetL1Args, …)`), Tauri does NOT flatten — the wrapper must
 * send `{ args: { l1, variety } }`, because the parameter Tauri is looking
 * for is literally named `args`. Sending the inner fields directly causes
 * deserialization to fail at runtime ("`args` not found"). The three struct-
 * arg wrappers (`setL1`, `setDifficulty`, `setReportUploadsEnabled`) wrap
 * accordingly.
 *
 * Error wire shape: `AppError` on the Rust side serializes as
 * `{ kind, message, recoverable }` (see `shared/error.rs`). The wrappers catch
 * the rejected `invoke()` promise and rethrow as a typed `IpcError`. If the
 * rejection value is a string (legacy or pre-AppError path), the wrapper falls
 * back to `kind: "unknown"` with the raw string in `message`.
 *
 * Spec: ADD §3.6 (command surface), ADD §3.10 (error envelope), task CL-25.
 */

import { invoke } from "@tauri-apps/api/core";

import type {
  ConsentState,
  DifficultyLevel,
  EvaluationResult,
  FirstRunPhase,
  InstallState,
  Passage,
  PhonemeTrend,
  QueueStatus,
  SessionId,
  SessionSummary,
  Settings,
  UpdateInfo,
} from "./types";

// ---------------------------------------------------------------------------
// IpcError — typed rejection wrapper
// ---------------------------------------------------------------------------

/**
 * Discriminator for a Tauri-command failure. Mirrors `AppError::kind()` in
 * `src-tauri/src/shared/error.rs` plus a `"unknown"` fallback for any error
 * the parser can't classify (legacy path, plugin errors, malformed strings).
 *
 * Open-ended `string` rather than a closed union so new Rust kinds added in
 * later tasks (e.g. CL-7 / CL-23) don't force a coordinated TS update — the
 * frontend can switch on the known set and treat the rest as "unknown".
 */
export type IpcErrorKind = string;

/**
 * Typed error thrown by every command wrapper on failure. `kind` is the Rust
 * `AppError` variant in snake_case (e.g. `"storage"`, `"microphone"`,
 * `"invalid_state"`); `message` is the Display string; `recoverable` is the
 * Rust-side hint for retry-vs-banner UX (per ADD §3.10).
 *
 * `cause` is the raw rejection value (the original `invoke()` reject) — kept
 * for debugging / logging, but UI code should switch on `kind`, not `cause`.
 *
 * V1 wire shape (today): `{ kind, message, recoverable }` JSON object.
 * Legacy / fallback path: a bare string, parsed as
 * `"<kind>: <message>"` if it matches that shape, otherwise
 * `kind = "unknown"` with the full string as `message`.
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
 * or doesn't carry the field. The UX layer can override per-kind if needed.
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
// Identity (CL-7 / CL-8)
//
// Handlers are not yet registered in `lib.rs::invoke_handler!`; the wrappers
// land now so the consent / settings surfaces can compile and so CL-7/CL-8
// can wire the Rust side without touching the React layer again.
// ---------------------------------------------------------------------------

export async function getInstallState(): Promise<InstallState> {
  return ipcInvoke<InstallState>("get_install_state");
}

export async function acceptConsent(): Promise<void> {
  await ipcInvoke<void>("accept_consent");
}

export async function revokeConsent(): Promise<void> {
  await ipcInvoke<void>("revoke_consent");
}

// ---------------------------------------------------------------------------
// Settings (CL-9)
// ---------------------------------------------------------------------------

export async function getSettings(): Promise<Settings> {
  return ipcInvoke<Settings>("get_settings");
}

export async function setL1(l1: string, variety?: string | null): Promise<void> {
  await ipcInvoke<void>("set_l1", { args: { l1, variety: variety ?? null } });
}

export async function setDifficulty(level: DifficultyLevel): Promise<void> {
  await ipcInvoke<void>("set_difficulty", { args: { level } });
}

export async function setReportUploadsEnabled(enabled: boolean): Promise<void> {
  await ipcInvoke<void>("set_report_uploads_enabled", { args: { enabled } });
}

export async function setUpdateChecksEnabled(enabled: boolean): Promise<void> {
  await ipcInvoke<void>("set_update_checks_enabled", { args: { enabled } });
}

// ---------------------------------------------------------------------------
// Recording lifecycle (CL-13)
// ---------------------------------------------------------------------------

/**
 * ADD §3.6 documents this as returning `{ session_id }`. The current Rust
 * handler (`recording/commands.rs::start_session`) returns the bare
 * `SessionId` newtype (which `serde(transparent)`s to a JSON string). This
 * wrapper matches the Rust impl, not the doc — flagged as ADD doc lag
 * (close the gap when the ADD is refreshed or when start_session adds
 * sibling fields).
 */
export async function startSession(): Promise<SessionId> {
  return ipcInvoke<SessionId>("start_session");
}

export async function pauseSession(): Promise<void> {
  await ipcInvoke<void>("pause_session");
}

export async function resumeSession(): Promise<void> {
  await ipcInvoke<void>("resume_session");
}

export async function cancelSession(): Promise<void> {
  await ipcInvoke<void>("cancel_session");
}

export async function endSession(): Promise<void> {
  await ipcInvoke<void>("end_session");
}

// ---------------------------------------------------------------------------
// Evaluation (CL-19)
// ---------------------------------------------------------------------------

/**
 * Late-arrival fetch for an already-persisted result (replay after a window
 * reload, race against `eval:done`). Rust returns `Option<EvaluationResult>`;
 * serde renders `Some(x)` as `x` and `None` as `null`.
 */
export async function getEvaluationResult(
  sessionId: SessionId
): Promise<EvaluationResult | null> {
  return ipcInvoke<EvaluationResult | null>("get_evaluation_result", {
    sessionId,
  });
}

// ---------------------------------------------------------------------------
// Reporting (CL-10 — Rust handler deferred)
// ---------------------------------------------------------------------------

/**
 * NOTE(CL-10): the Rust handler is not authored yet. Three-param individual
 * signatures (the current shape below) and a single struct-arg signature
 * require different invoke shapes — see the header comment on struct-arg
 * wrapping. When CL-10 lands, verify the handler signature and adjust the
 * call to either keep individual params (current shape) or wrap as
 * `{ args: { sessionId, rating, note } }`.
 */
export async function submitSessionFeedback(
  sessionId: SessionId,
  rating: number,
  note?: string | null
): Promise<void> {
  await ipcInvoke<void>("submit_session_feedback", {
    sessionId,
    rating,
    note: note ?? null,
  });
}

export async function getQueueStatus(): Promise<QueueStatus> {
  return ipcInvoke<QueueStatus>("get_queue_status");
}

/**
 * App-level feedback from the "Give Feedback" modal. Free-form `note` and/or an
 * optional `rating`, persisted to the local `feedback` table. Stored on-device
 * only — nothing is transmitted (V1 privacy posture). At least one of `note` /
 * `rating` must be present or the Rust side rejects it as invalid_state.
 *
 * Distinct from `submitSessionFeedback`, which is session-scoped (CL-10).
 */
export async function submitFeedback(
  note?: string | null,
  rating?: number | null
): Promise<void> {
  await ipcInvoke<void>("submit_feedback", {
    args: { rating: rating ?? null, note: note ?? null },
  });
}

// ---------------------------------------------------------------------------
// Update (CL-23-lite — Rust handler deferred)
// ---------------------------------------------------------------------------

export async function checkForUpdate(): Promise<UpdateInfo> {
  return ipcInvoke<UpdateInfo>("check_for_update");
}

export async function applyUpdate(): Promise<void> {
  await ipcInvoke<void>("apply_update");
}

// ---------------------------------------------------------------------------
// Storage — read-only. `get_session_history` landed its Rust handler (#117 —
// storage/commands.rs); `get_phoneme_trends` landed its handler in #147.
// `get_passage` lives in `evaluation/reference_ipa.rs`.
// ---------------------------------------------------------------------------

export async function getSessionHistory(): Promise<{ sessions: SessionSummary[] }> {
  return ipcInvoke<{ sessions: SessionSummary[] }>("get_session_history");
}

export async function getPhonemeTrends(): Promise<{ per_phoneme: PhonemeTrend[] }> {
  return ipcInvoke<{ per_phoneme: PhonemeTrend[] }>("get_phoneme_trends");
}

/**
 * Deletes all on-disk session/history rows (the `sessions` table and its
 * dependent `upload_queue` rows). Settings and install identity are left
 * intact. Backs the Settings "Clear session data" control. Returns the number
 * of session rows deleted. Independent of the frontend "Reset app / UI state"
 * action, which only clears Zustand/localStorage.
 */
export async function clearSessionData(): Promise<number> {
  return ipcInvoke<number>("clear_session_data");
}

export async function getPassage(): Promise<Passage> {
  return ipcInvoke<Passage>("get_passage");
}

// ---------------------------------------------------------------------------
// Shared / first-run (CL-24 — model download handler deferred)
// ---------------------------------------------------------------------------

export async function getAppVersion(): Promise<string> {
  return ipcInvoke<string>("get_app_version");
}

export async function getModelVersion(): Promise<string | null> {
  return ipcInvoke<string | null>("get_model_version");
}

export async function getFirstRunPhase(): Promise<FirstRunPhase> {
  return ipcInvoke<FirstRunPhase>("get_first_run_phase");
}

export async function startFirstRunModelDownload(): Promise<void> {
  await ipcInvoke<void>("start_first_run_model_download");
}

// ---------------------------------------------------------------------------
// Re-exports for convenience
// ---------------------------------------------------------------------------

export type { ConsentState };
