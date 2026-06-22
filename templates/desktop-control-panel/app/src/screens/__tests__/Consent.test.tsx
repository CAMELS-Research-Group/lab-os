/**
 * Unit tests for the first-run Consent screen (CL-24, offline reviewer build).
 *
 * Two-step opt-in (#119): "Continue" opens a ConfirmDialog; only "I agree"
 * inside it acknowledges consent and navigates to /setup. Backdrop / Cancel
 * dismiss without acting.
 *
 * Crucially this build must NOT call any backend / `accept_consent` — the
 * consent is a local acknowledgment only. We assert the store flag via the
 * real store and the navigation via a probe route.
 *
 * Spec: CL-24 first-run flow; consent scoped to local acknowledgment only.
 */

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import Consent from "../Consent";
import { useSession } from "../../store/useSession";

function renderConsent() {
  return render(
    <MemoryRouter initialEntries={["/consent"]}>
      <Routes>
        <Route path="/consent" element={<Consent />} />
        <Route path="/setup" element={<div>setup-probe</div>} />
      </Routes>
    </MemoryRouter>
  );
}

describe("Consent screen", () => {
  beforeEach(() => {
    useSession.setState({ consentAcknowledged: false });
  });

  it("renders a Continue button (not I agree) and the on-device privacy copy", () => {
    renderConsent();
    // Primary action is now "Continue", not "I agree"
    expect(
      screen.getByRole("button", { name: /continue/i })
    ).toBeInTheDocument();
    // Privacy posture must be stated: voice never leaves the device.
    expect(screen.getByText(/voice never leaves/i)).toBeInTheDocument();
    // "I agree" is not yet visible before the dialog is opened
    expect(screen.queryByRole("button", { name: /i agree/i })).toBeNull();
  });

  it("does not acknowledge consent just by rendering", () => {
    renderConsent();
    expect(useSession.getState().consentAcknowledged).toBe(false);
  });

  it("clicking Continue opens the ConfirmDialog without acknowledging consent or navigating", () => {
    renderConsent();
    expect(useSession.getState().consentAcknowledged).toBe(false);
    expect(screen.queryByText("setup-probe")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /continue/i }));

    // Dialog is now open — "I agree" button visible inside it
    expect(
      screen.getByRole("button", { name: /i agree/i })
    ).toBeInTheDocument();
    // Consent still NOT acknowledged and no navigation yet
    expect(useSession.getState().consentAcknowledged).toBe(false);
    expect(screen.queryByText("setup-probe")).toBeNull();
  });

  it("clicking I agree in the dialog acknowledges consent and navigates to /setup", () => {
    renderConsent();

    fireEvent.click(screen.getByRole("button", { name: /continue/i }));
    fireEvent.click(screen.getByRole("button", { name: /i agree/i }));

    expect(useSession.getState().consentAcknowledged).toBe(true);
    expect(screen.getByText("setup-probe")).toBeInTheDocument();
  });

  it("cancelling the dialog does not acknowledge consent or navigate", () => {
    renderConsent();

    fireEvent.click(screen.getByRole("button", { name: /continue/i }));
    // Cancel closes the dialog
    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));

    expect(useSession.getState().consentAcknowledged).toBe(false);
    expect(screen.queryByText("setup-probe")).toBeNull();
    // "I agree" button gone after cancel
    expect(screen.queryByRole("button", { name: /i agree/i })).toBeNull();
  });
});
