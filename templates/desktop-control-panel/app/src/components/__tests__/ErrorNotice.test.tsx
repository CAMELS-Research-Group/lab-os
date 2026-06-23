/**
 * Unit tests for ErrorNotice + the errorCopy map.
 *
 * Coverage:
 *  - A known kind renders its friendly title/body (not the raw kind/message).
 *  - An unknown kind falls back to the generic message — never the raw string.
 *  - The raw message is reachable via the "Technical details" disclosure.
 *  - The optional action button fires its handler.
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import ErrorNotice from "../ErrorNotice";

describe("ErrorNotice — known kind", () => {
  it("renders the friendly title for a known kind", () => {
    render(<ErrorNotice kind="settings_load_failed" />);
    expect(screen.getByText("Couldn't load settings")).toBeInTheDocument();
  });

  it("does not surface the raw kind string as primary content", () => {
    render(<ErrorNotice kind="settings_load_failed" />);
    // The bare kind token should not appear as visible body/title text.
    expect(screen.queryByText("settings_load_failed")).toBeNull();
  });
});

describe("ErrorNotice — unknown kind", () => {
  it("falls back to the generic copy", () => {
    render(<ErrorNotice kind="totally_unmapped_kind" />);
    // Literal expected fallback title — not derived from the unit under test.
    expect(screen.getByText("Something went wrong")).toBeInTheDocument();
  });
});

describe("ErrorNotice — technical details", () => {
  it("keeps the raw message reachable behind a disclosure", () => {
    render(<ErrorNotice kind="settings_save_failed" message="db locked: code 5" />);
    // Friendly copy is primary…
    expect(screen.getByText("Couldn't save that setting")).toBeInTheDocument();
    // …raw message is present (inside the <details>), not discarded.
    expect(screen.getByText(/db locked: code 5/)).toBeInTheDocument();
  });

  it("omits the disclosure when no message is given", () => {
    render(<ErrorNotice kind="settings_save_failed" />);
    expect(document.querySelector(".error-notice-details")).toBeNull();
  });
});

describe("ErrorNotice — action button", () => {
  it("fires onAction when the action button is clicked", () => {
    const onAction = vi.fn();
    render(
      <ErrorNotice kind="update" actionLabel="Retry" onAction={onAction} />
    );
    fireEvent.click(screen.getByRole("button", { name: /retry/i }));
    expect(onAction).toHaveBeenCalledOnce();
  });

  it("renders no action button without actionLabel/onAction", () => {
    render(<ErrorNotice kind="update" />);
    expect(screen.queryByRole("button")).toBeNull();
  });
});
