import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { useNavigate } from "react-router-dom";
import { clearSessionData, getSettings, setL1 as persistL1, setUpdateChecksEnabled } from "../ipc/commands";
import { useSession } from "../store/useSession";
import ConfirmDialog from "../components/ConfirmDialog";
import ErrorNotice from "../components/ErrorNotice";
import LanguageSelect from "../components/LanguageSelect";
import "./Settings.css";

type DataConfirm = "reset-ui" | "clear-data" | null;

// Duration of the fade-to-black overlay before the reset tears down the UI and
// routes back to first-run. Kept in sync with the `reset-fade` animation in
// Settings.css so the teardown lands as the screen finishes darkening.
const RESET_FADE_MS = 350;

export default function Settings() {
  const nav = useNavigate();
  const [enabled, setEnabled] = useState<boolean | null>(null);
  // Error kinds (not raw strings) so the shared ErrorNotice copy map drives
  // the user-facing text. null = no error.
  const [settingsErrorKind, setSettingsErrorKind] = useState<string | null>(null);

  const resetAll = useSession((s) => s.resetAll);
  const clearSessions = useSession((s) => s.clearSessions);
  const sessionCount = useSession((s) => s.sessions.length);
  const setL1 = useSession((s) => s.setL1);
  const storeL1 = useSession((s) => s.l1);
  const storeVariety = useSession((s) => s.regionalVariety);

  // Language section — local draft state, written to the store only on Save.
  const [l1Draft, setL1Draft] = useState(storeL1);
  const [varietyDraft, setVarietyDraft] = useState(storeVariety);
  const [l1Saved, setL1Saved] = useState(false);

  const [confirm, setConfirm] = useState<DataConfirm>(null);
  const [dataErrorKind, setDataErrorKind] = useState<string | null>(null);
  const [clearedNote, setClearedNote] = useState<string | null>(null);
  const [resetting, setResetting] = useState(false);

  useEffect(() => {
    getSettings()
      .then((s) => setEnabled(s.update_checks_enabled))
      .catch(() => setSettingsErrorKind("settings_load_failed"));
  }, []);

  async function handleToggle() {
    if (enabled === null) return;
    const next = !enabled;
    setSettingsErrorKind(null);
    // Don't leave a stale "Cleared N sessions" / data-error banner sitting on
    // the screen while an unrelated action runs.
    setClearedNote(null);
    setDataErrorKind(null);
    try {
      await setUpdateChecksEnabled(next);
      setEnabled(next);
    } catch {
      // Leave prior value; surface a transient error.
      setSettingsErrorKind("settings_save_failed");
    }
  }

  // Opening either confirm clears any stale status/error banner from a prior
  // data action so it doesn't linger alongside an unrelated dialog.
  function openConfirm(which: DataConfirm) {
    setClearedNote(null);
    setDataErrorKind(null);
    setConfirm(which);
  }

  function handleResetUi() {
    setConfirm(null);
    // Fade to black first so the reset reads as a deliberate transition rather
    // than a hard jump back to first-run. The actual teardown + route happens
    // once the overlay has darkened (RESET_FADE_MS).
    setResetting(true);
    window.setTimeout(() => {
      resetAll();
      nav("/");
    }, RESET_FADE_MS);
  }

  async function handleClearData() {
    setConfirm(null);
    setDataErrorKind(null);
    setClearedNote(null);
    try {
      const deleted = await clearSessionData();
      clearSessions();
      setClearedNote(
        deleted === 0
          ? "No saved sessions to clear."
          : `Cleared ${deleted} saved ${deleted === 1 ? "session" : "sessions"}.`
      );
    } catch {
      setDataErrorKind("session_clear_failed");
    }
  }

  return (
    <div className="screen settings-screen">
      <button className="ghost settings-back" onClick={() => nav(-1)}>
        ← Back
      </button>

      <h1>Settings</h1>
      <p className="lede">App preferences. Additional settings will appear here in future releases.</p>

      <div className="card">
        <div className="settings-section-label">Language</div>

        {l1Saved && (
          <p className="settings-note" role="status">Saved.</p>
        )}

        <LanguageSelect
          l1={l1Draft}
          variety={varietyDraft}
          onL1Change={(v) => { setL1Draft(v); setL1Saved(false); }}
          onVarietyChange={(v) => { setVarietyDraft(v); setL1Saved(false); }}
          idPrefix="settings"
        />

        <div className="actions">
          <button
            className="primary"
            disabled={l1Draft.trim().length === 0}
            onClick={() => {
              const l1 = l1Draft.trim();
              const variety = varietyDraft.trim();
              setL1(l1, variety);
              setL1Saved(true);
              // Also persist to the SQLite settings row via the Rust IPC so the
              // eval pipeline (which reads settings.l1 and stamps l1_at_session)
              // sees the change — not just the local store. Don't block the
              // "Saved." confirmation on it; swallow the rejection so a
              // no-backend build (React-dev / tests) doesn't throw an unhandled
              // promise rejection (same offline-tolerant pattern as getSettings).
              persistL1(l1, variety).catch(() => {});
            }}
          >
            Save
          </button>
        </div>
      </div>

      <div className="card">
        <div className="settings-section-label">Updates</div>

        {settingsErrorKind && (
          <ErrorNotice kind={settingsErrorKind} variant="inline" />
        )}

        <div className="settings-row">
          <div className="settings-row-text">
            <span className="settings-row-title">Check for updates</span>
            <span className="settings-row-desc">
              Off by default — this is an opt-in feature. When enabled, the app
              contacts the public GitHub releases feed over the network to check
              for a newer version. No identifying information is sent: the
              request carries only the standard HTTP headers a browser would send
              to a public URL.
            </span>
          </div>

          <button
            className={`toggle-btn ${enabled ? "toggle-on" : "toggle-off"}`}
            role="switch"
            aria-checked={enabled ?? false}
            aria-label="Check for updates"
            disabled={enabled === null}
            onClick={handleToggle}
          >
            <span className="toggle-thumb" />
          </button>
        </div>
      </div>

      <div className="card">
        <div className="settings-section-label">Your data</div>

        {dataErrorKind && <ErrorNotice kind={dataErrorKind} variant="inline" />}
        {clearedNote && <p className="settings-note" role="status">{clearedNote}</p>}

        <div className="settings-row">
          <div className="settings-row-text">
            <span className="settings-row-title">Clear session data</span>
            <span className="settings-row-desc">
              Permanently deletes your saved practice history on this device
              {sessionCount > 0 ? ` (${sessionCount} saved)` : ""}. Your
              language preferences and settings are kept.
            </span>
          </div>
          <button
            className="secondary settings-data-btn"
            onClick={() => openConfirm("clear-data")}
          >
            Clear data
          </button>
        </div>

        <div className="settings-row settings-row-divided">
          <div className="settings-row-text">
            <span className="settings-row-title">Reset app</span>
            <span className="settings-row-desc">
              Start over from first-run setup — useful if you want to practice
              with a different first language, or hand the device to a new
              learner. This clears your language profile but does
              <strong> not</strong> delete your saved session history; use
              “Clear session data” for that.
            </span>
          </div>
          <button
            className="secondary settings-data-btn"
            onClick={() => openConfirm("reset-ui")}
          >
            Reset app
          </button>
        </div>
      </div>

      <ConfirmDialog
        open={confirm === "clear-data"}
        title="Clear session data?"
        destructive
        confirmLabel="Clear data"
        onClose={() => setConfirm(null)}
        onConfirm={handleClearData}
        body={
          <p>
            This permanently deletes your saved practice history on this
            device. Your language preferences and settings are kept. This can’t
            be undone.
          </p>
        }
      />

      <ConfirmDialog
        open={confirm === "reset-ui"}
        title="Reset app?"
        destructive
        confirmLabel="Reset app"
        onClose={() => setConfirm(null)}
        onConfirm={handleResetUi}
        body={
          <p>
            This clears your language profile and returns the app to first-run
            setup — use it to start fresh with a different first language or to
            hand the device to a new learner. Your saved session history is{" "}
            <strong>not</strong> deleted.
          </p>
        }
      />

      {resetting &&
        createPortal(
          <div className="reset-fade" aria-hidden="true" />,
          document.body
        )}
    </div>
  );
}
