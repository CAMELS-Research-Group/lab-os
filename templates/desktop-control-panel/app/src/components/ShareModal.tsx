import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { submitFeedback } from "../ipc/commands";
import { useModalA11y } from "../lib/useModalA11y";
import "./ShareModal.css";

type Props = {
  open: boolean;
  onClose: () => void;
};

type Status = "idle" | "saving" | "sent" | "error";

/**
 * The "Give Feedback" modal. Captures a free-form suggestion and stores it on
 * this device (local `feedback` table) — nothing is transmitted. Opened from
 * the global header (`AppHeader`).
 */
export default function ShareModal({ open, onClose }: Props) {
  const [note, setNote] = useState("");
  const [status, setStatus] = useState<Status>("idle");
  const modalRef = useRef<HTMLDivElement>(null);
  useModalA11y(open, onClose, modalRef);

  // Reset the form each time the modal is dismissed/reopened.
  useEffect(() => {
    if (!open) {
      setNote("");
      setStatus("idle");
    }
  }, [open]);

  if (!open) return null;

  // The comment is optional, so Send is only disabled while a submit is
  // in flight — never on an empty note.
  const canSend = status !== "saving";

  async function handleSend() {
    const trimmed = note.trim();
    setStatus("saving");
    try {
      // The comment is optional. An empty submission is still persisted
      // on-device as an empty feedback row — `submit_feedback` accepts a
      // note-less/rating-less send (#121); the act of sending is itself a
      // (weak) signal, and V1 has no rating UI. An empty note stores as NULL
      // on the Rust side. `submitFeedback`/`submit_feedback` and the `feedback`
      // table also accept an optional 1..5 rating; that arm is intentionally
      // unused until a rating UI lands (schema is forward-shaped).
      await submitFeedback(trimmed);
      setStatus("sent");
    } catch {
      setStatus("error");
    }
  }

  return createPortal(
    <div
      className="share-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="Give Feedback"
      onClick={onClose}
    >
      <div
        className="share-modal"
        ref={modalRef}
        tabIndex={-1}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="share-modal-header">
          <h2>Give Feedback</h2>
          <button className="ghost" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        {status === "sent" ? (
          <>
            <div className="share-confirm">
              <strong>Thanks!</strong> Your feedback was saved on this device.
            </div>
            <div className="share-actions">
              <button className="primary" onClick={onClose}>
                Done
              </button>
            </div>
          </>
        ) : (
          <>
            <p className="share-intro">
              Have a suggestion, or did something feel confusing? Tell us in
              your own words. Your note is saved on this device for the team to
              review — nothing is sent over the network.
            </p>

            <label className="feedback-field-label" htmlFor="feedback-note">
              Your suggestions (optional)
            </label>
            <textarea
              id="feedback-note"
              className="feedback-textarea"
              value={note}
              onChange={(e) => setNote(e.target.value)}
              placeholder="What would make this better?"
              rows={5}
              autoFocus
            />

            {status === "error" && (
              <p className="feedback-error" role="alert">
                Couldn’t save your feedback. Please try again.
              </p>
            )}

            <div className="share-actions">
              <button className="secondary" onClick={onClose}>
                Cancel
              </button>
              <button
                className="primary"
                onClick={handleSend}
                disabled={!canSend}
              >
                {status === "saving" ? "Saving…" : "Send"}
              </button>
            </div>
          </>
        )}
      </div>
    </div>,
    document.body
  );
}
