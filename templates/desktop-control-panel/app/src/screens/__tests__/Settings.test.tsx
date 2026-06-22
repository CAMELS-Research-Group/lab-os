/**
 * Unit tests for the Settings screen — Language editor + reset-fade overlay.
 *
 * Coverage:
 *  (a) Language card (LanguageSelect): dropdown selection persists; "Other…"
 *      reveals free text and persists; a saved custom value reopens as "Other…";
 *      dialect input persists; Save is disabled when empty.
 *  (b) Reset fade (#114): click "Reset app", confirm, assert .reset-fade overlay
 *      exists. Fake timers prevent the post-confirm setTimeout from firing
 *      resetAll/navigation on an unmounted tree.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { useSession } from "../../store/useSession";
import { setL1 } from "../../ipc/commands";
import Settings from "../Settings";

// ---------------------------------------------------------------------------
// Module mock — ipc/commands (same pattern as UpdateBanner.test.tsx)
// ---------------------------------------------------------------------------

vi.mock("../../ipc/commands", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../ipc/commands")>();
  return {
    ...real,
    getSettings: vi.fn().mockResolvedValue({ update_checks_enabled: false }),
    setUpdateChecksEnabled: vi.fn().mockResolvedValue(undefined),
    setL1: vi.fn().mockResolvedValue(undefined),
    clearSessionData: vi.fn().mockResolvedValue(0),
  };
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function renderSettings() {
  return render(
    <MemoryRouter>
      <Settings />
    </MemoryRouter>
  );
}

// ---------------------------------------------------------------------------
// Reset store state between tests
// ---------------------------------------------------------------------------

beforeEach(() => {
  useSession.setState({
    l1: "",
    regionalVariety: "",
    hasCompletedFirstRun: false,
  });
  vi.clearAllMocks();
});

// ---------------------------------------------------------------------------
// (a) Language card — LanguageSelect dropdown + Other free-text + dialect
// ---------------------------------------------------------------------------

describe("Settings — Language card", () => {
  it("persists a dropdown selection to the store and shows Saved", () => {
    // Seed with a suggested language so the select reflects it on mount.
    useSession.setState({ l1: "Spanish", regionalVariety: "" });

    renderSettings();

    const select = screen.getByLabelText(
      "What is your first language?"
    ) as HTMLSelectElement;
    expect(select.value).toBe("Spanish");

    // Change to another suggestion.
    fireEvent.change(select, { target: { value: "Korean" } });

    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(useSession.getState().l1).toBe("Korean");
    expect(screen.getByRole("status")).toHaveTextContent("Saved.");
    // Save must also persist to SQLite via the set_l1 IPC, not just the store.
    expect(setL1).toHaveBeenCalledWith("Korean", "");
  });

  it("reveals a free-text input on 'Other…' and persists the typed language", () => {
    useSession.setState({ l1: "", regionalVariety: "" });

    renderSettings();

    const select = screen.getByLabelText(
      "What is your first language?"
    ) as HTMLSelectElement;

    // Selecting "Other…" (sentinel value) reveals the custom-language input.
    fireEvent.change(select, { target: { value: "__other__" } });

    const otherInput = screen.getByLabelText("Your first language");
    expect(otherInput).toBeInTheDocument();

    fireEvent.change(otherInput, { target: { value: "Swahili" } });

    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(useSession.getState().l1).toBe("Swahili");
  });

  it("reopens a saved custom language as 'Other…' with the free-text input", () => {
    // "Swahili" is not in L1_SUGGESTIONS — it must derive to the Other branch.
    useSession.setState({ l1: "Swahili", regionalVariety: "" });

    renderSettings();

    const otherInput = screen.getByLabelText(
      "Your first language"
    ) as HTMLInputElement;
    expect(otherInput).toBeInTheDocument();
    expect(otherInput.value).toBe("Swahili");
  });

  it("persists the dialect / regional-variety input to the store", () => {
    // Seed a non-empty l1 so Save isn't disabled.
    useSession.setState({ l1: "Spanish", regionalVariety: "" });

    renderSettings();

    const dialect = screen.getByLabelText(
      "Regional variety or dialect (optional)"
    );
    fireEvent.change(dialect, { target: { value: "Andalusian" } });

    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(useSession.getState().regionalVariety).toBe("Andalusian");
    // The IPC carries the saved (l1, variety) pair too.
    expect(setL1).toHaveBeenCalledWith("Spanish", "Andalusian");
  });

  it("disables Save when the L1 field is empty", () => {
    useSession.setState({ l1: "", regionalVariety: "" });

    renderSettings();

    const saveBtn = screen.getByRole("button", { name: "Save" });
    expect(saveBtn).toBeDisabled();
  });
});

// ---------------------------------------------------------------------------
// (b) Reset-fade overlay (#114)
// ---------------------------------------------------------------------------

describe("Settings — reset-fade overlay", () => {
  it("renders the .reset-fade overlay after confirming Reset app", () => {
    vi.useFakeTimers();

    useSession.setState({ l1: "Spanish", regionalVariety: "" });

    renderSettings();

    // Settings renders TWO "Reset app" buttons: the trigger row button and
    // (once open) the confirm-danger button inside the dialog. Use getAllBy
    // to grab the trigger (first occurrence in DOM order = the row button).
    const triggerBtns = screen.getAllByRole("button", { name: "Reset app" });
    // The row trigger button has class "secondary settings-data-btn".
    const triggerBtn = triggerBtns.find(
      (el) => el.classList.contains("settings-data-btn")
    ) as HTMLElement;
    expect(triggerBtn).toBeTruthy();
    fireEvent.click(triggerBtn);

    // Dialog should now be open — there are two dialogs (clear-data and
    // reset-ui); target the reset one by its aria-label.
    const dialog = screen.getByRole("dialog", { name: "Reset app?" });
    expect(dialog).toBeInTheDocument();

    // Click the confirm-danger button scoped inside the reset dialog.
    const dialogConfirmBtn = dialog.querySelector(".confirm-danger") as HTMLElement;
    expect(dialogConfirmBtn).not.toBeNull();
    fireEvent.click(dialogConfirmBtn);

    // The .reset-fade overlay must exist in the document immediately
    // (before the RESET_FADE_MS setTimeout has fired).
    expect(document.querySelector(".reset-fade")).not.toBeNull();

    vi.useRealTimers();
  });
});
