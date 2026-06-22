/**
 * Typed listeners for the Rust-emitted event surface.
 *
 * Mirrors the wire payloads defined in:
 *   - `src-tauri/src/recording/commands.rs::RecordingLevelEvent` (CL-13)
 *   - `src-tauri/src/evaluation/orchestrator.rs::{ProgressPayload, DonePayload, ErrorPayload}` (CL-19)
 *   - Identity / upload / model-download / update events per ADD §3.7 (deferred
 *     emitters land with CL-7 / CL-8 / CL-10 / CL-23-lite / CL-24).
 *
 * Snake_case field names match the Rust `#[derive(Serialize)]` output. Every
 * helper returns the `UnlistenFn` from `@tauri-apps/api/event`; callers MUST
 * invoke it on unmount to avoid leaking listeners across re-mounts.
 *
 * Channel names are exported as `const` strings (`EVT_*`) so call sites can
 * reference them by identifier rather than literal — typo-safety at the type
 * checker rather than at runtime.
 *
 * Spec: ADD §3.7 (event surface), tasks CL-13 + CL-19 (landed),
 * CL-7/CL-8/CL-10/CL-23-lite/CL-24 (deferred emitters), task CL-25.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { EvaluationResult, FeedbackEntry, FlaggedPhoneme, SessionId, TerminalErrorCode } from "./types";

// ---------------------------------------------------------------------------
// Channel name constants
// ---------------------------------------------------------------------------

export const EVT_IDENTITY_REGISTRATION_SUCCEEDED = "identity:registration_succeeded" as const;
export const EVT_IDENTITY_REGISTRATION_FAILED = "identity:registration_failed" as const;

export const EVT_RECORDING_LEVEL = "recording:level" as const;
export const EVT_RECORDING_ERROR = "recording:error" as const;

export const EVT_EVAL_PROGRESS = "eval:progress" as const;
export const EVT_EVAL_DONE = "eval:done" as const;
export const EVT_EVAL_ERROR = "eval:error" as const;

export const EVT_UPLOAD_SUCCEEDED = "upload:succeeded" as const;
export const EVT_UPLOAD_TERMINAL_ERROR = "upload:terminal_error" as const;

export const EVT_MODEL_DOWNLOAD_PROGRESS = "model_download:progress" as const;
export const EVT_MODEL_DOWNLOAD_DONE = "model_download:done" as const;
export const EVT_MODEL_DOWNLOAD_ERROR = "model_download:error" as const;

export const EVT_UPDATE_DOWNLOAD_PROGRESS = "update:download_progress" as const;
export const EVT_UPDATE_READY = "update:ready" as const;
export const EVT_UPDATE_ERROR = "update:error" as const;

/**
 * Union of every supported event channel — typo-safe references at call
 * sites. Open-ended `string` intentionally NOT used; this is the closed
 * compile-time check.
 */
export type EventChannel =
  | typeof EVT_IDENTITY_REGISTRATION_SUCCEEDED
  | typeof EVT_IDENTITY_REGISTRATION_FAILED
  | typeof EVT_RECORDING_LEVEL
  | typeof EVT_RECORDING_ERROR
  | typeof EVT_EVAL_PROGRESS
  | typeof EVT_EVAL_DONE
  | typeof EVT_EVAL_ERROR
  | typeof EVT_UPLOAD_SUCCEEDED
  | typeof EVT_UPLOAD_TERMINAL_ERROR
  | typeof EVT_MODEL_DOWNLOAD_PROGRESS
  | typeof EVT_MODEL_DOWNLOAD_DONE
  | typeof EVT_MODEL_DOWNLOAD_ERROR
  | typeof EVT_UPDATE_DOWNLOAD_PROGRESS
  | typeof EVT_UPDATE_READY
  | typeof EVT_UPDATE_ERROR;

// ---------------------------------------------------------------------------
// Generic subscribe helper
// ---------------------------------------------------------------------------

