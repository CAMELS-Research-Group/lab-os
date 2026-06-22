/**
 * Unit tests for the first-run Welcome screen (CL-24).
 *
 * Renders inside a MemoryRouter and asserts the "Get started" CTA navigates to
 * /consent. Navigation is observed via a probe route that renders a sentinel.
 *
 * Spec: CL-24 first-run flow (Welcome → Consent → L1Setup → ModelDownload).
 */

import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import Welcome from "../Welcome";

function renderWelcome() {
  return render(
    <MemoryRouter initialEntries={["/welcome"]}>
      <Routes>
        <Route path="/welcome" element={<Welcome />} />
        <Route path="/consent" element={<div>consent-probe</div>} />
      </Routes>
    </MemoryRouter>
  );
}

describe("Welcome screen", () => {
  it("renders the intro heading and CTA", () => {
    renderWelcome();
    expect(
      screen.getByRole("button", { name: /get started/i })
    ).toBeInTheDocument();
  });

  it("frames feedback as 'Get' (visual), not 'Hear', and leads privacy with PII", () => {
    renderWelcome();
    // The feedback is visual (per-phoneme on Results), so the copy must not
    // promise the learner will "hear" guidance.
    expect(screen.getByText(/get specific guidance/i)).toBeInTheDocument();
    expect(screen.queryByText(/hear what to adjust/i)).toBeNull();
    // Privacy point: titled "Privacy", subtext leads with PII remaining private.
    expect(screen.getByText("Privacy")).toBeInTheDocument();
    expect(screen.queryByText(/stays with you/i)).toBeNull();
    expect(screen.getByText(/PII remains private/i)).toBeInTheDocument();
  });

  it("navigates to /consent when Get started is clicked", () => {
    renderWelcome();
    expect(screen.queryByText("consent-probe")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /get started/i }));

    expect(screen.getByText("consent-probe")).toBeInTheDocument();
  });
});
