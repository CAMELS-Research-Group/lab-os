/**
 * Unit tests for the first-run Welcome gate.
 *
 * Renders inside a MemoryRouter and asserts the "Get started" CTA records
 * first-run completion in the session store and navigates to the root gate.
 * Navigation is observed via a probe route that renders a sentinel.
 */

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import Welcome from "../Welcome";
import { useSession } from "../../store/useSession";

function renderWelcome() {
  return render(
    <MemoryRouter initialEntries={["/welcome"]}>
      <Routes>
        <Route path="/welcome" element={<Welcome />} />
        <Route path="/" element={<div>root-gate-probe</div>} />
      </Routes>
    </MemoryRouter>
  );
}

beforeEach(() => {
  useSession.setState({ hasCompletedFirstRun: false });
});

describe("Welcome gate", () => {
  it("renders the intro heading and CTA", () => {
    renderWelcome();
    expect(
      screen.getByRole("button", { name: /get started/i })
    ).toBeInTheDocument();
  });

  it("leads privacy with PII remaining private", () => {
    renderWelcome();
    expect(screen.getByText("Privacy")).toBeInTheDocument();
    expect(screen.getByText(/PII remains private/i)).toBeInTheDocument();
  });

  it("records first-run completion and navigates to the root gate on Get started", () => {
    renderWelcome();
    expect(screen.queryByText("root-gate-probe")).toBeNull();
    expect(useSession.getState().hasCompletedFirstRun).toBe(false);

    fireEvent.click(screen.getByRole("button", { name: /get started/i }));

    expect(useSession.getState().hasCompletedFirstRun).toBe(true);
    expect(screen.getByText("root-gate-probe")).toBeInTheDocument();
  });
});