/**
 * Generic subscription helper. Wraps `@tauri-apps/api/event::listen` so the
 * caller receives the unwrapped `payload` directly instead of the full
 * `Event<T>` envelope.
 *
 * Returns the `UnlistenFn` promise; the caller MUST invoke the resolved
 * function on unmount (or component teardown) to avoid leaking listeners.
 *
 * Every channel-specific `listen*` helper below is a thin wrapper over this.
 */
export function subscribe<T>(
  channel: EventChannel,
  handler: (payload: T) => void
): Promise<UnlistenFn> {
  return listen<T>(channel, (e) => handler(e.payload));
}

// ---------------------------------------------------------------------------
// Identity (CL-7 / CL-8 — emitters deferred)
// ---------------------------------------------------------------------------

/** `identity:registration_succeeded` — empty payload. */
export type IdentityRegistrationSucceededEvent = Record<string, never>;

/** `identity:registration_failed` — backend rejection or network failure. */
export type IdentityRegistrationFailedEvent = {
  error: string;
};

export function listenIdentityRegistrationSucceeded(
  cb: (e: IdentityRegistrationSucceededEvent) => void
): Promise<UnlistenFn> {
  return subscribe<IdentityRegistrationSucceededEvent>(
    EVT_IDENTITY_REGISTRATION_SUCCEEDED,
    cb
  );
}

export function listenIdentityRegistrationFailed(
  cb: (e: IdentityRegistrationFailedEvent) => void
): Promise<UnlistenFn> {
  return subscribe<IdentityRegistrationFailedEvent>(
    EVT_IDENTITY_REGISTRATION_FAILED,
    cb
  );
}

// ---------------------------------------------------------------------------
// recording:level (CL-13)
// ---------------------------------------------------------------------------

/** RMS level forwarded from the cpal audio thread at ~10 Hz. */
export type RecordingLevelEvent = {
  rms: number;
};

export function listenRecordingLevel(
  cb: (e: RecordingLevelEvent) => void
): Promise<UnlistenFn> {
  return subscribe<RecordingLevelEvent>(EVT_RECORDING_LEVEL, cb);
}

// ---------------------------------------------------------------------------
// recording:error (CL-13 follow-up — defined but NOT yet emitted)
// ---------------------------------------------------------------------------

/**
 * Recovery-path error from the recording session. The Rust side mentions this
 * event in a comment (`recording/session.rs:257`) but does not yet emit it; it
 * is tracked as a CL-13 follow-up. Defining the type + helper here lets
 * downstream callers compile against it ahead of the Rust-side wiring.
 */
export type RecordingErrorEvent = {
  kind: string;
  message: string;
};

export function listenRecordingError(
  cb: (e: RecordingErrorEvent) => void
): Promise<UnlistenFn> {
  return subscribe<RecordingErrorEvent>(EVT_RECORDING_ERROR, cb);
}

// ---------------------------------------------------------------------------
// eval:progress / eval:done / eval:error (CL-19)
// ---------------------------------------------------------------------------

/**
 * Lightweight per-chunk partial. Mirrors `orchestrator::WireChunkPartial`.
 * Always `null` in the current production flow — the chunked-then-aligned
 * pipeline runs forced alignment once after the last chunk, so a progressive
 * partial isn't available at chunk-emission time. Reserved on the wire shape
 * for a future progressive-UI path.
 */
export type PartialChunkResult = {
  flagged_phonemes_so_far: FlaggedPhoneme[];
  mean_certainty_so_far: Record<string, number>;
};

export type EvalProgressEvent = {
  session_id: SessionId;
  stage: "chunk" | "stopping" | "building_feedback" | "persisting" | string;
  pct: number;
  partial_result: PartialChunkResult | null;
};

/**
 * `eval:done` payload.
 *
 * ADD §3.7 documents this as `{ session_id, result }`. The current Rust
 * orchestrator (`evaluation/orchestrator.rs`) emits `{ result, feedback }` —
 * `result` already carries `session_id`, and CL-18 adds `feedback`. This
 * wrapper matches the live impl; flagged as ADD doc lag.
 */
