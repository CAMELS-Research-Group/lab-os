/**
 * Typed listeners for the Rust-emitted event surface.
 *
 * Snake_case field names match the Rust `#[derive(Serialize)]` output. Every
 * helper returns the `UnlistenFn` from `@tauri-apps/api/event`; callers MUST
 * invoke it on unmount to avoid leaking listeners across re-mounts.
 *
 * Channel names are exported as `const` strings (`EVT_*`) so call sites can
 * reference them by identifier rather than literal — typo-safety at the type
 * checker rather than at runtime.
 *
 * SCOPE: the generic-shell event surface — the updater channels. The former
 * ESL event surface (recording / eval / upload / model-download / identity)
 * was removed with the ESL UI strip.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ---------------------------------------------------------------------------
// Channel name constants
// ---------------------------------------------------------------------------

export const EVT_UPDATE_DOWNLOAD_PROGRESS = "update:download_progress" as const;
export const EVT_UPDATE_READY = "update:ready" as const;
export const EVT_UPDATE_ERROR = "update:error" as const;

/**
 * Union of every supported event channel — typo-safe references at call
 * sites. Open-ended `string` intentionally NOT used; this is the closed
 * compile-time check.
 */
export type EventChannel =
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
 */
export function subscribe<T>(
  channel: EventChannel,
  handler: (payload: T) => void
): Promise<UnlistenFn> {
  return listen<T>(channel, (e) => handler(e.payload));
}

// ---------------------------------------------------------------------------
// update:* (updater spine)
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
