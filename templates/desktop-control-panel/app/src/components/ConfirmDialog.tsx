import { useRef } from "react";
import { createPortal } from "react-dom";
import { useModalA11y } from "../lib/useModalA11y";
import "./ConfirmDialog.css";

type Props = {
  open: boolean;
  title: string;
  /** Body copy describing exactly what will happen. String or rich nodes. */
  body: React.ReactNode;
  confirmLabel: string;
  cancelLabel?: string;
  /** Style the confirm button as destructive (terracotta). Default false. */
  destructive?: boolean;
  onConfirm: () => void;
  onClose: () => void;
};

/**
 * Reusable confirmation dialog. Portal + backdrop pattern, generalized so any
 * flow needing a "are you sure?" gate can reuse it. Nothing happens until the
 * user picks Confirm; backdrop click and Cancel both dismiss without acting.
 */
export default function ConfirmDialog({
  open,
  title,
  body,
  confirmLabel,
  cancelLabel = "Cancel",
  destructive = false,
  onConfirm,
  onClose,
}: Props) {
  const modalRef = useRef<HTMLDivElement>(null);
  useModalA11y(open, onClose, modalRef);

  if (!open) return null;

  return createPortal(
    <div
      className="confirm-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      onClick={onClose}
    >
      <div
        className="confirm-modal"
        ref={modalRef}
        tabIndex={-1}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="confirm-title">{title}</h2>
        <div className="confirm-body">{body}</div>
        <div className="confirm-actions">
          <button className="secondary" onClick={onClose}>
            {cancelLabel}
          </button>
          <button
            className={destructive ? "confirm-danger" : "primary"}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
