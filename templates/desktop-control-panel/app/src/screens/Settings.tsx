import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { useNavigate } from "react-router-dom";
import {
  clearAppData,
  getSettings,
  setTheme as persistTheme,
  setUpdateChecksEnabled,
} from "../ipc/commands";
import type { Theme } from "../ipc/types";
import { useSession } from "../store/useSession";
import ConfirmDialog from "../components/ConfirmDialog";
import ErrorNotice from "../components/ErrorNotice";
import "./Settings.css";

type DataConfirm = "reset-ui" | "clear-data" | null;

// Duration of the fade-to-black overlay before the reset tears down the UI and
// routes back to first-run. Kept in sync with the `reset-fade` animation in
// Settings.css so the teardown lands as the screen finishes darkening.
const RESET_FADE_MS = 350;

const THEMES: Theme[] = ["system", "light", "dark"];

export default function Settings() {
  const nav = useNavigate();
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [theme, setThemeState] = useState<Theme | null>(null);
  // Error kinds (not raw strings) so the shared ErrorNotice copy map drives
  // the user-facing text. null = no error.
  const [settingsErrorKind, setSettingsErrorKind] = useState<string | null>(null);

  const resetAll = useSession((s) => s.resetAll);

  const [confirm, setConfirm] = useState<DataConfirm>(null);
  const [dataErrorKind, setDataErrorKind] = useState<string | null>(null);
  const [clearedNote, setClearedNote] = useState<string | null>(null);
  const [resetting, setResetting] = useState(false);

  useEffect(() => {
    getSettings()
      .then((s) => {
        setEnabled(s.update_checks_enabled);
        setThemeState(s.theme);
      })
      .catch(() => setSettingsErrorKind("settings_load_failed"));
  }, []);

  async function handleToggle() {
    if (enabled === null) return;
    const next = !enabled;
    setSettingsErrorKind(null);
    setClearedNote(null);
    setDataErrorKind(null);
    try {
      await setUpdateChecksEnabled(next);
      setEnabled(next);
    } catch {
      setSettingsErrorKind("settings_save_failed");
    }
  }

  async function handleThemeChange(next: Theme) {
    if (theme === null) return;
    const prev = theme;
    setSettingsErrorKind(null);
    setClearedNote(null);
    setDataErrorKind(null);
    // Optimistic: reflect the choice immediately, roll back on failure.
    setThemeState(next);
    try {
      await persistTheme(next);
    } catch {
      setThemeState(prev);
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
      const deleted = await clearAppData();
      setClearedNote(
        deleted === 0
          ? "No saved data to clear."
          : `Cleared ${deleted} saved ${deleted === 1 ? "item" : "items"}.`
      );
    } catch {
      setDataErrorKind("data_clear_failed");
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
        <div className="settings-section-label">Appearance</div>

        {settingsErrorKind && (
          <ErrorNotice kind={settingsErrorKind} variant="inline" />
        )}

        <div className="settings-row">
          <div className="settings-row-text">
            <span className="settings-row-title">Theme</span>
            <span className="settings-row-desc">
              Choose the app's color theme, or follow your system setting.
            </span>
          </div>
          <select
            className="settings-theme-select"
            aria-label="Theme"
            value={theme ?? "system"}
            disabled={theme === null}
            onChange={(e) => handleThemeChange(e.target.value as Theme)}
          >
            {THEMES.map((t) => (
              <option key={t} value={t}>
                {t === "system" ? "System" : t === "light" ? "Light" : "Dark"}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="card">
        <div className="settings-section-label">Updates</div>

        <div className="settings-row">
          <div className="settings-row-text">
            <span className="settings-row-title">Check for updates</span>
            <span className="settings-row-desc">
              Off by default — this is an opt-in feature. When enabled, the app
              contacts the public releases feed over the network to check for a
              newer version. No identifying information is sent: the request
              carries only the standard HTTP headers a browser would send to a
              public URL.
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
            <span className="settings-row-title">Clear app data</span>
            <span className="settings-row-desc">
              Permanently deletes the saved app data on this device. Your
              settings are kept.
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
              Start over from first-run setup. This clears local UI state but
              does <strong>not</strong> delete saved app data; use “Clear app
              data” for that.
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
        title="Clear app data?"
        destructive
        confirmLabel="Clear data"
        onClose={() => setConfirm(null)}
        onConfirm={handleClearData}
        body={
          <p>
            This permanently deletes the saved app data on this device. Your
            settings are kept. This can’t be undone.
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
            This returns the app to first-run setup and clears local UI state.
            Your saved app data is <strong>not</strong> deleted.
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
