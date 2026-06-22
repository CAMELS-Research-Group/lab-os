/**
 * Routing tests for the FirstRunGate (CL-24).
 *
 * The gate (in App.tsx) redirects from "/" based on two persisted store flags:
 *   - !hasCompletedFirstRun        → /welcome
 *   - hasCompletedFirstRun, !modelReady → /model-download
 *   - both true                    → /home (inside the app shell)
 *
 * App is rendered inside a MemoryRouter at "/" with the store seeded per branch.
 * The IPC command + event modules are mocked so neither the UpdateBanner's
 * background check nor the ModelDownload screen's effect needs a Tauri runtime.
 *
 * Spec: CL-24 first-run flow gating.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

// Mock the whole IPC surface used transitively by App's children so jsdom never
// reaches a real `invoke`. checkForUpdate resolves to "no update".
vi.mock("../ipc/commands", () => ({
  checkForUpdate: vi.fn().mockResolvedValue({
    available: false,
    version: null,
    notes: null,
  }),
  applyUpdate: vi.fn().mockResolvedValue(undefined),
  startFirstRunModelDownload: vi.fn().mockResolvedValue(undefined),
  // Home (the post-first-run landing) refreshes session history on mount.
  getSessionHistory: vi.fn().mockResolvedValue({ sessions: [] }),
}));

const noopUnlisten = vi.fn();
vi.mock("../ipc/events", () => ({
  listenModelDownloadProgress: vi.fn(() => Promise.resolve(noopUnlisten)),
  listenModelDownloadDone: vi.fn(() => Promise.resolve(noopUnlisten)),
  listenModelDownloadError: vi.fn(() => Promise.resolve(noopUnlisten)),
}));

import App from "../App";
import { useSession } from "../store/useSession";

function renderAppAtRoot() {
  return render(
    <MemoryRouter initialEntries={["/"]}>
      <App />
    </MemoryRouter>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("FirstRunGate routing", () => {
  it("routes to /welcome when first-run is not complete", () => {
    useSession.setState({ hasCompletedFirstRun: false, modelReady: false });
    renderAppAtRoot();
    expect(
      screen.getByRole("button", { name: /get started/i })
    ).toBeInTheDocument();
  });

  it("routes to /model-download when first-run done but model not ready", () => {
    useSession.setState({ hasCompletedFirstRun: true, modelReady: false });
    renderAppAtRoot();
    expect(
      screen.getByRole("heading", { name: /practice model ready/i })
    ).toBeInTheDocument();
  });

  it("routes to /home when first-run done and model ready", () => {
    useSession.setState({ hasCompletedFirstRun: true, modelReady: true });
    renderAppAtRoot();
    expect(
      screen.getByRole("heading", { name: /clear up/i })
    ).toBeInTheDocument();
  });
});
