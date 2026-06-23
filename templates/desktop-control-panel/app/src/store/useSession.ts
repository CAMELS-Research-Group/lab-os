import { create } from "zustand";
import { persist } from "zustand/middleware";

// Generic shell session/UI state. The ESL session fields (L1 profile, practice
// history, recording, evaluation, feedback) were removed with the ESL UI strip;
// what remains is the first-run gate plus generic reset.

export type SessionState = {
  // First-run gate. Flips true once the user has completed the generic
  // first-run flow (the Welcome gate). The only persisted state.
  hasCompletedFirstRun: boolean;

  completeFirstRun: () => void;
  resetAll: () => void;
};

export const useSession = create<SessionState>()(
  persist(
    (set) => ({
      hasCompletedFirstRun: false,

      completeFirstRun: () => set({ hasCompletedFirstRun: true }),

      resetAll: () => set({ hasCompletedFirstRun: false }),
    }),
    {
      // Local-storage key namespace — rename after forking (machine identifier).
      name: "default-app:session",
      // Persist only the first-run flag so a relaunch lands on the right gate.
      partialize: (s) => ({
        hasCompletedFirstRun: s.hasCompletedFirstRun,
      }),
    }
  )
);
