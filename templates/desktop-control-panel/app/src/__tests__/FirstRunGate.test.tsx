/**
 * Routing tests for the FirstRunGate.
 *
 * The gate (in App.tsx) redirects from "/" based on a single persisted flag:
 *   - !hasCompletedFirstRun → /welcome
 *   - hasCompletedFirstRun  → /home (inside the app shell)
 *
 * App is rendered inside a MemoryRouter at "/" with the store seeded per branch.
 * The IPC command module is mocked so the UpdateBanner's background check never
 * reaches a real `invoke`.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

// Mock the IPC surface used transitively by App's children so jsdom never
// reaches a real `invoke`. checkForUpdate resolves to "no update".
vi.mock("../ipc/commands", () => ({
  checkForUpdate: vi.fn().mockResolvedValue({
    available: false,
    version: null,
    notes: null,
  }),
  applyUpdate: vi.fn().mockResolvedValue(undefined),
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
    useSession.setState({ hasCompletedFirstRun: false });
    renderAppAtRoot();
    expect(
      screen.getByRole("button", { name: /get started/i })
    ).toBeInTheDocument();
  });

  it("routes to /home when first-run is complete", () => {
    useSession.setState({ hasCompletedFirstRun: true });
    renderAppAtRoot();
    expect(
      screen.getByRole("heading", { name: /welcome back/i })
    ).toBeInTheDocument();
  });
});
