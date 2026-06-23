import "./UpdateBanner.css";
import { useUpdate } from "../store/useUpdate";
import { errorCopyFor } from "../lib/errorCopy";

/**
 * App-level update notification banner.
 *
 * Renders only when `available && !dismissed`. Shows the new version string,
 * optional release notes, an action button that opens the release download
 * page (apply()), and a dismiss control that hides the banner for the current
 * session.
 */
export default function UpdateBanner() {
  const available = useUpdate((s) => s.available);
  const dismissed = useUpdate((s) => s.dismissed);
  const version = useUpdate((s) => s.version);
  const notes = useUpdate((s) => s.notes);
  const applyFailed = useUpdate((s) => s.applyFailed);
  const apply = useUpdate((s) => s.apply);
  const dismiss = useUpdate((s) => s.dismiss);

  if (!available || dismissed) return null;

  return (
    <div className="update-banner" role="status" aria-live="polite">
      <div className="update-banner-body">
        <span className="update-banner-label">
          Update available{version ? `: v${version}` : ""}
        </span>
        {notes && notes.trim().length > 0 && (
          <span className="update-banner-notes">{notes}</span>
        )}
        {applyFailed && (
          <span className="update-banner-error" role="alert">
            {errorCopyFor("update_apply_failed").body}
          </span>
        )}
      </div>
      <div className="update-banner-actions">
        <button
          className="update-banner-btn update-banner-btn--apply"
          onClick={() => void apply()}
        >
          Download update
        </button>
        <button
          className="update-banner-btn update-banner-btn--dismiss"
          onClick={dismiss}
          aria-label="Dismiss update notification"
        >
          Later
        </button>
      </div>
    </div>
  );
}
