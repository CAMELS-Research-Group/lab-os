import { create } from "zustand";

import { checkForUpdate, applyUpdate } from "../ipc/commands";

// ---------------------------------------------------------------------------
// UpdateState — shape
// ---------------------------------------------------------------------------

export type UpdateState = {
  /** True while `checkForUpdate()` is in-flight. */
  checking: boolean;
  /** True when the Rust layer reports a newer version is available. */
  available: boolean;
  /** Version string from the manifest, or null when unavailable / not checked. */
  version: string | null;
  /** Release notes from the manifest, or null when unavailable / not checked. */
  notes: string | null;
  /**
   * In-memory dismissal flag. Flipped by the user clicking "Later" on the
   * banner. Not persisted — the banner reappears next launch if the update is
   * still pending (acceptable for the CL-23-lite path).
   */
  dismissed: boolean;
  /**
   * Set to true when the most recent user-initiated `apply()` call failed
   * (e.g. the Rust side returned ChecksDisabled or OpenBrowserFailed). Reset
   * to false at the start of each new `apply()` call.
   *
   * Note: the background `check()` intentionally does NOT touch this flag —
   * silent-swallow is correct for the background check; surfacing failure is
   * only appropriate for the user-initiated apply action.
   */
  applyFailed: boolean;

  /**
   * Poll the Rust side for a new version. Sets `checking=true`, calls
   * `checkForUpdate()`, populates state from the result, then clears `checking`.
   * Swallows `IpcError` — callers never need to catch. On failure, `available`
   * remains false and `checking` is cleared.
   */
  check: () => Promise<void>;

  /**
   * Trigger the update flow. Delegates to `applyUpdate()` and swallows any
   * `IpcError` so the store never crashes the banner. The Rust side opens the
   * release download page in the system browser (which resolves the newest
   * installer and carries install instructions); on failure `applyFailed` is
   * set to true so the banner can prompt a retry. `apply()` itself never throws.
   *
   * Silent-swallow is correct only for the background `check()`. A user-initiated
   * apply must surface failure so the banner can react.
   */
  apply: () => Promise<void>;

  /**
   * Mark the banner as dismissed for this session. The flag is volatile — it
   * resets when the app relaunches.
   */
  dismiss: () => void;
};

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

export const useUpdate = create<UpdateState>()((set) => ({
  checking: false,
  available: false,
  version: null,
  notes: null,
  dismissed: false,
  applyFailed: false,

  check: async () => {
    set({ checking: true });
    try {
      const info = await checkForUpdate();
      set({
        checking: false,
        available: info.available,
        version: info.version,
        notes: info.notes,
      });
    } catch {
      // IpcError (or any unexpected throw) — leave a safe state.
      // Silent-swallow is correct here: the background check must never nag.
      set({ checking: false, available: false, version: null, notes: null });
    }
  },

  apply: async () => {
    // Reset any prior failure before each attempt.
    set({ applyFailed: false });
    try {
      await applyUpdate();
    } catch {
      // Silent-swallow is correct only for the background check. A user-
      // initiated apply must surface failure so the banner can prompt a retry.
      set({ applyFailed: true });
    }
  },

  dismiss: () => set({ dismissed: true }),
}));
