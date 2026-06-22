/**
 * Friendly, user-facing copy keyed by error `kind`. Errors reach the frontend
 * as `{ kind, message, recoverable }` (see `ipc/commands.ts::IpcError` and the
 * Rust `AppError` serializer). The raw `message` is a developer diagnostic —
 * never the primary thing a learner should read. This map turns each `kind`
 * into a plain title + body; `ErrorNotice` renders it and tucks the raw
 * message behind a "Technical details" disclosure.
 *
 * Kind set mirrors the Rust side: `AppError::kind()` (shared/error.rs) and the
 * narrowed evaluation kinds from `orchestrator.rs::error_kind()`, plus a few
 * frontend-only kinds raised in the React layer.
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
  // --- Evaluation: model / data loading -----------------------------------
  model_load_failed: {
    title: "The practice model couldn't load",
    body: "The pronunciation model is missing or didn't pass its integrity check. Restart the app to try again; if it keeps happening, reinstall.",
  },
  model_download_failed: {
    title: "The download didn't finish",
    body: "We couldn't finish downloading the practice model. Check your internet connection and try again.",
  },
  threshold_load_failed: {
    title: "Couldn't load scoring settings",
    body: "Part of the analyzer's configuration couldn't be read. Restart the app; if it persists, reinstall.",
  },
  allophone_load_failed: {
    title: "Couldn't load pronunciation data",
    body: "Part of the analyzer's reference data couldn't be read. Restart the app; if it persists, reinstall.",
  },
  reference_load_failed: {
    title: "Couldn't load the reference passage",
    body: "The reference pronunciation for this passage couldn't be read. Restart the app; if it persists, reinstall.",
  },

  // --- Evaluation: analysis runtime ---------------------------------------
  vocab_mismatch: {
    title: "This passage isn't supported yet",
    body: "The analyzer doesn't recognize one of the sounds in this passage. Try a different passage; this one needs a fix on our side.",
  },
  scoring_failed: {
    title: "We couldn't analyze this reading",
    body: "The analyzer couldn't line up your reading with the passage. Try recording the passage again at a steady pace.",
  },
  audio_too_short: {
    title: "Your recording was too short",
    body: "We didn't catch enough audio to analyze. Read the whole passage aloud at a normal pace, then try again.",
  },
  inference_runtime: {
    title: "The analyzer couldn't run",
    body: "The on-device analyzer hit a problem on this device. Restart the app and try again.",
  },
  evaluation_failed: {
    title: "We couldn't analyze this reading",
    body: "Something interrupted the analysis. Your reading was still captured — try again on your next session.",
  },

  // --- Recording -----------------------------------------------------------
  microphone: {
    title: "Microphone unavailable",
    body: "We couldn't use your microphone. Check that the app has microphone permission and that a microphone is connected, then try again.",
  },
  simulated_capture_failed: {
    title: "Preview only",
    body: "Microphone access wasn't available, so this was a practice run with no analysis. Grant microphone permission to get feedback.",
  },

  // --- Storage -------------------------------------------------------------
  storage: {
    title: "Couldn't save to this device",
    body: "Saving to local storage failed. Make sure the device has free space, then try again.",
  },
  persistence_failed: {
    title: "Couldn't save this session",
    body: "Your reading was analyzed but couldn't be saved. Make sure the device has free space, then try again.",
  },
  session_clear_failed: {
    title: "Couldn't clear your data",
    body: "Your saved sessions couldn't be deleted just now. Please try again.",
  },
  end_session_failed: {
    title: "Couldn't finish this session",
    body: "Something interrupted saving this session. Your reading was still captured; try again on your next session.",
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

  // --- Feedback ------------------------------------------------------------
  feedback_save_failed: {
    title: "Couldn't save your feedback",
    body: "Your note wasn't saved just now. Please try again.",
  },

  // --- Update --------------------------------------------------------------
  update: {
    title: "Update check failed",
    body: "We couldn't check for updates right now. This won't affect practice — try again later.",
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
