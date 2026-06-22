/**
 * Unit tests for the "Give Feedback" ShareModal.
 *
 * Focus: the comment is OPTIONAL (#121). Send is never disabled on an empty
 * note; an empty Send dismisses the modal without calling the on-device
 * `submit_feedback` (which requires a note or rating), while a non-empty Send
 * persists the note and shows the confirmation.
 *
 * `ipc/commands` is mocked so no Tauri runtime is required.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

const { submitFeedback } = vi.hoisted(() => ({ submitFeedback: vi.fn() }));

vi.mock("../../ipc/commands", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../ipc/commands")>();
  return { ...real, submitFeedback };
});

import ShareModal from "../ShareModal";

describe("ShareModal — optional comment (#121)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("leaves Send enabled with an empty comment", () => {
    render(<ShareModal open onClose={() => {}} />);
    expect(screen.getByRole("button", { name: /^send$/i })).not.toBeDisabled();
  });

  it("submits an empty note and confirms when Send is clicked with no comment", async () => {
    submitFeedback.mockResolvedValue(undefined);
    render(<ShareModal open onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: /^send$/i }));

    // The comment is optional: an empty send is persisted on-device (stored as
    // a NULL note by the Rust side) and confirmed, not silently dropped.
    await waitFor(() => expect(submitFeedback).toHaveBeenCalledWith(""));
    expect(await screen.findByText(/thanks/i)).toBeInTheDocument();
  });

  it("submits the trimmed note and confirms when a comment is present", async () => {
    submitFeedback.mockResolvedValue(undefined);
    render(<ShareModal open onClose={() => {}} />);

    fireEvent.change(screen.getByLabelText(/your suggestions/i), {
      target: { value: "  the timer is distracting  " },
    });
    fireEvent.click(screen.getByRole("button", { name: /^send$/i }));

    await waitFor(() =>
      expect(submitFeedback).toHaveBeenCalledWith("the timer is distracting")
    );
    expect(await screen.findByText(/thanks/i)).toBeInTheDocument();
  });
});
