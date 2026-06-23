/**
 * Friendly, user-facing copy keyed by error `kind`. Errors reach the frontend
 * as `{ kind, message, recoverable }` (see `ipc/commands.ts::IpcError` and the
 * Rust `AppError` serializer). The raw `message` is a developer diagnostic —
 * never the primary thing a user should read. This map turns each `kind` into a
 * plain title + body; `ErrorNotice` renders it and tucks the raw message behind
 * a "Technical details" disclosure.
 *
 * Kind set mirrors the generic-shell Rust surface: `AppError::kind()`
 * (shared/error.rs) plus a few frontend-only kinds raised in the React layer.
 * The ESL-specific kinds (recording / evaluation / model-download) were removed
 * with the ESL UI strip.
 *
 * Any kind without an entry falls back to a safe generic message, so an
 * unmapped kind never leaks a raw string to the UI.
 */
export type ErrorCopy = {
  title: string;
  body: string;
};

const FALLBACK: ErrorCopy = {
  title: "Something went wrong",
  body: "An unexpected problem occurred. Please try again — if it keeps happening, restart the app.",
};

const COPY: Record<string, ErrorCopy> = {
  // --- Storage -------------------------------------------------------------
  storage: {
    title: "Couldn't save to this device",
    body: "Saving to local storage failed. Make sure the device has free space, then try again.",
  },
  data_clear_failed: {
    title: "Couldn't clear your data",
    body: "Your saved data couldn't be deleted just now. Please try again.",
  },

  // --- Settings ------------------------------------------------------------
  settings_load_failed: {
    title: "Couldn't load settings",
    body: "Your settings couldn't be read. Please restart the app.",
  },
  settings_save_failed: {
    title: "Couldn't save that setting",
    body: "The change wasn't saved. Please try again.",
  },

  // --- Update --------------------------------------------------------------
  update: {
    title: "Update check failed",
    body: "We couldn't check for updates right now. This won't affect normal use — try again later.",
  },
  update_apply_failed: {
    title: "Couldn't open the download",
    body: "We couldn't open the update download. Please try again.",
  },

  // --- Catch-alls for lower-level kinds -----------------------------------
  backend: {
    title: "Couldn't reach the service",
    body: "A network request didn't go through. Check your connection and try again.",
  },
  keyring: {
    title: "Couldn't access secure storage",
    body: "The app couldn't read its secure storage. Some features may be limited until you restart.",
  },
  invalid_state: {
    title: "Something went out of sync",
    body: "The app got into an unexpected state. Please restart the app and try again.",
  },
  config: {
    title: "The app isn't set up correctly",
    body: "A required component is missing or misconfigured. Reinstalling the app usually fixes this.",
  },
};

/** Look up friendly copy for an error kind, falling back to a generic message. */
export function errorCopyFor(kind: string): ErrorCopy {
  return COPY[kind] ?? FALLBACK;
}
