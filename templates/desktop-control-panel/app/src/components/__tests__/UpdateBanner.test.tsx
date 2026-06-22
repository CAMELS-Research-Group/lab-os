/**
 * Unit tests for UpdateBanner component.
 *
 * Drives the real useUpdate Zustand store via `useUpdate.setState(...)` to
 * control component state without spinning up actual IPC. The ipc/commands
 * module is mocked so no Tauri runtime is required.
 *
 * Coverage:
 *  - Hidden when available=false
 *  - Hidden when dismissed=true (even if available=true)
 *  - Visible with version and notes when available && !dismissed
 *  - Empty notes → no notes block rendered (no blank gap)
 *  - null notes → no notes block rendered
 *  - Action button calls apply()
 *  - Dismiss button calls dismiss() and hides the banner
 *
 * Spec: CL-23-lite update banner design.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { useUpdate } from "../../store/useUpdate";
import UpdateBanner from "../UpdateBanner";

// ---------------------------------------------------------------------------
// Module mocks — hoisted by vitest
// ---------------------------------------------------------------------------

vi.mock("../../ipc/commands", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../ipc/commands")>();
  return {
    ...real,
    checkForUpdate: vi.fn(),
    applyUpdate: vi.fn(),
  };
});

// ---------------------------------------------------------------------------
// Reset helpers
// ---------------------------------------------------------------------------

// Capture the original action functions BEFORE any test runs so resetStore()
// can restore them after tests that swap them out with spies.
const originalActions = {
  check: useUpdate.getState().check,
  apply: useUpdate.getState().apply,
  dismiss: useUpdate.getState().dismiss,
};

function resetStore() {
  useUpdate.setState({
    checking: false,
    available: false,
    version: null,
    notes: null,
    dismissed: false,
    applyFailed: false,
    // Restore original actions so spies from previous tests don't bleed over.
    ...originalActions,
  });
  vi.clearAllMocks();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("UpdateBanner — hidden states", () => {
  beforeEach(resetStore);

  it("renders nothing when available is false", () => {
    useUpdate.setState({ available: false, dismissed: false });
    const { container } = render(<UpdateBanner />);
    expect(container.firstChild).toBeNull();
  });

  it("renders nothing when available is true but dismissed is true", () => {
    useUpdate.setState({ available: true, version: "0.3.0", notes: "fixes", dismissed: true });
    const { container } = render(<UpdateBanner />);
    expect(container.firstChild).toBeNull();
  });
});

describe("UpdateBanner — visible state", () => {
  beforeEach(resetStore);

  it("renders version and notes when available && !dismissed", () => {
    useUpdate.setState({
      available: true,
      version: "0.3.0",
      notes: "Bug fixes and improvements.",
      dismissed: false,
    });

    render(<UpdateBanner />);

    expect(screen.getByText(/v0\.3\.0/)).toBeInTheDocument();
    expect(screen.getByText("Bug fixes and improvements.")).toBeInTheDocument();
  });

  it("renders action button and dismiss control", () => {
    useUpdate.setState({ available: true, version: "0.3.0", notes: null, dismissed: false });

    render(<UpdateBanner />);

    expect(screen.getByRole("button", { name: /download update/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /dismiss/i })).toBeInTheDocument();
  });
});

describe("UpdateBanner — empty notes case", () => {
  beforeEach(resetStore);

  it("renders no notes block when notes is an empty string", () => {
    useUpdate.setState({ available: true, version: "0.3.0", notes: "", dismissed: false });

    render(<UpdateBanner />);

    // The banner itself renders
    expect(screen.getByRole("status")).toBeInTheDocument();
    // But no notes element (the .update-banner-notes span)
    expect(document.querySelector(".update-banner-notes")).toBeNull();
  });

  it("renders no notes block when notes is whitespace-only", () => {
    useUpdate.setState({ available: true, version: "0.3.0", notes: "   ", dismissed: false });

    render(<UpdateBanner />);

    expect(document.querySelector(".update-banner-notes")).toBeNull();
  });

  it("renders no notes block when notes is null", () => {
    useUpdate.setState({ available: true, version: "0.3.0", notes: null, dismissed: false });

    render(<UpdateBanner />);

    expect(document.querySelector(".update-banner-notes")).toBeNull();
  });
});

describe("UpdateBanner — action button", () => {
  beforeEach(resetStore);

  it("calls apply() when the action button is clicked", async () => {
    // Replace the store's apply action with a spy
    const applySpy = vi.fn().mockResolvedValue(undefined);
    useUpdate.setState({
      available: true,
      version: "0.3.0",
      notes: null,
      dismissed: false,
      apply: applySpy,
    });

    render(<UpdateBanner />);

    fireEvent.click(screen.getByRole("button", { name: /download update/i }));

    // Give the void promise a tick to schedule
    await Promise.resolve();

    expect(applySpy).toHaveBeenCalledOnce();
  });
});

describe("UpdateBanner — dismiss", () => {
  beforeEach(resetStore);

  it("calls dismiss() when the dismiss button is clicked", () => {
    const dismissSpy = vi.fn();
    useUpdate.setState({
      available: true,
      version: "0.3.0",
      notes: null,
      dismissed: false,
      dismiss: dismissSpy,
    });

    render(<UpdateBanner />);

    fireEvent.click(screen.getByRole("button", { name: /dismiss/i }));

    expect(dismissSpy).toHaveBeenCalledOnce();
  });

  it("banner disappears after dismiss() is invoked (real store flow)", () => {
    // Use the real dismiss action so we can test the full round-trip.
    useUpdate.setState({ available: true, version: "0.3.0", notes: null, dismissed: false });

    render(<UpdateBanner />);

    // Banner is visible before dismiss.
    expect(screen.getByRole("status")).toBeInTheDocument();

    // Wrap in act() so React flushes the Zustand state update + re-render
    // synchronously before we assert.
    act(() => {
      fireEvent.click(screen.getByRole("button", { name: /dismiss/i }));
    });

    // After dismiss, the real store sets dismissed=true; the component
    // re-renders and returns null → the status element is gone.
    expect(screen.queryByRole("status")).toBeNull();
  });
});

describe("UpdateBanner — applyFailed error alert", () => {
  beforeEach(resetStore);

  it("renders error alert when banner is visible and applyFailed is true", () => {
    useUpdate.setState({
      available: true,
      version: "0.3.0",
      notes: null,
      dismissed: false,
      applyFailed: true,
    });

    render(<UpdateBanner />);

    const alert = screen.getByRole("alert");
    expect(alert).toBeInTheDocument();
    // Copy now comes from the shared errorCopy map (update_apply_failed).
    expect(alert).toHaveTextContent(/couldn't open the update download/i);
  });

  it("does not render error alert when applyFailed is false", () => {
    useUpdate.setState({
      available: true,
      version: "0.3.0",
      notes: null,
      dismissed: false,
      applyFailed: false,
    });

    render(<UpdateBanner />);

    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("does not render error alert when banner is not visible (available=false)", () => {
    useUpdate.setState({
      available: false,
      dismissed: false,
      applyFailed: true,
    });

    const { container } = render(<UpdateBanner />);

    // Banner itself is hidden — no alert either.
    expect(container.firstChild).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("does not render error alert when dismissed (even if applyFailed=true)", () => {
    useUpdate.setState({
      available: true,
      version: "0.3.0",
      notes: null,
      dismissed: true,
      applyFailed: true,
    });

    const { container } = render(<UpdateBanner />);

    expect(container.firstChild).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