export type EvalDoneEvent = {
  result: EvaluationResult;
  feedback: FeedbackEntry[];
};

export type EvalErrorEvent = {
  session_id: SessionId;
  kind: string;
  message: string;
};

export function listenEvalProgress(
  cb: (e: EvalProgressEvent) => void
): Promise<UnlistenFn> {
  return subscribe<EvalProgressEvent>(EVT_EVAL_PROGRESS, cb);
}

export function listenEvalDone(
  cb: (e: EvalDoneEvent) => void
): Promise<UnlistenFn> {
  return subscribe<EvalDoneEvent>(EVT_EVAL_DONE, cb);
}

export function listenEvalError(
  cb: (e: EvalErrorEvent) => void
): Promise<UnlistenFn> {
  return subscribe<EvalErrorEvent>(EVT_EVAL_ERROR, cb);
}

// ---------------------------------------------------------------------------
// upload:succeeded / upload:terminal_error (CL-10 — emitters deferred)
// ---------------------------------------------------------------------------

export type UploadSucceededEvent = {
  session_id: SessionId;
};

export type UploadTerminalErrorEvent = {
  code: TerminalErrorCode;
};

export function listenUploadSucceeded(
  cb: (e: UploadSucceededEvent) => void
): Promise<UnlistenFn> {
  return subscribe<UploadSucceededEvent>(EVT_UPLOAD_SUCCEEDED, cb);
}

export function listenUploadTerminalError(
  cb: (e: UploadTerminalErrorEvent) => void
): Promise<UnlistenFn> {
  return subscribe<UploadTerminalErrorEvent>(EVT_UPLOAD_TERMINAL_ERROR, cb);
}

// ---------------------------------------------------------------------------
// model_download:* (CL-24 — emitters deferred)
// ---------------------------------------------------------------------------

export type ModelDownloadProgressEvent = {
  bytes_done: number;
  bytes_total: number;
};

/** `model_download:done` — empty payload. */
export type ModelDownloadDoneEvent = Record<string, never>;

export type ModelDownloadErrorEvent = {
  error: string;
};

export function listenModelDownloadProgress(
  cb: (e: ModelDownloadProgressEvent) => void
): Promise<UnlistenFn> {
  return subscribe<ModelDownloadProgressEvent>(EVT_MODEL_DOWNLOAD_PROGRESS, cb);
}

export function listenModelDownloadDone(
  cb: (e: ModelDownloadDoneEvent) => void
): Promise<UnlistenFn> {
  return subscribe<ModelDownloadDoneEvent>(EVT_MODEL_DOWNLOAD_DONE, cb);
}

export function listenModelDownloadError(
  cb: (e: ModelDownloadErrorEvent) => void
): Promise<UnlistenFn> {
  return subscribe<ModelDownloadErrorEvent>(EVT_MODEL_DOWNLOAD_ERROR, cb);
}

// ---------------------------------------------------------------------------
// update:* (CL-23-lite — emitters deferred)
// ---------------------------------------------------------------------------

export type UpdateDownloadProgressEvent = {
  pct: number;
};

/** `update:ready` — empty payload. */
export type UpdateReadyEvent = Record<string, never>;

export type UpdateErrorEvent = {
  error: string;
};

export function listenUpdateDownloadProgress(
  cb: (e: UpdateDownloadProgressEvent) => void
): Promise<UnlistenFn> {
  return subscribe<UpdateDownloadProgressEvent>(EVT_UPDATE_DOWNLOAD_PROGRESS, cb);
}

export function listenUpdateReady(
  cb: (e: UpdateReadyEvent) => void
): Promise<UnlistenFn> {
  return subscribe<UpdateReadyEvent>(EVT_UPDATE_READY, cb);
}

export function listenUpdateError(
  cb: (e: UpdateErrorEvent) => void
): Promise<UnlistenFn> {
  return subscribe<UpdateErrorEvent>(EVT_UPDATE_ERROR, cb);
}
