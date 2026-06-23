/**
 * Unit tests for the Settings screen — theme control, update toggle,
 * clear-app-data, and the reset-fade overlay.
 *
 * The ipc/commands module is mocked so no Tauri runtime is required.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { useSession } from "../../store/useSession";
import { setTheme, clearAppData } from "../../ipc/commands";
import Settings from "../Settings";

// ---------------------------------------------------------------------------
// Module mock — ipc/commands
// ---------------------------------------------------------------------------

vi.mock("../../ipc/commands", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../ipc/commands")>();
  return {
    ...real,
    getSettings: vi.fn().mockResolvedValue({
      theme: "system",
      report_uploads_enabled: false,
      update_checks_enabled: false,
    }),
    setUpdateChecksEnabled: vi.fn().mockResolvedValue(undefined),
    setTheme: vi.fn().mockResolvedValue(undefined),
    clearAppData: vi.fn().mockResolvedValue(0),
  };
});

function renderSettings() {
  return render(
    <MemoryRouter>
      <Settings />
    </MemoryRouter>
  );
}

beforeEach(() => {
  useSession.setState({ hasCompletedFirstRun: true });
  vi.clearAllMocks();
});

// ---------------------------------------------------------------------------
// (a) Theme control
// ---------------------------------------------------------------------------

describe("Settings — theme control", () => {
  it("loads the current theme from settings and reflects it in the select", async () => {
    renderSettings();
    const select = (await screen.findByLabelText("Theme")) as HTMLSelectElement;
    expect(select.value).toBe("system");
  });

  it("persists a theme change via setTheme", async () => {
    renderSettings();
    const select = (await screen.findByLabelText("Theme")) as HTMLSelectElement;

    fireEvent.change(select, { target: { value: "dark" } });

    await waitFor(() => expect(setTheme).toHaveBeenCalledWith("dark"));
    expect(select.value).toBe("dark");
  });
});

// ---------------------------------------------------------------------------
// (b) Clear app data
// ---------------------------------------------------------------------------

describe("Settings — clear app data", () => {
  it("calls clearAppData when the clear-data dialog is confirmed", async () => {
    renderSettings();
    // Wait for settings to load so the screen is interactive.
    await screen.findByLabelText("Theme");

    fireEvent.click(screen.getByRole("button", { name: "Clear data" }));

    const dialog = screen.getByRole("dialog", { name: "Clear app data?" });
    const confirmBtn = dialog.querySelector(".confirm-danger") as HTMLElement;
    fireEvent.click(confirmBtn);

    await waitFor(() => expect(clearAppData).toHaveBeenCalledOnce());
  });
});

// ---------------------------------------------------------------------------
// (c) Reset-fade overlay
// ---------------------------------------------------------------------------

describe("Settings — reset-fade overlay", () => {
  it("renders the .reset-fade overlay after confirming Reset app", async () => {
    vi.useFakeTimers();
    try {
      renderSettings();
      // getSettings resolves on a microtask; flush it under fake timers,
      // wrapped in act so the resulting state updates don't warn.
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });

      // Two "Reset app" buttons exist once the dialog opens: the row trigger
      // and the confirm button. The row trigger has class settings-data-btn.
      const triggerBtns = screen.getAllByRole("button", { name: "Reset app" });
      const triggerBtn = triggerBtns.find((el) =>
        el.classList.contains("settings-data-btn")
      ) as HTMLElement;
      expect(triggerBtn).toBeTruthy();
      fireEvent.click(triggerBtn);

      const dialog = screen.getByRole("dialog", { name: "Reset app?" });
      const dialogConfirmBtn = dialog.querySelector(".confirm-danger") as HTMLElement;
      expect(dialogConfirmBtn).not.toBeNull();
      fireEvent.click(dialogConfirmBtn);

      // The .reset-fade overlay must exist immediately (before RESET_FADE_MS).
      expect(document.querySelector(".reset-fade")).not.toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });
});
