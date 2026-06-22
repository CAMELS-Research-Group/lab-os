import { errorCopyFor } from "../lib/errorCopy";
import "./ErrorNotice.css";

type Props = {
  /** Error `kind` from the IPC payload (or a frontend-local kind). */
  kind: string;
  /** Raw developer message — tucked behind a "Technical details" disclosure. */
  message?: string | null;
  /** Optional action button (e.g. Retry). */
  actionLabel?: string;
  onAction?: () => void;
  /** `card` (default) draws a bordered block; `inline` is borderless for use
   *  inside an existing card. */
  variant?: "card" | "inline";
};

/**
 * Renders a friendly, per-kind error message instead of a raw error string.
 * Looks copy up via `errorCopyFor`; unknown kinds get a safe generic message.
 * The raw `message` stays reachable (collapsed disclosure) for support/debug,
 * but is never the primary thing the user reads.
 */
export default function ErrorNotice({
  kind,
  message,
  actionLabel,
  onAction,
  variant = "card",
}: Props) {
  const copy = errorCopyFor(kind);

  return (
    <div className={`error-notice error-notice--${variant}`} role="alert">
      <p className="error-notice-title">{copy.title}</p>
      <p className="error-notice-body">{copy.body}</p>

      {actionLabel && onAction && (
        <div className="error-notice-actions">
          <button className="primary" onClick={onAction}>
            {actionLabel}
          </button>
        </div>
      )}

      {message && (
        <details className="error-notice-details">
          <summary>Technical details</summary>
          <code>
            {kind}: {message}
          </code>
        </details>
      )}
    </div>
  );
}
